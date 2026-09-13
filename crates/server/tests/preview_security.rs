use axum::{body::Body, http::Request};
use devfoundry_core::PreviewRegistry;
use devfoundry_server::{ApiSecurity, ExecutionRegistry, ServerState, router_with_previews};
use devfoundry_storage::SqliteStore;
use devfoundry_tools::AllowAllPermissions;
use http_body_util::BodyExt;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

async fn fixture() -> axum::Router {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("preview.db");
    let store = Arc::new(
        SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
            .await
            .unwrap(),
    );
    let provider: Arc<dyn devfoundry_llm::LlmProvider> = Arc::new(TestProvider);
    let runner = Arc::new(devfoundry_core::SessionRunner::new(
        store.clone(),
        provider,
        Arc::new(devfoundry_tools::ToolRegistry::default()),
    ));
    let service = Arc::new(tokio::sync::Mutex::new(PreviewRegistry::default()));
    router_with_previews(
        ServerState {
            store,
            runner,
            executions: Arc::new(ExecutionRegistry::default()),
            previews: service.clone(),
            preview_permissions: Arc::new(AllowAllPermissions),
            terminals: Arc::new(tokio::sync::Mutex::new(Default::default())),
        },
        ApiSecurity::default(),
        service,
    )
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

#[tokio::test]
async fn preview_routes_reject_missing_preview_without_leaking_paths() {
    let app = fixture().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v2/previews/01H00000000000000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value.get("path"), None);
}

#[tokio::test]
async fn preview_start_requires_bounded_worktree_and_revision_fields() {
    let app = fixture().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/projects/project-1/previews")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "worktree": "../outside",
                        "artifact_root": "../outside/dist",
                        "revision": "",
                        "port": 0
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}
