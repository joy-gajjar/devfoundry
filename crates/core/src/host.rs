use crate::{CoreError, EvidenceBundle, WorkerBrief, WorkerFailure};
use devfoundry_schema::AttemptId;
use devfoundry_storage::{SchedulerRepository, WorkerAttempt};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Supervised boundary for durable workers. Actual provider/tool execution is
/// intentionally admitted only after the durable attempt has been started.
pub struct WorkerHost<R> {
    repository: Arc<R>,
    cancellation: CancellationToken,
}

impl<R: SchedulerRepository> WorkerHost<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            cancellation: CancellationToken::new(),
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
}
