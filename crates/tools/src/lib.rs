//! Permission-aware tools exposed to the session runner.

use async_trait::async_trait;
use devfoundry_schema::DomainError;
use serde::{Deserialize, Serialize};
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

mod integration;
mod terminal;
mod worktree;

mod lsp;
#[cfg(target_os = "macos")]
mod mcp;

pub use integration::{
    IntegrationCapability, IntegrationContractError, IntegrationDescriptor, IntegrationKind,
    IntegrationRequest,
};
pub use terminal::{
    InputLease, MAX_PTY_INPUT_BYTES, MAX_PTY_OUTPUT_BYTES, NativePtyError, NativePtyService,
    PtyCapability, PtyInput, PtyOpenRequest, PtyOutput, PtyResize,
};
pub use worktree::{
    HumanIntegrationAuthorization, WorktreeManager, WorktreeRequest, WorktreeStatus,
};

pub use lsp::{
    LspDiagnostic, LspDiagnosticSeverity, LspError, LspProcessConfig, LspSession,
    MAX_LSP_DIAGNOSTICS, MAX_LSP_MESSAGE_BYTES, MAX_LSP_OUTPUT_BYTES,
};
#[cfg(target_os = "macos")]
pub use mcp::{McpClient, McpError, McpServerConfig, McpToolResult};

const MAX_OUTPUT_BYTES: usize = 64 * 1024;
const MAX_SEARCH_RESULTS: usize = 1_000;
const MAX_SEARCH_FILES: usize = 10_000;
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionDecision {
    Allow,
    Deny,
}

#[async_trait]
pub trait PermissionBroker: Send + Sync {
    async fn authorize(
        &self,
        operation: &str,
        target: &str,
    ) -> Result<PermissionDecision, DomainError>;
}

#[derive(Clone, Default)]
pub struct DefaultPermissions;

#[async_trait]
impl PermissionBroker for DefaultPermissions {
    async fn authorize(
        &self,
        _operation: &str,
        target: &str,
    ) -> Result<PermissionDecision, DomainError> {
        Ok(if is_sensitive_path(target) {
            PermissionDecision::Deny
        } else {
            PermissionDecision::Allow
        })
    }
}

#[derive(Clone, Default)]
pub struct AllowAllPermissions;

#[async_trait]
impl PermissionBroker for AllowAllPermissions {
    async fn authorize(
        &self,
        _operation: &str,
        _target: &str,
    ) -> Result<PermissionDecision, DomainError> {
        Ok(PermissionDecision::Allow)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolOutput {
    pub text: String,
    pub truncated: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("tool is not registered: {0}")]
    Unknown(String),
    #[error("tool failed: {0}")]
    Failed(String),
    #[error("domain error: {0}")]
    Domain(#[from] DomainError),
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError>;
}

#[derive(Clone)]
pub struct ToolContext {
    pub root: PathBuf,
    pub permissions: Arc<dyn PermissionBroker>,
    pub cancellation: CancellationToken,
}

impl ToolContext {
    pub fn resolve(&self, relative: &str) -> Result<PathBuf, ToolError> {
        let root = self
            .root
            .canonicalize()
            .map_err(|error| ToolError::Failed(error.to_string()))?;
        let requested = Path::new(relative);
        if requested.is_absolute()
            || requested
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(ToolError::Failed("path escapes project root".into()));
        }
        let path = root.join(requested);
        let mut parent = path
            .parent()
            .ok_or_else(|| ToolError::Failed("path has no parent".into()))?
            .to_path_buf();
        let mut suffix = Vec::new();
        while !parent.exists() {
            if let Some(name) = parent.file_name() {
                suffix.push(name.to_os_string());
            }
            parent = parent
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| ToolError::Failed("path has no existing parent".into()))?;
        }
        let canonical_parent = parent
            .canonicalize()
            .map_err(|error| ToolError::Failed(error.to_string()))?;
        let file_name = path
            .file_name()
            .ok_or_else(|| ToolError::Failed("path has no file name".into()))?;
        let candidate = suffix
            .iter()
            .rev()
            .fold(canonical_parent, |path, component| path.join(component))
            .join(file_name);
        if !candidate.starts_with(&root) {
            return Err(ToolError::Failed("path escapes project root".into()));
        }
        if candidate.symlink_metadata().is_ok() {
            let canonical_candidate = candidate
                .canonicalize()
                .map_err(|error| ToolError::Failed(error.to_string()))?;
            if !canonical_candidate.starts_with(&root) {
                return Err(ToolError::Failed("path escapes project root".into()));
            }
        }
        Ok(candidate)
    }
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: Vec<Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn standard() -> Self {
        let mut registry = Self::default();
        registry.register(Arc::new(ReadTool));
        registry.register(Arc::new(WriteTool));
        registry.register(Arc::new(EditTool));
        registry.register(Arc::new(ApplyPatchTool));
        registry.register(Arc::new(GlobTool));
        registry.register(Arc::new(GrepTool));
        registry.register(Arc::new(GitStatusTool));
        registry.register(Arc::new(BashTool));
        registry.register(Arc::new(PtyTool));
        registry
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.push(tool);
    }
    pub fn names(&self) -> Vec<&'static str> {
        self.tools.iter().map(|tool| tool.name()).collect()
    }
    pub fn definitions(&self) -> Vec<devfoundry_llm::ToolDefinition> {
        self.definitions_for(self.tools.iter().map(|tool| tool.name()))
    }
    pub fn definitions_for<'a, I>(&self, allowed: I) -> Vec<devfoundry_llm::ToolDefinition>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let allowed = allowed
            .into_iter()
            .collect::<std::collections::HashSet<_>>();
        self.tools
            .iter()
            .filter(|tool| allowed.contains(tool.name()))
            .map(|tool| devfoundry_llm::ToolDefinition {
                name: tool.name().into(),
                description: tool.description().into(),
                parameters: serde_json::json!({ "type": "object" }),
            })
            .collect()
    }
    pub async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let tool = self
            .tools
            .iter()
            .find(|tool| tool.name() == request.name)
            .ok_or_else(|| ToolError::Unknown(request.name.clone()))?;
        tool.execute(request, context).await
    }
}

pub struct ReadTool;

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &'static str {
        "read"
    }
    fn description(&self) -> &'static str {
        "Read a project file."
    }
    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let path = request
            .arguments
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| ToolError::Failed("missing path".into()))?;
        authorize_path(&context, "read", path).await?;
        let content = tokio::fs::read_to_string(context.resolve(path)?)
            .await
            .map_err(|error| ToolError::Failed(error.to_string()))?;
        let truncated = content.len() > 64 * 1024;
        Ok(ToolOutput {
            text: content.chars().take(64 * 1024).collect(),
            truncated,
        })
    }
}

