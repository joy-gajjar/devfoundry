use chrono::Utc;
use devfoundry_schema::{Project, ProjectId, Task, TaskStatus};
use devfoundry_storage::{ProjectRepository, SqliteStore, TaskRepository, TaskUpdate};

async fn store() -> (SqliteStore, tempfile::TempDir, ProjectId) {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("w10.db");
    let store = SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
        .await
        .unwrap();
    let project = Project {
        id: ProjectId::new(),
        root: directory.path().to_string_lossy().into_owned(),
        name: "w10".into(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    store.create_project(project.clone()).await.unwrap();
    (store, directory, project.id)
}

#[tokio::test]
async fn task_dependencies_reject_indirect_cycles_and_preserve_revision() {
    let (store, _directory, project_id) = store().await;
    let a = Task::new_for_project(project_id, "A", "first");
    let b = Task::new_for_project(project_id, "B", "second");
    let c = Task::new_for_project(project_id, "C", "third");
    store.create_task(a.clone()).await.unwrap();
    store.create_task(b.clone()).await.unwrap();
    store.create_task(c.clone()).await.unwrap();
    store
        .update_task(a.id, TaskUpdate::dependencies(vec![b.id]), 0)
        .await
        .unwrap();
    store
        .update_task(b.id, TaskUpdate::dependencies(vec![c.id]), 0)
        .await
        .unwrap();

    let error = store
        .update_task(c.id, TaskUpdate::dependencies(vec![a.id]), 0)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("cycle"));
    assert_eq!(store.get_task(c.id).await.unwrap().unwrap().revision.0, 0);
}

#[tokio::test]
async fn stale_task_revision_is_rejected_without_mutation() {
    let (store, _directory, project_id) = store().await;
    let task = Task::new_for_project(project_id, "A", "first");
    store.create_task(task.clone()).await.unwrap();
    store
        .update_task(task.id, TaskUpdate::status(TaskStatus::Ready), 0)
        .await
        .unwrap();
    let error = store
        .update_task(task.id, TaskUpdate::title("changed"), 0)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("revision"));
    assert_eq!(store.get_task(task.id).await.unwrap().unwrap().title, "A");
}
