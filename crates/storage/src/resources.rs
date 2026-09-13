use crate::{SqliteStore, StorageResult};
use devfoundry_schema::{
    InstalledResource, InstalledResourceFile, ProjectId, ResourceManifest, Revision,
};

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

    pub async fn replace_resource_files(
        &self,
        resource_id: &str,
        project_id: ProjectId,
        manifest: &ResourceManifest,
        files: &[InstalledResourceFile],
    ) -> StorageResult<InstalledResource> {
        let now = chrono::Utc::now();
        let mut tx = self.pool().begin().await?;
        sqlx::query("DELETE FROM resource_files WHERE resource_id = ? AND project_id = ?")
            .bind(resource_id)
            .bind(project_id.to_string())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM resources WHERE id = ? AND project_id = ?")
            .bind(resource_id)
            .bind(project_id.to_string())
            .execute(&mut *tx)
            .await?;
        let resource = InstalledResource {
            id: resource_id.into(),
            project_id,
            version: manifest.version.clone(),
            revision: Revision(0),
            manifest_hash: manifest.manifest_hash.clone(),
        };
        sqlx::query("INSERT INTO resources (id, project_id, version, source, manifest_hash, required_capabilities, revision, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?)")
            .bind(&resource.id)
            .bind(project_id.to_string())
            .bind(&resource.version)
            .bind(&manifest.source)
            .bind(&resource.manifest_hash)
            .bind(serde_json::to_string(&manifest.required_capabilities).unwrap_or_else(|_| "[]".into()))
            .bind(now)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        for file in files {
            sqlx::query("INSERT INTO resource_files (resource_id, project_id, target, expected_hash, installed_hash, size) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(&resource.id)
                .bind(project_id.to_string())
                .bind(&file.target)
                .bind(&file.expected_hash)
                .bind(&file.installed_hash)
                .bind(file.size as i64)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(resource)
    }

    pub async fn mark_resource_files_installed(
        &self,
        resource_id: &str,
        project_id: ProjectId,
        files: &[InstalledResourceFile],
    ) -> StorageResult<()> {
        let mut tx = self.pool().begin().await?;
        for file in files {
            sqlx::query(
                "UPDATE resource_files SET installed_hash = ? WHERE resource_id = ? AND project_id = ? AND target = ?",
            )
            .bind(&file.installed_hash)
            .bind(resource_id)
            .bind(project_id.to_string())
            .bind(&file.target)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_resource_files(
        &self,
        resource_id: &str,
        project_id: ProjectId,
    ) -> StorageResult<Vec<InstalledResourceFile>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64)>(
            "SELECT target, expected_hash, installed_hash, size FROM resource_files WHERE resource_id = ? AND project_id = ? ORDER BY target",
        )
        .bind(resource_id)
        .bind(project_id.to_string())
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(target, expected_hash, installed_hash, size)| InstalledResourceFile {
                    resource_id: resource_id.into(),
                    project_id,
                    target,
                    expected_hash,
                    installed_hash,
                    size: size as u64,
                },
            )
            .collect())
    }
}
