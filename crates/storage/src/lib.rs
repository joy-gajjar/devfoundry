//! Persistence boundaries for projects, sessions, messages, and events.
//!
//! The repository traits are intentionally storage-engine agnostic. SQLite and
//! migrations will be added in the persistence phase without leaking a
//! connection type into the core or protocol crates.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use devfoundry_schema::{
    AttemptId, AttemptStatus, DomainError, Event, EventEnvelope, EventSequence, IdempotencyResult,
    Message, MessageId, PermissionRequest, PermissionRequestId, PermissionStatus, Project,
    ProjectId, Revision, RunId, RunStatus, Session, SessionExport, SessionId,
};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::sync::broadcast;

mod documents;
mod tasks;
pub use documents::DocumentRepository;
pub use tasks::{TaskRepository, TaskUpdate};

#[async_trait]
pub trait ContextRepository {
    async fn save_context_manifest(
        &self,
        project_id: ProjectId,
        manifest: &devfoundry_schema::ContextManifest,
    ) -> StorageResult<String>;
    async fn get_context_manifest(
        &self,
        id: &str,
    ) -> StorageResult<Option<devfoundry_schema::ContextManifest>>;
}

#[async_trait]
impl ContextRepository for SqliteStore {
    async fn save_context_manifest(
        &self,
        project_id: ProjectId,
        manifest: &devfoundry_schema::ContextManifest,
    ) -> StorageResult<String> {
        let id = ulid::Ulid::new().to_string();
        sqlx::query("INSERT INTO context_manifests (id, project_id, payload, created_at) VALUES (?, ?, ?, datetime('now'))")
            .bind(&id).bind(project_id.to_string()).bind(serde_json::to_string(manifest).map_err(|e| sqlx::Error::Protocol(e.to_string()))?)
            .execute(self.pool()).await?;
        Ok(id)
    }

    async fn get_context_manifest(
        &self,
        id: &str,
    ) -> StorageResult<Option<devfoundry_schema::ContextManifest>> {
        let payload =
            sqlx::query_scalar::<_, String>("SELECT payload FROM context_manifests WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool())
                .await?;
        payload
            .map(|value| {
                serde_json::from_str(&value)
                    .map_err(|e| StorageError::Database(sqlx::Error::Protocol(e.to_string())))
            })
            .transpose()
    }
}

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("backup I/O error: {0}")]
    BackupIo(#[from] std::io::Error),
    #[error("backup destination already exists: {0}")]
    BackupExists(PathBuf),
    #[error("invalid session export: {0}")]
    InvalidExport(String),
    #[error("session already exists: {0}")]
    ImportConflict(SessionId),
}

pub struct SqliteStore {
    pool: SqlitePool,
    events: broadcast::Sender<PublishedEvent>,
}

#[derive(Clone, Debug)]
pub struct PublishedEvent {
    pub session_id: SessionId,
    pub sequence: EventSequence,
    pub event: Event,
}

