use crate::{ApiResult, ServerState, error, storage_error};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use devfoundry_schema::ProjectId;
use devfoundry_storage::NotificationRepository;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct SetupRequest {
    pub user_id: String,
    pub chat_id: String,
    pub nonce: String,
    pub revision: u64,
}

pub(crate) async fn status(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let project_id = parse_project(project_id)?;
    let status = state
        .store
        .notification_status(project_id)
        .await
        .map_err(storage_error)?;
    Ok(Json(
        serde_json::json!({"enabled": status.enabled, "paired": status.paired, "revoked": status.revoked}),
    ))
}

pub(crate) async fn setup(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
    Json(request): Json<SetupRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let project_id = parse_project(project_id)?;
    if request.user_id.trim().is_empty()
        || request.chat_id.trim().is_empty()
        || request.nonce.trim().is_empty()
        || request.revision == 0
    {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_notification_pairing",
            "notification pairing metadata is invalid",
        ));
    }
    state
        .store
        .save_notification_binding(&devfoundry_storage::NotificationBinding::pair(
            project_id,
            request.user_id,
            request.chat_id,
            request.nonce,
            request.revision,
        ))
        .await
        .map_err(storage_error)?;
    Ok(Json(
        serde_json::json!({"enabled": true, "paired": true, "revoked": false}),
    ))
}

pub(crate) async fn revoke(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .store
        .revoke_notification(parse_project(project_id)?)
        .await
        .map_err(storage_error)?;
    Ok(Json(
        serde_json::json!({"enabled": false, "paired": false, "revoked": true}),
    ))
}

fn parse_project(value: String) -> ApiResult<ProjectId> {
    value.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_project_id",
            "project id is invalid",
        )
    })
}
