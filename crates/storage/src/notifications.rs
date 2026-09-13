use crate::{SqliteStore, StorageError, StorageResult};
use async_trait::async_trait;
use devfoundry_schema::{ProjectId, Revision, TaskId};
use serde::Serialize;

const MAX_PAYLOAD_BYTES: usize = 2048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationStatus {
    pub enabled: bool,
    pub paired: bool,
    pub revoked: bool,
}

impl NotificationStatus {
    pub fn is_disabled(&self) -> bool {
        !self.enabled
    }
}

#[derive(Clone, Debug)]
pub struct NotificationBinding {
    pub project_id: ProjectId,
    pub user_id: String,
    pub chat_id: String,
    pub nonce: String,
    pub revision: Revision,
}

impl NotificationBinding {
    pub fn pair(
        project_id: ProjectId,
        user_id: impl Into<String>,
        chat_id: impl Into<String>,
        nonce: impl Into<String>,
        revision: u64,
    ) -> Self {
        Self {
            project_id,
            user_id: user_id.into(),
            chat_id: chat_id.into(),
            nonce: nonce.into(),
            revision: Revision(revision),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct NotificationPayload {
    pub task_id: TaskId,
    pub revision: Revision,
    pub status: String,
    pub deep_link: String,
}

#[derive(Clone, Debug)]
pub struct NotificationOutbox {
    pub project_id: ProjectId,
    pub task_id: TaskId,
    pub task_revision: Revision,
    payload: String,
    dedup_key: String,
}

impl NotificationOutbox {
    pub fn sanitized(task_id: TaskId, revision: Revision, status: &str, deep_link: &str) -> Self {
        let payload = serde_json::to_string(&NotificationPayload {
            task_id,
            revision,
            status: status.to_owned(),
            deep_link: deep_link.to_owned(),
        })
        .unwrap();
        Self {
            project_id: ProjectId::new(),
            task_id,
            task_revision: revision,
            dedup_key: format!("{task_id}:{}:{status}", revision.0),
            payload,
        }
    }

    pub fn for_project(mut self, project_id: ProjectId) -> Self {
        self.project_id = project_id;
        self
    }
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

#[async_trait]
pub trait NotificationRepository {
    async fn notification_status(&self, project_id: ProjectId)
    -> StorageResult<NotificationStatus>;
    async fn save_notification_binding(&self, binding: &NotificationBinding) -> StorageResult<()>;
    async fn revoke_notification(&self, project_id: ProjectId) -> StorageResult<()>;
    async fn enqueue_notification(&self, notification: NotificationOutbox) -> StorageResult<bool>;
    async fn validate_notification_reply(
        &self,
        project_id: ProjectId,
        user_id: &str,
        chat_id: &str,
        nonce: &str,
        revision: Revision,
    ) -> StorageResult<()>;
    async fn consume_notification_nonce(
        &self,
        project_id: ProjectId,
        nonce: &str,
    ) -> StorageResult<()>;
}

#[async_trait]
impl NotificationRepository for SqliteStore {
    async fn notification_status(
        &self,
        project_id: ProjectId,
    ) -> StorageResult<NotificationStatus> {
        let row = sqlx::query_as::<_, (i64, i64, i64)>(
            "SELECT enabled, revoked, 1 FROM notification_bindings WHERE project_id = ?",
        )
        .bind(project_id.to_string())
        .fetch_optional(self.pool())
        .await?;
        Ok(row
            .map(|(enabled, revoked, paired)| NotificationStatus {
                enabled: enabled != 0,
                paired: paired != 0,
                revoked: revoked != 0,
            })
            .unwrap_or(NotificationStatus {
                enabled: false,
                paired: false,
                revoked: false,
            }))
    }

    async fn save_notification_binding(&self, binding: &NotificationBinding) -> StorageResult<()> {
        if binding.user_id.len() > 128 || binding.chat_id.len() > 128 || binding.nonce.len() > 256 {
            return Err(StorageError::InvalidExport(
                "notification binding is too large".into(),
            ));
        }
        sqlx::query("INSERT INTO notification_bindings (project_id,user_id,chat_id,nonce,revision,enabled,revoked,nonce_consumed,created_at,updated_at) VALUES (?,?,?,?,?,1,0,0,datetime('now'),datetime('now')) ON CONFLICT(project_id) DO UPDATE SET user_id=excluded.user_id, chat_id=excluded.chat_id, nonce=excluded.nonce, revision=excluded.revision, enabled=1, revoked=0, nonce_consumed=0, updated_at=datetime('now')")
            .bind(binding.project_id.to_string()).bind(&binding.user_id).bind(&binding.chat_id).bind(&binding.nonce).bind(binding.revision.0 as i64).execute(self.pool()).await?;
        Ok(())
    }

    async fn enqueue_notification(&self, notification: NotificationOutbox) -> StorageResult<bool> {
        if notification.payload.len() > MAX_PAYLOAD_BYTES {
            return Err(StorageError::InvalidExport(
                "notification payload is too large".into(),
            ));
        }
        let result = sqlx::query("INSERT OR IGNORE INTO notification_outbox (id,project_id,task_id,task_revision,status,payload,dedup_key,attempts,next_attempt_at,created_at) VALUES (?,?,?,?,?,?,?,?,datetime('now'),datetime('now'))")
            .bind(ulid::Ulid::new().to_string()).bind(notification.project_id.to_string()).bind(notification.task_id.to_string()).bind(notification.task_revision.0 as i64).bind("pending").bind(notification.payload).bind(notification.dedup_key).bind(0_i64).execute(self.pool()).await?;
        Ok(result.rows_affected() == 1)
    }

    async fn revoke_notification(&self, project_id: ProjectId) -> StorageResult<()> {
        sqlx::query("UPDATE notification_bindings SET enabled=0, revoked=1, nonce_consumed=1, updated_at=datetime('now') WHERE project_id=?")
            .bind(project_id.to_string()).execute(self.pool()).await?;
        Ok(())
    }

    async fn validate_notification_reply(
        &self,
        project_id: ProjectId,
        user_id: &str,
        chat_id: &str,
        nonce: &str,
        revision: Revision,
    ) -> StorageResult<()> {
        let row = sqlx::query_as::<_, (String, String, String, i64, i64, i64)>("SELECT user_id,chat_id,nonce,revision,enabled,nonce_consumed FROM notification_bindings WHERE project_id = ?")
            .bind(project_id.to_string()).fetch_optional(self.pool()).await?.ok_or_else(|| StorageError::InvalidExport("notification binding is unavailable".into()))?;
        if row.0 != user_id
            || row.1 != chat_id
            || row.2 != nonce
            || row.3 != revision.0 as i64
            || row.4 == 0
            || row.5 != 0
        {
            return Err(StorageError::InvalidExport(
                "notification reply rejected".into(),
            ));
        }
        Ok(())
    }

    async fn consume_notification_nonce(
        &self,
        project_id: ProjectId,
        nonce: &str,
    ) -> StorageResult<()> {
        let result = sqlx::query("UPDATE notification_bindings SET nonce_consumed=1, updated_at=datetime('now') WHERE project_id=? AND nonce=? AND nonce_consumed=0")
            .bind(project_id.to_string()).bind(nonce).execute(self.pool()).await?;
        if result.rows_affected() != 1 {
            return Err(StorageError::InvalidExport(
                "notification nonce is already consumed".into(),
            ));
        }
        Ok(())
    }
}
