//! Local HTTP and SSE transport for the DevFoundry runtime.

use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::sse::{Event as SseEvent, KeepAlive, Sse},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use devfoundry_core::{PreviewRegistry, SessionRunner};
use devfoundry_protocol::{
    CreateProjectRequest, CreateSessionRequest, ErrorResponse, ModelOption, PromptRequest,
    PromptStatusResponse, UpdateSessionRequest,
};
use devfoundry_schema::{
    AgentName, ModelName, ModelRef, PermissionRequest, Project, ProjectId, ProviderName, Session,
    SessionId, SessionStatus,
};
use devfoundry_storage::{EventRepository, ProjectRepository, SessionRepository, SqliteStore};
use futures_util::stream::Stream;
use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

mod browser;
mod routes_notifications;
mod routes_previews;
mod routes_resources;
mod routes_terminals;
mod routes_v2;
mod routes_w10;
pub mod routes_workers;
pub mod w10;

pub use browser::BrowserConfig;

#[derive(Clone)]
pub struct ServerState {
    pub store: Arc<SqliteStore>,
    pub runner: Arc<SessionRunner>,
    pub executions: Arc<ExecutionRegistry>,
    pub previews: Arc<Mutex<PreviewRegistry>>,
    pub preview_permissions: Arc<dyn devfoundry_tools::PermissionBroker>,
    pub terminals: Arc<Mutex<routes_terminals::TerminalRegistry>>,
    pub workers: Arc<routes_workers::WorkerRegistry>,
}

#[derive(Clone, Debug, Default)]
pub struct ApiSecurity {
    pub bearer_token: Option<String>,
    pub allowed_origins: Vec<HeaderValue>,
}

pub struct ExecutionRegistry {
    active: Mutex<HashMap<SessionId, CancellationToken>>,
    shutdown: CancellationToken,
}

impl Default for ExecutionRegistry {
    fn default() -> Self {
        Self {
            active: Mutex::new(HashMap::new()),
            shutdown: CancellationToken::new(),
        }
    }
}

impl ExecutionRegistry {
    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }
}

impl ExecutionRegistry {
    async fn start(&self, session_id: SessionId) -> Option<CancellationToken> {
        let mut active = self.active.lock().await;
        if active.contains_key(&session_id) {
            return None;
        }
        let token = CancellationToken::new();
        active.insert(session_id, token.clone());
        Some(token)
    }

    async fn finish(&self, session_id: SessionId) {
        self.active.lock().await.remove(&session_id);
    }

    async fn cancel(&self, session_id: SessionId) -> bool {
        let active = self.active.lock().await;
        if let Some(token) = active.get(&session_id) {
            token.cancel();
            true
        } else {
            false
        }
    }
}

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_PROMPT_BYTES: usize = 256 * 1024;
const MAX_REPLAY_EVENTS: usize = 1_000;

#[derive(Clone, Debug)]
struct RequestId;

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    response: ErrorResponse,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = (self.status, Json(self.response)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        response
    }
}

type ApiResult<T> = Result<T, ApiError>;

async fn request_context(
    mut request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .map(str::to_owned)
        .unwrap_or_else(|| ulid::Ulid::new().to_string());
    request.extensions_mut().insert(RequestId);
    let mut response = next.run(request).await;
    if response.status().is_client_error() || response.status().is_server_error() {
        let (parts, body) = response.into_parts();
        if let Ok(bytes) = axum::body::to_bytes(body, MAX_REQUEST_BYTES).await {
            if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                if value.get("request_id").is_some() {
                    value["request_id"] = serde_json::Value::String(request_id.clone());
                    response = Response::from_parts(
                        parts,
                        axum::body::Body::from(
                            serde_json::to_vec(&value).unwrap_or_else(|_| bytes.to_vec()),
                        ),
                    );
                } else {
                    response = Response::from_parts(parts, axum::body::Body::from(bytes));
                }
            } else {
                response = Response::from_parts(parts, axum::body::Body::from(bytes));
            }
        } else {
            response = Response::from_parts(parts, axum::body::Body::empty());
        }
    }
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

pub fn router(state: ServerState) -> Router {
    router_with_security(state, ApiSecurity::default())
}

pub fn router_with_previews(
    state: ServerState,
    security: ApiSecurity,
    previews: Arc<Mutex<PreviewRegistry>>,
) -> Router {
    let mut state = state;
    state.previews = previews;
    router_with_security(state, security)
}

