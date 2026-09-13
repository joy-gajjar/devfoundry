use async_trait::async_trait;
use devfoundry_schema::DomainError;
use devfoundry_tools::{
    AllowAllPermissions, NativePtyService, PermissionBroker, PermissionDecision, PtyInput,
    PtyOpenRequest, PtyResize, ToolContext,
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

fn shell_request(command: &str) -> PtyOpenRequest {
    PtyOpenRequest {
        program: "/bin/sh".into(),
        arguments: vec!["-c".into(), command.into()],
        cwd: None,
        rows: 24,
        columns: 80,
    }
}

#[tokio::test]
async fn native_pty_reports_native_capability_without_pipe_substitution() {
    let capability = NativePtyService::capability();
    assert!(capability.native);
    assert!(capability.platform.contains("macos") || capability.platform.contains("unix"));
}

#[tokio::test]
async fn native_pty_open_input_and_output_preserve_terminal_bytes() {
    let root = tempfile::tempdir().unwrap();
    let service = NativePtyService::open(
        shell_request("read line; printf 'reply:%s\\n' \"$line\""),
        context(root.path(), Arc::new(AllowAllPermissions)),
    )
    .await
    .unwrap();
    service
        .input(PtyInput::new(service.input_lease(), b"hello\n".to_vec()))
        .await
        .unwrap();
    let mut offset = 0;
    let mut collected = Vec::new();
    for _ in 0..3 {
        let output = service
            .read_output(offset, Duration::from_secs(2))
            .await
            .unwrap();
        collected.extend_from_slice(&output.bytes);
        offset = output.next_offset;
        if collected.windows(11).any(|window| window == b"reply:hello") {
            break;
        }
    }
    assert!(collected.windows(11).any(|window| window == b"reply:hello"));
    service.close().await.unwrap();
}

#[tokio::test]
async fn native_pty_resize_changes_reported_dimensions() {
    let root = tempfile::tempdir().unwrap();
    let service = NativePtyService::open(
        shell_request("sleep 0.1; stty size"),
        context(root.path(), Arc::new(AllowAllPermissions)),
    )
    .await
    .unwrap();
    service
        .resize(PtyResize {
            rows: 40,
            columns: 120,
        })
        .await
        .unwrap();
    let output = service
        .read_output(0, Duration::from_secs(2))
        .await
        .unwrap();
    assert!(String::from_utf8_lossy(&output.bytes).contains("40 120"));
    service.close().await.unwrap();
}

#[tokio::test]
async fn native_pty_permission_denial_prevents_spawn() {
    struct Deny;
    #[async_trait]
    impl PermissionBroker for Deny {
        async fn authorize(
            &self,
            operation: &str,
            _target: &str,
        ) -> Result<PermissionDecision, DomainError> {
            assert_eq!(operation, "pty_session");
            Ok(PermissionDecision::Deny)
        }
    }
    let root = tempfile::tempdir().unwrap();
    let result = NativePtyService::open(
        shell_request("printf should-not-run"),
        context(root.path(), Arc::new(Deny)),
    )
    .await;
    assert!(matches!(
        result,
        Err(devfoundry_tools::NativePtyError::PermissionDenied)
    ));
}

#[tokio::test]
async fn native_pty_rejects_external_cwd_and_unbounded_input() {
    let root = tempfile::tempdir().unwrap();
    let mut request = shell_request("cat");
    request.cwd = Some("../outside".into());
    assert!(
        NativePtyService::open(request, context(root.path(), Arc::new(AllowAllPermissions)))
            .await
            .is_err()
    );

    let service = NativePtyService::open(
        shell_request("cat"),
        context(root.path(), Arc::new(AllowAllPermissions)),
    )
    .await
    .unwrap();
    let oversized = PtyInput::new(service.input_lease(), vec![0; 128 * 1024]);
    assert!(matches!(
        service.input(oversized).await,
        Err(devfoundry_tools::NativePtyError::InputTooLarge)
    ));
    service.close().await.unwrap();
}

#[tokio::test]
async fn native_pty_rejects_stale_input_lease() {
    let root = tempfile::tempdir().unwrap();
    let service = NativePtyService::open(
        shell_request("cat"),
        context(root.path(), Arc::new(AllowAllPermissions)),
    )
    .await
    .unwrap();
    let stale = service.input_lease().next_generation();
    assert!(matches!(
        service.input(PtyInput::new(stale, b"nope".to_vec())).await,
        Err(devfoundry_tools::NativePtyError::InvalidInputLease)
    ));
    service.close().await.unwrap();
}

#[tokio::test]
async fn native_pty_close_cleans_up_and_offset_gap_is_explicit() {
    let root = tempfile::tempdir().unwrap();
    let service = NativePtyService::open(
        shell_request("yes X | head -c 200000"),
        context(root.path(), Arc::new(AllowAllPermissions)),
    )
    .await
    .unwrap();
    let output = service
        .read_output(0, Duration::from_secs(2))
        .await
        .unwrap();
    assert!(output.truncated || output.next_offset <= 64 * 1024);
    tokio::time::sleep(Duration::from_millis(100)).await;
    if output.truncated {
        assert!(service.read_output(0, Duration::ZERO).await.is_err());
    }
    service.close().await.unwrap();
    service.close().await.unwrap();
}

#[tokio::test]
async fn native_pty_cancellation_closes_the_owned_process() {
    let root = tempfile::tempdir().unwrap();
    let cancellation = CancellationToken::new();
    let context = ToolContext {
        root: root.path().to_path_buf(),
        permissions: Arc::new(AllowAllPermissions),
        cancellation: cancellation.clone(),
    };
    let service = NativePtyService::open(shell_request("sleep 30"), context)
        .await
        .unwrap();
    cancellation.cancel();
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(matches!(
        service
            .input(PtyInput::new(service.input_lease(), b"x".to_vec()))
            .await,
        Err(devfoundry_tools::NativePtyError::Cancelled)
    ));
    service.close().await.unwrap();
}

#[allow(dead_code)]
fn _path_type_check(path: PathBuf) -> PathBuf {
    path
}
