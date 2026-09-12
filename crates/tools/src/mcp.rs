//! A deliberately small, macOS-only MCP stdio client.
//!
//! The client is not a general process runner: callers provide one executable
//! and fixed argv values, the server must advertise `tools`, and every tool
//! call passes through the existing permission broker.

use crate::{
    IntegrationCapability, IntegrationDescriptor, IntegrationKind, PermissionBroker,
    PermissionDecision,
};
use devfoundry_schema::DomainError;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{io, path::Path, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    process::{Child, ChildStdin, ChildStdout},
};

const MAX_FRAME_BYTES: usize = 256 * 1024;
const MAX_NAME_BYTES: usize = 128;
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_HEADER_BYTES: usize = 8 * 1024;

#[derive(Clone, Debug)]
pub struct McpServerConfig {
    pub descriptor: IntegrationDescriptor,
    pub program: String,
    pub args: Vec<String>,
}

impl McpServerConfig {
    fn validate(&self) -> Result<(), McpError> {
        self.descriptor.validate().map_err(McpError::Contract)?;
        if self.descriptor.kind != IntegrationKind::Mcp
            || !self
                .descriptor
                .capabilities
                .contains(&IntegrationCapability::McpTool)
        {
            return Err(McpError::Contract(
                crate::IntegrationContractError::CapabilityMismatch,
            ));
        }
        validate_text(&self.program, MAX_NAME_BYTES)?;
        let program = Path::new(&self.program);
        if !program.is_absolute() {
            return Err(McpError::Protocol(
                "MCP executable must be an absolute path".into(),
            ));
        }
        if !program.is_file() {
            return Err(McpError::Protocol(
                "MCP executable must be an existing file".into(),
            ));
        }
        for argument in &self.args {
            validate_text(argument, MAX_FRAME_BYTES)?;
        }
        Ok(())
    }
}

