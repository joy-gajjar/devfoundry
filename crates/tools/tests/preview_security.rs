use devfoundry_tools::{
    AllowAllPermissions, PermissionBroker, PermissionDecision, PreviewLaunch, PreviewProcess,
    ToolContext, ToolError,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

fn context(root: &Path, permissions: Arc<dyn PermissionBroker>) -> ToolContext {
    ToolContext {
        root: root.to_path_buf(),
        permissions,
        cancellation: CancellationToken::new(),
    }
}

fn launch(worktree: PathBuf, artifact_root: PathBuf) -> PreviewLaunch {
    PreviewLaunch {
        executable: "/bin/sh".into(),
        arguments: vec!["-c".into(), "sleep 30".into()],
        worktree,
        artifact_root,
        port: 4173,
        readiness_deadline: Duration::from_millis(100),
    }
}

#[tokio::test]
async fn preview_start_rejects_artifact_root_outside_owned_worktree() {
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let error = PreviewProcess::start(
        launch(project.path().to_path_buf(), outside.path().to_path_buf()),
        context(project.path(), Arc::new(AllowAllPermissions)),
    )
    .await
    .unwrap_err();

    assert!(matches!(error, ToolError::Failed(message) if message.contains("artifact root")));
}

#[tokio::test]
async fn denied_preview_start_does_not_spawn() {
    struct Deny;
    #[async_trait::async_trait]
    impl PermissionBroker for Deny {
        async fn authorize(
            &self,
            operation: &str,
            _target: &str,
        ) -> Result<PermissionDecision, devfoundry_schema::DomainError> {
            assert_eq!(operation, "preview_start");
            Ok(PermissionDecision::Deny)
        }
    }

    let project = tempfile::tempdir().unwrap();
    let artifact = project.path().join("dist");
    tokio::fs::create_dir(&artifact).await.unwrap();
    let error = PreviewProcess::start(
        launch(project.path().to_path_buf(), artifact),
        context(project.path(), Arc::new(Deny)),
    )
    .await
    .unwrap_err();

    assert!(matches!(error, ToolError::Domain(_)));
}

#[tokio::test]
async fn stop_reaps_preview_process() {
    let project = tempfile::tempdir().unwrap();
    let artifact = project.path().join("dist");
    tokio::fs::create_dir(&artifact).await.unwrap();
    let process = PreviewProcess::start(
        launch(project.path().to_path_buf(), artifact),
        context(project.path(), Arc::new(AllowAllPermissions)),
    )
    .await
    .unwrap();

    process.stop().await.unwrap();
    assert!(process.is_stopped().await);
}
