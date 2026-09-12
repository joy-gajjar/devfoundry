use crate::{ApiResult, ServerState, error, storage_error};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, State},
};
use devfoundry_core::TaskService;
use devfoundry_protocol::{
    CreateTaskRequest, DocumentListResponse, RoadmapResponse, TaskResponse, UpdateTaskRequest,
};
use devfoundry_schema::{ProjectId, TaskId};
use devfoundry_storage::DocumentRepository;
use devfoundry_storage::TaskUpdate;

pub(crate) async fn list_tasks(
    State(state): State<ServerState>,
    Path(workspace_id): Path<String>,
) -> ApiResult<Json<Vec<devfoundry_schema::Task>>> {
    let project_id = parse_project(&workspace_id)?;
    let tasks = TaskService::new(state.store)
        .list(project_id)
        .await
        .map_err(storage_error)?;
    Ok(Json(tasks))
}

pub(crate) async fn create_task(
    State(state): State<ServerState>,
    Path(workspace_id): Path<String>,
    Json(request): Json<CreateTaskRequest>,
) -> ApiResult<(StatusCode, Json<TaskResponse>)> {
    if request.title.trim().is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_task",
            "title is required",
        ));
    }
    let task = TaskService::new(state.store)
        .create(
            parse_project(&workspace_id)?,
            request.title,
            request.description,
        )
        .await
        .map_err(storage_error)?;
    Ok((StatusCode::CREATED, Json(TaskResponse { task })))
}

pub(crate) async fn update_task(
    State(state): State<ServerState>,
    Path(task_id): Path<String>,
    Json(request): Json<UpdateTaskRequest>,
) -> ApiResult<Json<TaskResponse>> {
    let id = task_id.parse::<TaskId>().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_task_id",
            "task id is invalid",
        )
    })?;
    let update = if let Some(title) = request.title {
        TaskUpdate::Title(title)
    } else if let Some(status) = request.status {
        TaskUpdate::Status(status)
    } else if let Some(dependencies) = request.dependencies {
        TaskUpdate::Dependencies(dependencies)
    } else {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "empty_update",
            "task update is empty",
        ));
    };
    let task = TaskService::new(state.store)
        .update(id, update, request.expected_revision.0)
        .await
        .map_err(|storage_error_value| match storage_error_value {
            devfoundry_storage::StorageError::Domain(
                devfoundry_schema::DomainError::Validation(message),
            ) if message.contains("revision") || message.contains("cycle") => {
                error(StatusCode::CONFLICT, "task_conflict", &message)
            }
            other => storage_error(other),
        })?;
    Ok(Json(TaskResponse { task }))
}

pub(crate) async fn get_task(
    State(state): State<ServerState>,
    Path(task_id): Path<String>,
) -> ApiResult<Json<TaskResponse>> {
    let id = task_id.parse::<TaskId>().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_task_id",
            "task id is invalid",
        )
    })?;
    let task = TaskService::new(state.store)
        .get(id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "task_not_found",
                "task was not found",
            )
        })?;
    Ok(Json(TaskResponse { task }))
}

pub(crate) async fn export_roadmap(
    State(state): State<ServerState>,
    Path(workspace_id): Path<String>,
) -> ApiResult<Json<RoadmapResponse>> {
    let markdown = TaskService::new(state.store)
        .export_roadmap(parse_project(&workspace_id)?)
        .await
        .map_err(storage_error)?;
    Ok(Json(RoadmapResponse {
        markdown,
        revision: devfoundry_schema::Revision(1),
    }))
}

pub(crate) async fn import_roadmap(
    State(state): State<ServerState>,
    Path(workspace_id): Path<String>,
    Json(request): Json<devfoundry_protocol::ImportRoadmapRequest>,
) -> ApiResult<Json<RoadmapResponse>> {
    let project = parse_project(&workspace_id)?;
    TaskService::new(state.store)
        .import_roadmap(project, &request.markdown)
        .await
        .map_err(|storage_error_value| match storage_error_value {
            devfoundry_storage::StorageError::InvalidExport(message) => error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_roadmap",
                &message,
            ),
            devfoundry_storage::StorageError::Domain(
                devfoundry_schema::DomainError::Validation(message),
            ) => error(StatusCode::CONFLICT, "roadmap_conflict", &message),
            other => storage_error(other),
        })?;
    Ok(Json(RoadmapResponse {
        markdown: request.markdown,
        revision: devfoundry_schema::Revision(1),
    }))
}

pub(crate) async fn list_documents(
    State(state): State<ServerState>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<DocumentListResponse>> {
    let project = parse_project(&project_id)?;
    state
        .store
        .rebuild_documents(project)
        .await
        .map_err(storage_error)?;
    let documents = state
        .store
        .list_documents(project)
        .await
        .map_err(storage_error)?;
    Ok(Json(DocumentListResponse { documents }))
}

fn parse_project(value: &str) -> ApiResult<ProjectId> {
    value.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_workspace_id",
            "workspace id is invalid",
        )
    })
}