pub struct McpClient {
    stdin: ChildStdin,
    stdout: ChildStdout,
    child: Child,
    permissions: Arc<dyn PermissionBroker>,
    next_id: u64,
    server_name: String,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    #[allow(dead_code)]
    id: Value,
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("MCP is only supported on macOS")]
    UnsupportedPlatform,
    #[error("MCP integration contract rejected: {0}")]
    Contract(#[from] crate::IntegrationContractError),
    #[error("MCP permission denied")]
    PermissionDenied,
    #[error("MCP domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("MCP process failed: {0}")]
    Process(String),
    #[error("MCP protocol error: {0}")]
    Protocol(String),
    #[error("MCP request timed out")]
    Timeout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct McpToolResult {
    pub content: Value,
    pub is_error: bool,
}

impl McpClient {
    pub async fn connect(
        config: McpServerConfig,
        permissions: Arc<dyn PermissionBroker>,
    ) -> Result<Self, McpError> {
        config.validate()?;
        authorize(&permissions, "mcp_tool", &config.descriptor.name).await?;
        let mut command = tokio::process::Command::new(&config.program);
        command
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0);
        let mut child = command
            .spawn()
            .map_err(|error| McpError::Process(error.to_string()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Process("missing MCP stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Process("missing MCP stdout".into()))?;
        let mut client = Self {
            stdin,
            stdout,
            child,
            permissions,
            next_id: 1,
            server_name: config.descriptor.name,
        };
        if let Err(error) = client.initialize().await {
            client.kill().await;
            return Err(error);
        }
        Ok(client)
    }

    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: Value,
    ) -> Result<McpToolResult, McpError> {
        validate_text(name, MAX_NAME_BYTES)?;
        let target = format!("{}/{}", self.server_name, name);
        authorize(&self.permissions, "mcp_tool", &target).await?;
        let response = self
            .request("tools/call", json!({"name": name, "arguments": arguments}))
            .await?;
        Ok(McpToolResult {
            content: response
                .get("content")
                .cloned()
                .ok_or_else(|| McpError::Protocol("tools/call omitted content".into()))?,
            is_error: response
                .get("isError")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
    }

    pub async fn shutdown(mut self) -> Result<(), McpError> {
        self.stdin
            .shutdown()
            .await
            .map_err(|error| McpError::Process(error.to_string()))?;
        if tokio::time::timeout(Duration::from_secs(2), self.child.wait())
            .await
            .is_err()
        {
            self.child
                .start_kill()
                .map_err(|error| McpError::Process(error.to_string()))?;
            self.child
                .wait()
                .await
                .map_err(|error| McpError::Process(error.to_string()))?;
        }
        Ok(())
    }

    async fn initialize(&mut self) -> Result<(), McpError> {
        let result = self
            .request(
                "initialize",
                json!({
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": {"name": "devfoundry", "version": env!("CARGO_PKG_VERSION")}
                }),
            )
            .await?;
        validate_server_capabilities(&result)?;
        self.notify("notifications/initialized", json!({})).await
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value, McpError> {
        self.request_with_timeout(method, params, REQUEST_TIMEOUT)
            .await
    }

    async fn request_with_timeout(
        &mut self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, McpError> {
        let id = self.next_id;
        self.next_id += 1;
        tokio::time::timeout(timeout, async {
            write_frame(
                &mut self.stdin,
                &json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
            )
            .await?;
            read_response(&mut self.stdout, id).await
        })
        .await
        .map_err(|_| McpError::Timeout)?
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<(), McpError> {
        write_frame(
            &mut self.stdin,
            &json!({"jsonrpc": "2.0", "method": method, "params": params}),
        )
        .await
    }

    async fn kill(&mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

async fn authorize(
    permissions: &Arc<dyn PermissionBroker>,
    operation: &str,
    target: &str,
) -> Result<(), McpError> {
    if permissions.authorize(operation, target).await? != PermissionDecision::Allow {
        return Err(McpError::PermissionDenied);
    }
    Ok(())
}

fn validate_text(value: &str, max_bytes: usize) -> Result<(), McpError> {
    if value.is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        return Err(McpError::Protocol(
            "MCP configuration contains an invalid value".into(),
        ));
    }
    Ok(())
}

fn validate_server_capabilities(result: &Value) -> Result<(), McpError> {
    let tools = result
        .get("capabilities")
        .and_then(|value| value.get("tools"));
    if !tools.is_some_and(Value::is_object) {
        return Err(McpError::Protocol(
            "MCP server did not advertise tools capability".into(),
        ));
    }
    Ok(())
}

async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, value: &Value) -> Result<(), McpError> {
    let body = serde_json::to_vec(value).map_err(|error| McpError::Protocol(error.to_string()))?;
    if body.len() > MAX_FRAME_BYTES {
        return Err(McpError::Protocol("MCP request exceeds frame limit".into()));
    }
    writer
        .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
        .await
        .map_err(io_error)?;
    writer.write_all(&body).await.map_err(io_error)?;
    writer.flush().await.map_err(io_error)?;
    Ok(())
}

async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Value, McpError> {
    let mut headers = Vec::new();
    loop {
        let byte = read_byte(reader).await?;
        headers.push(byte);
        if headers.ends_with(b"\r\n\r\n") {
            break;
        }
        if headers.len() > MAX_HEADER_BYTES {
            return Err(McpError::Protocol("MCP headers exceed limit".into()));
        }
    }
    let headers = String::from_utf8(headers)
        .map_err(|_| McpError::Protocol("MCP headers are not UTF-8".into()))?;
    let mut length = None;
    for line in headers.lines() {
        if let Some(value) = line.strip_prefix("Content-Length:") {
            if length.is_some() {
                return Err(McpError::Protocol(
                    "MCP response contains duplicate Content-Length".into(),
                ));
            }
            length = Some(value.trim().parse::<usize>().map_err(|_| {
                McpError::Protocol("MCP response has invalid Content-Length".into())
            })?);
        }
    }
    let length =
        length.ok_or_else(|| McpError::Protocol("MCP response omitted Content-Length".into()))?;
    if length > MAX_FRAME_BYTES {
        return Err(McpError::Protocol(
            "MCP response exceeds frame limit".into(),
        ));
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).await.map_err(io_error)?;
    serde_json::from_slice(&body).map_err(|error| McpError::Protocol(error.to_string()))
}

async fn read_response<R: AsyncRead + Unpin>(reader: &mut R, id: u64) -> Result<Value, McpError> {
    loop {
        let value = read_frame(reader).await?;
        let Some(response_id) = value.get("id") else {
            // Server notifications are legal between a request and its response.
            continue;
        };
        if response_id != &json!(id) {
            return Err(McpError::Protocol(
                "MCP response id did not match request".into(),
            ));
        }
        let response: RpcResponse =
            serde_json::from_value(value).map_err(|error| McpError::Protocol(error.to_string()))?;
        if let Some(error) = response.error {
            return Err(McpError::Protocol(format!(
                "{} ({})",
                error.message, error.code
            )));
        }
        return response
            .result
            .ok_or_else(|| McpError::Protocol("MCP response omitted result".into()));
    }
}

#[cfg(test)]
async fn read_response_with_timeout<R: AsyncRead + Unpin>(
    reader: &mut R,
    id: u64,
    timeout: Duration,
) -> Result<Value, McpError> {
    tokio::time::timeout(timeout, read_response(reader, id))
        .await
        .map_err(|_| McpError::Timeout)?
}

async fn read_byte<R: AsyncRead + Unpin>(reader: &mut R) -> Result<u8, McpError> {
    let mut byte = [0; 1];
    reader.read_exact(&mut byte).await.map_err(io_error)?;
    Ok(byte[0])
}

fn io_error(error: io::Error) -> McpError {
    McpError::Process(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AllowAllPermissions;
    use tokio::io::duplex;

    fn config(capabilities: Vec<IntegrationCapability>) -> McpServerConfig {
        McpServerConfig {
            descriptor: IntegrationDescriptor {
                kind: IntegrationKind::Mcp,
                name: "fixture".into(),
                capabilities,
            },
            program: "/bin/echo".into(),
            args: vec!["fixture".into()],
        }
    }

    #[test]
    fn config_requires_mcp_tools_capability() {
        let error = config(vec![]).validate().unwrap_err();
        assert!(matches!(
            error,
            McpError::Contract(crate::IntegrationContractError::NoCapabilities)
        ));
        let error = config(vec![IntegrationCapability::LspDiagnostics])
            .validate()
            .unwrap_err();
        assert!(matches!(
            error,
            McpError::Contract(crate::IntegrationContractError::CapabilityMismatch)
        ));
    }

    #[test]
    fn config_rejects_control_characters_without_shell_parsing() {
        let mut config = config(vec![IntegrationCapability::McpTool]);
        config.args = vec!["--label\nnot-a-shell-command".into()];
        assert!(matches!(config.validate(), Err(McpError::Protocol(_))));
    }

    #[test]
    fn config_requires_an_existing_absolute_executable() {
        let mut config = config(vec![IntegrationCapability::McpTool]);
        config.program = "echo".into();
        assert!(
            matches!(config.validate(), Err(McpError::Protocol(message)) if message.contains("absolute"))
        );
        config.program = "/definitely/not-an-mcp-server".into();
        assert!(
            matches!(config.validate(), Err(McpError::Protocol(message)) if message.contains("existing file"))
        );
    }

    #[test]
    fn server_must_advertise_tools_as_an_object() {
        assert!(validate_server_capabilities(&json!({"capabilities": {"tools": {}}})).is_ok());
        assert!(matches!(
            validate_server_capabilities(&json!({"capabilities": {}})),
            Err(McpError::Protocol(message)) if message.contains("tools capability")
        ));
        assert!(matches!(
            validate_server_capabilities(&json!({"capabilities": {"tools": true}})),
            Err(McpError::Protocol(message)) if message.contains("tools capability")
        ));
    }

    #[tokio::test]
    async fn frames_are_content_length_bounded() {
        let (mut writer, mut reader) = duplex(4096);
        let value = json!({"jsonrpc": "2.0", "id": 1, "result": {"ok": true}});
        write_frame(&mut writer, &value).await.unwrap();
        assert_eq!(read_frame(&mut reader).await.unwrap(), value);
    }

    #[tokio::test]
    async fn oversized_response_is_rejected_before_allocation() {
        let (mut writer, mut reader) = duplex(4096);
        writer
            .write_all(b"Content-Length: 262145\r\n\r\n")
            .await
            .unwrap();
        assert!(
            matches!(read_frame(&mut reader).await, Err(McpError::Protocol(message)) if message.contains("frame limit"))
        );
    }

    #[tokio::test]
    async fn malformed_content_length_is_rejected() {
        let (mut writer, mut reader) = duplex(4096);
        writer
            .write_all(b"Content-Length: nope\r\n\r\n")
            .await
            .unwrap();
        assert!(matches!(
            read_frame(&mut reader).await,
            Err(McpError::Protocol(message)) if message.contains("invalid Content-Length")
        ));
    }

    #[tokio::test]
    async fn duplicate_content_length_is_rejected() {
        let (mut writer, mut reader) = duplex(4096);
        writer
            .write_all(b"Content-Length: 0\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
        assert!(matches!(
            read_frame(&mut reader).await,
            Err(McpError::Protocol(message)) if message.contains("duplicate Content-Length")
        ));
    }

    #[tokio::test]
    async fn response_correlation_skips_notifications_and_requires_exact_id() {
        let (mut writer, mut reader) = duplex(4096);
        write_frame(
            &mut writer,
            &json!({"jsonrpc": "2.0", "method": "notifications/progress", "params": {}}),
        )
        .await
        .unwrap();
        write_frame(
            &mut writer,
            &json!({"jsonrpc": "2.0", "id": 7, "result": {"ok": true}}),
        )
        .await
        .unwrap();
        assert_eq!(
            read_response(&mut reader, 7).await.unwrap(),
            json!({"ok": true})
        );

        write_frame(
            &mut writer,
            &json!({"jsonrpc": "2.0", "id": 8, "result": {}}),
        )
        .await
        .unwrap();
        assert!(matches!(
            read_response(&mut reader, 9).await,
            Err(McpError::Protocol(message)) if message.contains("did not match")
        ));
    }

    #[tokio::test]
    async fn response_wait_times_out() {
        let (_writer, mut reader) = duplex(64);
        assert!(matches!(
            read_response_with_timeout(&mut reader, 1, Duration::from_millis(5)).await,
            Err(McpError::Timeout)
        ));
    }

    #[tokio::test]
    async fn permission_broker_is_available_for_mcp_operation() {
        let permissions: Arc<dyn PermissionBroker> = Arc::new(AllowAllPermissions);
        authorize(&permissions, "mcp_tool", "fixture/tool")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn admission_uses_the_mcp_tool_permission_boundary() {
        struct Deny;
        #[async_trait::async_trait]
        impl PermissionBroker for Deny {
            async fn authorize(
                &self,
                operation: &str,
                target: &str,
            ) -> Result<PermissionDecision, DomainError> {
                assert_eq!(operation, "mcp_tool");
                assert_eq!(target, "fixture");
                Ok(PermissionDecision::Deny)
            }
        }
        let permissions: Arc<dyn PermissionBroker> = Arc::new(Deny);
        assert!(matches!(
            authorize(&permissions, "mcp_tool", "fixture").await,
            Err(McpError::PermissionDenied)
        ));
    }
}