pub struct WriteTool;

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &'static str {
        "write"
    }
    fn description(&self) -> &'static str {
        "Write a project file after permission."
    }
    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let path = request
            .arguments
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| ToolError::Failed("missing path".into()))?;
        let content = request
            .arguments
            .get("content")
            .and_then(|value| value.as_str())
            .ok_or_else(|| ToolError::Failed("missing content".into()))?;
        authorize_path(&context, "write", path).await?;
        if context.cancellation.is_cancelled() {
            return Err(ToolError::Domain(DomainError::Cancelled));
        }
        let target = context.resolve(path)?;
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| ToolError::Failed(error.to_string()))?;
        }
        atomic_write(&target, content).await?;
        Ok(ToolOutput {
            text: format!("wrote {path}"),
            truncated: false,
        })
    }
}

pub struct EditTool;

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &'static str {
        "edit"
    }
    fn description(&self) -> &'static str {
        "Replace exact text in a project file with stale-content detection."
    }
    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let path = string_arg(&request, "path")?;
        let old = string_arg(&request, "old_string")?;
        let new = string_arg(&request, "new_string")?;
        let replace_all = request
            .arguments
            .get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        authorize_path(&context, "edit", path).await?;
        let target = context.resolve(path)?;
        let content = tokio::fs::read_to_string(&target)
            .await
            .map_err(|e| ToolError::Failed(e.to_string()))?;
        let count = content.matches(old).count();
        if count == 0 {
            return Err(ToolError::Failed(
                "stale content: old_string was not found".into(),
            ));
        }
        if count > 1 && !replace_all {
            return Err(ToolError::Failed(
                "stale content: old_string matched more than once".into(),
            ));
        }
        let updated = if replace_all {
            content.replace(old, new)
        } else {
            content.replacen(old, new, 1)
        };
        atomic_write(&target, &updated).await?;
        Ok(ToolOutput {
            text: format!("edited {path}"),
            truncated: false,
        })
    }
}

pub struct ApplyPatchTool;

#[async_trait]
impl Tool for ApplyPatchTool {
    fn name(&self) -> &'static str {
        "apply_patch"
    }
    fn description(&self) -> &'static str {
        "Apply a constrained multi-file patch atomically after validation."
    }
    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let patch = string_arg(&request, "patch")?;
        let changes = parse_patch(patch)?;
        let mut resolved = Vec::new();
        for change in changes {
            authorize_path(&context, "apply_patch", &change.path).await?;
            let target = context.resolve(&change.path)?;
            let exists = target.exists();
            let original = match tokio::fs::read_to_string(&target).await {
                Ok(value) => value,
                Err(error)
                    if change.kind == PatchKind::Add && error.kind() == io::ErrorKind::NotFound =>
                {
                    String::new()
                }
                Err(error) => return Err(ToolError::Failed(error.to_string())),
            };
            let content = match change.kind {
                PatchKind::Add if exists => {
                    return Err(ToolError::Failed(format!(
                        "file already exists: {}",
                        change.path
                    )));
                }
                PatchKind::Add | PatchKind::Update => apply_hunks(&original, &change.hunks)?,
                PatchKind::Delete if !change.hunks.is_empty() => {
                    return Err(ToolError::Failed(
                        "delete patch cannot contain hunks".into(),
                    ));
                }
                PatchKind::Delete => String::new(),
            };
            if change.kind == PatchKind::Delete && !exists {
                return Err(ToolError::Failed(format!(
                    "file does not exist: {}",
                    change.path
                )));
            }
            resolved.push((target, change.kind, content));
        }
        apply_patch_transaction(resolved).await?;
        Ok(ToolOutput {
            text: "patch applied".into(),
            truncated: false,
        })
    }
}

pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &'static str {
        "glob"
    }
    fn description(&self) -> &'static str {
        "Find project files with bounded recursive glob matching."
    }
    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let pattern = string_arg(&request, "pattern")?;
        authorize(&context, "glob", pattern).await?;
        let mut results = Vec::new();
        walk_files(&context.root, &context.root, |relative, _| {
            if results.len() < MAX_SEARCH_RESULTS {
                if !is_sensitive_path(relative) && glob_matches(pattern, relative) {
                    results.push(relative.to_owned());
                }
                true
            } else {
                false
            }
        })
        .await?;
        results.sort();
        let truncated = results.len() > MAX_SEARCH_RESULTS;
        results.truncate(MAX_SEARCH_RESULTS);
        Ok(ToolOutput {
            text: results.join("\n"),
            truncated,
        })
    }
}

pub struct GrepTool;

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn description(&self) -> &'static str {
        "Search project text files with bounded results."
    }
    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let pattern = string_arg(&request, "pattern")?;
        authorize(&context, "grep", pattern).await?;
        let mut results = Vec::new();
        walk_files(&context.root, &context.root, |relative, path| {
            if results.len() >= MAX_SEARCH_RESULTS
                || relative.starts_with(".git/")
                || is_sensitive_path(relative)
            {
                return false;
            }
            let Ok(metadata) = std::fs::metadata(path) else {
                return false;
            };
            if metadata.len() > MAX_FILE_BYTES {
                return false;
            }
            let Ok(bytes) = std::fs::read(path) else {
                return false;
            };
            let Ok(text) = std::str::from_utf8(&bytes) else {
                return false;
            };
            for (line_number, line) in text.lines().enumerate() {
                if line.contains(pattern) {
                    results.push(format!("{}:{}:{}", relative, line_number + 1, line));
                }
                if results.len() >= MAX_SEARCH_RESULTS {
                    break;
                }
            }
            false
        })
        .await?;
        let truncated = results.len() >= MAX_SEARCH_RESULTS;
        Ok(ToolOutput {
            text: results.join("\n"),
            truncated,
        })
    }
}

pub struct BashTool;

/// Execute a terminal session through the bounded platform process boundary.
///
/// This is intentionally pipe-backed until a platform PTY adapter can guarantee
/// terminal restoration and resize handling on every supported target.
pub struct PtyTool;

