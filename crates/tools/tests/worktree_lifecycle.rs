use devfoundry_schema::DomainError;
use devfoundry_tools::{
    AllowAllPermissions, PermissionBroker, ToolError, WorktreeManager, WorktreeRequest,
    WorktreeStatus,
};
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio_util::sync::CancellationToken;

fn git(root: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap())
        .status()
        .unwrap();
    assert!(status.success(), "git {:?} failed: {status}", args);
}

fn repo() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "--initial-branch=main"]);
    git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "W08 Test"]);
    std::fs::write(root.path().join("README.md"), "base\n").unwrap();
    git(root.path(), &["add", "README.md"]);
    git(root.path(), &["commit", "-m", "base"]);
    root
}

fn manager(root: &Path, permissions: Arc<dyn PermissionBroker>) -> WorktreeManager {
    let metadata = root.parent().unwrap().join(format!(
        ".w08-{}",
        root.file_name().unwrap().to_string_lossy()
    ));
    WorktreeManager::new(root.to_path_buf(), metadata, permissions).unwrap()
}

#[tokio::test]
async fn dirty_primary_checkout_is_rejected_without_stashing_or_resetting() {
    let root = repo();
    std::fs::write(root.path().join("README.md"), "user changes\n").unwrap();
    let manager = manager(root.path(), Arc::new(AllowAllPermissions));

    let result = manager
        .allocate(
            WorktreeRequest::new("worker-1", "main"),
            CancellationToken::new(),
        )
        .await;

    assert!(matches!(result, Err(ToolError::Failed(message)) if message.contains("dirty")));
    assert_eq!(
        std::fs::read_to_string(root.path().join("README.md")).unwrap(),
        "user changes\n"
    );
}

#[tokio::test]
async fn allocation_requires_exact_expected_target_head() {
    let root = repo();
    let manager = manager(root.path(), Arc::new(AllowAllPermissions));
    let result = manager
        .allocate(
            WorktreeRequest::new("worker-1", "main").with_expected_target_head("deadbeef"),
            CancellationToken::new(),
        )
        .await;

    assert!(
        matches!(result, Err(ToolError::Failed(message)) if message.contains("expected target HEAD"))
    );
}

#[tokio::test]
async fn hostile_git_integrations_are_disabled_by_default() {
    let root = repo();
    let hostile = tempfile::tempdir().unwrap();
    let marker = root.path().join("hook-ran");
    let hook_dir = hostile.path().join("hooks");
    std::fs::create_dir(&hook_dir).unwrap();
    std::fs::write(
        hook_dir.join("post-checkout"),
        format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            hook_dir.join("post-checkout"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }
    git(root.path(), &["config", "core.hooksPath", "hooks"]);
    git(
        root.path(),
        &["config", "core.fsmonitor", "./fsmonitor-helper"],
    );
    git(root.path(), &["config", "diff.external", "./diff-helper"]);

    let first_manager = manager(root.path(), Arc::new(AllowAllPermissions));
    let allocated = first_manager
        .allocate(
            WorktreeRequest::new("worker-1", "main"),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(!marker.exists());

    let review = first_manager
        .review(&allocated.metadata, CancellationToken::new())
        .await
        .unwrap();
    assert!(review.diff.is_empty());
    assert!(!marker.exists());
}

#[tokio::test]
async fn stale_target_is_preserved_and_reported_as_conflict() {
    let root = repo();
    let manager = manager(root.path(), Arc::new(AllowAllPermissions));
    let allocated = manager
        .allocate(
            WorktreeRequest::new("worker-1", "main"),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    std::fs::write(allocated.path.join("README.md"), "worker\n").unwrap();
    git(&allocated.path, &["add", "README.md"]);
    git(&allocated.path, &["commit", "-m", "worker change"]);

    std::fs::write(root.path().join("README.md"), "primary\n").unwrap();
    git(root.path(), &["add", "README.md"]);
    git(root.path(), &["commit", "-m", "primary change"]);

    let review = manager
        .review(&allocated.metadata, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(review.status, WorktreeStatus::Ready);
    let result = manager
        .integrate(
            &allocated.metadata,
            review.evidence_digest.clone(),
            devfoundry_tools::HumanIntegrationAuthorization::new(
                "main",
                allocated.metadata.expected_target_head.clone(),
                review.proposed_head.clone(),
                review.evidence_digest,
            ),
            CancellationToken::new(),
        )
        .await;
    assert!(
        matches!(result, Err(ToolError::Failed(message)) if message.contains("stale") || message.contains("conflict"))
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("README.md")).unwrap(),
        "primary\n"
    );
}

#[tokio::test]
async fn integration_requires_separate_human_authorization() {
    let root = repo();
    let manager = manager(root.path(), Arc::new(AllowAllPermissions));
    let allocated = manager
        .allocate(
            WorktreeRequest::new("worker-1", "main"),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    let review = manager
        .review(&allocated.metadata, CancellationToken::new())
        .await
        .unwrap();

    let result = manager
        .integrate_without_human_authorization(
            &allocated.metadata,
            review.evidence_digest,
            CancellationToken::new(),
        )
        .await;
    assert!(
        matches!(result, Err(ToolError::Domain(DomainError::PermissionDenied { operation })) if operation == "worktree_integrate")
    );
}

#[tokio::test]
async fn cancelled_allocation_does_not_create_owned_worktree() {
    let root = repo();
    let manager = manager(root.path(), Arc::new(AllowAllPermissions));
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let result = manager
        .allocate(WorktreeRequest::new("worker-1", "main"), cancellation)
        .await;
    assert!(matches!(
        result,
        Err(ToolError::Domain(DomainError::Cancelled))
    ));
    assert!(
        !root
            .path()
            .parent()
            .unwrap()
            .join(format!(
                ".w08-{}/worker-1",
                root.path().file_name().unwrap().to_string_lossy()
            ))
            .exists()
    );
}

#[tokio::test]
async fn unknown_worktree_status_is_reported_without_cleanup() {
    let root = repo();
    let manager = manager(root.path(), Arc::new(AllowAllPermissions));
    let allocated = manager
        .allocate(
            WorktreeRequest::new("worker-1", "main"),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    std::fs::remove_dir_all(&allocated.path).unwrap();
    assert_eq!(
        manager
            .status(&allocated.metadata, CancellationToken::new())
            .await
            .unwrap(),
        WorktreeStatus::Missing
    );
    assert!(!allocated.path.exists());
}

#[tokio::test]
async fn reconcile_after_restart_retains_orphan_metadata() {
    let root = repo();
    let first_manager = manager(root.path(), Arc::new(AllowAllPermissions));
    let allocated = first_manager
        .allocate(
            WorktreeRequest::new("worker-1", "main"),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    std::fs::remove_dir_all(&allocated.path).unwrap();
    let restarted = manager(root.path(), Arc::new(AllowAllPermissions));
    let retained = restarted.reconcile(CancellationToken::new()).await.unwrap();
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].worker_id, "worker-1");
}
