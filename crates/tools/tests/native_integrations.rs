use async_trait::async_trait;
use devfoundry_schema::DomainError;
use devfoundry_tools::{
    AllowAllPermissions, LspDiagnostic, LspDiagnosticSeverity, LspProcessConfig, LspSession,
    PermissionBroker, PermissionDecision, PtyTool, Tool, ToolContext, ToolRequest,
};
#[cfg(target_os = "macos")]
use devfoundry_tools::{IntegrationCapability, IntegrationDescriptor, IntegrationKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_local-integration-fixture"))
}

fn context(root: &Path, permissions: Arc<dyn PermissionBroker>) -> ToolContext {
    ToolContext {
        root: root.to_path_buf(),
        permissions,
        cancellation: CancellationToken::new(),
    }
}

#[tokio::test]
async fn lsp_fixture_process_and_diagnostics_are_local_and_bounded() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("main.rs");
    std::fs::write(&source, "fn main() {}\n").unwrap();
    let cancellation = CancellationToken::new();
    let session = LspSession::start(
        LspProcessConfig {
            executable: fixture(),
            arguments: vec!["lsp".into()],
            workspace_root: root.path().to_path_buf(),
        },
        Arc::new(AllowAllPermissions),
        cancellation.clone(),
    )
    .await
    .unwrap();
    session
        .publish_diagnostics(
            Path::new("main.rs"),
            vec![LspDiagnostic {
                path: source.canonicalize().unwrap(),
                line: 1,
                column: 1,
                severity: LspDiagnosticSeverity::Warning,
                message: "local fixture warning".into(),
            }],
        )
        .await
        .unwrap();
    assert_eq!(session.diagnostics().await.len(), 1);
    cancellation.cancel();
    tokio::time::timeout(std::time::Duration::from_secs(2), session.shutdown())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn pty_contract_runs_local_command_with_bounds_and_permission() {
    let root = tempfile::tempdir().unwrap();
    let output = PtyTool
        .execute(
            ToolRequest {
                name: "pty".into(),
                arguments: serde_json::json!({"command": "printf local-pty"}),
            },
            context(root.path(), Arc::new(AllowAllPermissions)),
        )
        .await
        .unwrap();
    assert_eq!(output.text, "local-pty");
    assert!(!output.truncated);
}

#[tokio::test]
async fn pty_contract_bounds_local_output() {
    let root = tempfile::tempdir().unwrap();
    let output = PtyTool
        .execute(
            ToolRequest {
                name: "pty".into(),
                arguments: serde_json::json!({"command": "yes x | head -c 100000"}),
            },
            context(root.path(), Arc::new(AllowAllPermissions)),
        )
        .await
        .unwrap();
    assert!(output.truncated);
    assert!(output.text.len() <= 64 * 1024);
}

#[tokio::test]
async fn pty_contract_denies_before_spawning() {
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
    let result = PtyTool
        .execute(
            ToolRequest {
                name: "pty".into(),
                arguments: serde_json::json!({"command": "printf should-not-run"}),
            },
            context(root.path(), Arc::new(Deny)),
        )
        .await;
    assert!(matches!(
        result,
        Err(devfoundry_tools::ToolError::Domain(DomainError::PermissionDenied { operation }))
            if operation == "pty_session"
    ));
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn mcp_fixture_completes_initialize_and_tool_call_over_stdio() {
    use devfoundry_tools::{McpClient, McpServerConfig};
    let descriptor = IntegrationDescriptor {
        kind: IntegrationKind::Mcp,
        name: "local-fixture".into(),
        capabilities: vec![IntegrationCapability::McpTool],
    };
    let mut client = McpClient::connect(
        McpServerConfig {
            descriptor,
            program: fixture().to_string_lossy().into_owned(),
            args: vec!["mcp".into()],
        },
        Arc::new(AllowAllPermissions),
    )
    .await
    .unwrap();
    let result = client
        .call_tool("local", serde_json::json!({}))
        .await
        .unwrap();
    assert!(!result.is_error);
    assert_eq!(result.content[0]["text"], "local fixture");
    client.shutdown().await.unwrap();
}
