use devfoundry_schema::{InstalledResource, ProjectId, ResourceManifest};
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
        for entry in entries {
            let target = root.join(&entry.path);
            if target.exists() {
                std::fs::remove_dir_all(&stage).ok();
                return Err(ResourceError::Conflict(entry.path.clone()));
            }
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(ResourceError::Io)?;
            }
            std::fs::rename(stage.join(&entry.path), &target).map_err(ResourceError::Io)?;
        }
        let result = self.store.save_resource(project_id, manifest).await?;
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
        _project_id: ProjectId,
        _root: &Path,
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
        Err(ResourceError::Unsupported(
            "edited-file-preserving removal requires a persisted file projection".into(),
        ))
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
