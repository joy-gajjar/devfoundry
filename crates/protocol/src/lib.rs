//! Transport-neutral request and response contracts.

use devfoundry_schema::{AgentName, ModelRef, ProjectId, SessionId};
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
}
