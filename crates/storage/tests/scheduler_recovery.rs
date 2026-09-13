use chrono::Utc;
use devfoundry_schema::{Project, ProjectId, Revision, Task, TaskStatus};
use devfoundry_storage::{
    EvidenceRecord, ProjectRepository, SchedulerRepository, SqliteStore, TaskRepository,
    WorkerAttemptStatus, WorkerLeaseStatus,
};

async fn store() -> SqliteStore {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("scheduler.db");
    std::mem::forget(directory);
    SqliteStore::connect_path(path).await.unwrap()
}

async fn task(store: &SqliteStore) -> Task {
    let project_id = ProjectId::new();
    let now = Utc::now();
    store
        .create_project(Project {
            id: project_id,
            root: "/tmp/w19-fixture".into(),
            name: "w19".into(),
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap();
    let mut task = Task::new_for_project(project_id, "worker", "bounded work");
    task.status = TaskStatus::Ready;
    store.create_task(task.clone()).await.unwrap();
    task
}

#[tokio::test]
async fn duplicate_lease_and_stale_revision_are_conflicts_without_mutation() {
    let store = store().await;
    let task = task(&store).await;

    let first = store
        .claim_task(task.id, task.revision, "host-a")
        .await
        .unwrap();
    assert_eq!(first.status, WorkerLeaseStatus::Claimed);
    assert!(
        store
            .claim_task(task.id, task.revision, "host-b")
            .await
            .is_err()
    );
    assert!(
        store
            .claim_task(task.id, Revision(task.revision.0 + 1), "host-c")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn attempt_is_idempotent_and_settlement_persists_bounded_evidence() {
    let store = store().await;
    let task = task(&store).await;
    let lease = store
        .claim_task(task.id, task.revision, "host-a")
        .await
        .unwrap();
    let attempt_id = devfoundry_schema::AttemptId::new();
    let first = store.begin_attempt(&lease.id, attempt_id).await.unwrap();
    let second = store.begin_attempt(&lease.id, attempt_id).await.unwrap();
    assert_eq!(first.id, second.id);

    let evidence = EvidenceRecord::new(
        task.id,
        task.revision,
        "base",
        "head",
        "digest",
        vec!["cargo test".into()],
    );
    let settled = store
        .settle_attempt(attempt_id, WorkerAttemptStatus::Completed, Some(&evidence))
        .await
        .unwrap();
    assert_eq!(settled.status, WorkerAttemptStatus::Completed);
    assert_eq!(store.list_evidence(attempt_id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn recovery_marks_started_attempt_unknown_without_replay() {
    let store = store().await;
    let task = task(&store).await;
    let lease = store
        .claim_task(task.id, task.revision, "host-a")
        .await
        .unwrap();
    let attempt_id = devfoundry_schema::AttemptId::new();
    store.begin_attempt(&lease.id, attempt_id).await.unwrap();

    let recovered = store.recover_orphaned_workers("host-b").await.unwrap();
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].status, WorkerAttemptStatus::OutcomeUnknown);
    assert!(
        store
            .list_recoverable_workers()
            .await
            .unwrap()
            .iter()
            .any(|attempt| attempt.id == attempt_id)
    );
}
