use crate::{ApiResult, ServerState, error, storage_error};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use devfoundry_core::ResourceService;
use devfoundry_schema::{ProjectId, ResourceManifest};
use devfoundry_storage::ProjectRepository;
use devfoundry_tools::archive::{ArchiveEntry, ArchiveEntryKind};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub(crate) struct ResourceRequest {
    pub manifest: ResourceManifest,
    pub entries: Vec<ArchiveEntryRequest>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ArchiveEntryRequest {
    pub path: String,
    pub size: u64,
    pub kind: ArchiveEntryKind,
    pub content: Vec<u8>,
}

impl From<ArchiveEntryRequest> for ArchiveEntry {
    fn from(value: ArchiveEntryRequest) -> Self {
        Self {
            path: value.path,
            size: value.size,
            kind: value.kind,
            content: value.content,
        }
    }
}

pub(crate) async fn inspect(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
    Json(request): Json<ResourceRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let project_id = parse_project(project_id)?;
    ensure_project(&state, project_id).await?;
    let service = ResourceService::new(
        state.store.clone(),
        Arc::new(devfoundry_tools::DenyPermissions),
    );
    let entries = request
        .entries
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();
    service.inspect(&entries).map_err(|archive_error| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_resource",
            &archive_error.to_string(),
        )
    })?;
    Ok(Json(
        serde_json::json!({"id": request.manifest.id, "files": entries.iter().map(|entry| &entry.path).collect::<Vec<_>>()}),
    ))
}

pub(crate) async fn preview(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
    Json(request): Json<ResourceRequest>,
) -> ApiResult<Json<Vec<String>>> {
    let project_id = parse_project(project_id)?;
    ensure_project(&state, project_id).await?;
    let service = ResourceService::new(
        state.store.clone(),
        Arc::new(devfoundry_tools::DenyPermissions),
    );
    let entries = request
        .entries
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();
    service
        .preview(&entries)
        .map(Json)
        .map_err(|archive_error| {
            error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_resource",
                &archive_error.to_string(),
            )
        })
}

pub(crate) async fn install(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
    Json(request): Json<ResourceRequest>,
) -> ApiResult<(StatusCode, Json<devfoundry_schema::InstalledResource>)> {
    let project_id = parse_project(project_id)?;
    let project = ensure_project(&state, project_id).await?;
    let _ = project;
    let _ = request;
    Err(error(
        StatusCode::FORBIDDEN,
        "permission_required",
        "resource installation requires central permission broker authorization",
    ))
}

pub(crate) async fn update(
    State(_state): State<ServerState>,
    Path(_resource_id): Path<String>,
    Json(_request): Json<ResourceRequest>,
) -> ApiResult<StatusCode> {
    Err(error(
        StatusCode::FORBIDDEN,
        "permission_required",
        "resource update requires central permission broker authorization",
    ))
}

pub(crate) async fn remove(
    State(_state): State<ServerState>,
    Path(_resource_id): Path<String>,
) -> ApiResult<StatusCode> {
    Err(error(
        StatusCode::FORBIDDEN,
        "permission_required",
        "resource removal requires central permission broker authorization",
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

async fn ensure_project(
    state: &ServerState,
    id: ProjectId,
) -> ApiResult<devfoundry_schema::Project> {
    state
        .store
        .get_project(id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "project_not_found",
                "project was not found",
            )
        })
}
