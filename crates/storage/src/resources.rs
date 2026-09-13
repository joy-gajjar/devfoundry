use crate::{SqliteStore, StorageResult};
use devfoundry_schema::{InstalledResource, ProjectId, ResourceManifest, Revision};

pub trait ResourceRepository {
    fn _resource_repository_marker(&self) {}
}

impl ResourceRepository for SqliteStore {}

impl SqliteStore {
    pub async fn save_resource(
        &self,
        project_id: ProjectId,
        manifest: &ResourceManifest,
    ) -> StorageResult<InstalledResource> {
        let now = chrono::Utc::now();
        let resource = InstalledResource {
            id: manifest.id.clone(),
            project_id,
            version: manifest.version.clone(),
            revision: Revision(0),
            manifest_hash: manifest.manifest_hash.clone(),
        };
        let mut tx = self.pool().begin().await?;
        sqlx::query("INSERT INTO resources (id, project_id, version, source, manifest_hash, required_capabilities, revision, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?)")
            .bind(&resource.id).bind(project_id.to_string()).bind(&resource.version).bind(&manifest.source).bind(&resource.manifest_hash)
            .bind(serde_json::to_string(&manifest.required_capabilities).unwrap_or_else(|_| "[]".into())).bind(now).bind(now).execute(&mut *tx).await?;
        for file in &manifest.files {
            sqlx::query("INSERT INTO resource_files (resource_id, project_id, target, expected_hash, installed_hash, size) VALUES (?, ?, ?, ?, '', ?)")
                .bind(&resource.id).bind(project_id.to_string()).bind(&file.target).bind(&file.sha256).bind(file.size as i64).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(resource)
    }
}
