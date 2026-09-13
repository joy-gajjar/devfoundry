use crate::{ApiResult, ServerState, error, storage_error};
use async_trait::async_trait;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use devfoundry_core::{WorkerExecutionInput, WorkerFailure, WorkerHost, WorkerWorktree};
use devfoundry_schema::{AttemptId, Revision, TaskId};
use devfoundry_storage::{
    ProjectRepository, SchedulerRepository, SessionRepository, TaskRepository, WorkerAttemptStatus,
};
use devfoundry_tools::{WorktreeManager, WorktreeRequest};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Deserialize)]
pub(crate) struct AssignRequest {
    pub expected_revision: Revision,
    pub owner: String,
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AssignmentResponse {
    lease_id: String,
    task_id: TaskId,
    revision: Revision,
    status: &'static str,
    execution: &'static str,
}

struct ServerWorktreeAdapter {
    manager: Arc<WorktreeManager>,
    worker_id: String,
}

#[async_trait]
impl WorkerWorktree for ServerWorktreeAdapter {
    async fn prepare(&self, input: &WorkerExecutionInput) -> Result<PathBuf, WorkerFailure> {
        let allocated = self
            .manager
            .allocate(
                WorktreeRequest::new(self.worker_id.clone(), "HEAD"),
                CancellationToken::new(),
            )
            .await
            .map_err(|error| WorkerFailure::Execution(error.to_string()))?;
        if allocated.metadata.base_commit.is_empty() {
            return Err(WorkerFailure::Execution(
                "worktree base commit is empty".into(),
            ));
        }
        if input.task_revision.0 == 0 {
            return Err(WorkerFailure::Execution("task revision is invalid".into()));
        }
        Ok(allocated.path)
    }
}

pub(crate) async fn assign(
    State(state): State<ServerState>,
    Path(task_id): Path<String>,
    Json(request): Json<AssignRequest>,
) -> ApiResult<(StatusCode, Json<AssignmentResponse>)> {
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
    if request.prompt.trim().is_empty() || request.prompt.len() > 256 * 1024 {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_prompt",
            "worker prompt is empty or too large",
        ));
    }
    let task = state
        .store
        .get_task(task_id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "task_not_found",
                "task was not found",
            )
        })?;
    if task.revision != request.expected_revision {
        return Err(error(
            StatusCode::CONFLICT,
            "task_conflict",
            "task revision conflict",
        ));
    }
    let session_id = task.session_id.ok_or_else(|| {
        error(
            StatusCode::CONFLICT,
            "worker_session_required",
            "task has no assigned session",
        )
    })?;
    let session = state
        .store
        .get_session(session_id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "session_not_found",
                "worker session was not found",
            )
        })?;
    if session.model.provider.0 != "github-copilot" {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "unsupported_worker_provider",
            "worker execution requires GitHub Copilot",
        ));
    }
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
    let root = PathBuf::from(project.root);
    if !root.join(".git").exists() {
        return Err(error(
            StatusCode::CONFLICT,
            "worker_worktree_required",
            "worker execution requires a Git project",
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
    let worktree_root = root
        .parent()
        .ok_or_else(|| {
            error(
                StatusCode::CONFLICT,
                "worker_worktree_required",
                "project has no worktree parent",
            )
        })?
        .join(".devfoundry-worktrees");
    let manager = WorktreeManager::new(root, worktree_root, state.preview_permissions.clone())
        .map_err(|worktree_error| {
            error(
                StatusCode::CONFLICT,
                "worker_worktree_required",
                &worktree_error.to_string(),
            )
        })?;
    let lease_id = lease.id.clone();
    let attempt_id = AttemptId::new();
    let worker_id = request.owner.clone();
    let adapter = ServerWorktreeAdapter {
        manager: Arc::new(manager),
        worker_id,
    };
    let input = WorkerExecutionInput {
        task_id,
        task_revision: request.expected_revision,
        session,
        prompt: request.prompt,
        root: PathBuf::from("."),
        idempotency_key: format!("worker:{task_id}:{lease_id}"),
        lease_id: lease_id.clone(),
        attempt_id,
    };
    let runner = state.runner.clone();
    let store = state.store.clone();
    tokio::spawn(async move {
        let host = WorkerHost::new(store);
        let _ = host
            .execute_admitted(&runner, &adapter, input, "github-copilot")
            .await;
    });
    Ok((
        StatusCode::ACCEPTED,
        Json(AssignmentResponse {
            lease_id: lease.id,
            task_id: lease.task_id,
            revision: lease.task_revision,
            status: "claimed",
            execution: "started",
        }),
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
