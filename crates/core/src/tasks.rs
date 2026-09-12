use devfoundry_schema::{ProjectId, Task, TaskId};
use devfoundry_storage::{SqliteStore, StorageResult, TaskRepository, TaskUpdate};

pub struct TaskService {
    store: std::sync::Arc<SqliteStore>,
}

impl TaskService {
    pub fn new(store: std::sync::Arc<SqliteStore>) -> Self {
        Self { store }
    }
    pub async fn create(
        &self,
        project_id: ProjectId,
        title: String,
        description: String,
    ) -> StorageResult<Task> {
        self.store
            .create_task(Task::new_for_project(project_id, title, description))
            .await
    }
    pub async fn list(&self, project_id: ProjectId) -> StorageResult<Vec<Task>> {
        self.store.list_tasks(project_id).await
    }
    pub async fn get(&self, id: TaskId) -> StorageResult<Option<Task>> {
        self.store.get_task(id).await
    }
    pub async fn update(
        &self,
        id: TaskId,
        update: TaskUpdate,
        revision: u64,
    ) -> StorageResult<Task> {
        self.store.update_task(id, update, revision).await
    }

    pub async fn export_roadmap(&self, project_id: ProjectId) -> StorageResult<String> {
        let tasks = self.list(project_id).await?;
        let mut markdown = String::from("# Roadmap\n\n");
        for task in tasks {
            markdown.push_str(&format!(
                "- [{}] {} <!-- id:{} rev:{} deps:{} -->\n",
                if matches!(task.status, devfoundry_schema::TaskStatus::Accepted) {
                    'x'
                } else {
                    ' '
                },
                task.title,
                task.id,
                task.revision.0,
                task.dependencies
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        Ok(markdown)
    }

    pub async fn import_roadmap(
        &self,
        project_id: ProjectId,
        markdown: &str,
    ) -> StorageResult<Vec<Task>> {
        let mut records = Vec::new();
        for line in markdown
            .lines()
            .filter(|line| line.trim_start().starts_with("- ["))
        {
            let marker = line
                .split("<!-- id:")
                .nth(1)
                .and_then(|value| value.split(" -->").next())
                .ok_or_else(|| {
                    devfoundry_storage::StorageError::InvalidExport(
                        "missing roadmap task marker".into(),
                    )
                })?;
            let mut fields = marker.split(" rev:");
            let id = fields
                .next()
                .ok_or_else(|| {
                    devfoundry_storage::StorageError::InvalidExport(
                        "missing roadmap task id".into(),
                    )
                })?
                .parse()
                .map_err(|_| {
                    devfoundry_storage::StorageError::InvalidExport(
                        "invalid roadmap task id".into(),
                    )
                })?;
            let revision_and_deps = fields.next().ok_or_else(|| {
                devfoundry_storage::StorageError::InvalidExport("missing roadmap revision".into())
            })?;
            let mut revision_fields = revision_and_deps.split(" deps:");
            let revision = revision_fields
                .next()
                .ok_or_else(|| {
                    devfoundry_storage::StorageError::InvalidExport(
                        "missing roadmap revision".into(),
                    )
                })?
                .parse()
                .map_err(|_| {
                    devfoundry_storage::StorageError::InvalidExport(
                        "invalid roadmap revision".into(),
                    )
                })?;
            let dependencies = revision_fields
                .next()
                .unwrap_or_default()
                .split(',')
                .filter(|value| !value.is_empty())
                .map(|value| {
                    value.parse().map_err(|_| {
                        devfoundry_storage::StorageError::InvalidExport(
                            "invalid roadmap dependency".into(),
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let title = line
                .split("] ")
                .nth(1)
                .and_then(|value| value.split(" <!--").next())
                .ok_or_else(|| {
                    devfoundry_storage::StorageError::InvalidExport("missing roadmap title".into())
                })?
                .to_owned();
            records.push((id, revision, title, dependencies));
        }
        self.store.import_roadmap(project_id, &records).await
    }
}
