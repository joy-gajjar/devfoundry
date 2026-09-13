use chrono::Utc;
use devfoundry_schema::{Project, ProjectId, Revision, Task, TaskStatus};
use devfoundry_storage::{
    NotificationBinding, NotificationOutbox, NotificationRepository, ProjectRepository,
    SqliteStore, TaskRepository,
};

async fn store() -> SqliteStore {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("notifications.db");
    std::mem::forget(directory);
    SqliteStore::connect_path(path).await.unwrap()
}

#[tokio::test]
async fn notifications_are_disabled_until_explicit_pairing_and_outbox_is_deduplicated() {
    let store = store().await;
    let project_id = ProjectId::new();
    let now = Utc::now();
    store
        .create_project(Project {
            id: project_id,
            root: "/tmp/w25-fixture".into(),
            name: "w25".into(),
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap();
    let mut task = Task::new_for_project(project_id, "notify", "bounded task");
    task.status = TaskStatus::Blocked;
    store.create_task(task.clone()).await.unwrap();

    assert!(
        store
            .notification_status(project_id)
            .await
            .unwrap()
            .is_disabled()
    );

    let binding = NotificationBinding::pair(project_id, "user-1", "chat-1", "nonce-1", 7);
    store.save_notification_binding(&binding).await.unwrap();
    let outbox =
        NotificationOutbox::sanitized(task.id, task.revision, "blocked", "https://local/tasks/1")
            .for_project(project_id);
    assert!(outbox.payload().len() < 2048);
    assert!(store.enqueue_notification(outbox.clone()).await.unwrap());
    assert!(!store.enqueue_notification(outbox).await.unwrap());
}

#[tokio::test]
async fn notification_pairing_rejects_wrong_chat_replay_and_stale_revision() {
    let store = store().await;
    let project_id = ProjectId::new();
    let now = Utc::now();
    store
        .create_project(Project {
            id: project_id,
            root: "/tmp/w25-pairing-fixture".into(),
            name: "w25-pairing".into(),
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap();
    let binding = NotificationBinding::pair(project_id, "user-1", "chat-1", "nonce-1", 7);
    store.save_notification_binding(&binding).await.unwrap();

    assert!(
        store
            .validate_notification_reply(project_id, "user-1", "wrong-chat", "nonce-1", Revision(7))
            .await
            .is_err()
    );
    assert!(
        store
            .validate_notification_reply(project_id, "user-1", "chat-1", "nonce-1", Revision(6))
            .await
            .is_err()
    );
    store
        .consume_notification_nonce(project_id, "nonce-1")
        .await
        .unwrap();
    assert!(
        store
            .validate_notification_reply(project_id, "user-1", "chat-1", "nonce-1", Revision(7))
            .await
            .is_err()
    );
}