pub fn router_with_security(state: ServerState, security: ApiSecurity) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/models", get(list_models))
        .route("/api/v1/projects", get(list_projects).post(create_project))
        .route(
            "/api/v1/projects/{project_id}/sessions",
            get(list_sessions).post(create_session),
        )
        .route(
            "/api/v1/sessions/{session_id}",
            get(get_session).patch(update_session),
        )
        .route("/api/v1/sessions/{session_id}/prompt", post(prompt))
        .route("/api/v1/prompts/{prompt_id}", get(get_prompt_status))
        .route("/api/v1/sessions/{session_id}/interrupt", post(interrupt))
        .route(
            "/api/v1/sessions/{session_id}/permissions",
            get(list_permissions),
        )
        .route(
            "/api/v1/permissions/{permission_id}/resolve",
            post(resolve_permission),
        )
        .route("/api/v1/sessions/{session_id}/events", get(events))
        .route(
            "/api/v2/sessions/{session_id}/messages",
            get(routes_v2::history),
        )
        .route(
            "/api/v2/sessions/{session_id}/snapshot",
            get(routes_v2::snapshot),
        )
        .route(
            "/api/v2/sessions/{session_id}/prompt",
            post(routes_v2::idempotent_prompt),
        )
        .route("/api/v2/sessions/{session_id}/events", get(events))
        .route(
            "/api/v2/workspaces/{workspace_id}/tasks",
            get(routes_w10::list_tasks).post(routes_w10::create_task),
        )
        .route(
            "/api/v2/workspaces/{workspace_id}/roadmap/export",
            post(routes_w10::export_roadmap),
        )
        .route(
            "/api/v2/workspaces/{workspace_id}/roadmap/import",
            post(routes_w10::import_roadmap),
        )
        .route(
            "/api/v2/projects/{project_id}/documents",
            get(routes_w10::list_documents),
        )
        .route(
            "/api/v2/tasks/{task_id}",
            get(routes_w10::get_task).patch(routes_w10::update_task),
        )
        .route(
            "/api/v2/workspaces/{workspace_id}/workers",
            get(routes_workers::workers),
        )
        .route(
            "/api/v2/tasks/{task_id}/assign",
            post(routes_workers::assign),
        )
        .route(
            "/api/v2/attempts/{attempt_id}/cancel",
            post(routes_workers::cancel),
        )
        .route(
            "/api/v2/attempts/{attempt_id}/evidence",
            get(routes_workers::evidence),
        )
        .route(
            "/api/v2/projects/{project_id}/notifications/status",
            get(routes_notifications::status),
        )
        .route(
            "/api/v2/projects/{project_id}/notifications/setup",
            post(routes_notifications::setup),
        )
        .route(
            "/api/v2/projects/{project_id}/notifications/revoke",
            post(routes_notifications::revoke),
        )
        .route(
            "/api/v2/projects/{project_id}/resources/inspect",
            post(routes_resources::inspect),
        )
        .route(
            "/api/v2/projects/{project_id}/resources/preview",
            post(routes_resources::preview),
        )
        .route(
            "/api/v2/projects/{project_id}/resources/install",
            post(routes_resources::install),
        )
        .route(
            "/api/v2/resources/{resource_id}/update",
            post(routes_resources::update),
        )
        .route(
            "/api/v2/resources/{resource_id}/remove",
            post(routes_resources::remove),
        )
        .route(
            "/api/v2/projects/{project_id}/previews",
            post(routes_previews::start),
        )
        .route(
            "/api/v2/previews/{preview_id}",
            get(routes_previews::status),
        )
        .route(
            "/api/v2/previews/{preview_id}/stop",
            post(routes_previews::stop),
        )
        .route(
            "/api/v2/previews/{preview_id}/screenshots",
            post(routes_previews::screenshot),
        )
        .route(
            "/api/v2/previews/{preview_id}/annotations",
            post(routes_previews::annotation),
        )
        .route(
            "/api/v2/projects/{project_id}/terminals",
            post(routes_terminals::create),
        )
        .route(
            "/api/v2/terminals/{terminal_id}/input-lease",
            post(routes_terminals::acquire_lease).delete(routes_terminals::release_lease),
        )
        .route(
            "/api/v2/terminals/{terminal_id}/input",
            post(routes_terminals::input),
        )
        .route(
            "/api/v2/terminals/{terminal_id}/resize",
            post(routes_terminals::resize),
        )
        .route(
            "/api/v2/terminals/{terminal_id}/output",
            get(routes_terminals::output),
        )
        .route(
            "/api/v2/terminals/{terminal_id}/close",
            post(routes_terminals::close),
        )
        .layer(tower_http::limit::RequestBodyLimitLayer::new(
            MAX_REQUEST_BYTES,
        ))
        .layer(middleware::from_fn(request_context))
        .layer(middleware::from_fn(api_authentication))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(security.allowed_origins.clone())
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                    header::HeaderName::from_static("x-request-id"),
                ]),
        )
        .layer(axum::Extension(security))
        .with_state(state)
}

