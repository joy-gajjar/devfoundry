use crate::{ApiError, ApiResult, ServerState};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use devfoundry_schema::ProjectId;
use devfoundry_storage::ProjectRepository;
use devfoundry_tools::{
    NativePtyError, NativePtyService, PtyInput, PtyOpenRequest, PtyResize, ToolContext,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct TerminalRegistry {
    terminals: HashMap<String, TerminalHandle>,
}

struct TerminalHandle {
    service: Arc<NativePtyService>,
    lease: Option<(String, String, devfoundry_tools::InputLease)>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateRequest {
    pub program: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub cwd: Option<String>,
    pub rows: u16,
    pub columns: u16,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LeaseRequest {
    pub client_id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InputRequest {
    pub lease_id: String,
    pub bytes: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ResizeRequest {
    pub rows: u16,
    pub columns: u16,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OutputQuery {
    pub after: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TerminalResponse {
    pub terminal_id: String,
    pub project_id: ProjectId,
    pub platform: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct OutputResponse {
    pub terminal_id: String,
    pub offset: u64,
    pub next_offset: u64,
    pub bytes: String,
    pub truncated: bool,
    pub gap: bool,
}

pub(crate) async fn create(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
    Json(request): Json<CreateRequest>,
) -> ApiResult<(StatusCode, Json<TerminalResponse>)> {
    let project_id = project_id
        .parse::<ProjectId>()
        .map_err(|_| ApiError::bad_request("invalid_project_id", "project id is invalid"))?;
    let project = state
        .store
        .get_project(project_id)
        .await
        .map_err(crate::storage_error)?
        .ok_or_else(|| ApiError::not_found("project_not_found", "project was not found"))?;
    if request.rows == 0 || request.columns == 0 || request.program.trim().is_empty() {
        return Err(ApiError::bad_request(
            "invalid_terminal_request",
            "terminal dimensions and program are required",
        ));
    }
    let service = NativePtyService::open(
        PtyOpenRequest {
            program: request.program,
            arguments: request.arguments,
            cwd: request.cwd,
            rows: request.rows,
            columns: request.columns,
        },
        ToolContext {
            root: PathBuf::from(project.root),
            permissions: state.preview_permissions.clone(),
            cancellation: CancellationToken::new(),
        },
    )
    .await
    .map_err(map_pty_error)?;
    let id = ulid::Ulid::new().to_string();
    state.terminals.lock().await.terminals.insert(
        id.clone(),
        TerminalHandle {
            service: Arc::new(service),
            lease: None,
        },
    );
    Ok((
        StatusCode::CREATED,
        Json(TerminalResponse {
            terminal_id: id,
            project_id,
            platform: NativePtyService::capability().platform,
        }),
    ))
}

pub(crate) async fn acquire_lease(
    State(state): State<ServerState>,
    Path(terminal_id): Path<String>,
    Json(request): Json<LeaseRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if request.client_id.trim().is_empty() {
        return Err(ApiError::bad_request(
            "invalid_client_id",
            "client_id is required",
        ));
    }
    let mut terminals = state.terminals.lock().await;
    let terminal = terminals
        .terminals
        .get_mut(&terminal_id)
        .ok_or_else(|| ApiError::not_found("terminal_not_found", "terminal was not found"))?;
    if terminal.lease.is_some() {
        return Err(ApiError::conflict(
            "terminal_input_lease_held",
            "terminal input is leased by another client",
        ));
    }
    let lease_id = ulid::Ulid::new().to_string();
    terminal.lease = Some((
        request.client_id.clone(),
        lease_id.clone(),
        terminal.service.input_lease(),
    ));
    Ok(Json(
        serde_json::json!({"terminal_id": terminal_id, "lease_id": lease_id, "client_id": request.client_id}),
    ))
}

pub(crate) async fn release_lease(
    State(state): State<ServerState>,
    Path(terminal_id): Path<String>,
    Json(request): Json<LeaseRequest>,
) -> ApiResult<StatusCode> {
    let mut terminals = state.terminals.lock().await;
    let terminal = terminals
        .terminals
        .get_mut(&terminal_id)
        .ok_or_else(|| ApiError::not_found("terminal_not_found", "terminal was not found"))?;
    if terminal
        .lease
        .as_ref()
        .is_some_and(|(client_id, _, _)| client_id == &request.client_id)
    {
        terminal.lease = None;
        return Ok(StatusCode::NO_CONTENT);
    }
    Err(ApiError::conflict(
        "terminal_input_lease_invalid",
        "input lease is not owned by this client",
    ))
}

pub(crate) async fn input(
    State(state): State<ServerState>,
    Path(terminal_id): Path<String>,
    Json(request): Json<InputRequest>,
) -> ApiResult<StatusCode> {
    let terminals = state.terminals.lock().await;
    let terminal = terminals
        .terminals
        .get(&terminal_id)
        .ok_or_else(|| ApiError::not_found("terminal_not_found", "terminal was not found"))?;
    let (_, _lease_id, lease) = terminal
        .lease
        .as_ref()
        .filter(|(_, id, _)| id == &request.lease_id)
        .ok_or_else(|| {
            ApiError::conflict("terminal_input_lease_invalid", "input lease is invalid")
        })?;
    let bytes = decode_hex(&request.bytes).map_err(|_| {
        ApiError::bad_request("invalid_terminal_input", "input bytes must be hexadecimal")
    })?;
    terminal
        .service
        .input(PtyInput::new(*lease, bytes))
        .await
        .map_err(map_pty_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn resize(
    State(state): State<ServerState>,
    Path(terminal_id): Path<String>,
    Json(request): Json<ResizeRequest>,
) -> ApiResult<StatusCode> {
    let terminals = state.terminals.lock().await;
    let terminal = terminals
        .terminals
        .get(&terminal_id)
        .ok_or_else(|| ApiError::not_found("terminal_not_found", "terminal was not found"))?;
    terminal
        .service
        .resize(PtyResize {
            rows: request.rows,
            columns: request.columns,
        })
        .await
        .map_err(map_pty_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn output(
    State(state): State<ServerState>,
    Path(terminal_id): Path<String>,
    Query(query): Query<OutputQuery>,
) -> ApiResult<Json<OutputResponse>> {
    let terminals = state.terminals.lock().await;
    let terminal = terminals
        .terminals
        .get(&terminal_id)
        .ok_or_else(|| ApiError::not_found("terminal_not_found", "terminal was not found"))?;
    let output = terminal
        .service
        .read_output(query.after.unwrap_or(0), Duration::from_millis(25))
        .await
        .map_err(map_pty_error)?;
    Ok(Json(OutputResponse {
        terminal_id,
        offset: output.offset,
        next_offset: output.next_offset,
        bytes: encode_hex(&output.bytes),
        truncated: output.truncated,
        gap: false,
    }))
}

pub(crate) async fn close(
    State(state): State<ServerState>,
    Path(terminal_id): Path<String>,
) -> ApiResult<StatusCode> {
    let terminal = state
        .terminals
        .lock()
        .await
        .terminals
        .remove(&terminal_id)
        .ok_or_else(|| ApiError::not_found("terminal_not_found", "terminal was not found"))?;
    terminal.service.close().await.map_err(map_pty_error)?;
    Ok(StatusCode::NO_CONTENT)
}

fn map_pty_error(error: NativePtyError) -> ApiError {
    match error {
        NativePtyError::UnsupportedPlatform => ApiError::bad_request(
            "terminal_unsupported",
            "native terminal is unsupported on this platform",
        ),
        NativePtyError::OutputGap => ApiError::conflict(
            "terminal_output_gap",
            "requested terminal output is no longer available",
        ),
        NativePtyError::InvalidInputLease => {
            ApiError::conflict("terminal_input_lease_invalid", "input lease is invalid")
        }
        NativePtyError::InputTooLarge => {
            ApiError::bad_request("terminal_input_too_large", "terminal input is too large")
        }
        NativePtyError::InvalidRequest(message) => {
            ApiError::bad_request("invalid_terminal_request", &message)
        }
        other => ApiError::bad_request("terminal_failed", &other.to_string()),
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>, ()> {
    if value.len() % 2 != 0 {
        return Err(());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).map_err(|_| ()))
        .collect()
}
