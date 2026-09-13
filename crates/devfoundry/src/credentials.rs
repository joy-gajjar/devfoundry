use async_trait::async_trait;
use devfoundry_core::{SecretBinding, SecretBindingError, SecretBindingRequest};
use std::{collections::HashMap, fmt, sync::Arc};
use tokio::sync::RwLock;

#[async_trait]
#[allow(dead_code)]
pub(crate) trait SecretStore: Send + Sync {
    async fn resolve(
        &self,
        request: SecretBindingRequest,
    ) -> Result<SecretHandle, SecretStoreError>;
}

#[allow(dead_code)]
pub(crate) struct SecretHandle {
    value: String,
}

impl SecretHandle {
    #[cfg(test)]
    fn expose_for_test(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for SecretHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretHandle(REDACTED)")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[allow(dead_code)]
pub(crate) enum SecretStoreError {
    #[error("secret store is unavailable or locked")]
    Unavailable,
    #[error("secret reference was not found")]
    NotFound,
    #[error("secret binding is disabled")]
    Disabled,
    #[error("secret binding is revoked")]
    Revoked,
    #[error("secret binding scope is not authorized")]
    ScopeMismatch,
}

impl From<SecretBindingError> for SecretStoreError {
    fn from(error: SecretBindingError) -> Self {
        match error {
            SecretBindingError::NotFound => Self::NotFound,
            SecretBindingError::Disabled => Self::Disabled,
            SecretBindingError::Revoked => Self::Revoked,
            SecretBindingError::ScopeMismatch => Self::ScopeMismatch,
        }
    }
}

#[derive(Clone, Default)]
#[allow(dead_code)]
struct FakeSecretStore {
    entries: Arc<RwLock<HashMap<String, (SecretBinding, String)>>>,
    locked: bool,
}

impl FakeSecretStore {
    #[cfg(test)]
    fn with_entry(binding: SecretBinding, value: impl Into<String>) -> Self {
        let mut entries = HashMap::new();
        entries.insert(binding.reference.id.clone(), (binding, value.into()));
        Self {
            entries: Arc::new(RwLock::new(entries)),
            locked: false,
        }
    }

    #[cfg(test)]
    fn locked() -> Self {
        Self {
            entries: Arc::default(),
            locked: true,
        }
    }
}

#[async_trait]
impl SecretStore for FakeSecretStore {
    async fn resolve(
        &self,
        request: SecretBindingRequest,
    ) -> Result<SecretHandle, SecretStoreError> {
        if self.locked {
            return Err(SecretStoreError::Unavailable);
        }
        let entries = self.entries.read().await;
        let Some((binding, value)) = entries.get(&request.reference.id) else {
            return Err(SecretStoreError::NotFound);
        };
        request.validate(binding)?;
        Ok(SecretHandle {
            value: value.clone(),
        })
    }
}

#[derive(Clone, Default)]
#[allow(dead_code)]
struct MacKeychainStore;

#[async_trait]
impl SecretStore for MacKeychainStore {
    async fn resolve(
        &self,
        _request: SecretBindingRequest,
    ) -> Result<SecretHandle, SecretStoreError> {
        // No vetted Keychain dependency is available in this workspace. Never
        // substitute an environment/config-file/CLI fallback.
        Err(SecretStoreError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devfoundry_core::{BindingScope, SecretReference};
    use devfoundry_schema::ProjectId;

    #[tokio::test]
    async fn fake_store_resolves_only_an_exact_enabled_binding() {
        let project = ProjectId::new();
        let reference = SecretReference::new("copilot", "Copilot");
        let binding = SecretBinding::new(reference.clone(), BindingScope::Project(project));
        let request = SecretBindingRequest::new(reference, project);
        let store = FakeSecretStore::with_entry(binding, "secret-value");

        let handle = store.resolve(request).await.unwrap();
        assert_eq!(handle.expose_for_test(), "secret-value");
    }

    #[tokio::test]
    async fn locked_store_and_missing_reference_fail_closed() {
        let project = ProjectId::new();
        let reference = SecretReference::new("copilot", "Copilot");
        let request = SecretBindingRequest::new(reference, project);
        let store = FakeSecretStore::locked();

        assert!(matches!(
            store.resolve(request.clone()).await,
            Err(SecretStoreError::Unavailable)
        ));
        let empty = FakeSecretStore::default();
        assert!(matches!(
            empty.resolve(request).await,
            Err(SecretStoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn secret_values_are_not_serialized_or_debugged() {
        let project = ProjectId::new();
        let reference = SecretReference::new("copilot", "Copilot");
        let binding = SecretBinding::new(reference.clone(), BindingScope::Project(project));
        let request = SecretBindingRequest::new(reference, project);
        let store = FakeSecretStore::with_entry(binding, "secret-value");
        let handle = store.resolve(request).await.unwrap();
        let debug = format!("{handle:?}");

        assert!(!debug.contains("secret-value"));
    }

    #[tokio::test]
    async fn provider_token_is_not_inherited_by_the_store() {
        let project = ProjectId::new();
        let request =
            SecretBindingRequest::new(SecretReference::new("copilot", "Copilot"), project);
        let store = FakeSecretStore::default();

        // The fake store has no ambient-environment path, so this remains a
        // deterministic missing-reference denial regardless of provider setup.
        let result = store.resolve(request).await;
        assert!(matches!(result, Err(SecretStoreError::NotFound)));
    }

    #[tokio::test]
    async fn unavailable_keychain_adapter_fails_closed() {
        let request =
            SecretBindingRequest::new(SecretReference::new("copilot", "Copilot"), ProjectId::new());
        assert!(matches!(
            MacKeychainStore.resolve(request).await,
            Err(SecretStoreError::Unavailable)
        ));
    }
}
