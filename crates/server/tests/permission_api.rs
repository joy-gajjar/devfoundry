use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use devfoundry_core::SessionRunner;
use devfoundry_llm::{LlmEvent, LlmProvider, LlmRequest, LlmStream};
use devfoundry_schema::PermissionRequestId;
use devfoundry_server::{ExecutionRegistry, ServerState, router};
use devfoundry_storage::SqliteStore;
use devfoundry_tools::{ToolRegistry, WriteTool};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;

struct ToolCallProvider {
    calls: AtomicUsize,
}

#[async_trait::async_trait]
impl LlmProvider for ToolCallProvider {
    async fn stream(
        &self,
        _request: LlmRequest,
        cancellation: CancellationToken,
    ) -> Result<LlmStream, devfoundry_llm::LlmError> {
        let turn = self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(Box::pin(async_stream::stream! {
            if cancellation.is_cancelled() {
                yield Err(devfoundry_llm::LlmError::Cancelled);
                return;
            }
            if turn == 0 {
                yield Ok(LlmEvent::ToolCall {
                    id: "fixture-call".into(),
                    name: "write".into(),
                    arguments: r#"{"path":"permission-fixture.txt","content":"approved"}"#.into(),
                });
            } else {
                yield Ok(LlmEvent::TextDelta { text: "finished".into() });
            }
            yield Ok(LlmEvent::Finished { reason: "stop".into() });
        }))
    }
}

async fn fixture() -> (Router, tempfile::TempDir) {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("permissions.db");
    let store = Arc::new(
        SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
            .await
            .unwrap(),
    );
    let mut tools = ToolRegistry::default();
    tools.register(Arc::new(WriteTool));
    let runner = Arc::new(SessionRunner::new(
        store.clone(),
        Arc::new(ToolCallProvider {
            calls: AtomicUsize::new(0),
        }),
        Arc::new(tools),
    ));
    let app = router(ServerState {
        store,
        runner,
        executions: Arc::new(ExecutionRegistry::default()),
        previews: Arc::new(tokio::sync::Mutex::new(
            devfoundry_core::PreviewRegistry::default(),
        )),
        preview_permissions: Arc::new(devfoundry_tools::AllowAllPermissions),
        terminals: Arc::new(tokio::sync::Mutex::new(Default::default())),
        workers: Arc::new(devfoundry_server::routes_workers::WorkerRegistry::default()),
    });
    (app, directory)
}

async fn json_response(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn session(app: &Router, directory: &tempfile::TempDir) -> String {
    let project = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"root": directory.path(), "name": "fixture"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let project = json_response(project).await;
    let project_id = project["id"].as_str().unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/projects/{project_id}/sessions"))
                .header("content-type", "application/json")
                .body(Body::from(json!({"project_id": project_id}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    json_response(response).await["id"].as_str().unwrap().into()
}

async fn start_prompt(app: &Router, session_id: &str) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{session_id}/prompt"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"session_id": session_id, "prompt": "write a file"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

async fn pending_permission(app: &Router, session_id: &str) -> Value {
    for _ in 0..50 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/sessions/{session_id}/permissions"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let permissions = json_response(response).await;
        if let Some(permission) = permissions.as_array().and_then(|items| items.first()) {
            return permission.clone();
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("permission request did not become pending");
}

async fn resolve(app: &Router, permission_id: &str, allowed: bool) -> StatusCode {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/permissions/{permission_id}/resolve"))
                .header("content-type", "application/json")
                .body(Body::from(json!({"allowed": allowed}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn discovers_pending_permission() {
    let (app, directory) = fixture().await;
    let session_id = session(&app, &directory).await;
    start_prompt(&app, &session_id).await;
    let permission = pending_permission(&app, &session_id).await;
    assert_eq!(permission["operation"], "write");
    assert_eq!(permission["target"], "permission-fixture.txt");
    assert_eq!(permission["status"], "pending");
    assert_eq!(
        resolve(&app, permission["id"].as_str().unwrap(), false).await,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn approves_pending_permission() {
    let (app, directory) = fixture().await;
    let session_id = session(&app, &directory).await;
    start_prompt(&app, &session_id).await;
    let permission = pending_permission(&app, &session_id).await;
    assert_eq!(
        resolve(&app, permission["id"].as_str().unwrap(), true).await,
        StatusCode::NO_CONTENT
    );
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(directory.path().join("permission-fixture.txt").is_file());
}

#[tokio::test]
async fn denies_pending_permission() {
    let (app, directory) = fixture().await;
    let session_id = session(&app, &directory).await;
    start_prompt(&app, &session_id).await;
    let permission = pending_permission(&app, &session_id).await;
    assert_eq!(
        resolve(&app, permission["id"].as_str().unwrap(), false).await,
        StatusCode::NO_CONTENT
    );
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!directory.path().join("permission-fixture.txt").exists());
}

#[tokio::test]
async fn cancellation_clears_pending_permission() {
    let (app, directory) = fixture().await;
    let session_id = session(&app, &directory).await;
    start_prompt(&app, &session_id).await;
    pending_permission(&app, &session_id).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{session_id}/interrupt"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    for _ in 0..50 {
        let permission_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/sessions/{session_id}/permissions"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if json_response(permission_response)
            .await
            .as_array()
            .unwrap()
            .is_empty()
        {
            assert!(!directory.path().join("permission-fixture.txt").exists());
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("cancelled permission remained pending");
}

#[tokio::test]
async fn rejects_invalid_permission_resolution() {
    let (app, _directory) = fixture().await;
    assert_eq!(
        resolve(&app, "not-a-permission", true).await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        resolve(&app, &PermissionRequestId::new().to_string(), true).await,
        StatusCode::NOT_FOUND
    );
}
