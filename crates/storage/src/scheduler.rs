use crate::{SqliteStore, StorageError, StorageResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use devfoundry_schema::{AttemptId, DomainError, Revision, TaskId, TaskStatus};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum WorkerLeaseStatus {
    Claimed,
    Released,
    Recovered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum WorkerAttemptStatus {
    Started,
    Completed,
    Failed,
    Cancelled,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct WorkerLease {
    pub id: String,
    pub task_id: TaskId,
    pub task_revision: Revision,
    pub owner: String,
    pub status: WorkerLeaseStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct WorkerAttempt {
    pub id: AttemptId,
    pub lease_id: String,
    pub task_id: TaskId,
    pub task_revision: Revision,
    pub status: WorkerAttemptStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct EvidenceRecord {
    pub id: String,
    pub task_id: TaskId,
    pub task_revision: Revision,
    pub base_commit: String,
    pub proposed_head: String,
    pub diff_digest: String,
    pub commands: Vec<String>,
}

impl EvidenceRecord {
    pub fn new(
        task_id: TaskId,
        revision: Revision,
        base: impl Into<String>,
        head: impl Into<String>,
        digest: impl Into<String>,
        commands: Vec<String>,
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            task_id,
            task_revision: revision,
            base_commit: base.into(),
            proposed_head: head.into(),
            diff_digest: digest.into(),
            commands,
        }
    }
}

#[async_trait]
pub trait SchedulerRepository {
    async fn claim_task(
        &self,
        task_id: TaskId,
        expected_revision: Revision,
        owner: &str,
    ) -> StorageResult<WorkerLease>;
    async fn begin_attempt(
        &self,
        lease_id: &str,
        attempt_id: AttemptId,
    ) -> StorageResult<WorkerAttempt>;
    async fn settle_attempt(
        &self,
        attempt_id: AttemptId,
        outcome: WorkerAttemptStatus,
        evidence: Option<&EvidenceRecord>,
    ) -> StorageResult<WorkerAttempt>;
    async fn release_task(&self, lease_id: &str, expected_revision: Revision) -> StorageResult<()>;
    async fn recover_orphaned_workers(&self, owner: &str) -> StorageResult<Vec<WorkerAttempt>>;
    async fn list_recoverable_workers(&self) -> StorageResult<Vec<WorkerAttempt>>;
    async fn list_evidence(&self, attempt_id: AttemptId) -> StorageResult<Vec<EvidenceRecord>>;
}

fn conflict(message: &str) -> StorageError {
    StorageError::Domain(DomainError::Validation(message.into()))
}
fn attempt_status(value: &str) -> WorkerAttemptStatus {
    match value {
        "completed" => WorkerAttemptStatus::Completed,
        "failed" => WorkerAttemptStatus::Failed,
        "cancelled" => WorkerAttemptStatus::Cancelled,
        "outcome_unknown" => WorkerAttemptStatus::OutcomeUnknown,
        _ => WorkerAttemptStatus::Started,
    }
}

#[async_trait]
impl SchedulerRepository for SqliteStore {
    async fn claim_task(
        &self,
        task_id: TaskId,
        expected_revision: Revision,
        owner: &str,
    ) -> StorageResult<WorkerLease> {
        let mut tx = self.pool().begin().await?;
        let row = sqlx::query_as::<_, (String, i64, String)>(
            "SELECT project_id, revision, status FROM tasks WHERE id = ?",
        )
        .bind(task_id.to_string())
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            StorageError::Domain(DomainError::NotFound {
                resource: "task".into(),
            })
        })?;
        if row.1 as u64 != expected_revision.0 {
            return Err(conflict("task revision conflict"));
        }
        if row.2 != serde_json::to_string(&TaskStatus::Ready).unwrap()
            && row.2 != serde_json::to_string(&TaskStatus::Assigned).unwrap()
        {
            return Err(conflict("task is not ready"));
        }
        let id = ulid::Ulid::new().to_string();
        let now = Utc::now();
        let result = sqlx::query("INSERT INTO scheduler_leases (id, task_id, task_revision, owner, status, created_at, updated_at) VALUES (?, ?, ?, ?, 'claimed', ?, ?)")
            .bind(&id).bind(task_id.to_string()).bind(expected_revision.0 as i64).bind(owner).bind(now).bind(now).execute(&mut *tx).await;
        if let Err(error) = result {
            if matches!(error, sqlx::Error::Database(_)) {
                return Err(conflict("task lease conflict"));
            }
            return Err(error.into());
        }
        sqlx::query("UPDATE tasks SET status = ?, updated_at = ? WHERE id = ? AND revision = ?")
            .bind(serde_json::to_string(&TaskStatus::Assigned).unwrap())
            .bind(now)
            .bind(task_id.to_string())
            .bind(expected_revision.0 as i64)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(WorkerLease {
            id,
            task_id,
            task_revision: expected_revision,
            owner: owner.into(),
            status: WorkerLeaseStatus::Claimed,
            created_at: now,
        })
    }

    async fn begin_attempt(
        &self,
        lease_id: &str,
        attempt_id: AttemptId,
    ) -> StorageResult<WorkerAttempt> {
        let mut tx = self.pool().begin().await?;
        let lease = sqlx::query_as::<_, (String, i64, String)>(
            "SELECT task_id, task_revision, status FROM scheduler_leases WHERE id = ?",
        )
        .bind(lease_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            StorageError::Domain(DomainError::NotFound {
                resource: "lease".into(),
            })
        })?;
        let task_id: TaskId = lease
            .0
            .parse()
            .map_err(DomainError::InvalidIdentifier)
            .map_err(StorageError::from)?;
        if lease.2 != "claimed" {
            return Err(conflict("lease is not active"));
        }
        if let Some(row) = sqlx::query_as::<_, (String, i64, String)>(
            "SELECT task_id, task_revision, status FROM worker_attempts WHERE id = ?",
        )
        .bind(attempt_id.to_string())
        .fetch_optional(&mut *tx)
        .await?
        {
            if row.0 == lease.0 && row.1 == lease.1 {
                tx.commit().await?;
                return Ok(WorkerAttempt {
                    id: attempt_id,
                    lease_id: lease_id.into(),
                    task_id,
                    task_revision: Revision(lease.1 as u64),
                    status: attempt_status(&row.2),
                });
            }
            return Err(conflict("attempt identity conflict"));
        }
        let now = Utc::now();
        sqlx::query("INSERT INTO worker_attempts (id, lease_id, task_id, task_revision, status, created_at, updated_at) VALUES (?, ?, ?, ?, 'started', ?, ?)")
            .bind(attempt_id.to_string()).bind(lease_id).bind(&lease.0).bind(lease.1).bind(now).bind(now).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(WorkerAttempt {
            id: attempt_id,
            lease_id: lease_id.into(),
            task_id,
            task_revision: Revision(lease.1 as u64),
            status: WorkerAttemptStatus::Started,
        })
    }

    async fn settle_attempt(
        &self,
        attempt_id: AttemptId,
        outcome: WorkerAttemptStatus,
        evidence: Option<&EvidenceRecord>,
    ) -> StorageResult<WorkerAttempt> {
        let mut tx = self.pool().begin().await?;
        let row = sqlx::query_as::<_, (String, String, i64, String)>(
            "SELECT lease_id, task_id, task_revision, status FROM worker_attempts WHERE id = ?",
        )
        .bind(attempt_id.to_string())
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            StorageError::Domain(DomainError::NotFound {
                resource: "attempt".into(),
            })
        })?;
        let task_id: TaskId = row
            .1
            .parse()
            .map_err(DomainError::InvalidIdentifier)
            .map_err(StorageError::from)?;
        if row.3 != "started" {
            return Err(conflict("attempt already settled"));
        }
        if matches!(outcome, WorkerAttemptStatus::Completed) && evidence.is_none() {
            return Err(conflict("completed attempt requires evidence"));
        }
        let status = match outcome {
            WorkerAttemptStatus::Completed => "completed",
            WorkerAttemptStatus::Failed => "failed",
            WorkerAttemptStatus::Cancelled => "cancelled",
            WorkerAttemptStatus::OutcomeUnknown => "outcome_unknown",
            WorkerAttemptStatus::Started => "started",
        };
        let now = Utc::now();
        if let Some(evidence) = evidence {
            sqlx::query("INSERT INTO worker_evidence (id, attempt_id, task_id, task_revision, base_commit, proposed_head, diff_digest, commands, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)").bind(&evidence.id).bind(attempt_id.to_string()).bind(task_id.to_string()).bind(evidence.task_revision.0 as i64).bind(&evidence.base_commit).bind(&evidence.proposed_head).bind(&evidence.diff_digest).bind(serde_json::to_string(&evidence.commands).unwrap()).bind(now).execute(&mut *tx).await?;
        }
        sqlx::query("UPDATE worker_attempts SET status = ?, updated_at = ? WHERE id = ? AND status = 'started'").bind(status).bind(now).bind(attempt_id.to_string()).execute(&mut *tx).await?;
        if evidence.is_some() {
            sqlx::query("UPDATE tasks SET status = ?, evidence_id = ?, updated_at = ? WHERE id = ? AND revision = ?").bind(serde_json::to_string(&TaskStatus::Review).unwrap()).bind(evidence.map(|e| e.id.clone())).bind(now).bind(task_id.to_string()).bind(row.2).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(WorkerAttempt {
            id: attempt_id,
            lease_id: row.0,
            task_id,
            task_revision: Revision(row.2 as u64),
            status: outcome,
        })
    }

    async fn release_task(&self, lease_id: &str, expected_revision: Revision) -> StorageResult<()> {
        let result = sqlx::query("UPDATE scheduler_leases SET status = 'released', updated_at = ? WHERE id = ? AND task_revision = ? AND status = 'claimed'").bind(Utc::now()).bind(lease_id).bind(expected_revision.0 as i64).execute(self.pool()).await?;
        if result.rows_affected() == 0 {
            return Err(conflict("lease release conflict"));
        }
        Ok(())
    }

    async fn recover_orphaned_workers(&self, _owner: &str) -> StorageResult<Vec<WorkerAttempt>> {
        let mut tx = self.pool().begin().await?;
        sqlx::query("UPDATE worker_attempts SET status = 'outcome_unknown', updated_at = ? WHERE status = 'started'").bind(Utc::now()).execute(&mut *tx).await?;
        sqlx::query("UPDATE scheduler_leases SET status = 'recovered', updated_at = ? WHERE status = 'claimed'").bind(Utc::now()).execute(&mut *tx).await?;
        tx.commit().await?;
        self.list_recoverable_workers().await
    }

    async fn list_recoverable_workers(&self) -> StorageResult<Vec<WorkerAttempt>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64, String)>("SELECT id, lease_id, task_id, task_revision, status FROM worker_attempts WHERE status = 'outcome_unknown' ORDER BY created_at, id").fetch_all(self.pool()).await?;
        rows.into_iter()
            .map(|(id, lease, task, revision, status)| {
                Ok(WorkerAttempt {
                    id: id
                        .parse()
                        .map_err(DomainError::InvalidIdentifier)
                        .map_err(StorageError::from)?,
                    lease_id: lease,
                    task_id: task
                        .parse()
                        .map_err(DomainError::InvalidIdentifier)
                        .map_err(StorageError::from)?,
                    task_revision: Revision(revision as u64),
                    status: attempt_status(&status),
                })
            })
            .collect()
    }

    async fn list_evidence(&self, attempt_id: AttemptId) -> StorageResult<Vec<EvidenceRecord>> {
        let rows = sqlx::query_as::<_, (String, String, i64, String, String, String, String)>("SELECT id, task_id, task_revision, base_commit, proposed_head, diff_digest, commands FROM worker_evidence WHERE attempt_id = ? ORDER BY created_at, id").bind(attempt_id.to_string()).fetch_all(self.pool()).await?;
        rows.into_iter()
            .map(|(id, task, revision, base, head, digest, commands)| {
                Ok(EvidenceRecord {
                    id,
                    task_id: task
                        .parse()
                        .map_err(DomainError::InvalidIdentifier)
                        .map_err(StorageError::from)?,
                    task_revision: Revision(revision as u64),
                    base_commit: base,
                    proposed_head: head,
                    diff_digest: digest,
                    commands: serde_json::from_str(&commands).map_err(|e| {
                        StorageError::Database(sqlx::Error::Protocol(e.to_string()))
                    })?,
                })
            })
            .collect()
    }
}
