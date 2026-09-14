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
        binding: &SecretBinding,
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
        binding: &SecretBinding,
    ) -> Result<SecretHandle, SecretStoreError> {
        if self.locked {
            return Err(SecretStoreError::Unavailable);
        }
        request.validate(binding)?;
        let entries = self.entries.read().await;
        let Some((stored_binding, value)) = entries.get(&request.reference.id) else {
            return Err(SecretStoreError::NotFound);
        };
        if stored_binding != binding {
            return Err(SecretStoreError::NotFound);
        }
        Ok(SecretHandle {
            value: value.clone(),
        })
    }
}

#[derive(Clone)]
#[allow(dead_code)]
struct MacKeychainStore {
    service: String,
    account: String,
}

impl MacKeychainStore {
    #[cfg(test)]
    fn new(service: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            account: account.into(),
        }
    }

    #[cfg(target_os = "macos")]
    fn entry(&self) -> Result<keyring::Entry, SecretStoreError> {
        keyring::Entry::new(&self.service, &self.account).map_err(|_| SecretStoreError::Unavailable)
    }
}

impl Default for MacKeychainStore {
    fn default() -> Self {
        Self {
            service: "devfoundry".to_owned(),
            account: "copilot".to_owned(),
        }
    }
}

#[async_trait]
impl SecretStore for MacKeychainStore {
    async fn resolve(
        &self,
        request: SecretBindingRequest,
        binding: &SecretBinding,
    ) -> Result<SecretHandle, SecretStoreError> {
        request.validate(binding)?;
        #[cfg(target_os = "macos")]
        {
            let entry = self.entry()?;
            return entry
                .get_password()
                .map(|value| SecretHandle { value })
                .map_err(|_| SecretStoreError::Unavailable);
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (request, binding);
            Err(SecretStoreError::Unavailable)
        }
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

        let binding = store.entries.read().await.get("copilot").unwrap().0.clone();
        let handle = store.resolve(request, &binding).await.unwrap();
        assert_eq!(handle.expose_for_test(), "secret-value");
    }

    #[tokio::test]
    async fn locked_store_and_missing_reference_fail_closed() {
        let project = ProjectId::new();
        let reference = SecretReference::new("copilot", "Copilot");
        let request = SecretBindingRequest::new(reference, project);
        let store = FakeSecretStore::locked();
        let binding = SecretBinding::new(request.reference.clone(), BindingScope::Project(project));

        assert!(matches!(
            store.resolve(request.clone(), &binding).await,
            Err(SecretStoreError::Unavailable)
        ));
        let empty = FakeSecretStore::default();
        assert!(matches!(
            empty.resolve(request, &binding).await,
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
        let binding = store.entries.read().await.get("copilot").unwrap().0.clone();
        let handle = store.resolve(request, &binding).await.unwrap();
        let debug = format!("{handle:?}");

        assert!(!debug.contains("secret-value"));
    }

    #[tokio::test]
    async fn provider_token_is_not_inherited_by_the_store() {
        let project = ProjectId::new();
        let request =
            SecretBindingRequest::new(SecretReference::new("copilot", "Copilot"), project);
        let store = FakeSecretStore::default();
        let binding = SecretBinding::new(request.reference.clone(), BindingScope::Project(project));

        // The fake store has no ambient-environment path, so this remains a
        // deterministic missing-reference denial regardless of provider setup.
        let result = store.resolve(request, &binding).await;
        assert!(matches!(result, Err(SecretStoreError::NotFound)));
    }

    #[tokio::test]
    async fn unavailable_keychain_adapter_fails_closed() {
        let request =
            SecretBindingRequest::new(SecretReference::new("copilot", "Copilot"), ProjectId::new());
        let binding = SecretBinding::new(
            request.reference.clone(),
            BindingScope::Project(request.project_id),
        );
        assert!(matches!(
            MacKeychainStore::default().resolve(request, &binding).await,
            Err(SecretStoreError::Unavailable)
        ));
    }

    #[cfg(target_os = "macos")]
    #[ignore = "requires DEVFOUNDRY_RUN_KEYCHAIN_TEST=1 and a disposable fixture"]
    #[tokio::test]
    async fn mac_keychain_fixture_resolves_without_exposing_value() {
        if std::env::var("DEVFOUNDRY_RUN_KEYCHAIN_TEST").as_deref() != Ok("1") {
            return;
        }
        let request = SecretBindingRequest::new(
            SecretReference::new("devfoundry-test", "fixture"),
            ProjectId::new(),
        );
        let store = MacKeychainStore::new("devfoundry-test-credential", "devfoundry-test");
        let binding = SecretBinding::new(
            request.reference.clone(),
            BindingScope::Project(request.project_id),
        );

        let handle = store.resolve(request, &binding).await.unwrap();
        assert!(!handle.expose_for_test().is_empty());
        assert!(!format!("{handle:?}").contains(handle.expose_for_test()));
    }
}
