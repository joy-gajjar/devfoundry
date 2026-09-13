use devfoundry_schema::{ContextManifest, TaskId};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerBrief {
    pub task_id: TaskId,
    pub task_revision: u64,
    pub objective: String,
    pub base_commit: String,
    pub owned_scope: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub context: ContextManifest,
}

impl WorkerBrief {
    pub fn new(
        task_id: TaskId,
        task_revision: u64,
        objective: impl Into<String>,
        base_commit: impl Into<String>,
        owned_scope: Vec<String>,
        allowed_tools: Vec<String>,
        context: ContextManifest,
    ) -> Self {
        Self {
            task_id,
            task_revision,
            objective: objective.into(),
            base_commit: base_commit.into(),
            owned_scope,
            allowed_tools,
            context,
        }
    }

    pub fn validate_provider(provider: &str) -> Result<(), WorkerFailure> {
        if provider == "github-copilot" || provider == "copilot" {
            Ok(())
        } else {
            Err(WorkerFailure::UnsupportedProvider)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceBundle {
    pub task_id: TaskId,
    pub base_commit: String,
    pub proposed_head: String,
    pub diff_digest: String,
}

impl EvidenceBundle {
    pub fn new(
        task_id: TaskId,
        base_commit: impl Into<String>,
        proposed_head: impl Into<String>,
        diff_digest: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            base_commit: base_commit.into(),
            proposed_head: proposed_head.into(),
            diff_digest: diff_digest.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkerFailure {
    UnsupportedProvider,
    ExecutionUnavailable,
    Execution(String),
}

impl fmt::Display for WorkerFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::UnsupportedProvider => "unsupported provider",
                Self::ExecutionUnavailable => "worker execution adapter unavailable",
                Self::Execution(message) => message,
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkerOutcome {
    Evidence(EvidenceBundle),
    UnknownSideEffect(TaskId),
}

impl WorkerOutcome {
    pub fn unknown(task_id: TaskId) -> Self {
        Self::UnknownSideEffect(task_id)
    }
    pub fn review_status(&self) -> &'static str {
        "review"
    }
}