pub struct GitStatusTool;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GitStatusArguments {
    #[serde(default)]
    operation: GitOperation,
    #[serde(default)]
    include_worktrees: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GitOperation {
    #[default]
    Status,
    Diff,
    Log,
}

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &'static str {
        "git_status"
    }
    fn description(&self) -> &'static str {
        "Read fixed-argument Git status, diff, log, and optional worktree metadata without changing repository state."
    }
    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let arguments: GitStatusArguments = serde_json::from_value(request.arguments)
            .map_err(|error| ToolError::Failed(format!("invalid git_status arguments: {error}")))?;
        let (operation, command) = match arguments.operation {
            GitOperation::Status => (
                "git_status",
                "git --no-optional-locks status --short --branch --porcelain=v1",
            ),
            GitOperation::Diff => (
                "git_diff",
                "git --no-optional-locks diff --no-ext-diff --no-color --binary",
            ),
            GitOperation::Log => (
                "git_log",
                "git --no-optional-locks log --oneline --decorate --no-color -n 50",
            ),
        };
        let mut output =
            run_command_with_operation(operation, command, &context, COMMAND_TIMEOUT).await?;
        if arguments.include_worktrees {
            let worktrees = run_command_with_operation(
                "git_worktree",
                "git --no-optional-locks worktree list --porcelain",
                &context,
                COMMAND_TIMEOUT,
            )
            .await?;
            if !worktrees.text.is_empty() {
                if !output.text.is_empty() {
                    output.text.push_str("\n\n");
                }
                output.text.push_str("Worktrees:\n");
                output.text.push_str(&worktrees.text);
            }
            output.truncated |= worktrees.truncated;
            if output.text.len() > MAX_OUTPUT_BYTES {
                while output.text.len() > MAX_OUTPUT_BYTES {
                    output.text.pop();
                }
                output.truncated = true;
            }
        }
        Ok(output)
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &'static str {
        "bash"
    }
    fn description(&self) -> &'static str {
        "Run a shell command after explicit bash permission."
    }
    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        run_command(string_arg(&request, "command")?, &context).await
    }
}

#[async_trait]
impl Tool for PtyTool {
    fn name(&self) -> &'static str {
        "pty"
    }

    fn description(&self) -> &'static str {
        "Run a bounded terminal command with PTY permission and process-tree cleanup."
    }

    async fn execute(
        &self,
        request: ToolRequest,
        context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let command = string_arg(&request, "command")?;
        let integration = IntegrationRequest {
            kind: IntegrationKind::Pty,
            capability: IntegrationCapability::PtySession,
            target: command.to_owned(),
        };
        integration
            .validate()
            .map_err(|error| ToolError::Failed(error.to_string()))?;
        if context.cancellation.is_cancelled() {
            return Err(ToolError::Domain(DomainError::Cancelled));
        }
        let root = context
            .root
            .canonicalize()
            .map_err(|error| ToolError::Failed(error.to_string()))?;
        if !root.is_dir() {
            return Err(ToolError::Failed("project root is not a directory".into()));
        }
        let context = ToolContext { root, ..context };
        run_command_with_operation(
            integration.permission_operation(),
            command,
            &context,
            COMMAND_TIMEOUT,
        )
        .await
    }
}

async fn authorize(context: &ToolContext, operation: &str, target: &str) -> Result<(), ToolError> {
    if context.permissions.authorize(operation, target).await? != PermissionDecision::Allow {
        return Err(ToolError::Domain(DomainError::PermissionDenied {
            operation: operation.into(),
        }));
    }
    Ok(())
}

async fn authorize_path(
    context: &ToolContext,
    operation: &str,
    target: &str,
) -> Result<(), ToolError> {
    if is_sensitive_path(target) {
        return Err(ToolError::Domain(DomainError::PermissionDenied {
            operation: operation.into(),
        }));
    }
    authorize(context, operation, target).await
}

fn is_sensitive_path(path: &str) -> bool {
    path.replace('\\', "/")
        .split('/')
        .map(str::to_ascii_lowercase)
        .any(|component| {
            component == ".env"
                || component.starts_with(".env.")
                || component == "credentials"
                || component.starts_with("credentials.")
                || component == "secrets"
                || component.starts_with("secrets.")
                || component == "id_rsa"
                || component == "id_dsa"
                || component.ends_with(".pem")
                || component.ends_with(".key")
                || component.ends_with(".p12")
                || component.ends_with(".pfx")
                || component.ends_with(".secret")
        })
}

fn string_arg<'a>(request: &'a ToolRequest, name: &str) -> Result<&'a str, ToolError> {
    request
        .arguments
        .get(name)
        .and_then(|value| value.as_str())
        .ok_or_else(|| ToolError::Failed(format!("missing {name}")))
}

