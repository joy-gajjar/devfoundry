//! Narrow contracts for future MCP, LSP, and PTY adapters.
//!
//! This module deliberately contains no transport, process spawning, or provider
//! registration. Adapters must validate a request and ask the normal permission
//! broker before doing any external work.

use serde::{Deserialize, Serialize};
use std::fmt;

const MAX_LABEL_BYTES: usize = 128;
const MAX_TARGET_BYTES: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationKind {
    Mcp,
    Lsp,
    Pty,
}

impl fmt::Display for IntegrationKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Mcp => "mcp",
            Self::Lsp => "lsp",
            Self::Pty => "pty",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationCapability {
    McpTool,
    LspDiagnostics,
    PtySession,
}

impl IntegrationCapability {
    fn kind(self) -> IntegrationKind {
        match self {
            Self::McpTool => IntegrationKind::Mcp,
            Self::LspDiagnostics => IntegrationKind::Lsp,
            Self::PtySession => IntegrationKind::Pty,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationRequest {
    pub kind: IntegrationKind,
    pub capability: IntegrationCapability,
    /// A server name, workspace identifier, or executable label supplied by the adapter.
    pub target: String,
}

impl IntegrationRequest {
    pub fn validate(&self) -> Result<(), IntegrationContractError> {
        if self.capability.kind() != self.kind {
            return Err(IntegrationContractError::CapabilityMismatch);
        }
        validate_text(&self.target, MAX_TARGET_BYTES, "target")
    }

    /// The operation class adapters must pass to `PermissionBroker::authorize`.
    pub fn permission_operation(&self) -> &'static str {
        match self.capability {
            IntegrationCapability::McpTool => "mcp_tool",
            IntegrationCapability::LspDiagnostics => "lsp_diagnostics",
            IntegrationCapability::PtySession => "pty_session",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrationDescriptor {
    pub kind: IntegrationKind,
    pub name: String,
    pub capabilities: Vec<IntegrationCapability>,
}

impl IntegrationDescriptor {
    pub fn validate(&self) -> Result<(), IntegrationContractError> {
        validate_text(&self.name, MAX_LABEL_BYTES, "name")?;
        if self.capabilities.is_empty() {
            return Err(IntegrationContractError::NoCapabilities);
        }
        if self
            .capabilities
            .iter()
            .any(|capability| capability.kind() != self.kind)
        {
            return Err(IntegrationContractError::CapabilityMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum IntegrationContractError {
    #[error("integration name is empty or exceeds {0} bytes")]
    InvalidName(usize),
    #[error("integration target is empty or exceeds {0} bytes")]
    InvalidTarget(usize),
    #[error("integration value contains control characters")]
    ControlCharacter,
    #[error("integration capability does not match integration kind")]
    CapabilityMismatch,
    #[error("integration must declare at least one capability")]
    NoCapabilities,
}

fn validate_text(
    value: &str,
    max_bytes: usize,
    field: &str,
) -> Result<(), IntegrationContractError> {
    if value.is_empty() || value.len() > max_bytes {
        return Err(if field == "name" {
            IntegrationContractError::InvalidName(max_bytes)
        } else {
            IntegrationContractError::InvalidTarget(max_bytes)
        });
    }
    if value.chars().any(char::is_control) {
        return Err(IntegrationContractError::ControlCharacter);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contracts_keep_capabilities_in_their_own_trust_domain() {
        let request = IntegrationRequest {
            kind: IntegrationKind::Lsp,
            capability: IntegrationCapability::LspDiagnostics,
            target: "workspace".into(),
        };
        assert!(request.validate().is_ok());
        assert_eq!(request.permission_operation(), "lsp_diagnostics");
    }

    #[test]
    fn mismatched_capability_is_rejected() {
        let request = IntegrationRequest {
            kind: IntegrationKind::Pty,
            capability: IntegrationCapability::McpTool,
            target: "server".into(),
        };
        assert_eq!(
            request.validate(),
            Err(IntegrationContractError::CapabilityMismatch)
        );
    }

    #[test]
    fn control_characters_and_empty_capability_sets_are_rejected() {
        let request = IntegrationRequest {
            kind: IntegrationKind::Pty,
            capability: IntegrationCapability::PtySession,
            target: "terminal\ncommand".into(),
        };
        assert_eq!(
            request.validate(),
            Err(IntegrationContractError::ControlCharacter)
        );
        let descriptor = IntegrationDescriptor {
            kind: IntegrationKind::Mcp,
            name: "server".into(),
            capabilities: vec![],
        };
        assert_eq!(
            descriptor.validate(),
            Err(IntegrationContractError::NoCapabilities)
        );
    }

    #[test]
    fn descriptors_cannot_cross_capability_boundaries() {
        let descriptor = IntegrationDescriptor {
            kind: IntegrationKind::Lsp,
            name: "rust-analyzer".into(),
            capabilities: vec![IntegrationCapability::PtySession],
        };
        assert_eq!(
            descriptor.validate(),
            Err(IntegrationContractError::CapabilityMismatch)
        );
    }
}
