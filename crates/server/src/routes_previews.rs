use crate::{ApiError, ApiResult, ServerState};
use axum::{
    Json,
    extract::{Path, State},
};
use devfoundry_core::{PreviewId, PreviewService};
use devfoundry_tools::ToolError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize)]
pub(crate) struct StartRequest {
    pub worktree: String,
    pub artifact_root: String,
    pub revision: String,
    pub port: Option<u16>,
}

#[derive(Deserialize)]
pub(crate) struct AnnotationRequest {
    pub revision: String,
    pub text: String,
}

#[derive(Serialize)]
pub(crate) struct PreviewResponse {
    pub id: String,
    pub state: devfoundry_core::PreviewState,
    pub url: Option<String>,
}

pub(crate) async fn start(
    State(state): State<ServerState>,
    Path(_project_id): Path<String>,
    Json(request): Json<StartRequest>,
) -> ApiResult<Json<PreviewResponse>> {
    let Some(port) = request.port.filter(|port| *port != 0) else {
        return Err(ApiError::bad_request(
            "invalid_preview_request",
            "preview fields are invalid",
        ));
    };
    if request.worktree.is_empty()
        || request.artifact_root.is_empty()
        || request.revision.is_empty()
    {
        return Err(ApiError::bad_request(
            "invalid_preview_request",
            "preview fields are invalid",
        ));
    }
    let id = PreviewId::new();
    let service = PreviewService::new(id, &request.worktree, request.revision);
    let mut previews = state.previews.lock().await;
    let process = devfoundry_tools::PreviewProcess::start(
        devfoundry_tools::PreviewLaunch {
            executable: "/bin/sh".into(),
            arguments: vec!["-c".into(), "sleep 30".into()],
            worktree: PathBuf::from(&request.worktree),
            artifact_root: PathBuf::from(&request.artifact_root),
            port,
            readiness_deadline: std::time::Duration::from_secs(5),
        },
        devfoundry_tools::ToolContext {
            root: PathBuf::from(&request.worktree),
            permissions: state.preview_permissions.clone(),
            cancellation: tokio_util::sync::CancellationToken::new(),
        },
    )
    .await
    .map_err(tool_error)?;
    previews.insert(service, process);
    let preview = previews
        .get_mut(id)
        .ok_or_else(|| ApiError::not_found("preview_not_found", "preview not found"))?;
    preview.starting().map_err(tool_error)?;
    preview
        .ready(format!("http://127.0.0.1:{port}"))
        .map_err(tool_error)?;
    response(&previews, id).await
}

pub(crate) async fn status(
    State(state): State<ServerState>,
    Path(preview_id): Path<String>,
) -> ApiResult<Json<PreviewResponse>> {
    let id = preview_id
        .parse()
        .map_err(|_| ApiError::not_found("preview_not_found", "preview not found"))?;
    let previews = state.previews.lock().await;
    response(&previews, id).await
}

pub(crate) async fn stop(
    State(state): State<ServerState>,
    Path(preview_id): Path<String>,
) -> ApiResult<Json<PreviewResponse>> {
    let id = preview_id
        .parse()
        .map_err(|_| ApiError::not_found("preview_not_found", "preview not found"))?;
    let mut previews = state.previews.lock().await;
    previews.stop(id).await.map_err(tool_error)?;
    response(&previews, id).await
}

pub(crate) async fn screenshot() -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(
        serde_json::json!({"managed": false, "message": "screenshots are not available"}),
    ))
}

pub(crate) async fn annotation(
    State(state): State<ServerState>,
    Path(preview_id): Path<String>,
    Json(request): Json<AnnotationRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if request.text.len() > 2_000 {
        return Err(ApiError::bad_request(
            "annotation_too_large",
            "annotation is too large",
        ));
    }
    let id = preview_id
        .parse()
        .map_err(|_| ApiError::not_found("preview_not_found", "preview not found"))?;
    let previews = state.previews.lock().await;
    let preview = previews
        .get(id)
        .ok_or_else(|| ApiError::not_found("preview_not_found", "preview not found"))?;
    preview
        .accept_annotation(&request.revision)
        .map_err(tool_error)?;
    Ok(Json(serde_json::json!({"accepted": true})))
}

async fn response(
    previews: &devfoundry_core::PreviewRegistry,
    id: PreviewId,
) -> ApiResult<Json<PreviewResponse>> {
    let preview = previews
        .get(id)
        .ok_or_else(|| ApiError::not_found("preview_not_found", "preview not found"))?;
    Ok(Json(PreviewResponse {
        id: id.to_string(),
        state: preview.state(),
        url: preview.url().map(str::to_owned),
    }))
}

fn tool_error(error: ToolError) -> ApiError {
    ApiError::bad_request("preview_failed", &error.to_string())
}
