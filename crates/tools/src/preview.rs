use crate::{PermissionDecision, ToolContext, ToolError};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct PreviewLaunch {
    pub executable: String,
    pub arguments: Vec<String>,
    pub worktree: PathBuf,
    pub artifact_root: PathBuf,
    pub port: u16,
    pub readiness_deadline: Duration,
}

#[derive(Debug)]
pub struct PreviewProcess {
    child: Arc<Mutex<Option<Child>>>,
    stopped: Arc<tokio::sync::Notify>,
}

impl PreviewProcess {
    pub async fn start(launch: PreviewLaunch, context: ToolContext) -> Result<Self, ToolError> {
        if context
            .permissions
            .authorize("preview_start", &launch.worktree.to_string_lossy())
            .await?
            != PermissionDecision::Allow
        {
            return Err(ToolError::Domain(
                devfoundry_schema::DomainError::PermissionDenied {
                    operation: "preview_start".into(),
                },
            ));
        }
        let worktree = canonical_dir(&launch.worktree, "worktree")?;
        let artifact = canonical_dir(&launch.artifact_root, "artifact root")?;
        if !artifact.starts_with(&worktree) {
            return Err(ToolError::Failed(
                "artifact root is outside owned worktree".into(),
            ));
        }
        if launch.port == 0
            || launch.executable.is_empty()
            || launch.arguments.iter().any(|a| a.contains('\0'))
        {
            return Err(ToolError::Failed("invalid preview launch".into()));
        }
        let mut command = Command::new(&launch.executable);
        command
            .args(&launch.arguments)
            .current_dir(worktree)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("PORT", launch.port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        configure_process_group(&mut command);
        let child = command
            .spawn()
            .map_err(|e| ToolError::Failed(e.to_string()))?;
        Ok(Self {
            child: Arc::new(Mutex::new(Some(child))),
            stopped: Arc::new(tokio::sync::Notify::new()),
        })
    }

    pub async fn stop(&self) -> Result<(), ToolError> {
        let mut child = self.child.lock().await;
        if let Some(mut child) = child.take() {
            terminate_child(&mut child).await?;
        }
        self.stopped.notify_waiters();
        Ok(())
    }

    pub async fn is_stopped(&self) -> bool {
        self.child.lock().await.is_none()
    }
}

fn canonical_dir(path: &Path, label: &str) -> Result<PathBuf, ToolError> {
    let canonical = path
        .canonicalize()
        .map_err(|_| ToolError::Failed(format!("invalid {label}")))?;
    if !canonical.is_dir() {
        return Err(ToolError::Failed(format!("{label} is not a directory")));
    }
    Ok(canonical)
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    command.process_group(0);
}
#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

async fn terminate_child(child: &mut Child) -> Result<(), ToolError> {
    #[cfg(unix)]
    if let Some(pid) = child.id() {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
    let _ = child.kill().await;
    child
        .wait()
        .await
        .map_err(|e| ToolError::Failed(e.to_string()))?;
    Ok(())
}
