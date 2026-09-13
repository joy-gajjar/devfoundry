use async_trait::async_trait;
use axum::{body::Body, http::Request};
use devfoundry_core::SessionRunner;
use devfoundry_llm::LlmProvider;
use devfoundry_llm::{LlmError, LlmEvent, LlmRequest, LlmStream};
use devfoundry_server::{
    ApiSecurity, BrowserConfig, ExecutionRegistry, ServerState, router_with_browser,
};
use devfoundry_storage::SqliteStore;
use devfoundry_tools::ToolRegistry;
use http_body_util::BodyExt;
use serde_json::Value;
use std::{path::PathBuf, sync::Arc};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;

struct TestProvider;

#[async_trait]
impl LlmProvider for TestProvider {
    async fn stream(
        &self,
        _request: LlmRequest,
        _cancellation: CancellationToken,
    ) -> Result<LlmStream, LlmError> {
        Ok(Box::pin(futures_util::stream::iter([Ok(
            LlmEvent::Finished {
                reason: "stop".into(),
            },
        )])))
    }
}

async fn fixture(config: BrowserConfig) -> (axum::Router, tempfile::TempDir) {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("browser.db");
    let store = Arc::new(
        SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
            .await
            .unwrap(),
    );
    let provider: Arc<dyn LlmProvider> = Arc::new(TestProvider);
    let runner = Arc::new(SessionRunner::new(
        store.clone(),
        provider,
        Arc::new(ToolRegistry::default()),
    ));
    let app = router_with_browser(
        ServerState {
            store,
            runner,
            executions: Arc::new(ExecutionRegistry::default()),
            previews: Arc::new(tokio::sync::Mutex::new(
                devfoundry_core::PreviewRegistry::default(),
            )),
            preview_permissions: Arc::new(devfoundry_tools::AllowAllPermissions),
            terminals: Arc::new(tokio::sync::Mutex::new(Default::default())),
            workers: Arc::new(devfoundry_server::routes_workers::WorkerRegistry::default()),
        },
        ApiSecurity::default(),
        config,
    );
    (app, directory)
}

#[tokio::test]
async fn browser_host_is_disabled_unless_explicitly_enabled() {
    let (app, _directory) = fixture(BrowserConfig::default()).await;
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn browser_assets_have_csp_and_reject_root_escape() {
    let directory = tempfile::tempdir().unwrap();
    tokio::fs::write(directory.path().join("index.html"), "<html>ok</html>")
        .await
        .unwrap();
    let (app, _directory) = fixture(BrowserConfig {
        enabled: true,
        asset_root: Some(PathBuf::from(directory.path())),
        allowed_origin: Some("http://127.0.0.1:4096".parse().unwrap()),
    })
    .await;

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    assert!(
        response.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("default-src 'self'")
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/../Cargo.toml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn bootstrap_is_non_secret_and_browser_mutations_require_csrf_and_origin() {
    let directory = tempfile::tempdir().unwrap();
    tokio::fs::write(directory.path().join("index.html"), "ok")
        .await
        .unwrap();
    let (app, _directory) = fixture(BrowserConfig {
        enabled: true,
        asset_root: Some(PathBuf::from(directory.path())),
        allowed_origin: Some("http://127.0.0.1:4096".parse().unwrap()),
    })
    .await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v2/browser/bootstrap")
                .header("host", "127.0.0.1:4096")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["version"], 2);
    assert!(body.get("token").is_none());
    assert!(body.get("authorization").is_none());
    assert!(body.get("csrf").is_none());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sessions/not-a-session/interrupt")
                .header("origin", "http://127.0.0.1:4096")
                .header("host", "127.0.0.1:4096")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["code"], "csrf_required");
}
