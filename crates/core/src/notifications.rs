use devfoundry_schema::{ProjectId, Revision};
use devfoundry_storage::{
    NotificationBinding, NotificationRepository, NotificationStatus, SqliteStore, StorageResult,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct NotificationService {
    store: Arc<SqliteStore>,
}

impl NotificationService {
    pub fn new(store: Arc<SqliteStore>) -> Self {
        Self { store }
    }

    pub async fn status(&self, project_id: ProjectId) -> StorageResult<NotificationStatus> {
        self.store.notification_status(project_id).await
    }

    pub async fn pair(&self, binding: NotificationBinding) -> StorageResult<()> {
        self.store.save_notification_binding(&binding).await
    }

    pub async fn revoke(&self, project_id: ProjectId) -> StorageResult<()> {
        self.store.revoke_notification(project_id).await
    }

    pub async fn validate_reply(
        &self,
        project_id: ProjectId,
        user_id: &str,
        chat_id: &str,
        nonce: &str,
        revision: Revision,
    ) -> StorageResult<()> {
        self.store
            .validate_notification_reply(project_id, user_id, chat_id, nonce, revision)
            .await
    }
}
