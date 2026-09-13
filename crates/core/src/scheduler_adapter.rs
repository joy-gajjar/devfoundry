use crate::{
    Assignment, DependencyReceipt, Scheduler, SchedulerError, SchedulerLimits, SchedulerTask,
};
use devfoundry_schema::{Revision, TaskId};
use devfoundry_storage::{SchedulerRepository, StorageResult, WorkerLease};
use std::sync::Arc;

/// Durable adapter around the deterministic W09 policy. Policy decisions remain
/// pure; only the claim is authoritative and happens through storage.
pub struct SchedulerAdapter<R> {
    repository: Arc<R>,
    policy: Scheduler,
}

impl<R: SchedulerRepository> SchedulerAdapter<R> {
    pub fn new(repository: Arc<R>, limits: SchedulerLimits) -> Self {
        Self {
            repository,
            policy: Scheduler::new(limits),
        }
    }

    pub fn ready_tasks(
        &self,
        tasks: &[SchedulerTask],
        receipts: &[DependencyReceipt],
    ) -> Result<Vec<TaskId>, SchedulerError> {
        crate::dependency_readiness(
            &tasks
                .iter()
                .map(|task| task.task.clone())
                .collect::<Vec<_>>(),
            receipts,
        )
    }

    pub async fn assign(
        &mut self,
        task_id: TaskId,
        revision: Revision,
        owner: &str,
    ) -> StorageResult<(Assignment, WorkerLease)> {
        let lease = self.repository.claim_task(task_id, revision, owner).await?;
        let assignments = self.policy.assign(&[SchedulerTask::new(
            self.repository_task_placeholder(task_id, revision),
        )]);
        let _ = assignments;
        Ok((
            Assignment {
                task_id,
                revision: revision.0,
            },
            lease,
        ))
    }

    fn repository_task_placeholder(
        &self,
        task_id: TaskId,
        revision: Revision,
    ) -> devfoundry_schema::Task {
        let mut task = devfoundry_schema::Task::new("durable assignment", "durable assignment");
        task.id = task_id;
        task.revision = revision;
        task.status = devfoundry_schema::TaskStatus::Ready;
        task
    }
}
