use crate::{ApiResult, ServerState, error, storage_error};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use devfoundry_schema::{AttemptId, Revision, TaskId};
use devfoundry_storage::{SchedulerRepository, WorkerAttemptStatus};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct AssignRequest {
    pub expected_revision: Revision,
    pub owner: String,
}

pub(crate) async fn assign(
    State(state): State<ServerState>,
    Path(task_id): Path<String>,
    Json(request): Json<AssignRequest>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let task_id = task_id.parse::<TaskId>().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_task_id",
            "task id is invalid",
        )
    })?;
    if request.owner.trim().is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_owner",
            "owner is required",
        ));
    }
    let lease = state
        .store
        .claim_task(task_id, request.expected_revision, &request.owner)
        .await
        .map_err(|e| match e {
            devfoundry_storage::StorageError::Domain(
                devfoundry_schema::DomainError::Validation(message),
            ) => error(StatusCode::CONFLICT, "task_conflict", &message),
            other => storage_error(other),
        })?;
    Ok((
        StatusCode::ACCEPTED,
        Json(
            serde_json::json!({"lease_id": lease.id, "task_id": lease.task_id, "revision": lease.task_revision, "status": "claimed"}),
        ),
    ))
}

pub(crate) async fn cancel(
    State(state): State<ServerState>,
    Path(attempt_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let attempt_id = attempt_id.parse::<AttemptId>().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_attempt_id",
            "attempt id is invalid",
        )
    })?;
    let attempts = state
        .store
        .list_recoverable_workers()
        .await
        .map_err(storage_error)?;
    if attempts.iter().any(|attempt| {
        attempt.id == attempt_id && attempt.status == WorkerAttemptStatus::OutcomeUnknown
    }) {
        return Ok(Json(
            serde_json::json!({"attempt_id": attempt_id, "status": "outcome_unknown"}),
        ));
    }
    Err(error(
        StatusCode::NOT_FOUND,
        "attempt_not_found",
        "attempt was not found",
    ))
}

pub(crate) async fn evidence(
    State(state): State<ServerState>,
    Path(attempt_id): Path<String>,
) -> ApiResult<Json<Vec<devfoundry_storage::EvidenceRecord>>> {
    let attempt_id = attempt_id.parse::<AttemptId>().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_attempt_id",
            "attempt id is invalid",
        )
    })?;
    Ok(Json(
        state
            .store
            .list_evidence(attempt_id)
            .await
            .map_err(storage_error)?,
    ))
}

pub(crate) async fn workers(
    State(state): State<ServerState>,
    Path(_workspace_id): Path<String>,
) -> ApiResult<Json<Vec<devfoundry_storage::WorkerAttempt>>> {
    Ok(Json(
        state
            .store
            .list_recoverable_workers()
            .await
            .map_err(storage_error)?,
    ))
}
