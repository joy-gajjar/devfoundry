//! Permission-gated Git worktree ownership and review.
//!
//! This module deliberately uses argv-only Git execution. Repository configuration and
//! inherited Git configuration are treated as hostile input; checkout and review commands
//! disable executable Git integrations and ignore global/system configuration.

use crate::{PermissionBroker, PermissionDecision, ToolError};
use devfoundry_schema::DomainError;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{process::Command, sync::Mutex};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct WorktreeManager {
    primary: PathBuf,
    metadata_root: PathBuf,
    permissions: Arc<dyn PermissionBroker>,
    admin_lock: Arc<Mutex<()>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorktreeRequest {
    pub worker_id: String,
    pub target_ref: String,
    pub expected_target_head: Option<String>,
}

impl WorktreeRequest {
    pub fn new(worker_id: impl Into<String>, target_ref: impl Into<String>) -> Self {
        Self {
            worker_id: worker_id.into(),
            target_ref: target_ref.into(),
            expected_target_head: None,
        }
    }

    pub fn with_expected_target_head(mut self, head: impl Into<String>) -> Self {
        self.expected_target_head = Some(head.into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorktreeMetadata {
    pub worker_id: String,
    pub worktree_path: PathBuf,
    pub branch: String,
    pub target_ref: String,
    pub expected_target_head: String,
    pub base_commit: String,
    pub proposed_head: Option<String>,
    pub lease: String,
}

#[derive(Clone, Debug)]
pub struct AllocatedWorktree {
    pub path: PathBuf,
    pub metadata: WorktreeMetadata,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorktreeStatus {
    Ready,
    Dirty,
    Missing,
    Conflict,
    Stale,
}

#[derive(Clone, Debug)]
pub struct WorktreeReview {
    pub status: WorktreeStatus,
    pub base_commit: String,
    pub proposed_head: String,
    pub diff: String,
    pub evidence_digest: String,
}

#[derive(Clone, Debug)]
pub struct HumanIntegrationAuthorization {
    target_ref: String,
    expected_target_head: String,
    proposed_head: String,
    evidence_digest: String,
}

impl HumanIntegrationAuthorization {
    pub fn new(
        target_ref: impl Into<String>,
        expected_target_head: impl Into<String>,
        proposed_head: impl Into<String>,
        evidence_digest: impl Into<String>,
    ) -> Self {
        Self {
            target_ref: target_ref.into(),
            expected_target_head: expected_target_head.into(),
            proposed_head: proposed_head.into(),
            evidence_digest: evidence_digest.into(),
        }
    }
}

impl WorktreeManager {
    pub fn new(
        primary: PathBuf,
        metadata_root: PathBuf,
        permissions: Arc<dyn PermissionBroker>,
    ) -> Result<Self, ToolError> {
        let primary = primary
            .canonicalize()
            .map_err(|e| ToolError::Failed(format!("invalid primary repository: {e}")))?;
        if !primary.is_dir() {
            return Err(ToolError::Failed(
                "primary repository is not a directory".into(),
            ));
        }
        let metadata_parent = metadata_root
            .parent()
            .ok_or_else(|| ToolError::Failed("metadata root has no parent".into()))?
            .canonicalize()
            .map_err(|e| ToolError::Failed(format!("invalid metadata parent: {e}")))?;
        let metadata_root = metadata_parent.join(
            metadata_root
                .file_name()
                .ok_or_else(|| ToolError::Failed("metadata root has no name".into()))?,
        );
        if metadata_root.starts_with(&primary) {
            return Err(ToolError::Failed(
                "metadata root must not be inside the primary repository".into(),
            ));
        }
        Ok(Self {
            primary,
            metadata_root,
            permissions,
            admin_lock: Arc::new(Mutex::new(())),
        })
    }

    pub async fn allocate(
        &self,
        request: WorktreeRequest,
        cancellation: CancellationToken,
    ) -> Result<AllocatedWorktree, ToolError> {
        self.authorize("worktree_allocate", &request.worker_id)
            .await?;
        let _guard = self.admin_lock.lock().await;
        let status = self
            .git(
                &["status", "--porcelain"],
                &self.primary,
                cancellation.clone(),
            )
            .await?;
        if !status.is_empty() {
            return Err(ToolError::Failed(
                "primary repository is dirty; worktree allocation refused".into(),
            ));
        }
        let target_head = self
            .rev_parse(&request.target_ref, cancellation.clone())
            .await?;
        if let Some(expected) = request.expected_target_head.as_deref() {
            if expected != target_head {
                return Err(ToolError::Failed(format!(
                    "expected target HEAD {expected}, found {target_head}"
                )));
            }
        }
        let branch = format!("w08/{}", safe_name(&request.worker_id)?);
        let path = self.metadata_root.join(&request.worker_id);
        tokio::fs::create_dir_all(&self.metadata_root)
            .await
            .map_err(io_error)?;
        let path_text = path.to_string_lossy().into_owned();
        self.git_with_options(
            &[
                "worktree",
                "add",
                "-b",
                &branch,
                &path_text,
                &request.target_ref,
            ],
            &self.primary,
            cancellation,
        )
        .await?;
        let metadata = WorktreeMetadata {
            worker_id: request.worker_id,
            worktree_path: path.clone(),
            branch,
            target_ref: request.target_ref,
            expected_target_head: target_head.clone(),
            base_commit: target_head,
            proposed_head: None,
            lease: format!("{}", std::process::id()),
        };
        self.save_metadata(&metadata).await?;
        Ok(AllocatedWorktree { path, metadata })
    }

    pub async fn status(
        &self,
        metadata: &WorktreeMetadata,
        cancellation: CancellationToken,
    ) -> Result<WorktreeStatus, ToolError> {
        self.authorize("worktree_status", &metadata.worker_id)
            .await?;
        if !metadata.worktree_path.is_dir() {
            return Ok(WorktreeStatus::Missing);
        }
        let current = self
            .rev_parse_at(&metadata.worktree_path, "HEAD", cancellation.clone())
            .await?;
        if current != metadata.base_commit && metadata.proposed_head.as_deref() != Some(&current) {
            return Ok(WorktreeStatus::Stale);
        }
        let dirty = self
            .git(
                &["status", "--porcelain"],
                &metadata.worktree_path,
                cancellation,
            )
            .await?;
        Ok(if dirty.is_empty() {
            WorktreeStatus::Ready
        } else {
            WorktreeStatus::Dirty
        })
    }

    pub async fn review(
        &self,
        metadata: &WorktreeMetadata,
        cancellation: CancellationToken,
    ) -> Result<WorktreeReview, ToolError> {
        self.authorize("worktree_review", &metadata.worker_id)
            .await?;
        let _guard = self.admin_lock.lock().await;
        let proposed_head = self
            .rev_parse_at(&metadata.worktree_path, "HEAD", cancellation.clone())
            .await?;
        let diff = self
            .git_with_options(
                &[
                    "diff",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--no-color",
                    &metadata.base_commit,
                    &proposed_head,
                ],
                &metadata.worktree_path,
                cancellation,
            )
            .await?;
        let evidence_digest = format!(
            "{}:{}:{}",
            metadata.base_commit,
            proposed_head,
            digest(&diff)
        );
        let mut reviewed = metadata.clone();
        reviewed.proposed_head = Some(proposed_head.clone());
        self.save_metadata(&reviewed).await?;
        Ok(WorktreeReview {
            status: WorktreeStatus::Ready,
            base_commit: metadata.base_commit.clone(),
            proposed_head,
            diff,
            evidence_digest,
        })
    }

    pub async fn integrate_without_human_authorization(
        &self,
        _metadata: &WorktreeMetadata,
        _evidence: String,
        _cancellation: CancellationToken,
    ) -> Result<(), ToolError> {
        Err(ToolError::Domain(DomainError::PermissionDenied {
            operation: "worktree_integrate".into(),
        }))
    }

    pub async fn integrate(
        &self,
        metadata: &WorktreeMetadata,
        evidence: String,
        authorization: HumanIntegrationAuthorization,
        cancellation: CancellationToken,
    ) -> Result<(), ToolError> {
        self.authorize("worktree_integrate", &metadata.target_ref)
            .await?;
        let owned_head = self
            .rev_parse_at(&metadata.worktree_path, "HEAD", cancellation.clone())
            .await?;
        if authorization.target_ref != metadata.target_ref
            || authorization.expected_target_head != metadata.expected_target_head
            || authorization.proposed_head != owned_head
            || authorization.evidence_digest != evidence
        {
            return Err(ToolError::Failed(
                "human integration authorization does not match reviewed evidence".into(),
            ));
        }
        let _guard = self.admin_lock.lock().await;
        let current = self
            .rev_parse(&metadata.target_ref, cancellation.clone())
            .await?;
        if current != metadata.expected_target_head {
            return Err(ToolError::Failed(
                "stale target HEAD; integration conflict preserved".into(),
            ));
        }
        self.git_with_options(
            &["merge", "--ff-only", &authorization.proposed_head],
            &self.primary,
            cancellation,
        )
        .await
        .map(|_| ())
    }

    pub async fn reconcile(
        &self,
        cancellation: CancellationToken,
    ) -> Result<Vec<WorktreeMetadata>, ToolError> {
        self.authorize("worktree_reconcile", "owned-worktrees")
            .await?;
        let mut retained = Vec::new();
        let mut entries = match tokio::fs::read_dir(&self.metadata_root).await {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(retained),
            Err(e) => return Err(io_error(e)),
        };
        while let Some(entry) = entries.next_entry().await.map_err(io_error)? {
            if !entry.file_type().await.map_err(io_error)?.is_file() {
                continue;
            }
            let metadata = self.load_metadata(&entry.path()).await?;
            let _ = self.status(&metadata, cancellation.clone()).await?;
            retained.push(metadata);
        }
        Ok(retained)
    }

    async fn authorize(&self, operation: &str, target: &str) -> Result<(), ToolError> {
        if self.permissions.authorize(operation, target).await? != PermissionDecision::Allow {
            return Err(ToolError::Domain(DomainError::PermissionDenied {
                operation: operation.into(),
            }));
        }
        Ok(())
    }

    async fn rev_parse(
        &self,
        reference: &str,
        cancellation: CancellationToken,
    ) -> Result<String, ToolError> {
        self.rev_parse_at(&self.primary, reference, cancellation)
            .await
    }

    async fn rev_parse_at(
        &self,
        cwd: &Path,
        reference: &str,
        cancellation: CancellationToken,
    ) -> Result<String, ToolError> {
        self.git(&["rev-parse", "--verify", reference], cwd, cancellation)
            .await
            .map(|s| s.trim().into())
    }

    async fn git(
        &self,
        args: &[&str],
        cwd: &Path,
        cancellation: CancellationToken,
    ) -> Result<String, ToolError> {
        self.git_with_options(args, cwd, cancellation).await
    }

    async fn git_with_options(
        &self,
        args: &[&str],
        cwd: &Path,
        cancellation: CancellationToken,
    ) -> Result<String, ToolError> {
        let mut command = Command::new("git");
        let mut argv = vec![
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.attributesFile=/dev/null",
            "-c",
            "diff.external=",
            "-c",
            "diff.trustExitCode=false",
        ];
        argv.extend_from_slice(args);
        command
            .args(argv)
            .current_dir(cwd)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_OPTIONAL_LOCKS", "0");
        let output = tokio::select! {
            _ = cancellation.cancelled() => return Err(ToolError::Domain(DomainError::Cancelled)),
            result = command.output() => result.map_err(io_error)?,
        };
        if !output.status.success() {
            return Err(ToolError::Failed(
                String::from_utf8_lossy(&output.stderr).trim().into(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn save_metadata(&self, metadata: &WorktreeMetadata) -> Result<(), ToolError> {
        let path = self
            .metadata_root
            .join(format!("{}.json", metadata.worker_id));
        let bytes =
            serde_json::to_vec_pretty(metadata).map_err(|e| ToolError::Failed(e.to_string()))?;
        tokio::fs::write(path, bytes).await.map_err(io_error)
    }

    async fn load_metadata(&self, path: &Path) -> Result<WorktreeMetadata, ToolError> {
        let bytes = tokio::fs::read(path).await.map_err(io_error)?;
        serde_json::from_slice(&bytes)
            .map_err(|e| ToolError::Failed(format!("invalid worktree metadata: {e}")))
    }
}

fn safe_name(name: &str) -> Result<String, ToolError> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(ToolError::Failed("invalid worker id".into()));
    }
    Ok(name.into())
}

fn digest(value: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
fn io_error(error: std::io::Error) -> ToolError {
    ToolError::Failed(error.to_string())
}
