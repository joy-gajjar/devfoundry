use async_trait::async_trait;
use devfoundry_core::{WorkerBrief, WorkerExecutionInput, WorkerHost, WorkerWorktree};
use devfoundry_llm::{LlmError, LlmEvent, LlmProvider, LlmRequest, LlmStream};
use devfoundry_schema::{
    AgentName, ContextManifest, ModelName, ModelRef, Project, ProjectId, ProviderName, Session,
    SessionId, SessionStatus, TaskId, TaskStatus,
};
use devfoundry_storage::{
    ProjectRepository, SchedulerRepository, SessionRepository, SqliteStore, TaskRepository,
};
use devfoundry_tools::ToolRegistry;
use std::{path::PathBuf, sync::Arc};
use tokio_util::sync::CancellationToken;

struct CopilotFixture;

#[async_trait]
impl LlmProvider for CopilotFixture {
    async fn stream(
        &self,
        _request: LlmRequest,
        _cancellation: CancellationToken,
    ) -> Result<LlmStream, LlmError> {
        Ok(Box::pin(futures_util::stream::iter([
            Ok(LlmEvent::TextDelta {
                text: "worker complete".into(),
            }),
            Ok(LlmEvent::Finished {
                reason: "stop".into(),
            }),
        ])))
    }
}

struct FixtureWorktree(PathBuf);

#[async_trait]
impl WorkerWorktree for FixtureWorktree {
    async fn prepare(
        &self,
        _input: &WorkerExecutionInput,
    ) -> Result<PathBuf, devfoundry_core::WorkerFailure> {
        Ok(self.0.clone())
    }
}

#[test]
fn worker_host_rejects_non_copilot_before_side_effects() {
    assert!(WorkerHost::validate_provider("anthropic").is_err());
    assert!(WorkerHost::validate_provider("github-copilot").is_ok());
}

#[test]
fn worker_brief_remains_immutable_and_bounded() {
    let brief = WorkerBrief::new(
        TaskId::new(),
        4,
        "objective",
        "base",
        vec!["crates/core".into()],
        vec!["read".into()],
        ContextManifest::new(Vec::new()),
    );
    assert_eq!(brief.task_revision, 4);
    assert_eq!(brief.owned_scope, vec!["crates/core"]);
}

#[tokio::test]
async fn admitted_worker_persists_evidence_and_enters_review() {
    let directory = tempfile::tempdir().unwrap();
    let store = Arc::new(
        SqliteStore::connect_path(directory.path().join("worker.db"))
            .await
            .unwrap(),
    );
    let now = chrono::Utc::now();
    let project_id = ProjectId::new();
    store
        .create_project(Project {
            id: project_id,
            root: directory.path().to_string_lossy().into(),
            name: "worker".into(),
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap();
    let session = Session {
        id: SessionId::new(),
        project_id,
        title: "worker".into(),
        agent: AgentName("build".into()),
        model: ModelRef {
            provider: ProviderName("github-copilot".into()),
            model: ModelName("test".into()),
        },
        status: SessionStatus::Idle,
        created_at: now,
        updated_at: now,
    };
    store.create_session(session.clone()).await.unwrap();
    let mut task = devfoundry_schema::Task::new_for_project(project_id, "worker", "run worker");
    task.status = TaskStatus::Ready;
    store.create_task(task.clone()).await.unwrap();
    let lease = store
        .claim_task(task.id, task.revision, "host")
        .await
        .unwrap();
    let attempt_id = devfoundry_schema::AttemptId::new();
    let runner = devfoundry_core::SessionRunner::new(
        store.clone(),
        Arc::new(CopilotFixture),
        Arc::new(ToolRegistry::default()),
    );
    let host = WorkerHost::new(store.clone());
    let result = host
        .execute_admitted(
            &runner,
            &FixtureWorktree(directory.path().to_path_buf()),
            WorkerExecutionInput {
                task_id: task.id,
                task_revision: task.revision,
                session,
                prompt: "run worker".into(),
                root: directory.path().to_path_buf(),
                idempotency_key: "worker-run".into(),
                lease_id: lease.id,
                attempt_id,
            },
            "github-copilot",
        )
        .await
        .unwrap();
    assert_eq!(result.task_id, task.id);
    assert_eq!(store.list_evidence(attempt_id).await.unwrap().len(), 1);
    assert_eq!(
        store.get_task(task.id).await.unwrap().unwrap().status,
        TaskStatus::Review
    );
}