async fn walk_files<F>(root: &Path, directory: &Path, mut visit: F) -> Result<(), ToolError>
where
    F: FnMut(&str, &Path) -> bool,
{
    let mut stack = vec![directory.to_path_buf()];
    let mut files = 0;
    while let Some(directory) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&directory)
            .await
            .map_err(|e| ToolError::Failed(e.to_string()))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ToolError::Failed(e.to_string()))?
        {
            if files >= MAX_SEARCH_FILES {
                return Ok(());
            }
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            let file_type = entry
                .file_type()
                .await
                .map_err(|e| ToolError::Failed(e.to_string()))?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if entry.file_name() != ".git" && entry.file_name() != "target" {
                    let canonical = match path.canonicalize() {
                        Ok(path) if path.starts_with(root) => path,
                        _ => continue,
                    };
                    stack.push(canonical);
                }
            } else {
                let canonical = match path.canonicalize() {
                    Ok(path) if path.starts_with(root) => path,
                    _ => continue,
                };
                files += 1;
                let _ = visit(&relative, &canonical);
            }
        }
    }
    Ok(())
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    glob_match_parts(
        &pattern.split('/').collect::<Vec<_>>(),
        &path.split('/').collect::<Vec<_>>(),
    )
}
fn glob_match_parts(pattern: &[&str], path: &[&str]) -> bool {
    match (pattern.first(), path.first()) {
        (None, None) => true,
        (Some(&"**"), _) => {
            glob_match_parts(&pattern[1..], path)
                || (!path.is_empty() && glob_match_parts(pattern, &path[1..]))
        }
        (Some(segment), Some(value)) => {
            segment_match(segment, value) && glob_match_parts(&pattern[1..], &path[1..])
        }
        _ => false,
    }
}
fn segment_match(pattern: &str, value: &str) -> bool {
    let (mut p, mut v, mut star, mut mark) = (0, 0, None, 0);
    let pc: Vec<char> = pattern.chars().collect();
    let vc: Vec<char> = value.chars().collect();
    while v < vc.len() {
        if p < pc.len() && (pc[p] == '?' || pc[p] == vc[v]) {
            p += 1;
            v += 1;
        } else if p < pc.len() && pc[p] == '*' {
            star = Some(p);
            mark = v;
            p += 1;
        } else if let Some(s) = star {
            p = s + 1;
            mark += 1;
            v = mark;
        } else {
            return false;
        }
    }
    while p < pc.len() && pc[p] == '*' {
        p += 1;
    }
    p == pc.len()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PatchKind {
    Add,
    Update,
    Delete,
}

struct PatchChange {
    path: String,
    kind: PatchKind,
    hunks: Vec<(String, String)>,
}

fn parse_patch(patch: &str) -> Result<Vec<PatchChange>, ToolError> {
    let lines: Vec<&str> = patch.lines().collect();
    if lines.first().copied() != Some("*** Begin Patch")
        || lines.last().copied() != Some("*** End Patch")
    {
        return Err(ToolError::Failed("invalid patch envelope".into()));
    }
    let mut changes = Vec::new();
    let mut index = 1;
    while index + 1 < lines.len() {
        let header = lines[index];
        let (kind, prefix) = if let Some(path) = header.strip_prefix("*** Add File: ") {
            (PatchKind::Add, path)
        } else if let Some(path) = header.strip_prefix("*** Update File: ") {
            (PatchKind::Update, path)
        } else if let Some(path) = header.strip_prefix("*** Delete File: ") {
            (PatchKind::Delete, path)
        } else {
            return Err(ToolError::Failed(format!("invalid patch header: {header}")));
        };
        index += 1;
        let mut hunks = Vec::new();
        if kind == PatchKind::Add {
            let mut content = String::new();
            while index < lines.len() && !lines[index].starts_with("*** ") {
                let line = lines[index];
                if !line.starts_with('+') {
                    return Err(ToolError::Failed("invalid add-file line".into()));
                }
                content.push_str(&line[1..]);
                content.push('\n');
                index += 1;
            }
            hunks.push((String::new(), content));
            changes.push(PatchChange {
                path: prefix.to_owned(),
                kind,
                hunks,
            });
            continue;
        }
        while index < lines.len() && !lines[index].starts_with("*** ") {
            if !lines[index].starts_with("@@") {
                return Err(ToolError::Failed("invalid patch hunk".into()));
            }
            index += 1;
            let mut old = String::new();
            let mut new = String::new();
            while index < lines.len()
                && !lines[index].starts_with("@@")
                && !lines[index].starts_with("*** ")
            {
                let line = lines[index];
                match line.as_bytes().first().copied() {
                    Some(b' ') => {
                        old.push_str(&line[1..]);
                        old.push('\n');
                        new.push_str(&line[1..]);
                        new.push('\n');
                    }
                    Some(b'-') => {
                        old.push_str(&line[1..]);
                        old.push('\n');
                    }
                    Some(b'+') => {
                        new.push_str(&line[1..]);
                        new.push('\n');
                    }
                    _ => return Err(ToolError::Failed("invalid patch line".into())),
                }
                index += 1;
            }
            hunks.push((old, new));
        }
        if kind == PatchKind::Delete && index < lines.len() && !hunks.is_empty() {
            return Err(ToolError::Failed(
                "delete patch cannot contain hunks".into(),
            ));
        }
        changes.push(PatchChange {
            path: prefix.to_owned(),
            kind,
            hunks,
        });
    }
    if changes.is_empty() {
        return Err(ToolError::Failed("patch contains no changes".into()));
    }
    let mut paths = std::collections::HashSet::new();
    for change in &changes {
        if !paths.insert(&change.path) {
            return Err(ToolError::Failed(format!(
                "patch contains duplicate path: {}",
                change.path
            )));
        }
    }
    Ok(changes)
}

fn apply_hunks(content: &str, hunks: &[(String, String)]) -> Result<String, ToolError> {
    let mut result = content.to_owned();
    for (old, new) in hunks {
        let old = old.strip_suffix('\n').unwrap_or(old);
        let new = new.strip_suffix('\n').unwrap_or(new);
        if old.is_empty() {
            result.push_str(new);
        } else if result.matches(old).count() != 1 {
            return Err(ToolError::Failed(
                "patch context is stale or ambiguous".into(),
            ));
        } else {
            result = result.replacen(old, new, 1);
        }
    }
    Ok(result)
}

pub async fn run_command(command: &str, context: &ToolContext) -> Result<ToolOutput, ToolError> {
    run_command_with_timeout(command, context, COMMAND_TIMEOUT).await
}

async fn run_command_with_timeout(
    command: &str,
    context: &ToolContext,
    timeout: Duration,
) -> Result<ToolOutput, ToolError> {
    run_command_with_operation("bash", command, context, timeout).await
}

async fn run_command_with_operation(
    operation: &str,
    command: &str,
    context: &ToolContext,
    timeout: Duration,
) -> Result<ToolOutput, ToolError> {
    if context.permissions.authorize(operation, command).await? != PermissionDecision::Allow {
        return Err(ToolError::Domain(DomainError::PermissionDenied {
            operation: operation.into(),
        }));
    }
    let root = context
        .root
        .canonicalize()
        .map_err(|error| ToolError::Failed(error.to_string()))?;
    if !root.is_dir() {
        return Err(ToolError::Failed("project root is not a directory".into()));
    }
    let shell = command_shell();
    let mut command_process = tokio::process::Command::new(shell);
    command_process
        .arg("-c")
        .arg(command)
        .current_dir(root)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    configure_process_group(&mut command_process);
    let mut child = command_process
        .spawn()
        .map_err(|error| ToolError::Failed(error.to_string()))?;
    #[cfg(unix)]
    let child_pid = child.id();
    #[cfg(windows)]
    let job = match attach_job_object(&child) {
        Ok(job) => job,
        Err(error) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(ToolError::Failed(format!(
                "failed to attach command to Windows job object: {error}"
            )));
        }
    };
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");
    let mut stdout_task = tokio::spawn(read_bounded(stdout));
    let mut stderr_task = tokio::spawn(read_bounded(stderr));
    let outcome = {
        let wait = tokio::time::timeout(timeout, child.wait());
        tokio::pin!(wait);
        tokio::select! {
            _ = context.cancellation.cancelled() => Err(ToolError::Domain(DomainError::Cancelled)),
            result = &mut wait => result
                .map_err(|_| ToolError::Failed("command timed out".into()))
                .and_then(|result| result.map_err(|error| ToolError::Failed(error.to_string())))
        }
    };
    let status = match outcome {
        Ok(status) => status,
        Err(failure) => {
            #[cfg(unix)]
            terminate_process_group(&mut child)
                .await
                .map_err(|error| ToolError::Failed(error.to_string()))?;
            #[cfg(windows)]
            terminate_process_group(&mut child, &job)
                .await
                .map_err(|error| ToolError::Failed(error.to_string()))?;
            #[cfg(all(not(unix), not(windows)))]
            terminate_process_group(&mut child)
                .await
                .map_err(|error| ToolError::Failed(error.to_string()))?;
            child
                .wait()
                .await
                .map_err(|error| ToolError::Failed(error.to_string()))?;
            stdout_task.abort();
            stderr_task.abort();
            return Err(failure);
        }
    };
    let output = tokio::time::timeout(Duration::from_millis(100), async {
        tokio::try_join!(&mut stdout_task, &mut stderr_task)
    })
    .await;
    let output = match output {
        Ok(output) => output.map_err(|error| ToolError::Failed(error.to_string()))?,
        Err(_) => {
            #[cfg(unix)]
            if let Some(pid) = child_pid {
                terminate_process_group_id(pid)
                    .map_err(|error| ToolError::Failed(error.to_string()))?;
            }
            stdout_task.abort();
            stderr_task.abort();
            return Err(ToolError::Failed("command output cleanup timed out".into()));
        }
    };
    let (stdout, stderr) = output;
    let stdout = stdout.map_err(|error| ToolError::Failed(error.to_string()))?;
    let stderr = stderr.map_err(|error| ToolError::Failed(error.to_string()))?;
    let mut text = stdout;
    text.extend(stderr);
    let truncated = text.len() > MAX_OUTPUT_BYTES;
    let mut output =
        String::from_utf8_lossy(&text[..text.len().min(MAX_OUTPUT_BYTES)]).into_owned();
    while output.len() > MAX_OUTPUT_BYTES {
        output.pop();
    }
    if !status.success() {
        return Err(ToolError::Failed(match status.code() {
            Some(code) => format!("command exited with exit code {code}"),
            None => "command terminated by signal".into(),
        }));
    }
    Ok(ToolOutput {
        truncated,
        text: output,
    })
}

