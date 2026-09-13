//! Permission-gated, read-only LSP lifecycle and diagnostics seam.
//!
//! This module deliberately speaks in structured process arguments. It never
//! invokes a shell, accepts an environment map, or turns project text into a
//! command. A future JSON-RPC adapter can use `LspSession` without widening
//! these process and workspace boundaries.

use crate::{
    IntegrationCapability, IntegrationKind, IntegrationRequest, PermissionBroker,
    PermissionDecision,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub const MAX_LSP_DIAGNOSTICS: usize = 1_000;
pub const MAX_LSP_MESSAGE_BYTES: usize = 64 * 1024;
pub const MAX_LSP_OUTPUT_BYTES: usize = 64 * 1024;
const MAX_LSP_ARGUMENT_BYTES: usize = 4 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspProcessConfig {
    /// An absolute executable path. Shell syntax and PATH lookup are not used.
    pub executable: PathBuf,
    /// Fixed argv entries supplied by trusted configuration, excluding argv[0].
    pub arguments: Vec<String>,
    pub workspace_root: PathBuf,
}

impl LspProcessConfig {
    pub fn validate(&self) -> Result<(PathBuf, PathBuf), LspError> {
        if !self.executable.is_absolute() {
            return Err(LspError::InvalidConfig(
                "executable must be an absolute path".into(),
            ));
        }
        let executable = self
            .executable
            .canonicalize()
            .map_err(LspError::Executable)?;
        if !executable.is_file() {
            return Err(LspError::InvalidConfig("executable is not a file".into()));
        }
        let root = self
            .workspace_root
            .canonicalize()
            .map_err(LspError::Workspace)?;
        if !root.is_dir() {
            return Err(LspError::InvalidConfig(
                "workspace root is not a directory".into(),
            ));
        }
        if self.arguments.iter().any(|argument| {
            argument.is_empty()
                || argument.len() > MAX_LSP_ARGUMENT_BYTES
                || argument.chars().any(char::is_control)
        }) {
            return Err(LspError::InvalidConfig("invalid LSP argument".into()));
        }
        Ok((executable, root))
    }

    pub fn validate_workspace_path(&self, path: &Path) -> Result<PathBuf, LspError> {
        let (_, root) = self.validate()?;
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        let canonical = candidate.canonicalize().map_err(LspError::Workspace)?;
        if !canonical.starts_with(&root) {
            return Err(LspError::WorkspaceEscape);
        }
        Ok(canonical)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LspDiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub severity: LspDiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("invalid LSP configuration: {0}")]
    InvalidConfig(String),
    #[error("failed to inspect LSP executable: {0}")]
    Executable(std::io::Error),
    #[error("failed to inspect LSP workspace: {0}")]
    Workspace(std::io::Error),
    #[error("LSP path escapes workspace root")]
    WorkspaceEscape,
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("LSP process failed: {0}")]
    Process(#[source] std::io::Error),
    #[error("operation cancelled")]
    Cancelled,
    #[error("diagnostic payload is invalid or exceeds its bound")]
    InvalidDiagnostics,
}

async fn lsp_notify(
    stdin: &Arc<Mutex<ChildStdin>>,
    method: &str,
    params: Value,
) -> Result<(), LspError> {
    let body =
        serde_json::to_vec(&serde_json::json!({"jsonrpc":"2.0","method":method,"params":params}))
            .map_err(|error| LspError::InvalidConfig(error.to_string()))?;
    if body.len() > MAX_LSP_MESSAGE_BYTES {
        return Err(LspError::InvalidDiagnostics);
    }
    let mut stdin = stdin.lock().await;
    stdin
        .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
        .await
        .map_err(LspError::Process)?;
    stdin.write_all(&body).await.map_err(LspError::Process)?;
    stdin.flush().await.map_err(LspError::Process)
}

async fn lsp_request(
    stdin: &Arc<Mutex<ChildStdin>>,
    stdout: &Arc<Mutex<ChildStdout>>,
    id: u64,
    method: &str,
    params: Value,
) -> Result<Value, LspError> {
    let body = serde_json::to_vec(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    }))
    .map_err(|error| LspError::InvalidConfig(error.to_string()))?;
    if body.len() > MAX_LSP_MESSAGE_BYTES {
        return Err(LspError::InvalidDiagnostics);
    }
    let mut stdin = stdin.lock().await;
    stdin
        .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
        .await
        .map_err(LspError::Process)?;
    stdin.write_all(&body).await.map_err(LspError::Process)?;
    stdin.flush().await.map_err(LspError::Process)?;
    drop(stdin);
    let mut stdout = stdout.lock().await;
    let mut headers = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        stdout
            .read_exact(&mut byte)
            .await
            .map_err(LspError::Process)?;
        headers.push(byte[0]);
        if headers.ends_with(b"\r\n\r\n") {
            break;
        }
        if headers.len() > 8 * 1024 {
            return Err(LspError::InvalidDiagnostics);
        }
    }
    let header = String::from_utf8(headers.clone()).map_err(|_| LspError::InvalidDiagnostics)?;
    let length = header
        .lines()
        .find_map(|line| {
            line.strip_prefix("Content-Length:")?
                .trim()
                .parse::<usize>()
                .ok()
        })
        .ok_or(LspError::InvalidDiagnostics)?;
    if length > MAX_LSP_MESSAGE_BYTES {
        return Err(LspError::InvalidDiagnostics);
    }
    let mut body = vec![0; length];
    stdout
        .read_exact(&mut body)
        .await
        .map_err(LspError::Process)?;
    let mut value: Value =
        serde_json::from_slice(&body).map_err(|_| LspError::InvalidDiagnostics)?;
    while value.get("id").is_none() {
        headers.clear();
        loop {
            stdout
                .read_exact(&mut byte)
                .await
                .map_err(LspError::Process)?;
            headers.push(byte[0]);
            if headers.ends_with(b"\r\n\r\n") {
                break;
            }
            if headers.len() > 8 * 1024 {
                return Err(LspError::InvalidDiagnostics);
            }
        }
        let header =
            String::from_utf8(headers.clone()).map_err(|_| LspError::InvalidDiagnostics)?;
        let length = header
            .lines()
            .find_map(|line| {
                line.strip_prefix("Content-Length:")?
                    .trim()
                    .parse::<usize>()
                    .ok()
            })
            .ok_or(LspError::InvalidDiagnostics)?;
        if length > MAX_LSP_MESSAGE_BYTES {
            return Err(LspError::InvalidDiagnostics);
        }
        let mut next = vec![0; length];
        stdout
            .read_exact(&mut next)
            .await
            .map_err(LspError::Process)?;
        value = serde_json::from_slice(&next).map_err(|_| LspError::InvalidDiagnostics)?;
    }
    if value.get("id") != Some(&Value::from(id)) {
        return Err(LspError::InvalidDiagnostics);
    }
    if let Some(error) = value.get("error") {
        return Err(LspError::InvalidConfig(error.to_string()));
    }
    Ok(value.get("result").cloned().unwrap_or(Value::Null))
}

