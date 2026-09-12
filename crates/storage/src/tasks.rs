use crate::{SqliteStore, StorageError, StorageResult};
use async_trait::async_trait;
use devfoundry_schema::{DomainError, ProjectId, Revision, Task, TaskId, TaskStatus};

#[derive(Clone, Debug)]
pub enum TaskUpdate {
    Title(String),
    Status(TaskStatus),
    Dependencies(Vec<TaskId>),
}

impl TaskUpdate {
    pub fn title(value: impl Into<String>) -> Self {
        Self::Title(value.into())
    }
    pub fn status(value: TaskStatus) -> Self {
        Self::Status(value)
    }
    pub fn dependencies(value: Vec<TaskId>) -> Self {
        Self::Dependencies(value)
    }
}

#[async_trait]
pub trait TaskRepository {
    async fn create_task(&self, task: Task) -> StorageResult<Task>;
    async fn get_task(&self, id: TaskId) -> StorageResult<Option<Task>>;
    async fn list_tasks(&self, project_id: ProjectId) -> StorageResult<Vec<Task>>;
    async fn update_task(
        &self,
        id: TaskId,
        update: TaskUpdate,
        expected_revision: u64,
    ) -> StorageResult<Task>;
    async fn import_roadmap(
        &self,
        project_id: ProjectId,
        records: &[(TaskId, u64, String, Vec<TaskId>)],
    ) -> StorageResult<Vec<Task>>;
}

impl SqliteStore {
    async fn task_dependencies(&self, task_id: TaskId) -> StorageResult<Vec<TaskId>> {
        let ids = sqlx::query_scalar::<_, String>(
            "SELECT dependency_id FROM task_dependencies WHERE task_id = ? ORDER BY dependency_id",
        )
        .bind(task_id.to_string())
        .fetch_all(self.pool())
        .await?;
        ids.into_iter()
            .map(|id| {
                id.parse()
                    .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))
            })
            .collect()
    }
}

