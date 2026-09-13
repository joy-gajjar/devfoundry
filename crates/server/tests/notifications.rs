use axum::{body::Body, http::Request};
use devfoundry_server::{ExecutionRegistry, ServerState, router};
use http_body_util::BodyExt;
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn notification_status_is_disabled_and_setup_never_returns_secret_material() {
    let directory = tempfile::tempdir().unwrap();
    let store = Arc::new(
        devfoundry_storage::SqliteStore::connect_path(directory.path().join("server.db"))
            .await
            .unwrap(),
    );
    let provider: Arc<dyn devfoundry_llm::LlmProvider> = Arc::new(TestProvider);
    let runner = Arc::new(devfoundry_core::SessionRunner::new(
        store.clone(),
        provider,
        Arc::new(devfoundry_tools::ToolRegistry::default()),
    ));
    let app = router(ServerState {
        store,
        runner,
        executions: Arc::new(ExecutionRegistry::default()),
        previews: Arc::new(tokio::sync::Mutex::new(Default::default())),
        preview_permissions: Arc::new(devfoundry_tools::AllowAllPermissions),
        terminals: Arc::new(tokio::sync::Mutex::new(Default::default())),
        workers: Arc::new(devfoundry_server::routes_workers::WorkerRegistry::default()),
    });
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v2/projects/01ARZ3NDEKTSV4RRFFQ69G5FAV/notifications/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["enabled"], false);
    assert!(body.get("token").is_none());
    assert!(body.get("secret").is_none());
    assert!(body.get("command").is_none());
}

struct TestProvider;

#[async_trait::async_trait]
impl devfoundry_llm::LlmProvider for TestProvider {
    async fn stream(
        &self,
        _request: devfoundry_llm::LlmRequest,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<devfoundry_llm::LlmStream, devfoundry_llm::LlmError> {
        Ok(Box::pin(futures_util::stream::empty()))
    }
}