async fn drain_bounded<R: AsyncRead + Unpin>(reader: R) -> std::io::Result<()> {
    let mut reader = reader;
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            return Ok(());
        }
    }
}

#[cfg(unix)]
async fn terminate_process_group(child: &mut Child) -> std::io::Result<()> {
    let Some(pid) = child.id() else {
        return Ok(());
    };
    let result = unsafe { libc::kill(-(pid as libc::pid_t), libc::SIGKILL) };
    let error = std::io::Error::last_os_error();
    if result == 0 || error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

#[cfg(not(unix))]
async fn terminate_process_group(child: &mut Child) -> std::io::Result<()> {
    child.kill().await
}

pub struct LspSession {
    child: Arc<Mutex<Option<Child>>>,
    cancellation: CancellationToken,
    diagnostics: Arc<Mutex<Vec<LspDiagnostic>>>,
    cancellation_task: Option<tokio::task::JoinHandle<()>>,
    output_tasks: Vec<tokio::task::JoinHandle<()>>,
    workspace_root: PathBuf,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<ChildStdout>>,
    next_id: Arc<Mutex<u64>>,
}

impl LspSession {
    pub async fn start(
        config: LspProcessConfig,
        permissions: Arc<dyn PermissionBroker>,
        cancellation: CancellationToken,
    ) -> Result<Self, LspError> {
        let (executable, workspace_root) = config.validate()?;
        let request = IntegrationRequest {
            kind: IntegrationKind::Lsp,
            capability: IntegrationCapability::LspDiagnostics,
            target: executable.to_string_lossy().into_owned(),
        };
        request
            .validate()
            .map_err(|error| LspError::InvalidConfig(error.to_string()))?;
        if permissions
            .authorize(request.permission_operation(), &request.target)
            .await
            .map_err(|error| LspError::PermissionDenied(error.to_string()))?
            != PermissionDecision::Allow
        {
            return Err(LspError::PermissionDenied(
                request.permission_operation().into(),
            ));
        }
        if cancellation.is_cancelled() {
            return Err(LspError::Cancelled);
        }

        let mut command = Command::new(&executable);
        command
            .args(&config.arguments)
            .current_dir(&workspace_root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(unix)]
        command.process_group(0);
        let child = command.spawn().map_err(LspError::Process)?;
        let mut child = child;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| LspError::Process(std::io::Error::other("missing LSP stdout")))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| LspError::Process(std::io::Error::other("missing LSP stderr")))?;
        let output_tasks = vec![tokio::spawn(async move {
            let _ = drain_bounded(stderr).await;
        })];
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LspError::Process(std::io::Error::other("missing LSP stdin")))?;
        let stdout = Arc::new(Mutex::new(stdout));
        let stdin = Arc::new(Mutex::new(stdin));
        if !config.arguments.is_empty() {
            lsp_request(&stdin, &stdout, 1, "initialize", serde_json::json!({
                "processId": std::process::id(), "rootUri": workspace_root.to_string_lossy(),
                "capabilities": {}, "clientInfo": {"name": "devfoundry", "version": env!("CARGO_PKG_VERSION")}
            })).await?;
            lsp_notify(&stdin, "initialized", serde_json::json!({})).await?;
        }
        let child = Arc::new(Mutex::new(Some(child)));
        let watcher_child = Arc::clone(&child);
        let watcher_token = cancellation.clone();
        let cancellation_task = tokio::spawn(async move {
            watcher_token.cancelled().await;
            let mut child = watcher_child.lock().await;
            if let Some(child) = child.as_mut() {
                let _ = terminate_process_group(child).await;
            }
        });
        Ok(Self {
            child,
            cancellation,
            diagnostics: Arc::new(Mutex::new(Vec::new())),
            cancellation_task: Some(cancellation_task),
            output_tasks,
            workspace_root,
            stdin,
            stdout,
            next_id: Arc::new(Mutex::new(2)),
        })
    }

    pub async fn publish_diagnostics(
        &self,
        path: &Path,
        diagnostics: Vec<LspDiagnostic>,
    ) -> Result<(), LspError> {
        if self.cancellation.is_cancelled() {
            return Err(LspError::Cancelled);
        }
        let path = self.validate_workspace_path(path)?;
        if diagnostics.len() > MAX_LSP_DIAGNOSTICS
            || diagnostics.iter().any(|diagnostic| {
                diagnostic.message.is_empty()
                    || diagnostic.message.len() > MAX_LSP_MESSAGE_BYTES
                    || diagnostic.message.chars().any(char::is_control)
                    || diagnostic.path != path
            })
        {
            return Err(LspError::InvalidDiagnostics);
        }
        *self.diagnostics.lock().await = diagnostics;
        Ok(())
    }

    pub async fn diagnostics(&self) -> Vec<LspDiagnostic> {
        self.diagnostics.lock().await.clone()
    }

    pub async fn document(&self, path: &Path, text: &str, version: i32) -> Result<(), LspError> {
        if text.len() > MAX_LSP_OUTPUT_BYTES {
            return Err(LspError::InvalidDiagnostics);
        }
        let path = self.validate_workspace_path(path)?;
        let uri = format!("file://{}", path.display());
        let id = {
            let mut id = self.next_id.lock().await;
            let current = *id;
            *id += 1;
            current
        };
        let result = lsp_request(
            &self.stdin,
            &self.stdout,
            id,
            "textDocument/diagnostic",
            serde_json::json!({
                "textDocument": {"uri": uri, "version": version, "text": text}
            }),
        )
        .await?;
        if result.get("items").and_then(Value::as_array).is_none() {
            return Err(LspError::InvalidDiagnostics);
        }
        Ok(())
    }

    pub async fn shutdown(mut self) -> Result<(), LspError> {
        if !self.cancellation.is_cancelled() {
            let id = {
                let mut id = self.next_id.lock().await;
                let current = *id;
                *id += 1;
                current
            };
            let _ = lsp_request(
                &self.stdin,
                &self.stdout,
                id,
                "shutdown",
                serde_json::Value::Null,
            )
            .await;
            let _ = lsp_notify(&self.stdin, "exit", serde_json::json!({})).await;
        }
        self.cancellation.cancel();
        if let Some(task) = self.cancellation_task.take() {
            task.abort();
        }
        let mut child = self.child.lock().await;
        if let Some(child) = child.as_mut() {
            let _ = terminate_process_group(child).await;
            let _ = child.kill().await;
        }
        child.take();
        for task in self.output_tasks.drain(..) {
            task.abort();
        }
        Ok(())
    }

    fn validate_workspace_path(&self, path: &Path) -> Result<PathBuf, LspError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        };
        let canonical = candidate.canonicalize().map_err(LspError::Workspace)?;
        if !canonical.starts_with(&self.workspace_root) {
            return Err(LspError::WorkspaceEscape);
        }
        Ok(canonical)
    }
}

