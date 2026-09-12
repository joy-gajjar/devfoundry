//! Transport-neutral request and response contracts.

use devfoundry_schema::{AgentName, ArtifactId, ModelRef, ProjectId, Revision, SessionId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub project_id: ProjectId,
    pub title: Option<String>,
    pub agent: Option<AgentName>,
    pub model: Option<ModelRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub root: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub id: SessionId,
    pub project_id: ProjectId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelOption {
    pub provider: String,
    pub id: String,
    pub display_name: String,
    pub context_window: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UpdateSessionRequest {
    pub title: Option<String>,
    pub agent: Option<AgentName>,
    pub model: Option<ModelRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptRequest {
    pub session_id: SessionId,
    pub prompt: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttachmentRef {
    pub artifact_id: ArtifactId,
    pub sha256: String,
    pub media_type: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubmitPrompt {
    pub session_id: SessionId,
    pub idempotency_key: String,
    pub prompt: String,
    pub expected_session_revision: Revision,
    pub attachments: Vec<AttachmentRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptStatusResponse {
    pub prompt_id: devfoundry_schema::MessageId,
    pub session_id: SessionId,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use devfoundry_schema::{ArtifactId, EventEnvelope, Revision, RunId, RunOutcome};

    #[test]
    fn prompt_request_round_trips() {
        let request = PromptRequest {
            session_id: SessionId::new(),
            prompt: "inspect the project".to_owned(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: PromptRequest = serde_json::from_str(&encoded).unwrap();
        assert_eq!(request, decoded);
    }

    #[test]
    fn prompt_submission_preserves_idempotency_and_attachments() {
        let request = SubmitPrompt {
            session_id: SessionId::new(),
            idempotency_key: "prompt-1".into(),
            prompt: "inspect the project".into(),
            expected_session_revision: Revision(3),
            attachments: vec![AttachmentRef {
                artifact_id: ArtifactId::new(),
                sha256: "abc123".into(),
                media_type: "text/plain".into(),
            }],
        };
        let decoded: SubmitPrompt =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(request, decoded);
    }

    #[test]
    fn event_envelope_round_trips_terminal_outcome() {
        let envelope = EventEnvelope {
            version: 1,
            sequence: 12,
            run_id: RunId::new(),
            aggregate_revision: Revision(4),
            kind: "run.completed".into(),
            payload: serde_json::json!({ "outcome": RunOutcome::Completed }),
        };
        let decoded: EventEnvelope =
            serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
        assert_eq!(envelope, decoded);
    }
}
