use devfoundry_schema::{InstalledResource, InstalledResourceFile, ProjectId, ResourceManifest};
use devfoundry_storage::SqliteStore;
use devfoundry_tools::{
    PermissionBroker, PermissionDecision,
    archive::{ArchiveEntry, ArchiveLimits, sha256_hex, stage_entries},
};
use std::{path::Path, sync::Arc};

pub struct ResourceService {
    store: Arc<SqliteStore>,
    permissions: Arc<dyn PermissionBroker>,
    limits: ArchiveLimits,
}

impl ResourceService {
    pub fn new(store: Arc<SqliteStore>, permissions: Arc<dyn PermissionBroker>) -> Self {
        Self {
            store,
            permissions,
            limits: ArchiveLimits::default(),
        }
    }
    pub fn inspect(
        &self,
        entries: &[ArchiveEntry],
    ) -> Result<(), devfoundry_tools::archive::ArchiveError> {
        devfoundry_tools::archive::validate_entries(entries, &self.limits)
    }
    pub fn preview(
        &self,
        entries: &[ArchiveEntry],
    ) -> Result<Vec<String>, devfoundry_tools::archive::ArchiveError> {
        self.inspect(entries)?;
        Ok(entries.iter().map(|entry| entry.path.clone()).collect())
    }
    pub async fn install(
        &self,
        project_id: ProjectId,
        root: &Path,
        manifest: &ResourceManifest,
        entries: &[ArchiveEntry],
    ) -> Result<InstalledResource, ResourceError> {
        self.inspect(entries)?;
        if entries.len() != manifest.files.len() {
            return Err(ResourceError::HashMismatch("manifest file set".into()));
        }
        for entry in entries {
            if sha256_hex(&entry.content) != entry_hash(manifest, &entry.path) {
                return Err(ResourceError::HashMismatch(entry.path.clone()));
            }
        }
        if self
            .permissions
            .authorize("resource_install", &manifest.id)
            .await
            .map_err(ResourceError::Domain)?
            != PermissionDecision::Allow
        {
            return Err(ResourceError::Denied);
        }
        let root = root.canonicalize().map_err(ResourceError::Io)?;
        let stage = stage_entries(&root, entries, &self.limits)?;
        let mut published = Vec::new();
        for entry in entries {
            let target = root.join(&entry.path);
            if target.exists() {
                rollback(&published);
                std::fs::remove_dir_all(&stage).ok();
                return Err(ResourceError::Conflict(entry.path.clone()));
            }
            if let Some(parent) = target.parent() {
                if let Err(error) = std::fs::create_dir_all(parent) {
                    rollback(&published);
                    std::fs::remove_dir_all(&stage).ok();
                    return Err(ResourceError::Io(error));
                }
            }
            if let Err(error) = std::fs::rename(stage.join(&entry.path), &target) {
                rollback(&published);
                std::fs::remove_dir_all(&stage).ok();
                return Err(ResourceError::Io(error));
            }
            published.push(target);
        }
        let files = entries
            .iter()
            .map(|entry| InstalledResourceFile {
                resource_id: manifest.id.clone(),
                project_id,
                target: entry.path.clone(),
                expected_hash: entry_hash(manifest, &entry.path),
                installed_hash: sha256_hex(&entry.content),
                size: entry.size,
            })
            .collect::<Vec<_>>();
        let result = self
            .store
            .replace_resource_files(&manifest.id, project_id, manifest, &files)
            .await?;
        std::fs::remove_dir_all(&stage).ok();
        Ok(result)
    }
    pub async fn update(
        &self,
        project_id: ProjectId,
        root: &Path,
        manifest: &ResourceManifest,
        entries: &[ArchiveEntry],
    ) -> Result<InstalledResource, ResourceError> {
        self.install(project_id, root, manifest, entries).await
    }
    pub async fn remove(
        &self,
        project_id: ProjectId,
        root: &Path,
        resource_id: &str,
    ) -> Result<(), ResourceError> {
        if self
            .permissions
            .authorize("resource_remove", resource_id)
            .await
            .map_err(ResourceError::Domain)?
            != PermissionDecision::Allow
        {
            return Err(ResourceError::Denied);
        }
        let root = root.canonicalize().map_err(ResourceError::Io)?;
        let files = self
            .store
            .list_resource_files(resource_id, project_id)
            .await?;
        let mut conflicts = Vec::new();
        for file in &files {
            let target = root.join(&file.target);
            if !target.exists() {
                continue;
            }
            let content = std::fs::read(&target).map_err(ResourceError::Io)?;
            if sha256_hex(&content) != file.installed_hash {
                conflicts.push(file.target.clone());
                continue;
            }
            std::fs::remove_file(target).map_err(ResourceError::Io)?;
        }
        if !conflicts.is_empty() {
            return Err(ResourceError::Conflict(format!(
                "edited resource files preserved: {}",
                conflicts.join(", ")
            )));
        }
        Ok(())
    }
}

fn rollback(paths: &[std::path::PathBuf]) {
    for path in paths.iter().rev() {
        let _ = std::fs::remove_file(path);
    }
}

fn entry_hash(manifest: &ResourceManifest, path: &str) -> String {
    manifest
        .files
        .iter()
        .find(|file| file.target == path)
        .map(|file| file.sha256.clone())
        .unwrap_or_default()
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("archive: {0}")]
    Archive(#[from] devfoundry_tools::archive::ArchiveError),
    #[error("storage: {0}")]
    Storage(#[from] devfoundry_storage::StorageError),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("domain: {0}")]
    Domain(#[from] devfoundry_schema::DomainError),
    #[error("permission denied")]
    Denied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("hash mismatch: {0}")]
    HashMismatch(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
}