impl Drop for LspSession {
    fn drop(&mut self) {
        self.cancellation.cancel();
        if let Some(task) = self.cancellation_task.take() {
            task.abort();
        }
        for task in self.output_tasks.drain(..) {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AllowAllPermissions, PermissionDecision};
    use async_trait::async_trait;
    use std::fs;

    fn config(root: &Path) -> LspProcessConfig {
        LspProcessConfig {
            executable: std::env::current_exe().unwrap(),
            arguments: vec![],
            workspace_root: root.to_path_buf(),
        }
    }
    fn diagnostic(path: PathBuf) -> LspDiagnostic {
        LspDiagnostic {
            path,
            line: 0,
            column: 0,
            severity: LspDiagnosticSeverity::Error,
            message: "problem".into(),
        }
    }

    #[test]
    fn config_rejects_relative_executable_and_control_arguments() {
        let root = tempfile::tempdir().unwrap();
        let mut invalid = config(root.path());
        invalid.executable = PathBuf::from("lsp");
        assert!(matches!(
            invalid.validate(),
            Err(LspError::InvalidConfig(_))
        ));
        invalid.executable = std::env::current_exe().unwrap();
        invalid.arguments = vec!["bad\narg".into()];
        assert!(matches!(
            invalid.validate(),
            Err(LspError::InvalidConfig(_))
        ));
    }

    #[tokio::test]
    async fn permission_is_required_before_process_start() {
        let root = tempfile::tempdir().unwrap();
        struct Deny;
        #[async_trait]
        impl PermissionBroker for Deny {
            async fn authorize(
                &self,
                operation: &str,
                _target: &str,
            ) -> Result<PermissionDecision, devfoundry_schema::DomainError> {
                assert_eq!(operation, "lsp_diagnostics");
                Ok(PermissionDecision::Deny)
            }
        }
        let session = LspSession::start(
            config(root.path()),
            Arc::new(Deny),
            CancellationToken::new(),
        )
        .await;
        assert!(matches!(session, Err(LspError::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn diagnostics_are_contained_and_bounded() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("main.rs");
        fs::write(&file, "fn main() {}").unwrap();
        let session = LspSession::start(
            config(root.path()),
            Arc::new(AllowAllPermissions),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        session
            .publish_diagnostics(
                Path::new("main.rs"),
                vec![diagnostic(file.canonicalize().unwrap())],
            )
            .await
            .unwrap();
        assert_eq!(session.diagnostics().await.len(), 1);
        let outside = tempfile::NamedTempFile::new().unwrap();
        assert!(matches!(
            session.publish_diagnostics(outside.path(), vec![]).await,
            Err(LspError::WorkspaceEscape)
        ));
        session.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn cancellation_stops_the_process() {
        let root = tempfile::tempdir().unwrap();
        let token = CancellationToken::new();
        let session = LspSession::start(
            config(root.path()),
            Arc::new(AllowAllPermissions),
            token.clone(),
        )
        .await
        .unwrap();
        token.cancel();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            session.shutdown().await.unwrap();
        })
        .await
        .unwrap();
    }
}