#[async_trait]
impl TaskRepository for SqliteStore {
    async fn create_task(&self, task: Task) -> StorageResult<Task> {
        sqlx::query("INSERT INTO tasks (id, project_id, title, description, status, revision, session_id, evidence_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(task.id.to_string()).bind(task.project_id.to_string()).bind(&task.title).bind(&task.description)
            .bind(serde_json::to_string(&task.status).unwrap()).bind(task.revision.0 as i64)
            .bind(task.session_id.map(|v| v.to_string())).bind(task.evidence_id.map(|v| v.to_string()))
            .bind(task.created_at).bind(task.updated_at).execute(self.pool()).await?;
        Ok(task)
    }

    async fn get_task(&self, id: TaskId) -> StorageResult<Option<Task>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, i64, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>("SELECT id, project_id, title, description, status, revision, session_id, evidence_id, created_at, updated_at FROM tasks WHERE id = ?")
            .bind(id.to_string()).fetch_optional(self.pool()).await?;
        let Some((
            id,
            project,
            title,
            description,
            status,
            revision,
            session,
            evidence,
            created,
            updated,
        )) = row
        else {
            return Ok(None);
        };
        let dependencies = self
            .task_dependencies(
                id.parse()
                    .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))?,
            )
            .await?;
        Ok(Some(Task {
            id: id
                .parse()
                .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))?,
            project_id: project
                .parse()
                .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))?,
            title,
            description,
            status: serde_json::from_str(&status).map_err(|_| {
                StorageError::Domain(DomainError::Validation("invalid task status".into()))
            })?,
            revision: Revision(revision as u64),
            dependencies,
            session_id: session
                .map(|v| v.parse())
                .transpose()
                .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))?,
            evidence_id: evidence
                .map(|v| v.parse())
                .transpose()
                .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))?,
            created_at: created,
            updated_at: updated,
        }))
    }

    async fn list_tasks(&self, project_id: ProjectId) -> StorageResult<Vec<Task>> {
        let ids = sqlx::query_scalar::<_, String>(
            "SELECT id FROM tasks WHERE project_id = ? ORDER BY created_at, id",
        )
        .bind(project_id.to_string())
        .fetch_all(self.pool())
        .await?;
        let mut tasks = Vec::new();
        for id in ids {
            if let Some(task) = self
                .get_task(
                    id.parse()
                        .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))?,
                )
                .await?
            {
                tasks.push(task);
            }
        }
        Ok(tasks)
    }

    async fn update_task(
        &self,
        id: TaskId,
        update: TaskUpdate,
        expected_revision: u64,
    ) -> StorageResult<Task> {
        let mut tx = self.pool().begin().await?;
        let row = sqlx::query_as::<_, (String, String, String, String, i64, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>("SELECT project_id, title, description, status, revision, session_id, evidence_id, created_at, updated_at FROM tasks WHERE id = ?").bind(id.to_string()).fetch_optional(&mut *tx).await?.ok_or_else(|| StorageError::Domain(DomainError::NotFound { resource: "task".into() }))?;
        if row.4 as u64 != expected_revision {
            return Err(StorageError::Domain(DomainError::Validation(
                "task revision conflict".into(),
            )));
        }
        let mut dependencies = sqlx::query_scalar::<_, String>(
            "SELECT dependency_id FROM task_dependencies WHERE task_id = ?",
        )
        .bind(id.to_string())
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .map(|v| v.parse())
        .collect::<Result<Vec<TaskId>, _>>()
        .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))?;
        let old_title = row.1.clone();
        let old_status = row.3.clone();
        let is_dependency_update = matches!(&update, TaskUpdate::Dependencies(_));
        let (title, status) = match update {
            TaskUpdate::Title(v) => (v, old_title.clone()),
            TaskUpdate::Status(v) => (old_title, serde_json::to_string(&v).unwrap()),
            TaskUpdate::Dependencies(v) => {
                dependencies = v;
                (old_title, old_status)
            }
        };
        if is_dependency_update {
            let edges = sqlx::query_as::<_, (String, String)>(
                "SELECT task_id, dependency_id FROM task_dependencies",
            )
            .fetch_all(&mut *tx)
            .await?;
            let mut graph = std::collections::HashMap::<TaskId, Vec<TaskId>>::new();
            for (task, dependency) in edges {
                let task = task
                    .parse()
                    .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))?;
                let dependency = dependency
                    .parse()
                    .map_err(|e| StorageError::Domain(DomainError::InvalidIdentifier(e)))?;
                graph.entry(task).or_default().push(dependency);
            }
            graph.insert(id, dependencies.clone());
            fn reaches(
                graph: &std::collections::HashMap<TaskId, Vec<TaskId>>,
                start: TaskId,
                target: TaskId,
                seen: &mut std::collections::HashSet<TaskId>,
            ) -> bool {
                if start == target {
                    return true;
                }
                if !seen.insert(start) {
                    return false;
                }
                graph.get(&start).is_some_and(|dependencies| {
                    dependencies
                        .iter()
                        .any(|dependency| reaches(graph, *dependency, target, seen))
                })
            }
            if dependencies.iter().any(|dependency| {
                *dependency == id
                    || reaches(
                        &graph,
                        *dependency,
                        id,
                        &mut std::collections::HashSet::new(),
                    )
            }) {
                return Err(StorageError::Domain(DomainError::Validation(
                    "task dependency cycle".into(),
                )));
            }
        }
        sqlx::query("UPDATE tasks SET title = ?, status = ?, revision = revision + 1, updated_at = datetime('now') WHERE id = ? AND revision = ?").bind(title).bind(status).bind(id.to_string()).bind(expected_revision as i64).execute(&mut *tx).await?;
        if is_dependency_update {
            sqlx::query("DELETE FROM task_dependencies WHERE task_id = ?")
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
            for dependency in &dependencies {
                sqlx::query("INSERT INTO task_dependencies (task_id, dependency_id) VALUES (?, ?)")
                    .bind(id.to_string())
                    .bind(dependency.to_string())
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        self.get_task(id).await?.ok_or_else(|| {
            StorageError::Domain(DomainError::NotFound {
                resource: "task".into(),
            })
        })
    }

    async fn import_roadmap(
        &self,
        project_id: ProjectId,
        records: &[(TaskId, u64, String, Vec<TaskId>)],
    ) -> StorageResult<Vec<Task>> {
        let mut transaction = self.pool().begin().await?;
        for (id, revision, title, dependencies) in records {
            let row = sqlx::query_as::<_, (String, i64)>(
                "SELECT project_id, revision FROM tasks WHERE id = ?",
            )
            .bind(id.to_string())
            .fetch_optional(&mut *transaction)
            .await?
            .ok_or_else(|| {
                StorageError::Domain(DomainError::NotFound {
                    resource: "task".into(),
                })
            })?;
            if row.0 != project_id.to_string() || row.1 as u64 != *revision {
                return Err(StorageError::Domain(DomainError::Validation(
                    "roadmap task revision conflict".into(),
                )));
            }
            if dependencies.contains(id) {
                return Err(StorageError::Domain(DomainError::Validation(
                    "task dependency cycle".into(),
                )));
            }
            sqlx::query("UPDATE tasks SET title = ?, revision = revision + 1, updated_at = datetime('now') WHERE id = ? AND revision = ?")
                .bind(title).bind(id.to_string()).bind(*revision as i64).execute(&mut *transaction).await?;
            sqlx::query("DELETE FROM task_dependencies WHERE task_id = ?")
                .bind(id.to_string())
                .execute(&mut *transaction)
                .await?;
            for dependency in dependencies {
                let exists: Option<String> =
                    sqlx::query_scalar("SELECT id FROM tasks WHERE id = ? AND project_id = ?")
                        .bind(dependency.to_string())
                        .bind(project_id.to_string())
                        .fetch_optional(&mut *transaction)
                        .await?;
                if exists.is_none() {
                    return Err(StorageError::Domain(DomainError::Validation(
                        "roadmap dependency not found".into(),
                    )));
                }
                sqlx::query("INSERT INTO task_dependencies (task_id, dependency_id) VALUES (?, ?)")
                    .bind(id.to_string())
                    .bind(dependency.to_string())
                    .execute(&mut *transaction)
                    .await?;
            }
        }
        transaction.commit().await?;
        let mut result = Vec::new();
        for (id, _, _, _) in records {
            if let Some(task) = self.get_task(*id).await? {
                result.push(task);
            }
        }
        Ok(result)
    }
}