pub fn router_with_browser(
    state: ServerState,
    mut security: ApiSecurity,
    config: BrowserConfig,
) -> Router {
    if let Some(origin) = config.allowed_origin.clone() {
        security.allowed_origins = vec![origin];
    }
    let mut router = router_with_security(state, security);
    if !config.enabled {
        return router;
    }
    let root = Arc::new(
        config
            .asset_root
            .expect("browser asset_root is required when enabled")
            .canonicalize()
            .expect("browser asset_root must exist"),
    );
    router = router
        .route("/api/v2/browser/bootstrap", get(browser::bootstrap))
        .route("/", get(browser::index))
        .route("/{*path}", get(browser::asset))
        .layer(Extension(root));
    router
}

async fn api_authentication(
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let security = request.extensions().get::<ApiSecurity>();
    let origin_allowed = security.is_none_or(|security| security.allowed_origins.is_empty())
        || request.headers().get(header::ORIGIN).is_none_or(|origin| {
            security.is_some_and(|security| {
                security
                    .allowed_origins
                    .iter()
                    .any(|allowed| allowed == origin)
            })
        });
    if !origin_allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "origin_not_allowed",
                "message": "request origin is not allowed"
            })),
        )
            .into_response();
    }
    let host_allowed = security.is_none_or(|security| {
        security.allowed_origins.is_empty()
            || request.headers().get(header::HOST).is_none_or(|host| {
                security.allowed_origins.iter().any(|origin| {
                    origin
                        .to_str()
                        .ok()
                        .and_then(|value| {
                            value
                                .strip_prefix("http://")
                                .or_else(|| value.strip_prefix("https://"))
                        })
                        .is_some_and(|authority| authority == host.to_str().unwrap_or_default())
                })
            })
    });
    if !host_allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "host_not_allowed",
                "message": "request host is not allowed"
            })),
        )
            .into_response();
    }
    if request.method() != Method::GET
        && request.method() != Method::HEAD
        && request.method() != Method::OPTIONS
        && request.headers().get(header::ORIGIN).is_some()
        && request.headers().get("x-csrf-token").is_none()
    {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "csrf_required",
                "message": "a CSRF token is required for browser mutations"
            })),
        )
            .into_response();
    }
    let required_token = security.and_then(|security| security.bearer_token.as_deref());
    let authorized = request.method() == Method::OPTIONS
        || required_token.is_none_or(|expected| {
            request
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .is_some_and(|provided| {
                    constant_time_equal(provided.as_bytes(), expected.as_bytes())
                })
        });
    if authorized {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            Json(serde_json::json!({
                "code": "authentication_required",
                "message": "a valid API bearer token is required"
            })),
        )
            .into_response()
    }
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    for index in 0..left.len().max(right.len()) {
        difference |= left
            .get(index)
            .copied()
            .unwrap_or_default()
            .abs_diff(right.get(index).copied().unwrap_or_default()) as usize;
    }
    difference == 0
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

async fn list_models() -> Json<Vec<ModelOption>> {
    Json(vec![
        ModelOption {
            provider: "github-copilot".into(),
            id: "gpt-4o".into(),
            display_name: "GPT-4o".into(),
            context_window: Some(128_000),
        },
        ModelOption {
            provider: "github-copilot".into(),
            id: "gpt-4o-mini".into(),
            display_name: "GPT-4o mini".into(),
            context_window: Some(128_000),
        },
        ModelOption {
            provider: "github-copilot".into(),
            id: "claude-3.5-sonnet".into(),
            display_name: "Claude 3.5 Sonnet".into(),
            context_window: Some(200_000),
        },
        ModelOption {
            provider: "github-copilot".into(),
            id: "o1".into(),
            display_name: "o1".into(),
            context_window: Some(200_000),
        },
    ])
}

