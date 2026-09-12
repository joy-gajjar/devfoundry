//! Stable domain values shared by the DevFoundry components.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use ulid::Ulid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Ulid);

        impl $name {
            pub fn new() -> Self {
                Self(Ulid::new())
            }

            pub const fn from_ulid(value: Ulid) -> Self {
                Self(value)
            }

            pub const fn into_ulid(self) -> Ulid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                value
                    .parse::<Ulid>()
                    .map(Self)
                    .map_err(|error| error.to_string())
            }
        }
    };
}

id_type!(ProjectId);
id_type!(SessionId);
id_type!(MessageId);
id_type!(PartId);
id_type!(ToolCallId);
id_type!(PermissionRequestId);
id_type!(RunId);
id_type!(AttemptId);
id_type!(TaskId);
id_type!(ArtifactId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Draft,
    Ready,
    Assigned,
    Running,
    Blocked,
    Review,
    Accepted,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub revision: Revision,
    pub dependencies: Vec<TaskId>,
    pub session_id: Option<SessionId>,
    pub evidence_id: Option<ArtifactId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new_for_project(ProjectId::new(), title, description)
    }

    pub fn new_for_project(
        project_id: ProjectId,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: TaskId::new(),
            project_id,
            title: title.into(),
            description: description.into(),
            status: TaskStatus::Draft,
            revision: Revision(0),
            dependencies: Vec::new(),
            session_id: None,
            evidence_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextSource {
    pub identity: String,
    pub content_hash: String,
    pub provenance: String,
}

impl ContextSource {
    pub fn new(
        identity: impl Into<String>,
        content_hash: impl Into<String>,
        provenance: impl Into<String>,
    ) -> Self {
        Self {
            identity: identity.into(),
            content_hash: content_hash.into(),
            provenance: provenance.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextManifest {
    pub version: u16,
    pub sources: Vec<ContextSource>,
    pub unresolved_tool_groups: Vec<String>,
}

impl ContextManifest {
    pub fn new(sources: Vec<ContextSource>) -> Self {
        Self {
            version: 1,
            sources,
            unresolved_tool_groups: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub path: String,
    pub content_hash: String,
    pub links: Vec<String>,
}

impl DocumentMetadata {
    pub fn new(path: impl Into<String>, content_hash: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content_hash: content_hash.into(),
            links: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Revision(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RunOutcome {
    Completed,
    Failed,
    Cancelled,
    Interrupted,
    OutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RunStatus {
    Admitted,
    Running,
    WaitingPermission,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
    OutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AttemptStatus {
    Started,
    Completed,
    Failed,
    Cancelled,
    OutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum IdempotencyResult {
    New,
    Existing,
    Conflict,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutionLease {
    pub owner_id: String,
    pub revision: Revision,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub version: u16,
    pub sequence: u64,
    pub run_id: RunId,
    pub aggregate_revision: Revision,
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventSequence(pub u64);

impl fmt::Display for EventSequence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub root: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub project_id: ProjectId,
    pub title: String,
    pub agent: AgentName,
    pub model: ModelRef,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Portable session data. This intentionally excludes projects, events,
/// prompts, permissions, database paths, provider configuration, and secrets.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionExport {
    pub format: String,
    pub version: u32,
    pub session: ExportedSession,
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExportedSession {
    pub id: SessionId,
    pub title: String,
    pub agent: AgentName,
    pub model: ModelRef,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SessionExport {
    pub const FORMAT: &'static str = "devfoundry.session";
    pub const VERSION: u32 = 1;

    pub fn new(session: &Session, messages: Vec<Message>) -> Self {
        Self {
            format: Self::FORMAT.into(),
            version: Self::VERSION,
            session: ExportedSession {
                id: session.id,
                title: session.title.clone(),
                agent: session.agent.clone(),
                model: session.model.clone(),
                created_at: session.created_at,
                updated_at: session.updated_at,
            },
            messages,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.format != Self::FORMAT || self.version != Self::VERSION {
            return Err(DomainError::Validation(
                "unsupported session export format or version".into(),
            ));
        }
        if self
            .messages
            .iter()
            .any(|message| message.session_id != self.session.id)
        {
            return Err(DomainError::Validation(
                "export contains a message for another session".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentName(pub String);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderName(pub String);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelName(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelRef {
    pub provider: ProviderName,
    pub model: ModelName,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Idle,
    Running,
    WaitingPermission,
    WaitingQuestion,
    Cancelling,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub session_id: SessionId,
    pub role: MessageRole,
    pub parts: Vec<MessagePart>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessagePart {
    Text {
        id: PartId,
        text: String,
    },
    Reasoning {
        id: PartId,
        text: String,
    },
    ToolCall {
        id: PartId,
        call_id: ToolCallId,
        name: String,
        arguments: String,
    },
    ToolResult {
        id: PartId,
        call_id: ToolCallId,
        output: String,
        truncated: bool,
    },
    Error {
        id: PartId,
        message: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    SessionStatus {
        session_id: SessionId,
        status: SessionStatus,
    },
    MessageCreated {
        session_id: SessionId,
        message_id: MessageId,
    },
    MessageDelta {
        session_id: SessionId,
        message_id: MessageId,
        text: String,
    },
    ToolStarted {
        session_id: SessionId,
        call_id: ToolCallId,
        name: String,
    },
    ToolFinished {
        session_id: SessionId,
        call_id: ToolCallId,
        success: bool,
    },
    PermissionRequested {
        session_id: SessionId,
        request_id: PermissionRequestId,
    },
    PermissionResolved {
        session_id: SessionId,
        request_id: PermissionRequestId,
        allowed: bool,
    },
    Error {
        session_id: Option<SessionId>,
        message: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    Pending,
    Allowed,
    Denied,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: PermissionRequestId,
    pub session_id: SessionId,
    pub operation: String,
    pub target: String,
    pub status: PermissionStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DomainError {
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),
    #[error("invalid state transition: {0}")]
    InvalidStateTransition(String),
    #[error("resource not found: {resource}")]
    NotFound { resource: String },
    #[error("permission denied: {operation}")]
    PermissionDenied { operation: String },
    #[error("operation cancelled")]
    Cancelled,
    #[error("validation failed: {0}")]
    Validation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_through_json() {
        let id = SessionId::new();
        let encoded = serde_json::to_string(&id).unwrap();
        let decoded: SessionId = serde_json::from_str(&encoded).unwrap();
        assert_eq!(id, decoded);
    }

    #[test]
    fn message_parts_use_stable_tagged_json() {
        let part = MessagePart::Text {
            id: PartId::new(),
            text: "hello".to_owned(),
        };
        let value = serde_json::to_value(part).unwrap();
        assert_eq!(value["type"], "text");
        assert_eq!(value["text"], "hello");
    }

    #[test]
    fn session_export_has_explicit_version_and_no_project_or_status() {
        let now = Utc::now();
        let session = Session {
            id: SessionId::new(),
            project_id: ProjectId::new(),
            title: "portable".into(),
            agent: AgentName("build".into()),
            model: ModelRef {
                provider: ProviderName("fake".into()),
                model: ModelName("test".into()),
            },
            status: SessionStatus::Error,
            created_at: now,
            updated_at: now,
        };
        let export = SessionExport::new(&session, Vec::new());
        let value = serde_json::to_value(export).unwrap();
        assert_eq!(value["format"], "devfoundry.session");
        assert_eq!(value["version"], 1);
        assert!(value["session"].get("project_id").is_none());
        assert!(value["session"].get("status").is_none());
    }
}
