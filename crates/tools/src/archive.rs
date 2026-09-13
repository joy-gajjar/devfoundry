use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ArchiveEntryKind {
    File,
    Symlink,
    Hardlink,
    Special,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveEntry {
    pub path: String,
    pub size: u64,
    pub kind: ArchiveEntryKind,
    pub content: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveLimits {
    pub max_files: usize,
    pub max_expanded_bytes: u64,
    pub max_file_bytes: u64,
    pub max_path_bytes: usize,
}

impl Default for ArchiveLimits {
    fn default() -> Self {
        Self {
            max_files: 1_000,
            max_expanded_bytes: 64 * 1024 * 1024,
            max_file_bytes: 8 * 1024 * 1024,
            max_path_bytes: 4 * 1024,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("archive traversal path rejected")]
    Traversal,
    #[error("archive path is invalid")]
    InvalidPath,
    #[error("archive contains a link or special entry")]
    LinkOrSpecial,
    #[error("archive contains duplicate target")]
    DuplicateTarget,
    #[error("archive file count exceeds limit")]
    FileCount,
    #[error("archive expanded size exceeds limit")]
    ExpandedSize,
    #[error("archive file size exceeds limit")]
    FileSize,
    #[error("archive path exceeds limit")]
    PathSize,
    #[error("staging failed: {0}")]
    Io(#[from] std::io::Error),
}

pub fn validate_entries(
    entries: &[ArchiveEntry],
    limits: &ArchiveLimits,
) -> Result<(), ArchiveError> {
    if entries.len() > limits.max_files {
        return Err(ArchiveError::FileCount);
    }
    let mut targets = BTreeSet::new();
    let mut total: u64 = 0;
    for entry in entries {
        if entry.path.len() > limits.max_path_bytes {
            return Err(ArchiveError::PathSize);
        }
        if !matches!(entry.kind, ArchiveEntryKind::File) {
            return Err(ArchiveError::LinkOrSpecial);
        }
        let path = Path::new(&entry.path);
        if path.is_absolute()
            || path.components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(ArchiveError::Traversal);
        }
        if entry.path.trim().is_empty() || path.components().any(|c| matches!(c, Component::CurDir))
        {
            return Err(ArchiveError::InvalidPath);
        }
        if entry.size > limits.max_file_bytes {
            return Err(ArchiveError::FileSize);
        }
        total = total
            .checked_add(entry.size)
            .ok_or(ArchiveError::ExpandedSize)?;
        if total > limits.max_expanded_bytes {
            return Err(ArchiveError::ExpandedSize);
        }
        if !targets.insert(path.to_string_lossy().replace('\\', "/")) {
            return Err(ArchiveError::DuplicateTarget);
        }
    }
    Ok(())
}

pub fn stage_entries(
    root: &Path,
    entries: &[ArchiveEntry],
    limits: &ArchiveLimits,
) -> Result<PathBuf, ArchiveError> {
    validate_entries(entries, limits)?;
    let stage = root.join(format!(
        ".devfoundry-resource-stage-{}-{}",
        std::process::id(),
        STAGE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&stage)?;
    for entry in entries {
        let target = stage.join(&entry.path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, &entry.content)?;
    }
    Ok(stage)
}

pub fn sha256_hex(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}
