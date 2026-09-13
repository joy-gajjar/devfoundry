use devfoundry_schema::ProjectId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretReference {
    pub id: String,
    pub label: String,
}

impl SecretReference {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum BindingScope {
    Project(ProjectId),
    Process {
        project_id: ProjectId,
        process_id: String,
    },
    Account {
        project_id: ProjectId,
        account_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretBinding {
    pub reference: SecretReference,
    pub scope: BindingScope,
    pub enabled: bool,
    pub revoked: bool,
}

impl SecretBinding {
    pub fn new(reference: SecretReference, scope: BindingScope) -> Self {
        Self {
            reference,
            scope,
            enabled: true,
            revoked: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretBindingRequest {
    pub reference: SecretReference,
    pub project_id: ProjectId,
    pub process_id: Option<String>,
    pub account_id: Option<String>,
}

impl SecretBindingRequest {
    pub fn new(reference: SecretReference, project_id: ProjectId) -> Self {
        Self {
            reference,
            project_id,
            process_id: None,
            account_id: None,
        }
    }

    pub fn for_process(
        reference: SecretReference,
        project_id: ProjectId,
        process_id: impl Into<String>,
    ) -> Self {
        Self {
            process_id: Some(process_id.into()),
            ..Self::new(reference, project_id)
        }
    }

    pub fn for_account(
        reference: SecretReference,
        project_id: ProjectId,
        account_id: impl Into<String>,
    ) -> Self {
        Self {
            account_id: Some(account_id.into()),
            ..Self::new(reference, project_id)
        }
    }

    pub fn validate(&self, binding: &SecretBinding) -> Result<(), SecretBindingError> {
        if binding.reference != self.reference {
            return Err(SecretBindingError::NotFound);
        }
        if !binding.enabled {
            return Err(SecretBindingError::Disabled);
        }
        if binding.revoked {
            return Err(SecretBindingError::Revoked);
        }
        let matches = match &binding.scope {
            BindingScope::Project(project_id) => {
                *project_id == self.project_id
                    && self.process_id.is_none()
                    && self.account_id.is_none()
            }
            BindingScope::Process {
                project_id,
                process_id,
            } => *project_id == self.project_id && self.process_id.as_deref() == Some(process_id),
            BindingScope::Account {
                project_id,
                account_id,
            } => *project_id == self.project_id && self.account_id.as_deref() == Some(account_id),
        };
        if matches {
            Ok(())
        } else {
            Err(SecretBindingError::ScopeMismatch)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum SecretBindingError {
    #[error("secret binding not found")]
    NotFound,
    #[error("secret binding is disabled")]
    Disabled,
    #[error("secret binding is revoked")]
    Revoked,
    #[error("secret binding scope does not match the request")]
    ScopeMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_serialization_never_contains_a_secret_value() {
        let reference = SecretReference::new("copilot-account", "GitHub Copilot");
        let binding =
            SecretBinding::new(reference.clone(), BindingScope::Project(ProjectId::new()));
        let encoded = serde_json::to_string(&binding).unwrap();

        assert!(!encoded.contains("secret-value"));
        assert!(encoded.contains("copilot-account"));
    }

    #[test]
    fn exact_project_scope_is_required() {
        let project = ProjectId::new();
        let request = SecretBindingRequest::new(
            SecretReference::new("copilot-account", "GitHub Copilot"),
            project,
        );
        let binding = SecretBinding::new(request.reference.clone(), BindingScope::Project(project));

        assert!(request.validate(&binding).is_ok());
        let other_project = SecretBindingRequest::new(request.reference, ProjectId::new());
        assert_eq!(
            other_project.validate(&binding),
            Err(SecretBindingError::ScopeMismatch)
        );
    }

    #[test]
    fn disabled_and_revoked_bindings_fail_closed() {
        let project = ProjectId::new();
        let reference = SecretReference::new("copilot-account", "GitHub Copilot");
        let request = SecretBindingRequest::new(reference.clone(), project);
        let mut binding = SecretBinding::new(reference, BindingScope::Project(project));

        binding.enabled = false;
        assert_eq!(
            request.validate(&binding),
            Err(SecretBindingError::Disabled)
        );
        binding.enabled = true;
        binding.revoked = true;
        assert_eq!(request.validate(&binding), Err(SecretBindingError::Revoked));
    }

    #[test]
    fn process_and_account_scopes_do_not_match_project_requests() {
        let project = ProjectId::new();
        let reference = SecretReference::new("account", "Provider");
        let request = SecretBindingRequest::new(reference.clone(), project);
        let process_binding = SecretBinding::new(
            reference.clone(),
            BindingScope::Process {
                project_id: project,
                process_id: "worker-1".into(),
            },
        );
        let account_binding = SecretBinding::new(
            reference,
            BindingScope::Account {
                project_id: project,
                account_id: "account-1".into(),
            },
        );

        assert_eq!(
            request.validate(&process_binding),
            Err(SecretBindingError::ScopeMismatch)
        );
        assert_eq!(
            request.validate(&account_binding),
            Err(SecretBindingError::ScopeMismatch)
        );
    }

    #[test]
    fn process_and_account_scopes_cannot_cross_projects() {
        let project = ProjectId::new();
        let other_project = ProjectId::new();
        let reference = SecretReference::new("account", "Provider");
        let process_binding = SecretBinding::new(
            reference.clone(),
            BindingScope::Process {
                project_id: other_project,
                process_id: "worker-1".into(),
            },
        );
        let account_binding = SecretBinding::new(
            reference.clone(),
            BindingScope::Account {
                project_id: other_project,
                account_id: "account-1".into(),
            },
        );

        assert_eq!(
            SecretBindingRequest::for_process(reference.clone(), project, "worker-1")
                .validate(&process_binding),
            Err(SecretBindingError::ScopeMismatch)
        );
        assert_eq!(
            SecretBindingRequest::for_account(reference, project, "account-1")
                .validate(&account_binding),
            Err(SecretBindingError::ScopeMismatch)
        );
    }
}