#[cfg(unix)]
fn command_shell() -> &'static str {
    "/bin/sh"
}

#[cfg(not(unix))]
fn command_shell() -> &'static str {
    "sh"
}

#[cfg(unix)]
fn configure_process_group(command: &mut tokio::process::Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut tokio::process::Command) {}

#[cfg(unix)]
async fn terminate_process_group(child: &mut tokio::process::Child) -> io::Result<()> {
    let Some(pid) = child.id() else {
        return Ok(());
    };
    terminate_process_group_id(pid)
}

#[cfg(unix)]
fn terminate_process_group_id(pid: u32) -> io::Result<()> {
    let result = unsafe { libc::kill(-(pid as libc::pid_t), libc::SIGKILL) };
    let error = io::Error::last_os_error();
    if result == 0 || error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

#[cfg(windows)]
fn attach_job_object(child: &tokio::process::Child) -> io::Result<WindowsJob> {
    WindowsJob::attach(child)
}

#[cfg(windows)]
struct WindowsJob {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl WindowsJob {
    fn attach(child: &tokio::process::Child) -> io::Result<Self> {
        use std::mem::size_of;
        use std::ptr::null_mut;
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        };

        let handle = unsafe { CreateJobObjectW(null_mut(), null_mut()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &mut limits as *mut _ as *mut _,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
            return Err(io::Error::last_os_error());
        }
        let Some(process) = child.raw_handle() else {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "command exited before job setup",
            ));
        };
        let assigned = unsafe { AssignProcessToJobObject(handle, process) };
        if assigned == 0 {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
            return Err(io::Error::last_os_error());
        }
        Ok(Self { handle })
    }

    fn terminate(&self) -> io::Result<()> {
        use windows_sys::Win32::System::JobObjects::TerminateJobObject;

        if unsafe { TerminateJobObject(self.handle, 1) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(windows)]
impl Drop for WindowsJob {
    fn drop(&mut self) {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(self.handle) };
    }
}

#[cfg(windows)]
async fn terminate_process_group(
    _child: &mut tokio::process::Child,
    job: &WindowsJob,
) -> io::Result<()> {
    job.terminate()
}

#[cfg(all(not(unix), not(windows)))]
async fn terminate_process_group(child: &mut tokio::process::Child) -> io::Result<()> {
    child.kill().await
}

async fn read_bounded<R: AsyncRead + Unpin>(reader: R) -> io::Result<Vec<u8>> {
    let mut reader = reader;
    let mut bytes = Vec::with_capacity(MAX_OUTPUT_BYTES + 1);
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        if bytes.len() < MAX_OUTPUT_BYTES + 1 {
            let retained = count.min(MAX_OUTPUT_BYTES + 1 - bytes.len());
            bytes.extend_from_slice(&buffer[..retained]);
        }
    }
    Ok(bytes)
}

async fn atomic_write(target: &Path, content: &str) -> Result<(), ToolError> {
    let temp = stage_file(target, content).await?;
    if let Err(error) = tokio::fs::rename(&temp, target)
        .await
        .map_err(|error| ToolError::Failed(error.to_string()))
    {
        let _ = tokio::fs::remove_file(&temp).await;
        return Err(error);
    }
    Ok(())
}