#[derive(Clone, Debug)]
pub struct AdmissionInput {
    pub session_id: SessionId,
    pub idempotency_key: String,
    pub prompt: String,
    pub expected_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionReceipt {
    pub run_id: RunId,
    pub revision: Revision,
    pub idempotency: IdempotencyResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunStatusRecord {
    pub run_id: RunId,
    pub status: RunStatus,
    pub attempt_status: Option<AttemptStatus>,
    pub revision: Revision,
}

#[derive(Debug, serde::Serialize)]
pub struct StorageDiagnostics {
    pub foreign_keys: bool,
    pub integrity_check: String,
    pub row_counts: serde_json::Map<String, serde_json::Value>,
}

impl SqliteStore {
    pub async fn admit_run(&self, input: AdmissionInput) -> StorageResult<AdmissionReceipt> {
        let mut transaction = self.pool.begin().await?;
        let existing = sqlx::query_as::<_, (String, String, i64)>(
            "SELECT id, prompt, revision FROM runs WHERE session_id = ? AND idempotency_key = ?",
        )
        .bind(input.session_id.to_string())
        .bind(&input.idempotency_key)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some((id, prompt, revision)) = existing {
            transaction.commit().await?;
            let run_id = id
                .parse()
                .map_err(|error| StorageError::Domain(DomainError::InvalidIdentifier(error)))?;
            return Ok(AdmissionReceipt {
                run_id,
                revision: Revision(revision as u64),
                idempotency: if prompt == input.prompt {
                    IdempotencyResult::Existing
                } else {
                    IdempotencyResult::Conflict
                },
            });
        }

        let run_id = RunId::new();
        let now = Utc::now();
        let status = serde_json::to_string(&RunStatus::Admitted).unwrap();
        sqlx::query(
            "INSERT INTO runs (id, session_id, idempotency_key, prompt, expected_revision, status, revision, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(run_id.to_string())
        .bind(input.session_id.to_string())
        .bind(input.idempotency_key)
        .bind(input.prompt)
        .bind(input.expected_revision as i64)
        .bind(status)
        .bind(0_i64)
        .bind(now)
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        let event = EventEnvelope {
            version: 1,
            sequence: 1,
            run_id,
            aggregate_revision: Revision(0),
            kind: "run.admitted".into(),
            payload: serde_json::json!({"session_id": input.session_id}),
        };
        sqlx::query(
            "INSERT INTO run_events (run_id, sequence, payload, created_at) VALUES (?, 1, ?, ?)",
        )
        .bind(run_id.to_string())
        .bind(serde_json::to_string(&event).unwrap())
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(AdmissionReceipt {
            run_id,
            revision: Revision(0),
            idempotency: IdempotencyResult::New,
        })
    }

    pub async fn count_runs(&self) -> StorageResult<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM runs")
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn count_run_events(&self) -> StorageResult<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM run_events")
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn append_run_event(
        &self,
        run_id: RunId,
        mut event: EventEnvelope,
    ) -> StorageResult<EventEnvelope> {
        let mut transaction = self.pool.begin().await?;
        let sequence = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT MAX(sequence) FROM run_events WHERE run_id = ?",
        )
        .bind(run_id.to_string())
        .fetch_one(&mut *transaction)
        .await?
        .unwrap_or(0)
            + 1;
        event.sequence = sequence as u64;
        event.run_id = run_id;
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO run_events (run_id, sequence, payload, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(run_id.to_string())
        .bind(sequence)
        .bind(serde_json::to_string(&event).unwrap())
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(event)
    }

    pub async fn list_run_events_after(
        &self,
        run_id: RunId,
        after: u64,
        limit: u32,
        max_bytes: usize,
    ) -> StorageResult<Vec<EventEnvelope>> {
        let rows = sqlx::query_as::<_, (i64, String)>(
            "SELECT sequence, payload FROM run_events WHERE run_id = ? AND sequence > ? ORDER BY sequence ASC LIMIT ?",
        )
        .bind(run_id.to_string())
        .bind(after as i64)
        .bind(limit.min(1000) as i64)
        .fetch_all(&self.pool)
        .await?;
        let mut bytes = 0;
        let mut events = Vec::new();
        for (_, payload) in rows {
            if bytes + payload.len() > max_bytes {
                break;
            }
            bytes += payload.len();
            events.push(serde_json::from_str(&payload).map_err(|error| {
                StorageError::Database(sqlx::Error::Protocol(error.to_string()))
            })?);
        }
        Ok(events)
    }

    pub async fn list_messages_after(
        &self,
        session_id: SessionId,
        after: Option<MessageId>,
        limit: u32,
        max_bytes: usize,
    ) -> StorageResult<Vec<(MessageId, Message)>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT id, payload FROM messages WHERE session_id = ? AND (? IS NULL OR id > ?) ORDER BY id ASC LIMIT ?",
        )
        .bind(session_id.to_string())
        .bind(after.map(|id| id.to_string()))
        .bind(after.map(|id| id.to_string()))
        .bind(limit.min(1000) as i64)
        .fetch_all(&self.pool)
        .await?;
        let mut bytes = 0;
        let mut messages = Vec::new();
        for (id, payload) in rows {
            if bytes + payload.len() > max_bytes {
                break;
            }
            bytes += payload.len();
            let message = serde_json::from_str(&payload).map_err(|error| {
                StorageError::Database(sqlx::Error::Protocol(error.to_string()))
            })?;
            let id = id
                .parse()
                .map_err(|error| StorageError::Domain(DomainError::InvalidIdentifier(error)))?;
            messages.push((id, message));
        }
        Ok(messages)
    }

    pub async fn start_attempt(&self, run_id: RunId, attempt_id: AttemptId) -> StorageResult<()> {
        let now = Utc::now();
        let status = serde_json::to_string(&AttemptStatus::Started).unwrap();
        sqlx::query("INSERT INTO attempts (id, run_id, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
            .bind(attempt_id.to_string()).bind(run_id.to_string()).bind(status).bind(now).bind(now)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn start_tool(
        &self,
        run_id: RunId,
        attempt_id: AttemptId,
        tool_id: &str,
        name: &str,
    ) -> StorageResult<()> {
        sqlx::query("INSERT INTO tool_calls (id, run_id, attempt_id, name, status, created_at, updated_at) VALUES (?, ?, ?, ?, 'running', ?, ?)")
            .bind(tool_id).bind(run_id.to_string()).bind(attempt_id.to_string()).bind(name).bind(Utc::now()).bind(Utc::now())
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn settle_tool(
        &self,
        run_id: RunId,
        attempt_id: AttemptId,
        tool_id: &str,
        output: &str,
        truncated: bool,
    ) -> StorageResult<()> {
        let mut transaction = self.pool.begin().await?;
        let now = Utc::now();
        sqlx::query("UPDATE tool_calls SET status = 'succeeded', updated_at = ? WHERE id = ? AND run_id = ? AND attempt_id = ? AND status = 'running'")
            .bind(now).bind(tool_id).bind(run_id.to_string()).bind(attempt_id.to_string())
            .execute(&mut *transaction).await?;
        sqlx::query("INSERT INTO tool_outputs (tool_call_id, output, truncated, created_at) VALUES (?, ?, ?, ?)")
            .bind(tool_id).bind(output).bind(truncated as i64).bind(now)
            .execute(&mut *transaction).await?;
        let sequence = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT MAX(sequence) FROM run_events WHERE run_id = ?",
        )
        .bind(run_id.to_string())
        .fetch_one(&mut *transaction)
        .await?
        .unwrap_or(0)
            + 1;
        let event = EventEnvelope {
            version: 1,
            sequence: sequence as u64,
            run_id,
            aggregate_revision: Revision(sequence as u64),
            kind: "tool.finished".into(),
            payload: serde_json::json!({"tool_id": tool_id, "success": true}),
        };
        sqlx::query(
            "INSERT INTO run_events (run_id, sequence, payload, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(run_id.to_string())
        .bind(sequence)
        .bind(serde_json::to_string(&event).unwrap())
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn tool_output(&self, tool_id: &str) -> StorageResult<Option<(String, bool)>> {
        Ok(sqlx::query_as::<_, (String, i64)>(
            "SELECT output, truncated FROM tool_outputs WHERE tool_call_id = ?",
        )
        .bind(tool_id)
        .fetch_optional(&self.pool)
        .await?
        .map(|(output, truncated)| (output, truncated != 0)))
    }

    pub async fn tool_status(&self, tool_id: &str) -> StorageResult<Option<String>> {
        Ok(
            sqlx::query_scalar("SELECT status FROM tool_calls WHERE id = ?")
                .bind(tool_id)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    pub async fn get_run_status(&self, run_id: RunId) -> StorageResult<Option<RunStatusRecord>> {
        let row =
            sqlx::query_as::<_, (String, i64)>("SELECT status, revision FROM runs WHERE id = ?")
                .bind(run_id.to_string())
                .fetch_optional(&self.pool)
                .await?;
        let Some((status, revision)) = row else {
            return Ok(None);
        };
        let attempt = sqlx::query_scalar::<_, String>(
            "SELECT status FROM attempts WHERE run_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(run_id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .map(|value| serde_json::from_str(&value))
        .transpose()
        .map_err(|error| StorageError::Database(sqlx::Error::Protocol(error.to_string())))?;
        Ok(Some(RunStatusRecord {
            run_id,
            status: serde_json::from_str(&status).map_err(|error| {
                StorageError::Database(sqlx::Error::Protocol(error.to_string()))
            })?,
            attempt_status: attempt,
            revision: Revision(revision as u64),
        }))
    }

    pub async fn claim_execution(
        &self,
        run_id: RunId,
        owner_id: &str,
        seconds: i64,
    ) -> StorageResult<bool> {
        let result = sqlx::query("INSERT INTO execution_leases (run_id, owner_id, revision, expires_at) VALUES (?, ?, 0, ?) ON CONFLICT(run_id) DO UPDATE SET owner_id = excluded.owner_id, expires_at = excluded.expires_at WHERE execution_leases.expires_at <= datetime('now')")
            .bind(run_id.to_string()).bind(owner_id).bind(Utc::now() + Duration::seconds(seconds))
            .execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn update_session(&self, session: &Session) -> StorageResult<()> {
        sqlx::query("UPDATE sessions SET title = ?, agent = ?, provider = ?, model = ?, updated_at = ? WHERE id = ?")
            .bind(&session.title).bind(&session.agent.0).bind(&session.model.provider.0).bind(&session.model.model.0).bind(session.updated_at).bind(session.id.to_string()).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn export_session(&self, id: SessionId) -> StorageResult<SessionExport> {
        let session = self.get_session(id).await?.ok_or_else(|| {
            StorageError::Domain(DomainError::NotFound {
                resource: "session".into(),
            })
        })?;
        let messages = self.list_messages(id).await?;
        Ok(SessionExport::new(&session, messages))
    }

    pub async fn import_session(
        &self,
        project_id: ProjectId,
        export: &SessionExport,
    ) -> StorageResult<Session> {
        export
            .validate()
            .map_err(|error| StorageError::InvalidExport(error.to_string()))?;
        if self.get_project(project_id).await?.is_none() {
            return Err(StorageError::Domain(DomainError::NotFound {
                resource: "project".into(),
            }));
        }
        let session_id = SessionId::new();

        let session = Session {
            id: session_id,
            project_id,
            title: export.session.title.clone(),
            agent: export.session.agent.clone(),
            model: export.session.model.clone(),
            // Runtime state is never resumed from an offline export.
            status: devfoundry_schema::SessionStatus::Idle,
            created_at: export.session.created_at,
            updated_at: export.session.updated_at,
        };
        let mut transaction = self.pool.begin().await?;
        sqlx::query("INSERT INTO sessions (id, project_id, title, agent, provider, model, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(session.id.to_string())
            .bind(session.project_id.to_string())
            .bind(&session.title)
            .bind(session.agent.0.clone())
            .bind(session.model.provider.0.clone())
            .bind(session.model.model.0.clone())
            .bind(serde_json::to_string(&session.status).unwrap())
            .bind(session.created_at)
            .bind(session.updated_at)
            .execute(&mut *transaction)
            .await?;
        for message in &export.messages {
            let mut imported_message = message.clone();
            imported_message.id = MessageId::new();
            imported_message.session_id = session.id;
            let payload = serde_json::to_string(&imported_message)
                .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
            sqlx::query("INSERT INTO messages (id, session_id, role, payload, created_at) VALUES (?, ?, ?, ?, ?)")
                .bind(imported_message.id.to_string())
                .bind(session.id.to_string())
                .bind(serde_json::to_string(&imported_message.role).unwrap())
                .bind(payload)
                .bind(imported_message.created_at)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(session)
    }

    pub async fn connect(database_url: &str) -> StorageResult<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?.foreign_keys(true);
        Self::connect_with_options(options).await
    }

    /// Open a filesystem database without serializing its path into a URL.
    pub async fn connect_path(path: impl AsRef<Path>) -> StorageResult<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(true)
            .foreign_keys(true);
        Self::connect_with_options(options).await
    }

    async fn connect_with_options(options: SqliteConnectOptions) -> StorageResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        sqlx::migrate!("../../migrations").run(&pool).await?;
        let (events, _) = broadcast::channel(256);
        Ok(Self { pool, events })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<PublishedEvent> {
        self.events.subscribe()
    }

    pub async fn admit_prompt(
        &self,
        session_id: SessionId,
        prompt: &str,
    ) -> StorageResult<MessageId> {
        let id = MessageId::new();
        sqlx::query("INSERT INTO prompt_inbox (id, session_id, prompt, status, created_at) VALUES (?, ?, ?, 'admitted', datetime('now'))")
            .bind(id.to_string())
            .bind(session_id.to_string())
            .bind(prompt)
            .execute(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn mark_prompt_running(&self, id: MessageId) -> StorageResult<()> {
        sqlx::query("UPDATE prompt_inbox SET status = 'running' WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn settle_prompt(&self, id: MessageId, status: &str) -> StorageResult<()> {
        sqlx::query("UPDATE prompt_inbox SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_prompt_status(
        &self,
        id: MessageId,
    ) -> StorageResult<Option<(SessionId, String)>> {
        let row = sqlx::query_as::<_, (String, String)>(
            "SELECT session_id, status FROM prompt_inbox WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        row.map(|(session_id, status)| {
            session_id
                .parse::<SessionId>()
                .map(|session_id| (session_id, status))
                .map_err(|error| {
                    StorageError::Domain(DomainError::InvalidIdentifier(error.to_string()))
                })
        })
        .transpose()
    }

    pub async fn set_session_status(
        &self,
        session_id: SessionId,
        status: devfoundry_schema::SessionStatus,
    ) -> StorageResult<()> {
        sqlx::query("UPDATE sessions SET status = ?, updated_at = ? WHERE id = ?")
            .bind(serde_json::to_string(&status).unwrap_or_else(|_| "\"error\"".into()))
            .bind(Utc::now())
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn recover_interrupted_work(&self) -> StorageResult<()> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("UPDATE prompt_inbox SET status = 'interrupted' WHERE status IN ('admitted', 'running')")
            .execute(&mut *transaction)
            .await?;
        sqlx::query(
            "UPDATE permission_requests SET status = '\"cancelled\"' WHERE status = '\"pending\"'",
        )
        .execute(&mut *transaction)
        .await?;
        let unknown = serde_json::to_string(&RunStatus::OutcomeUnknown).unwrap();
        let attempt_unknown = serde_json::to_string(&AttemptStatus::OutcomeUnknown).unwrap();
        sqlx::query("UPDATE runs SET status = ?, revision = revision + 1, updated_at = ? WHERE status IN ('\"Admitted\"', '\"Running\"', '\"WaitingPermission\"')")
            .bind(&unknown).bind(Utc::now()).execute(&mut *transaction).await?;
        sqlx::query("UPDATE attempts SET status = ?, updated_at = ? WHERE status = '\"Started\"'")
            .bind(attempt_unknown)
            .bind(Utc::now())
            .execute(&mut *transaction)
            .await?;
        sqlx::query(
            "UPDATE tool_calls SET status = 'unknown', updated_at = ? WHERE status = 'running'",
        )
        .bind(Utc::now())
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn diagnostics(&self) -> StorageResult<StorageDiagnostics> {
        let foreign_keys = sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
            .fetch_one(&self.pool)
            .await?
            == 1;
        let integrity_check = sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;
        let mut row_counts = serde_json::Map::new();
        for table in [
            "projects",
            "sessions",
            "messages",
            "events",
            "prompt_inbox",
            "permission_requests",
        ] {
            let count = sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&self.pool)
                .await?;
            row_counts.insert(table.into(), count.into());
        }
        Ok(StorageDiagnostics {
            foreign_keys,
            integrity_check,
            row_counts,
        })
    }

    /// Create a consistent SQLite backup without copying a live database file.
    ///
    /// SQLite's VACUUM INTO performs the copy while holding the appropriate
    /// database locks and includes data from the WAL. The temporary destination
    /// makes the visible backup atomic if publication succeeds.
    pub async fn backup_to(&self, destination: impl AsRef<Path>) -> StorageResult<()> {
        let destination = destination.as_ref();
        if destination.exists() {
            return Err(StorageError::BackupExists(destination.to_path_buf()));
        }
        let parent = destination.parent().unwrap_or_else(|| Path::new("."));
        if !parent.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("backup directory does not exist: {}", parent.display()),
            )
            .into());
        }

        let temporary = parent.join(format!(
            ".{}.{}.tmp",
            destination
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("database-backup"),
            std::process::id()
        ));
        if temporary.exists() {
            return Err(StorageError::BackupExists(temporary));
        }

        let temporary_sql = temporary.to_string_lossy().replace('\'', "''");
        let result = sqlx::query(&format!("VACUUM INTO '{}'", temporary_sql))
            .execute(&self.pool)
            .await;
        if let Err(error) = result {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }

        // The temporary file is a sibling, so linking it is same-filesystem
        // and publishes without replacing a destination created concurrently.
        if let Err(error) = fs::hard_link(&temporary, destination) {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
        fs::remove_file(&temporary)?;
        Ok(())
    }

    pub async fn create_permission_request(
        &self,
        request: &PermissionRequest,
    ) -> StorageResult<()> {
        sqlx::query("INSERT INTO permission_requests (id, session_id, operation, target, status, created_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(request.id.to_string())
            .bind(request.session_id.to_string())
            .bind(&request.operation)
            .bind(&request.target)
            .bind(serde_json::to_string(&request.status).unwrap_or_else(|_| "\"pending\"".into()))
            .bind(request.created_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn resolve_permission_request(
        &self,
        id: PermissionRequestId,
        status: PermissionStatus,
    ) -> StorageResult<bool> {
        let result = sqlx::query(
            "UPDATE permission_requests SET status = ? WHERE id = ? AND status = '\"pending\"'",
        )
        .bind(serde_json::to_string(&status).unwrap_or_else(|_| "\"denied\"".into()))
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn get_permission_request(
        &self,
        id: PermissionRequestId,
    ) -> StorageResult<Option<PermissionRequest>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, chrono::DateTime<Utc>)>(
            "SELECT id, session_id, operation, target, status, created_at FROM permission_requests WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        row.map(|(id, session_id, operation, target, status, created_at)| -> Result<PermissionRequest, DomainError> { Ok(PermissionRequest {
            id: id.parse().map_err(|_| DomainError::InvalidIdentifier(id.clone()))?,
            session_id: session_id.parse().map_err(|_| DomainError::InvalidIdentifier(session_id.clone()))?,
            operation,
            target,
            status: serde_json::from_str(&status).map_err(|_| DomainError::Validation("invalid permission status".into()))?,
            created_at,
        }) }).transpose().map_err(StorageError::from)
    }

    pub async fn list_pending_permissions(
        &self,
        session_id: SessionId,
    ) -> StorageResult<Vec<PermissionRequest>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, chrono::DateTime<Utc>)>(
            "SELECT id, session_id, operation, target, status, created_at FROM permission_requests WHERE session_id = ? AND status = '\"pending\"' ORDER BY created_at ASC",
        )
        .bind(session_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(id, session_id, operation, target, status, created_at)| -> Result<PermissionRequest, DomainError> { Ok(PermissionRequest {
                id: id.parse().map_err(|_| DomainError::InvalidIdentifier(id.clone()))?,
                session_id: session_id.parse().map_err(|_| DomainError::InvalidIdentifier(session_id.clone()))?,
                operation,
                target,
                status: serde_json::from_str(&status).map_err(|_| DomainError::Validation("invalid permission status".into()))?,
                created_at,
            }) })
            .collect::<Result<Vec<_>, DomainError>>()
            .map_err(StorageError::from)
    }

    pub async fn append_message_and_event(
        &self,
        message: &Message,
        event: &Event,
    ) -> StorageResult<EventSequence> {
        let mut transaction = self.pool.begin().await?;
        let message_payload = serde_json::to_string(message)
            .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
        sqlx::query(
            "INSERT INTO messages (id, session_id, role, payload, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(message.id.to_string())
        .bind(message.session_id.to_string())
        .bind(serde_json::to_string(&message.role).unwrap_or_else(|_| "\"user\"".into()))
        .bind(message_payload)
        .bind(message.created_at)
        .execute(&mut *transaction)
        .await?;

        let event_payload = serde_json::to_string(event)
            .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
        let result = sqlx::query(
            "INSERT INTO events (session_id, payload, created_at) VALUES (?, ?, datetime('now'))",
        )
        .bind(message.session_id.to_string())
        .bind(event_payload)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        let sequence = EventSequence(result.last_insert_rowid() as u64);
        let _ = self.events.send(PublishedEvent {
            session_id: message.session_id,
            sequence,
            event: event.clone(),
        });
        Ok(sequence)
    }
}

#[async_trait]
pub trait ProjectRepository {
    async fn create_project(&self, project: Project) -> StorageResult<Project>;
    async fn get_project(&self, id: ProjectId) -> StorageResult<Option<Project>>;
}

impl SqliteStore {
    pub async fn list_projects(&self) -> StorageResult<Vec<Project>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            "SELECT id, root, name, created_at, updated_at FROM projects ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(
                |(id, root, name, created_at, updated_at)| -> Result<Project, DomainError> {
                    Ok(Project {
                        id: id
                            .parse()
                            .map_err(|_| DomainError::InvalidIdentifier(id.clone()))?,
                        root,
                        name,
                        created_at,
                        updated_at,
                    })
                },
            )
            .collect::<Result<Vec<_>, DomainError>>()
            .map_err(StorageError::from)
    }

    pub async fn list_sessions(&self, project_id: ProjectId) -> StorageResult<Vec<Session>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, project_id, title, agent, provider, model, status, created_at, updated_at FROM sessions WHERE project_id = ? ORDER BY updated_at DESC",
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(
                |(
                    id,
                    project_id,
                    title,
                    agent,
                    provider,
                    model,
                    status,
                    created_at,
                    updated_at,
                )|
                 -> Result<Session, DomainError> {
                    Ok(Session {
                        id: id
                            .parse()
                            .map_err(|_| DomainError::InvalidIdentifier(id.clone()))?,
                        project_id: project_id
                            .parse()
                            .map_err(|_| DomainError::InvalidIdentifier(project_id.clone()))?,
                        title,
                        agent: devfoundry_schema::AgentName(agent),
                        model: devfoundry_schema::ModelRef {
                            provider: devfoundry_schema::ProviderName(provider),
                            model: devfoundry_schema::ModelName(model),
                        },
                        status: serde_json::from_str(&status).map_err(|_| {
                            DomainError::Validation("invalid session status".into())
                        })?,
                        created_at,
                        updated_at,
                    })
                },
            )
            .collect::<Result<Vec<_>, DomainError>>()
            .map_err(StorageError::from)
    }
}

#[async_trait]
pub trait SessionRepository {
    async fn create_session(&self, session: Session) -> StorageResult<Session>;
    async fn get_session(&self, id: SessionId) -> StorageResult<Option<Session>>;
    async fn list_messages(&self, id: SessionId) -> StorageResult<Vec<Message>>;
    async fn append_message(&self, message: &Message) -> StorageResult<()>;
}

#[async_trait]
pub trait EventRepository {
    async fn append_event(
        &self,
        session_id: SessionId,
        event: &Event,
    ) -> StorageResult<EventSequence>;
    async fn list_events_after(
        &self,
        session_id: SessionId,
        after: Option<EventSequence>,
    ) -> StorageResult<Vec<(EventSequence, Event)>>;
}

#[async_trait]
impl EventRepository for SqliteStore {
    async fn append_event(
        &self,
        session_id: SessionId,
        event: &Event,
    ) -> StorageResult<EventSequence> {
        let payload = serde_json::to_string(event)
            .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
        let result = sqlx::query(
            "INSERT INTO events (session_id, payload, created_at) VALUES (?, ?, datetime('now'))",
        )
        .bind(session_id.to_string())
        .bind(payload)
        .execute(&self.pool)
        .await?;
        let sequence = EventSequence(result.last_insert_rowid() as u64);
        let _ = self.events.send(PublishedEvent {
            session_id,
            sequence,
            event: event.clone(),
        });
        Ok(sequence)
    }

    async fn list_events_after(
        &self,
        session_id: SessionId,
        after: Option<EventSequence>,
    ) -> StorageResult<Vec<(EventSequence, Event)>> {
        let rows = sqlx::query_as::<_, (i64, String)>(
            "SELECT sequence, payload FROM events WHERE session_id = ? AND sequence > ? ORDER BY sequence ASC",
        )
        .bind(session_id.to_string())
        .bind(after.map_or(0, |sequence| sequence.0) as i64)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(sequence, payload)| {
                let event = serde_json::from_str(&payload)
                    .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
                Ok((EventSequence(sequence as u64), event))
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()
            .map_err(StorageError::from)
    }
}

#[async_trait]
impl ProjectRepository for SqliteStore {
    async fn create_project(&self, project: Project) -> StorageResult<Project> {
        sqlx::query(
            "INSERT INTO projects (id, root, name, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(project.id.to_string())
        .bind(&project.root)
        .bind(&project.name)
        .bind(project.created_at)
        .bind(project.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(project)
    }

    async fn get_project(&self, id: ProjectId) -> StorageResult<Option<Project>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            "SELECT id, root, name, created_at, updated_at FROM projects WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        row.map(
            |(id, root, name, created_at, updated_at)| -> Result<Project, DomainError> {
                Ok(Project {
                    id: id.parse().map_err(|_| DomainError::InvalidIdentifier(id))?,
                    root,
                    name,
                    created_at,
                    updated_at,
                })
            },
        )
        .transpose()
        .map_err(StorageError::from)
    }
}

#[async_trait]
impl SessionRepository for SqliteStore {
    async fn create_session(&self, session: Session) -> StorageResult<Session> {
        sqlx::query("INSERT INTO sessions (id, project_id, title, agent, provider, model, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(session.id.to_string())
            .bind(session.project_id.to_string())
            .bind(&session.title)
            .bind(session.agent.0.clone())
            .bind(session.model.provider.0.clone())
            .bind(session.model.model.0.clone())
            .bind(serde_json::to_string(&session.status).unwrap_or_else(|_| "\"idle\"".into()))
            .bind(session.created_at)
            .bind(session.updated_at)
            .execute(&self.pool)
            .await?;
        Ok(session)
    }

    async fn get_session(&self, id: SessionId) -> StorageResult<Option<Session>> {
        Ok(sqlx::query_as::<_, (String, String, String, String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, project_id, title, agent, provider, model, status, created_at, updated_at FROM sessions WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .map(|(id, project_id, title, agent, provider, model, status, created_at, updated_at)| -> Result<Session, DomainError> {
            Ok(Session {
                id: id.parse().map_err(|_| DomainError::InvalidIdentifier(id.clone()))?,
                project_id: project_id.parse().map_err(|_| DomainError::InvalidIdentifier(project_id.clone()))?,
                title,
                agent: devfoundry_schema::AgentName(agent),
                model: devfoundry_schema::ModelRef { provider: devfoundry_schema::ProviderName(provider), model: devfoundry_schema::ModelName(model) },
                status: serde_json::from_str(&status).map_err(|_| DomainError::Validation("invalid session status".into()))?,
                created_at,
                updated_at,
            })
        })
        .transpose()
        .map_err(StorageError::from)?)
    }

    async fn list_messages(&self, id: SessionId) -> StorageResult<Vec<Message>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT payload FROM messages WHERE session_id = ? ORDER BY created_at ASC, id ASC",
        )
        .bind(id.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|payload| {
                serde_json::from_str(&payload).map_err(|error| {
                    StorageError::Database(sqlx::Error::Protocol(error.to_string()))
                })
            })
            .collect()
    }

    async fn append_message(&self, message: &Message) -> StorageResult<()> {
        let payload = serde_json::to_string(message)
            .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
        sqlx::query("INSERT INTO messages (id, session_id, role, payload, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(message.id.to_string())
            .bind(message.session_id.to_string())
            .bind(serde_json::to_string(&message.role).unwrap_or_else(|_| "\"user\"".into()))
            .bind(payload)
            .bind(message.created_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
