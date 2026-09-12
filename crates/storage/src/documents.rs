use crate::{SqliteStore, StorageError, StorageResult};
use async_trait::async_trait;
use devfoundry_schema::{DocumentMetadata, ProjectId};

#[async_trait]
pub trait DocumentRepository {
    async fn rebuild_documents(&self, project_id: ProjectId) -> StorageResult<()>;
    async fn list_documents(&self, project_id: ProjectId) -> StorageResult<Vec<DocumentMetadata>>;
    async fn backlinks(&self, project_id: ProjectId, path: &str) -> StorageResult<Vec<String>>;
}

fn safe_path(root: &std::path::Path, relative: &str) -> StorageResult<std::path::PathBuf> {
    let path = root.join(relative);
    if relative.starts_with('/')
        || relative.split('/').any(|part| part == "..")
        || path.is_symlink()
    {
        return Err(StorageError::InvalidExport(
            "document path escapes project".into(),
        ));
    }
    Ok(path)
}

#[async_trait]
impl DocumentRepository for SqliteStore {
    async fn rebuild_documents(&self, project_id: ProjectId) -> StorageResult<()> {
        let (root,) = sqlx::query_as::<_, (String,)>("SELECT root FROM projects WHERE id = ?")
            .bind(project_id.to_string())
            .fetch_one(self.pool())
            .await?;
        let root = std::path::PathBuf::from(root);
        let mut stack = vec![root.clone()];
        let mut documents = Vec::new();
        while let Some(directory) = stack.pop() {
            for entry in std::fs::read_dir(&directory)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_symlink() {
                    return Err(StorageError::InvalidExport(
                        "symlink in document tree".into(),
                    ));
                }
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().is_some_and(|extension| extension == "md") {
                    let relative = path
                        .strip_prefix(&root)
                        .unwrap()
                        .to_string_lossy()
                        .replace(std::path::MAIN_SEPARATOR, "/");
                    let content = std::fs::read_to_string(safe_path(&root, &relative)?)?;
                    let mut links = Vec::new();
                    for line in content.lines() {
                        let mut rest = line;
                        while let Some(start) = rest.find("[[") {
                            let Some(end) = rest[start + 2..].find("]]") else {
                                break;
                            };
                            links.push(rest[start + 2..start + 2 + end].to_owned());
                            rest = &rest[start + 4 + end..];
                        }
                    }
                    let content_hash = format!(
                        "{:x}",
                        content.bytes().fold(0_u128, |hash, byte| hash
                            .wrapping_mul(31)
                            .wrapping_add(byte as u128))
                    );
                    documents.push(DocumentMetadata {
                        path: relative,
                        content_hash,
                        links,
                    });
                }
            }
        }
        let mut transaction = self.pool().begin().await?;
        sqlx::query("DELETE FROM documents WHERE project_id = ?")
            .bind(project_id.to_string())
            .execute(&mut *transaction)
            .await?;
        for document in documents {
            sqlx::query("INSERT INTO documents (project_id, path, content_hash, links, updated_at) VALUES (?, ?, ?, ?, datetime('now'))")
                .bind(project_id.to_string()).bind(document.path).bind(document.content_hash)
                .bind(serde_json::to_string(&document.links).unwrap()).execute(&mut *transaction).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn backlinks(&self, project_id: ProjectId, path: &str) -> StorageResult<Vec<String>> {
        let target = path.strip_suffix(".md").unwrap_or(path);
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT path, links FROM documents WHERE project_id = ? ORDER BY path",
        )
        .bind(project_id.to_string())
        .fetch_all(self.pool())
        .await?;
        let basename = target.rsplit('/').next().unwrap_or(target);
        Ok(rows
            .into_iter()
            .filter_map(|(source, links)| {
                let links: Vec<String> = serde_json::from_str(&links).ok()?;
                links
                    .iter()
                    .any(|link| link == target || link == basename || format!("{link}.md") == path)
                    .then_some(source)
            })
            .collect())
    }

    async fn list_documents(&self, project_id: ProjectId) -> StorageResult<Vec<DocumentMetadata>> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT path, content_hash, links FROM documents WHERE project_id = ? ORDER BY path",
        )
        .bind(project_id.to_string())
        .fetch_all(self.pool())
        .await?;
        rows.into_iter()
            .map(|(path, content_hash, links)| {
                Ok(DocumentMetadata {
                    path,
                    content_hash,
                    links: serde_json::from_str(&links).map_err(|e| {
                        StorageError::Database(sqlx::Error::Protocol(e.to_string()))
                    })?,
                })
            })
            .collect()
    }
}