async fn list_projects(
    State(state): State<ServerState>,
    Extension(_request_id): Extension<RequestId>,
) -> ApiResult<Json<Vec<Project>>> {
    Ok(Json(
        state.store.list_projects().await.map_err(storage_error)?,
    ))
}

async fn create_project(
    State(state): State<ServerState>,
    Extension(_request_id): Extension<RequestId>,
    Json(request): Json<CreateProjectRequest>,
) -> ApiResult<(StatusCode, Json<Project>)> {
    let root = std::path::PathBuf::from(&request.root)
        .canonicalize()
        .map_err(|io_error| {
            error(
                StatusCode::BAD_REQUEST,
                "invalid_project_root",
                &io_error.to_string(),
            )
        })?;
    if !root.is_dir() {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "invalid_project_root",
            "project root is not a directory",
        ));
    }
    let now = Utc::now();
    let project = Project {
        id: ProjectId::new(),
        root: root.to_string_lossy().into_owned(),
        name: request.name.unwrap_or_else(|| {
            root.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("project")
                .to_owned()
        }),
        created_at: now,
        updated_at: now,
    };
    let project = state
        .store
        .create_project(project)
        .await
        .map_err(storage_error)?;
    Ok((StatusCode::CREATED, Json(project)))
}

async fn create_session(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
    Json(request): Json<CreateSessionRequest>,
) -> ApiResult<(StatusCode, Json<Session>)> {
    let project_id: ProjectId = project_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_project_id",
            "project_id is invalid",
        )
    })?;
    let project = state
        .store
        .get_project(project_id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "project_not_found",
                "project was not found",
            )
        })?;
    if request.project_id != project.id {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "project_id_mismatch",
            "path and body project IDs differ",
        ));
    }
    let agent = request.agent.unwrap_or_else(|| AgentName("build".into()));
    if !matches!(
        agent.0.as_str(),
        "boss" | "build" | "plan" | "review" | "test"
    ) {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "unknown_agent",
            "agent profile is not supported",
        ));
    }
    let now = Utc::now();
    let session = Session {
        id: SessionId::new(),
        project_id,
        title: request
            .title
            .unwrap_or_else(|| format!("Session {}", now.format("%Y-%m-%d %H:%M"))),
        agent,
        model: request.model.unwrap_or_else(|| ModelRef {
            provider: ProviderName("github-copilot".into()),
            model: ModelName("default".into()),
        }),
        status: SessionStatus::Idle,
        created_at: now,
        updated_at: now,
    };
    let session = state
        .store
        .create_session(session)
        .await
        .map_err(storage_error)?;
    Ok((StatusCode::CREATED, Json(session)))
}

async fn list_sessions(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<Vec<Session>>> {
    let project_id: ProjectId = project_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_project_id",
            "project_id is invalid",
        )
    })?;
    if state
        .store
        .get_project(project_id)
        .await
        .map_err(storage_error)?
        .is_none()
    {
        return Err(error(
            StatusCode::NOT_FOUND,
            "project_not_found",
            "project was not found",
        ));
    }
    Ok(Json(
        state
            .store
            .list_sessions(project_id)
            .await
            .map_err(storage_error)?,
    ))
}

async fn get_session(
    State(state): State<ServerState>,
    Path(session_id): Path<String>,
) -> ApiResult<Json<Session>> {
    let session_id: SessionId = session_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session_id is invalid",
        )
    })?;
    state
        .store
        .get_session(session_id)
        .await
        .map_err(storage_error)?
        .map(Json)
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "session_not_found",
                "session was not found",
            )
        })
}

