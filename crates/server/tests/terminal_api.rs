use axum::{body::Body, http::Request};
use devfoundry_core::PreviewRegistry;
use devfoundry_server::{ApiSecurity, ExecutionRegistry, ServerState, router};
use devfoundry_storage::SqliteStore;
use http_body_util::BodyExt;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

async fn fixture() -> (axum::Router, tempfile::TempDir) {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("terminal.db");
    let store = Arc::new(
        SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
            .await
            .unwrap(),
    );
    let runner = Arc::new(devfoundry_core::SessionRunner::new(
        store.clone(),
        Arc::new(TestProvider),
        Arc::new(devfoundry_tools::ToolRegistry::default()),
    ));
    let app = router(ServerState {
        store,
        runner,
        executions: Arc::new(ExecutionRegistry::default()),
        previews: Arc::new(tokio::sync::Mutex::new(PreviewRegistry::default())),
        preview_permissions: Arc::new(devfoundry_tools::AllowAllPermissions),
        terminals: Arc::new(tokio::sync::Mutex::new(Default::default())),
        workers: Arc::new(devfoundry_server::routes_workers::WorkerRegistry::default()),
    });
    let _ = ApiSecurity::default();
    (app, directory)
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
async fn terminal_routes_reject_unknown_terminal_without_leaking_paths() {
    let (app, _directory) = fixture().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v2/terminals/01H00000000000000000000000/output?after=0")
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
async fn terminal_creation_validates_dimensions_before_spawn() {
    let (app, directory) = fixture().await;
    let project = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({"root": directory.path(), "name": "terminal"}))
                        .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let project: serde_json::Value =
        serde_json::from_slice(&project.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v2/projects/{}/terminals",
                    project["id"].as_str().unwrap()
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "program": "/bin/sh",
                        "arguments": [],
                        "rows": 0,
                        "columns": 80
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}