async fn stage_file(target: &Path, content: &str) -> Result<PathBuf, ToolError> {
    let parent = target
        .parent()
        .ok_or_else(|| ToolError::Failed("target has no parent".into()))?;
    let name = target
        .file_name()
        .ok_or_else(|| ToolError::Failed("target has no file name".into()))?
        .to_string_lossy();
    let temp = parent.join(format!(
        ".{name}.tmp-{}",
        TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let result = async {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .await
            .map_err(|error| ToolError::Failed(error.to_string()))?;
        file.write_all(content.as_bytes())
            .await
            .map_err(|error| ToolError::Failed(error.to_string()))?;
        file.sync_all()
            .await
            .map_err(|error| ToolError::Failed(error.to_string()))?;
        Ok(temp.clone())
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&temp).await;
    }
    result
}

async fn apply_patch_transaction(
    changes: Vec<(PathBuf, PatchKind, String)>,
) -> Result<(), ToolError> {
    let mut staged = Vec::new();
    for (target, kind, content) in &changes {
        if *kind != PatchKind::Delete {
            match stage_file(target, content).await {
                Ok(temp) => staged.push((target.clone(), *kind, temp)),
                Err(error) => {
                    for (_, _, temp) in staged {
                        let _ = tokio::fs::remove_file(temp).await;
                    }
                    return Err(error);
                }
            }
        }
    }

    let mut backups = Vec::new();
    for (index, (target, kind, _)) in changes.iter().enumerate() {
        if target.exists() {
            let backup = target.with_file_name(format!(
                ".{}.patch-backup-{}",
                target.file_name().unwrap().to_string_lossy(),
                TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            if let Err(error) = tokio::fs::rename(target, &backup).await {
                rollback_patch(&staged, &backups, &[]).await;
                return Err(ToolError::Failed(error.to_string()));
            }
            backups.push((index, target.clone(), backup));
        } else if *kind == PatchKind::Delete {
            rollback_patch(&staged, &backups, &[]).await;
            return Err(ToolError::Failed(format!(
                "file disappeared: {}",
                target.display()
            )));
        }
    }

    let mut installed = Vec::new();
    for (target, kind, _) in &changes {
        if *kind != PatchKind::Delete {
            let temp = staged
                .iter()
                .find(|(path, _, _)| path == target)
                .unwrap()
                .2
                .clone();
            if let Err(error) = tokio::fs::rename(&temp, target).await {
                rollback_patch(&staged, &backups, &installed).await;
                return Err(ToolError::Failed(error.to_string()));
            }
            installed.push(target.clone());
        }
    }
    for (_, _, temp) in staged {
        let _ = tokio::fs::remove_file(temp).await;
    }
    for (_, _, backup) in backups {
        let _ = tokio::fs::remove_file(backup).await;
    }
    Ok(())
}

async fn rollback_patch(
    staged: &[(PathBuf, PatchKind, PathBuf)],
    backups: &[(usize, PathBuf, PathBuf)],
    installed: &[PathBuf],
) {
    for target in installed {
        remove_path(target).await;
    }
    for (_, target, backup) in backups.iter().rev() {
        remove_path(target).await;
        let _ = tokio::fs::rename(backup, target).await;
    }
    for (_, _, temp) in staged {
        let _ = tokio::fs::remove_file(temp).await;
    }
}

async fn remove_path(path: &Path) {
    if tokio::fs::symlink_metadata(path)
        .await
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
    {
        let _ = tokio::fs::remove_dir_all(path).await;
    } else {
        let _ = tokio::fs::remove_file(path).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn context(root: &Path) -> ToolContext {
        ToolContext {
            root: root.to_path_buf(),
            permissions: Arc::new(AllowAllPermissions),
            cancellation: CancellationToken::new(),
        }
    }

    fn default_context(root: &Path) -> ToolContext {
        ToolContext {
            root: root.to_path_buf(),
            permissions: Arc::new(DefaultPermissions),
            cancellation: CancellationToken::new(),
        }
    }

    fn temp_root() -> PathBuf {
        let root = tempfile::tempdir().unwrap().keep();
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[tokio::test]
    async fn resolve_rejects_traversal() {
        let root = temp_root();
        let result = context(&root).resolve("../outside");
        assert!(result.is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn resolve_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;
        let root = temp_root();
        let outside = temp_root();
        symlink(&outside, root.join("link")).unwrap();
        assert!(context(&root).resolve("link/secret.txt").is_err());
        fs::remove_file(root.join("link")).unwrap();
        fs::remove_dir(root).unwrap();
        fs::remove_dir(outside).unwrap();
    }

    #[tokio::test]
    async fn missing_nested_suffix_is_preserved() {
        let root = temp_root();
        let resolved = context(&root).resolve("new/nested/file.txt").unwrap();
        assert_eq!(
            resolved,
            root.canonicalize().unwrap().join("new/nested/file.txt")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn grep_rejects_external_symlink() {
        use std::os::unix::fs::symlink;
        let root = temp_root();
        let outside = temp_root();
        fs::write(outside.join("secret.txt"), "outside marker").unwrap();
        symlink(&outside, root.join("linked")).unwrap();

        let result = GrepTool
            .execute(
                ToolRequest {
                    name: "grep".into(),
                    arguments: serde_json::json!({"pattern": "outside marker"}),
                },
                context(&root),
            )
            .await
            .unwrap();
        assert!(!result.text.contains("outside marker"));

        fs::remove_file(root.join("linked")).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[tokio::test]
    async fn write_failure_does_not_replace_existing_path() {
        let root = temp_root();
        fs::create_dir(root.join("target")).unwrap();
        let result = WriteTool
            .execute(
                ToolRequest {
                    name: "write".into(),
                    arguments: serde_json::json!({"path": "target", "content": "new"}),
                },
                context(&root),
            )
            .await;
        assert!(result.is_err());
        assert!(root.join("target").is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn edit_rejects_stale_content_without_mutation() {
        let root = temp_root();
        fs::write(root.join("file.txt"), "current").unwrap();
        let result = EditTool
            .execute(
                ToolRequest {
                    name: "edit".into(),
                    arguments: serde_json::json!({
                        "path": "file.txt", "old_string": "stale", "new_string": "new"
                    }),
                },
                context(&root),
            )
            .await;
        assert!(
            matches!(result, Err(ToolError::Failed(message)) if message.contains("stale content"))
        );
        assert_eq!(
            fs::read_to_string(root.join("file.txt")).unwrap(),
            "current"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn default_policy_denies_sensitive_file_and_allows_normal_file() {
        let root = temp_root();
        fs::write(root.join(".env"), "TOKEN=secret").unwrap();
        fs::write(root.join("normal.txt"), "safe").unwrap();
        let denied = ReadTool
            .execute(
                ToolRequest {
                    name: "read".into(),
                    arguments: serde_json::json!({"path": ".env"}),
                },
                default_context(&root),
            )
            .await;
        assert!(matches!(
            denied,
            Err(ToolError::Domain(DomainError::PermissionDenied { operation }))
                if operation == "read"
        ));
        let allowed = ReadTool
            .execute(
                ToolRequest {
                    name: "read".into(),
                    arguments: serde_json::json!({"path": "normal.txt"}),
                },
                default_context(&root),
            )
            .await
            .unwrap();
        assert_eq!(allowed.text, "safe");
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn sensitive_alias_is_denied() {
        let root = temp_root();
        fs::write(root.join(".env.local"), "TOKEN=secret").unwrap();
        let result = ReadTool
            .execute(
                ToolRequest {
                    name: "read".into(),
                    arguments: serde_json::json!({"path": ".env.local"}),
                },
                context(&root),
            )
            .await;
        assert!(matches!(
            result,
            Err(ToolError::Domain(DomainError::PermissionDenied { operation }))
                if operation == "read"
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn apply_patch_denies_sensitive_file_even_with_allow_all_broker() {
        let root = temp_root();
        fs::write(root.join("credentials.json"), "old").unwrap();
        let patch =
            "*** Begin Patch\n*** Update File: credentials.json\n@@\n-old\n+new\n*** End Patch";
        let result = ApplyPatchTool
            .execute(
                ToolRequest {
                    name: "apply_patch".into(),
                    arguments: serde_json::json!({"patch": patch}),
                },
                context(&root),
            )
            .await;
        assert!(matches!(
            result,
            Err(ToolError::Domain(DomainError::PermissionDenied { operation }))
                if operation == "apply_patch"
        ));
        assert_eq!(
            fs::read_to_string(root.join("credentials.json")).unwrap(),
            "old"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn invalid_patch_leaves_all_files_unchanged() {
        let root = temp_root();
        fs::write(root.join("one.txt"), "one").unwrap();
        let patch = "*** Begin Patch\n*** Update File: one.txt\n@@\n-one\n+changed\n*** Update File: missing.txt\n@@\n-wrong\n+new\n*** End Patch";
        let result = ApplyPatchTool
            .execute(
                ToolRequest {
                    name: "apply_patch".into(),
                    arguments: serde_json::json!({"patch": patch}),
                },
                context(&root),
            )
            .await;
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(root.join("one.txt")).unwrap(), "one");
        assert!(!root.join("missing.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn patch_install_failure_rolls_back_prior_replacements() {
        let root = temp_root();
        let directory = root.join("directory");
        let first = directory.join("first.txt");
        fs::create_dir(&directory).unwrap();
        fs::write(&first, "one").unwrap();
        let result = apply_patch_transaction(vec![
            (directory.clone(), PatchKind::Delete, String::new()),
            (first.clone(), PatchKind::Update, "changed".into()),
        ])
        .await;
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(first).unwrap(), "one");
        assert!(directory.is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn patch_delete_failure_leaves_files_unchanged() {
        let root = temp_root();
        let first = root.join("first.txt");
        let missing = root.join("missing.txt");
        fs::write(&first, "one").unwrap();
        let result = apply_patch_transaction(vec![
            (first.clone(), PatchKind::Update, "changed".into()),
            (missing.clone(), PatchKind::Delete, String::new()),
        ])
        .await;
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(first).unwrap(), "one");
        assert!(!missing.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn standard_registry_includes_bash_and_search_tools() {
        let names = ToolRegistry::standard().names();
        assert!(names.contains(&"bash"));
        assert!(names.contains(&"pty"));
        assert!(names.contains(&"git_status"));
        assert!(names.contains(&"edit"));
        assert!(names.contains(&"apply_patch"));
        assert!(names.contains(&"glob"));
        assert!(names.contains(&"grep"));
    }

    #[tokio::test]
    async fn bash_uses_explicit_permission_operation() {
        struct Deny;
        #[async_trait]
        impl PermissionBroker for Deny {
            async fn authorize(
                &self,
                operation: &str,
                _target: &str,
            ) -> Result<PermissionDecision, DomainError> {
                assert_eq!(operation, "bash");
                Ok(PermissionDecision::Deny)
            }
        }
        let root = temp_root();
        let context = ToolContext {
            root: root.clone(),
            permissions: Arc::new(Deny),
            cancellation: CancellationToken::new(),
        };
        let result = BashTool
            .execute(
                ToolRequest {
                    name: "bash".into(),
                    arguments: serde_json::json!({"command": "printf no"}),
                },
                context,
            )
            .await;
        assert!(
            matches!(result, Err(ToolError::Domain(DomainError::PermissionDenied { operation })) if operation == "bash")
        );
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn pty_uses_explicit_permission_operation() {
        struct Deny;
        #[async_trait]
        impl PermissionBroker for Deny {
            async fn authorize(
                &self,
                operation: &str,
                target: &str,
            ) -> Result<PermissionDecision, DomainError> {
                assert_eq!(operation, "pty_session");
                assert_eq!(target, "printf terminal");
                Ok(PermissionDecision::Deny)
            }
        }
        let root = temp_root();
        let result = PtyTool
            .execute(
                ToolRequest {
                    name: "pty".into(),
                    arguments: serde_json::json!({"command": "printf terminal"}),
                },
                ToolContext {
                    root: root.clone(),
                    permissions: Arc::new(Deny),
                    cancellation: CancellationToken::new(),
                },
            )
            .await;
        assert!(matches!(
            result,
            Err(ToolError::Domain(DomainError::PermissionDenied { operation }))
                if operation == "pty_session"
        ));
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn pty_rejects_control_characters_before_permission() {
        let root = temp_root();
        let result = PtyTool
            .execute(
                ToolRequest {
                    name: "pty".into(),
                    arguments: serde_json::json!({"command": "printf 'unsafe\n'"}),
                },
                context(&root),
            )
            .await;
        assert!(matches!(result, Err(ToolError::Failed(message)) if message.contains("control")));
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn pty_timeout_and_cancellation_use_process_boundary() {
        let root = temp_root();
        let timed_out = run_command_with_operation(
            "pty_session",
            "sleep 1",
            &context(&root),
            Duration::from_millis(10),
        )
        .await;
        assert!(
            matches!(timed_out, Err(ToolError::Failed(message)) if message == "command timed out")
        );

        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let cancelled = run_command_with_operation(
            "pty_session",
            "sleep 1",
            &ToolContext {
                root: root.clone(),
                permissions: Arc::new(AllowAllPermissions),
                cancellation,
            },
            Duration::from_secs(1),
        )
        .await;
        assert!(matches!(
            cancelled,
            Err(ToolError::Domain(DomainError::Cancelled))
        ));
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn pty_bounds_output_and_rejects_cancelled_context_before_spawn() {
        let root = temp_root();
        let output = PtyTool
            .execute(
                ToolRequest {
                    name: "pty".into(),
                    arguments: serde_json::json!({"command": "yes x | head -c 100000"}),
                },
                context(&root),
            )
            .await
            .unwrap();
        assert!(output.truncated);
        assert!(output.text.len() <= MAX_OUTPUT_BYTES);

        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let result = PtyTool
            .execute(
                ToolRequest {
                    name: "pty".into(),
                    arguments: serde_json::json!({"command": "exit 0"}),
                },
                ToolContext {
                    root: root.clone(),
                    permissions: Arc::new(AllowAllPermissions),
                    cancellation,
                },
            )
            .await;
        assert!(matches!(
            result,
            Err(ToolError::Domain(DomainError::Cancelled))
        ));
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn git_status_is_read_only_and_uses_fixed_arguments() {
        let root = temp_root();
        fs::write(root.join("untracked.txt"), "content").unwrap();
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .unwrap();
        let output = GitStatusTool
            .execute(
                ToolRequest {
                    name: "git_status".into(),
                    arguments: serde_json::json!({}),
                },
                context(&root),
            )
            .await
            .unwrap();
        assert!(output.text.contains("untracked.txt"));
        assert!(root.join(".git").is_dir());
        assert!(root.join("untracked.txt").is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn git_status_rejects_unknown_arguments_before_execution() {
        let root = temp_root();
        let result = GitStatusTool
            .execute(
                ToolRequest {
                    name: "git_status".into(),
                    arguments: serde_json::json!({"command": "git reset --hard"}),
                },
                context(&root),
            )
            .await;
        assert!(matches!(
            result,
            Err(ToolError::Failed(message)) if message.starts_with("invalid git_status arguments:")
        ));
        assert!(!root.join(".git").exists());
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn git_status_requires_its_own_permission_operation() {
        struct Deny;
        #[async_trait]
        impl PermissionBroker for Deny {
            async fn authorize(
                &self,
                operation: &str,
                target: &str,
            ) -> Result<PermissionDecision, DomainError> {
                assert_eq!(operation, "git_status");
                assert!(target.contains("git --no-optional-locks status"));
                Ok(PermissionDecision::Deny)
            }
        }
        let root = temp_root();
        let context = ToolContext {
            root: root.clone(),
            permissions: Arc::new(Deny),
            cancellation: CancellationToken::new(),
        };
        let result = GitStatusTool
            .execute(
                ToolRequest {
                    name: "git_status".into(),
                    arguments: serde_json::json!({}),
                },
                context,
            )
            .await;
        assert!(matches!(
            result,
            Err(ToolError::Domain(DomainError::PermissionDenied { operation }))
                if operation == "git_status"
        ));
        assert!(!root.join(".git").exists());
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn git_read_queries_are_fixed_and_read_only() {
        let root = temp_root();
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .unwrap();
        fs::write(root.join("file.txt"), "one\n").unwrap();
        std::process::Command::new("git")
            .args(["add", "file.txt"])
            .current_dir(&root)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "--quiet",
                "-m",
                "test",
            ])
            .current_dir(&root)
            .status()
            .unwrap();
        for operation in ["diff", "log"] {
            GitStatusTool
                .execute(
                    ToolRequest {
                        name: "git_status".into(),
                        arguments: serde_json::json!({"operation": operation}),
                    },
                    context(&root),
                )
                .await
                .unwrap();
        }
        assert!(!root.join(".git/index.lock").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn git_worktree_metadata_has_separate_permission() {
        struct DenyWorktree;
        #[async_trait]
        impl PermissionBroker for DenyWorktree {
            async fn authorize(
                &self,
                operation: &str,
                _target: &str,
            ) -> Result<PermissionDecision, DomainError> {
                Ok(if operation == "git_worktree" {
                    PermissionDecision::Deny
                } else {
                    PermissionDecision::Allow
                })
            }
        }
        let root = temp_root();
        let context = ToolContext {
            root: root.clone(),
            permissions: Arc::new(DenyWorktree),
            cancellation: CancellationToken::new(),
        };
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .unwrap();
        let result = GitStatusTool
            .execute(
                ToolRequest {
                    name: "git_status".into(),
                    arguments: serde_json::json!({"include_worktrees": true}),
                },
                context,
            )
            .await;
        assert!(matches!(
            result,
            Err(ToolError::Domain(DomainError::PermissionDenied { operation }))
                if operation == "git_worktree"
        ));
        assert!(root.join(".git").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn command_output_is_bounded_in_bytes() {
        let root = temp_root();
        let output = run_command("printf 'あ%.0s' $(seq 1 100000)", &context(&root))
            .await
            .unwrap();
        assert!(output.truncated);
        assert!(output.text.len() <= MAX_OUTPUT_BYTES);
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn command_timeout_kills_child() {
        let root = temp_root();
        let result =
            run_command_with_timeout("sleep 1", &context(&root), Duration::from_millis(10)).await;
        assert!(
            matches!(result, Err(ToolError::Failed(message)) if message == "command timed out")
        );
        fs::remove_dir(root).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn command_timeout_kills_descendants_in_process_group() {
        let root = temp_root();
        let marker = root.join("descendant-marker");
        let command = format!(
            "sleep 0.2; printf marker > '{}'",
            marker.to_string_lossy().replace('\'', "'\\''")
        );
        let result =
            run_command_with_timeout(&command, &context(&root), Duration::from_millis(20)).await;
        assert!(matches!(
            result,
            Err(ToolError::Failed(message)) if message == "command timed out"
        ));
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(!marker.exists());
        fs::remove_dir(root).unwrap();
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn command_timeout_kills_descendants_in_job_object() {
        let root = temp_root();
        let marker = root.join("descendant-marker");
        let marker_path = marker.to_string_lossy().replace('\\', "/");
        let command = format!(
            "sleep 0.5; printf marker > '{}'",
            marker_path.replace('\'', "'\\''")
        );
        let result =
            run_command_with_timeout(&command, &context(&root), Duration::from_millis(20)).await;
        assert!(matches!(
            result,
            Err(ToolError::Failed(message)) if message == "command timed out"
        ));
        tokio::time::sleep(Duration::from_millis(800)).await;
        assert!(!marker.exists());
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn command_cancellation_kills_child() {
        let root = temp_root();
        let context = context(&root);
        context.cancellation.cancel();
        let result = run_command_with_timeout("sleep 1", &context, Duration::from_secs(1)).await;
        assert!(matches!(
            result,
            Err(ToolError::Domain(DomainError::Cancelled))
        ));
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn exit_seven_is_failed_with_exit_code() {
        let root = temp_root();
        let result = run_command("exit 7", &context(&root)).await;
        assert!(matches!(
            result,
            Err(ToolError::Failed(message)) if message.contains("exit code 7")
        ));
        fs::remove_dir(root).unwrap();
    }

    #[tokio::test]
    async fn grandchild_pipe_does_not_outlive_deadline() {
        let root = temp_root();
        let started = std::time::Instant::now();
        let result = run_command_with_timeout(
            "sleep 1 & exit 0",
            &context(&root),
            Duration::from_millis(100),
        )
        .await;
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_millis(500));
        fs::remove_dir(root).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn provider_token_is_not_in_child_environment() {
        let root = temp_root();
        // SAFETY: this test runs in the process-local test environment and restores no
        // shared application state; the child environment is the behavior under test.
        unsafe { std::env::set_var("GITHUB_COPILOT_TOKEN", "test-provider-secret") };
        let result = run_command("env", &context(&root)).await.unwrap();
        unsafe { std::env::remove_var("GITHUB_COPILOT_TOKEN") };
        assert!(
            !result
                .text
                .contains("GITHUB_COPILOT_TOKEN=test-provider-secret")
        );
        fs::remove_dir(root).unwrap();
    }
}