async fn update_session(
    State(state): State<ServerState>,
    Path(session_id): Path<String>,
    Json(request): Json<UpdateSessionRequest>,
) -> ApiResult<Json<Session>> {
    let session_id: SessionId = session_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session_id is invalid",
        )
    })?;
    let mut session = state
        .store
        .get_session(session_id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "session_not_found",
                "session was not found",
            )
        })?;
    if let Some(title) = request.title {
        session.title = title;
    }
    if let Some(agent) = request.agent {
        if !matches!(
            agent.0.as_str(),
            "boss" | "build" | "plan" | "review" | "test"
        ) {
            return Err(error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "unknown_agent",
                "agent profile is not supported",
            ));
        }
        session.agent = agent;
    }
    if let Some(model) = request.model {
        session.model = model;
    }
    session.updated_at = Utc::now();
    state
        .store
        .update_session(&session)
        .await
        .map_err(storage_error)?;
    Ok(Json(session))
}

async fn get_prompt_status(
    State(state): State<ServerState>,
    Path(prompt_id): Path<String>,
) -> ApiResult<Json<PromptStatusResponse>> {
    let prompt_id = prompt_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_prompt_id",
            "prompt_id is invalid",
        )
    })?;
    let (session_id, status) = state
        .store
        .get_prompt_status(prompt_id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "prompt_not_found",
                "prompt was not found",
            )
        })?;
    Ok(Json(PromptStatusResponse {
        prompt_id,
        session_id,
        status,
    }))
}

async fn prompt(
    State(state): State<ServerState>,
    Extension(_request_id): Extension<RequestId>,
    Path(path_session_id): Path<String>,
    Json(request): Json<PromptRequest>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let session_id: SessionId = path_session_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session_id is invalid",
        )
    })?;
    if session_id != request.session_id {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "session_id_mismatch",
            "path and body session IDs differ",
        ));
    }
    if request.prompt.trim().is_empty() {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "empty_prompt",
            "prompt must not be empty",
        ));
    }
    if request.prompt.len() > MAX_PROMPT_BYTES {
        return Err(error(
            StatusCode::PAYLOAD_TOO_LARGE,
            "prompt_too_large",
            "prompt exceeds the maximum size",
        ));
    }
    let session = state
        .store
        .get_session(session_id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "session_not_found",
                "session was not found",
            )
        })?;
    let project = state
        .store
        .get_project(session.project_id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "project_not_found",
                "project was not found",
            )
        })?;
    let cancellation = state.executions.start(session_id).await.ok_or_else(|| {
        error(
            StatusCode::CONFLICT,
            "session_busy",
            "session already has an active execution",
        )
    })?;
    let execution_cancellation = cancellation.clone();
    let shutdown = state.executions.shutdown.clone();
    tokio::spawn(async move {
        shutdown.cancelled().await;
        execution_cancellation.cancel();
    });
    let prompt_id = match state.store.admit_prompt(session_id, &request.prompt).await {
        Ok(prompt_id) => prompt_id,
        Err(error) => {
            state.executions.finish(session_id).await;
            return Err(storage_error(error));
        }
    };
    state
        .store
        .set_session_status(session_id, SessionStatus::Running)
        .await
        .map_err(storage_error)?;
    state
        .store
        .append_event(
            session_id,
            &devfoundry_schema::Event::SessionStatus {
                session_id,
                status: SessionStatus::Running,
            },
        )
        .await
        .map_err(storage_error)?;
    let store = state.store.clone();
    let runner = state.runner.clone();
    let executions = state.executions.clone();
    tokio::spawn(async move {
        let _ = store.mark_prompt_running(prompt_id).await;
        let interrupted_token = cancellation.clone();
        let result = runner
            .run(session, request.prompt, project.root.into(), cancellation)
            .await;
        let interrupted = interrupted_token.is_cancelled();
        let status = if result.is_ok() {
            SessionStatus::Idle
        } else {
            SessionStatus::Error
        };
        let prompt_status = if result.is_ok() {
            "completed"
        } else if interrupted {
            "interrupted"
        } else {
            "failed"
        };
        let _ = store.settle_prompt(prompt_id, prompt_status).await;
        let _ = store.set_session_status(session_id, status).await;
        let _ = store
            .append_event(
                session_id,
                &devfoundry_schema::Event::SessionStatus { session_id, status },
            )
            .await;
        executions.finish(session_id).await;
    });
    Ok((
        StatusCode::ACCEPTED,
        Json(
            serde_json::json!({ "session_id": session_id, "prompt_id": prompt_id, "accepted": true }),
        ),
    ))
}

