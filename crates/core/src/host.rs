use crate::{CoreError, EvidenceBundle, SessionRunner, WorkerBrief, WorkerFailure};
use async_trait::async_trait;
use devfoundry_schema::{AttemptId, Session};
use devfoundry_storage::{AdmissionInput, SchedulerRepository, WorkerAttempt, WorkerAttemptStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Supervised boundary for durable workers. Actual provider/tool execution is
/// intentionally admitted only after the durable attempt has been started.
pub struct WorkerHost<R> {
    repository: Arc<R>,
    cancellation: CancellationToken,
}

#[derive(Clone, Debug)]
pub struct WorkerExecutionInput {
    pub task_id: devfoundry_schema::TaskId,
    pub task_revision: devfoundry_schema::Revision,
    pub session: Session,
    pub prompt: String,
    pub root: PathBuf,
    pub idempotency_key: String,
    pub lease_id: String,
    pub attempt_id: AttemptId,
}

#[async_trait]
pub trait WorkerWorktree: Send + Sync {
    async fn prepare(&self, input: &WorkerExecutionInput) -> Result<PathBuf, WorkerFailure>;
}

impl<R: SchedulerRepository> WorkerHost<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self::with_cancellation(repository, CancellationToken::new())
    }

    pub fn with_cancellation(repository: Arc<R>, cancellation: CancellationToken) -> Self {
        Self {
            repository,
            cancellation,
        }
    }

    pub async fn recover_workers(&self, owner: &str) -> Result<Vec<WorkerAttempt>, CoreError> {
        Ok(self.repository.recover_orphaned_workers(owner).await?)
    }

    pub fn cancel(&self) {
        self.cancellation.cancel();
    }

    pub async fn execute_worker(
        &self,
        provider: &str,
        _attempt_id: AttemptId,
        _brief: WorkerBrief,
    ) -> Result<EvidenceBundle, WorkerFailure> {
        WorkerBrief::validate_provider(provider)?;
        Err(WorkerFailure::ExecutionUnavailable)
    }
}

impl WorkerHost<devfoundry_storage::SqliteStore> {
    pub fn validate_provider(provider: &str) -> Result<(), WorkerFailure> {
        WorkerBrief::validate_provider(provider)
    }

    pub async fn execute_admitted<W: WorkerWorktree>(
        &self,
        runner: &SessionRunner,
        worktree: &W,
        input: WorkerExecutionInput,
        provider: &str,
    ) -> Result<EvidenceBundle, WorkerFailure> {
        WorkerBrief::validate_provider(provider)?;
        let admission = self
            .repository
            .admit_run(AdmissionInput {
                session_id: input.session.id,
                idempotency_key: input.idempotency_key.clone(),
                prompt: input.prompt.clone(),
                expected_revision: 0,
            })
            .await
            .map_err(|error| WorkerFailure::Execution(error.to_string()))?;
        if matches!(
            admission.idempotency,
            devfoundry_schema::IdempotencyResult::Conflict
        ) {
            return Err(WorkerFailure::Execution("worker admission conflict".into()));
        }
        let attempt = self
            .repository
            .begin_attempt(&input.lease_id, input.attempt_id)
            .await
            .map_err(|error| WorkerFailure::Execution(error.to_string()))?;
        let root = match worktree.prepare(&input).await {
            Ok(root) => root,
            Err(error) => {
                let _ = self
                    .repository
                    .settle_attempt(attempt.id, WorkerAttemptStatus::Failed, None)
                    .await;
                return Err(error);
            }
        };
        let cancellation = self.cancellation.child_token();
        let result = runner
            .run(input.session, input.prompt, root, cancellation)
            .await;
        match result {
            Ok(message) => {
                let evidence = EvidenceBundle::new(
                    input.task_id,
                    "worker-execution",
                    "worker-execution",
                    "worker-execution",
                );
                let record = devfoundry_storage::EvidenceRecord::new(
                    input.task_id,
                    input.task_revision,
                    evidence.base_commit.clone(),
                    evidence.proposed_head.clone(),
                    evidence.diff_digest.clone(),
                    Vec::new(),
                );
                self.repository
                    .settle_attempt(attempt.id, WorkerAttemptStatus::Completed, Some(&record))
                    .await
                    .map_err(|error| WorkerFailure::Execution(error.to_string()))?;
                let _ = message;
                Ok(evidence)
            }
            Err(error) => {
                let outcome = if matches!(error, CoreError::Cancelled) {
                    WorkerAttemptStatus::Cancelled
                } else {
                    WorkerAttemptStatus::Failed
                };
                self.repository
                    .settle_attempt(attempt.id, outcome, None)
                    .await
                    .map_err(|storage_error| WorkerFailure::Execution(storage_error.to_string()))?;
                Err(WorkerFailure::Execution(error.to_string()))
            }
        }
    }
}
