use devfoundry_tools::{PermissionBroker, PreviewLaunch, PreviewProcess, ToolContext, ToolError};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
pub struct PreviewId(ulid::Ulid);
impl PreviewId {
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }
}
impl Default for PreviewId {
    fn default() -> Self {
        Self::new()
    }
}
impl std::fmt::Display for PreviewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::str::FromStr for PreviewId {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self).map_err(|_| ())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PreviewState {
    Requested,
    Starting,
    Ready,
    Failed,
    Stopped,
}

#[derive(Debug)]
pub struct PreviewService {
    id: PreviewId,
    worktree: String,
    revision: String,
    state: PreviewState,
    url: Option<String>,
}
impl PreviewService {
    pub fn new(id: PreviewId, worktree: impl Into<String>, revision: impl Into<String>) -> Self {
        Self {
            id,
            worktree: worktree.into(),
            revision: revision.into(),
            state: PreviewState::Requested,
            url: None,
        }
    }
    pub fn with_permissions(
        _permissions: Arc<dyn PermissionBroker>,
    ) -> Arc<Mutex<PreviewRegistry>> {
        Arc::new(Mutex::new(PreviewRegistry::default()))
    }
    pub fn state(&self) -> PreviewState {
        self.state
    }
    pub fn starting(&mut self) -> Result<(), ToolError> {
        self.transition(PreviewState::Starting)
    }
    pub fn ready(&mut self, url: impl Into<String>) -> Result<(), ToolError> {
        let url = url.into();
        if !url.starts_with("http://127.0.0.1:") {
            return Err(ToolError::Failed("preview URL is not loopback".into()));
        }
        self.url = Some(url);
        self.transition(PreviewState::Ready)
    }
    pub fn stopped(&mut self) -> Result<(), ToolError> {
        self.transition(PreviewState::Stopped)
    }
    pub fn accept_annotation(&self, revision: &str) -> Result<(), ToolError> {
        if self.state != PreviewState::Ready || revision != self.revision {
            return Err(ToolError::Failed("preview revision is stale".into()));
        }
        Ok(())
    }
    pub fn id(&self) -> PreviewId {
        self.id
    }
    pub fn worktree(&self) -> &str {
        &self.worktree
    }
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
    fn transition(&mut self, next: PreviewState) -> Result<(), ToolError> {
        if self.state == PreviewState::Stopped
            || (self.state == PreviewState::Ready && next != PreviewState::Stopped)
        {
            return Err(ToolError::Failed("invalid preview transition".into()));
        }
        self.state = next;
        Ok(())
    }
}

#[derive(Default)]
pub struct PreviewRegistry {
    previews: HashMap<PreviewId, (PreviewService, Option<PreviewProcess>)>,
}
impl PreviewRegistry {
    pub fn insert(&mut self, service: PreviewService, process: PreviewProcess) -> PreviewId {
        let id = service.id();
        self.previews.insert(id, (service, Some(process)));
        id
    }
    pub fn get(&self, id: PreviewId) -> Option<&PreviewService> {
        self.previews.get(&id).map(|v| &v.0)
    }
    pub fn get_mut(&mut self, id: PreviewId) -> Option<&mut PreviewService> {
        self.previews.get_mut(&id).map(|v| &mut v.0)
    }
    pub async fn stop(&mut self, id: PreviewId) -> Result<(), ToolError> {
        let (service, process) = self
            .previews
            .get_mut(&id)
            .ok_or_else(|| ToolError::Failed("preview not found".into()))?;
        if let Some(process) = process.take() {
            process.stop().await?;
        }
        service.stopped()
    }
    pub async fn start(
        &mut self,
        id: PreviewId,
        worktree: PathBuf,
        artifact_root: PathBuf,
        port: u16,
        permissions: Arc<dyn PermissionBroker>,
    ) -> Result<(), ToolError> {
        let context = ToolContext {
            root: worktree.clone(),
            permissions,
            cancellation: CancellationToken::new(),
        };
        let process = PreviewProcess::start(
            PreviewLaunch {
                executable: "/bin/sh".into(),
                arguments: vec!["-c".into(), "sleep 30".into()],
                worktree,
                artifact_root,
                port,
                readiness_deadline: std::time::Duration::from_secs(5),
            },
            context,
        )
        .await?;
        let service = self
            .previews
            .get_mut(&id)
            .ok_or_else(|| ToolError::Failed("preview not found".into()))?;
        service.0.starting()?;
        service.0.ready(format!("http://127.0.0.1:{port}"))?;
        service.1 = Some(process);
        Ok(())
    }
}