async fn interrupt(
    State(state): State<ServerState>,
    Path(path_session_id): Path<String>,
) -> ApiResult<StatusCode> {
    let session_id: SessionId = path_session_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session_id is invalid",
        )
    })?;
    if state
        .store
        .get_session(session_id)
        .await
        .map_err(storage_error)?
        .is_none()
    {
        return Err(error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "session was not found",
        ));
    }
    state.executions.cancel(session_id).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_permissions(
    State(state): State<ServerState>,
    Path(session_id): Path<String>,
) -> ApiResult<Json<Vec<PermissionRequest>>> {
    let session_id: SessionId = session_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session_id is invalid",
        )
    })?;
    if state
        .store
        .get_session(session_id)
        .await
        .map_err(storage_error)?
        .is_none()
    {
        return Err(error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "session was not found",
        ));
    }
    Ok(Json(
        state
            .store
            .list_pending_permissions(session_id)
            .await
            .map_err(storage_error)?,
    ))
}

#[derive(serde::Deserialize)]
struct PermissionResolution {
    allowed: bool,
}

async fn resolve_permission(
    State(state): State<ServerState>,
    Path(permission_id): Path<String>,
    Json(body): Json<PermissionResolution>,
) -> ApiResult<StatusCode> {
    let permission_id = permission_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_permission_id",
            "permission ID is invalid",
        )
    })?;
    if state
        .runner
        .resolve_permission(permission_id, body.allowed)
        .await
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "permission_error",
                "permission resolution failed",
            )
        })?
    {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(error(
            StatusCode::NOT_FOUND,
            "permission_not_found",
            "permission request is not pending",
        ))
    }
}

fn error(status: StatusCode, code: &str, message: &str) -> ApiError {
    ApiError {
        status,
        response: ErrorResponse {
            code: code.into(),
            message: message.into(),
            request_id: ulid::Ulid::new().to_string(),
            retryable: status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS,
            details: std::collections::BTreeMap::new(),
        },
    }
}

impl ApiError {
    pub(crate) fn bad_request(code: &str, message: &str) -> Self {
        error(StatusCode::BAD_REQUEST, code, message)
    }

    pub(crate) fn not_found(code: &str, message: &str) -> Self {
        error(StatusCode::NOT_FOUND, code, message)
    }

    pub(crate) fn conflict(code: &str, message: &str) -> Self {
        error(StatusCode::CONFLICT, code, message)
    }
}

fn storage_error(storage_error: devfoundry_storage::StorageError) -> ApiError {
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "storage_error",
        &storage_error.to_string(),
    )
}

#[derive(Debug, Deserialize)]
struct EventQuery {
    after: Option<u64>,
}

async fn events(
    State(state): State<ServerState>,
    Path(session_id): Path<String>,
    Query(query): Query<EventQuery>,
) -> ApiResult<Sse<impl Stream<Item = Result<SseEvent, Infallible>>>> {
    let session_id: SessionId = session_id.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session_id is invalid",
        )
    })?;
    if state
        .store
        .get_session(session_id)
        .await
        .map_err(storage_error)?
        .is_none()
    {
        return Err(error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "session was not found",
        ));
    }
    let mut live = state.store.subscribe_events();
    let events = state
        .store
        .list_events_after(
            session_id,
            query.after.map(devfoundry_schema::EventSequence),
        )
        .await
        .map_err(storage_error)?;
    let events = if events.len() > MAX_REPLAY_EVENTS {
        if query.after.is_none() {
            events
                .into_iter()
                .rev()
                .take(MAX_REPLAY_EVENTS)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        } else {
            return Err(error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "event_replay_too_large",
                "event replay exceeds the maximum size",
            ));
        }
    } else {
        events
    };
    let mut last_sent = events
        .last()
        .map_or(query.after.unwrap_or(0), |(sequence, _)| sequence.0);
    let stream = async_stream::stream! {
        for (sequence, event) in events {
            let payload = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());
            yield Ok(SseEvent::default().id(sequence.0.to_string()).event("message").data(payload));
            last_sent = sequence.0;
        }
        loop {
            match live.recv().await {
                Ok(published) if published.session_id == session_id && published.sequence.0 > last_sent => {
                    let payload = serde_json::to_string(&published.event).unwrap_or_else(|_| "{}".into());
                    yield Ok(SseEvent::default().id(published.sequence.0.to_string()).event("message").data(payload));
                    last_sent = published.sequence.0;
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => break,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
