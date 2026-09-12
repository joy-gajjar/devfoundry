use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use devfoundry_core::SessionRunner;
use devfoundry_llm::{LlmError, LlmEvent, LlmProvider, LlmRequest, LlmStream};
use devfoundry_server::{ApiSecurity, ExecutionRegistry, ServerState, router_with_security};
use devfoundry_storage::SqliteStore;
use devfoundry_tools::ToolRegistry;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;

struct BoundaryProvider;

#[async_trait]
impl LlmProvider for BoundaryProvider {
    async fn stream(
        &self,
        request: LlmRequest,
        cancellation: CancellationToken,
    ) -> Result<LlmStream, LlmError> {
        let prompt = request
            .messages
            .last()
            .map(|message| message.content.as_str())
            .unwrap_or_default()
            .to_owned();
        if prompt == "fail" {
            return Err(LlmError::Request("fixture failure".into()));
        }
        if prompt == "interrupt" || prompt == "shutdown" {
            return Ok(Box::pin(async_stream::stream! {
                cancellation.cancelled().await;
                yield Err(LlmError::Cancelled);
            }));
        }
        Ok(Box::pin(futures_util::stream::iter([
            Ok(LlmEvent::TextDelta {
                text: "fixture response".into(),
            }),
            Ok(LlmEvent::Finished {
                reason: "stop".into(),
            }),
        ])))
    }
}

async fn fixture() -> (Router, tempfile::TempDir, Arc<ExecutionRegistry>) {
    fixture_with_security(ApiSecurity::default()).await
}

async fn fixture_with_security(
    security: ApiSecurity,
) -> (Router, tempfile::TempDir, Arc<ExecutionRegistry>) {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("api.db");
    let store = Arc::new(
        SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
            .await
            .unwrap(),
    );
    let provider: Arc<dyn devfoundry_llm::LlmProvider> = Arc::new(BoundaryProvider);
    let runner = Arc::new(SessionRunner::new(
        store.clone(),
        provider,
        Arc::new(ToolRegistry::default()),
    ));
    let executions = Arc::new(ExecutionRegistry::default());
    let app = router_with_security(
        ServerState {
            store,
            runner,
            executions: executions.clone(),
        },
        security,
    );
    (app, directory, executions)
}

#[tokio::test]
async fn loopback_compatibility_keeps_health_unauthenticated() {
    let (app, _directory, _executions) = fixture().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn protected_router_requires_bearer_token_without_leaking_it() {
    let (app, _directory, _executions) = fixture_with_security(ApiSecurity {
        bearer_token: Some("test-secret-value".into()),
        allowed_origins: vec!["https://client.example".parse().unwrap()],
    })
    .await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_response(response).await;
    assert!(!body.to_string().contains("test-secret-value"));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("authorization", "Bearer test-secret-value")
                .header("origin", "https://client.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["access-control-allow-origin"],
        "https://client.example"
    );
}

#[tokio::test]
async fn protected_router_allows_cors_preflight_without_token() {
    let (app, _directory, _executions) = fixture_with_security(ApiSecurity {
        bearer_token: Some("test-secret-value".into()),
        allowed_origins: vec!["https://client.example".parse().unwrap()],
    })
    .await;
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header("origin", "https://client.example")
                .header("access-control-request-method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["access-control-allow-origin"],
        "https://client.example"
    );
}

#[tokio::test]
async fn protected_router_rejects_unconfigured_origin() {
    let (app, _directory, _executions) = fixture_with_security(ApiSecurity {
        bearer_token: Some("test-secret-value".into()),
        allowed_origins: vec!["https://client.example".parse().unwrap()],
    })
    .await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("origin", "https://other.example")
                .header("authorization", "Bearer test-secret-value")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

async fn json_response(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn next_sse_frame(body: &mut axum::body::Body) -> String {
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(2), body.frame())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let data = String::from_utf8(frame.into_data().unwrap().to_vec()).unwrap();
        if !data.starts_with(":") {
            return data;
        }
    }
}

async fn wait_for_prompt(app: &Router, prompt_id: &str, expected: &str) -> Value {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/v1/prompts/{prompt_id}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let body = json_response(response).await;
            if body["status"] == expected {
                return body;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap()
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
    json_response(response).await["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn creates_session_accepts_prompt_and_persists_event() {
    let (app, directory, _executions) = fixture().await;
    let project_response = app
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
    assert_eq!(project_response.status(), StatusCode::CREATED);
    let project: Value = json_response(project_response).await;
    let project_id = project["id"].as_str().unwrap();
    let session_response = app
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
    assert_eq!(session_response.status(), StatusCode::CREATED);
    let session: Value = json_response(session_response).await;
    let session_id = session["id"].as_str().unwrap();
    let prompt_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{session_id}/prompt"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"session_id": session_id, "prompt": "hello"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(prompt_response.status(), StatusCode::ACCEPTED);
    let accepted = json_response(prompt_response).await;
    let prompt_id = accepted["prompt_id"].as_str().unwrap();
    let status = wait_for_prompt(&app, prompt_id, "completed").await;
    assert_eq!(status["session_id"], session_id);
    let events_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sessions/{session_id}/events"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(events_response.status(), StatusCode::OK);
    assert_eq!(
        events_response.headers()["content-type"],
        "text/event-stream"
    );
    let body = events_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    assert!(String::from_utf8_lossy(&body).contains("message_created"));
}

#[tokio::test]
async fn prompt_status_not_found_and_invalid_id_are_structured() {
    let (app, _directory, _executions) = fixture().await;
    for (prompt_id, status, code) in [
        ("not-a-prompt", StatusCode::BAD_REQUEST, "invalid_prompt_id"),
        (
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            StatusCode::NOT_FOUND,
            "prompt_not_found",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/prompts/{prompt_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(json_response(response).await["code"], code);
    }
}

#[tokio::test]
async fn prompt_failure_is_durably_signaled() {
    let (app, directory, _executions) = fixture().await;
    let session_id = session(&app, &directory).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{session_id}/prompt"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"session_id": session_id, "prompt": "fail"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let prompt_id = json_response(response).await["prompt_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let status = wait_for_prompt(&app, &prompt_id, "failed").await;
    assert_eq!(status["status"], "failed");
}

#[tokio::test]
async fn interrupt_and_registry_shutdown_signal_interrupted_prompt() {
    let (app, directory, _executions) = fixture().await;
    let session_id = session(&app, &directory).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{session_id}/prompt"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"session_id": session_id, "prompt": "interrupt"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let prompt_id = json_response(response).await["prompt_id"]
        .as_str()
        .unwrap()
        .to_owned();
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{session_id}/interrupt"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        wait_for_prompt(&app, &prompt_id, "interrupted").await["status"],
        "interrupted"
    );

    let (app, directory, executions) = fixture().await;
    let session_id = session(&app, &directory).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{session_id}/prompt"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"session_id": session_id, "prompt": "shutdown"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let prompt_id = json_response(response).await["prompt_id"]
        .as_str()
        .unwrap()
        .to_owned();
    executions.shutdown();
    assert_eq!(
        wait_for_prompt(&app, &prompt_id, "interrupted").await["status"],
        "interrupted"
    );
}

#[tokio::test]
async fn rejects_invalid_prompt_payload() {
    let (app, _directory, _executions) = fixture().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sessions/not-a-session/prompt")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"session_id": "not-a-session", "prompt": ""}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn invalid_route_returns_structured_error_and_request_id() {
    let (app, _directory, _executions) = fixture().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/sessions/not-a-session")
                .header("x-request-id", "contract-request-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()["x-request-id"], "contract-request-1");
    let body = json_response(response).await;
    assert_eq!(body["code"], "invalid_session_id");
    assert_eq!(body["request_id"], "contract-request-1");
    assert_eq!(body["retryable"], false);
}

#[tokio::test]
async fn missing_session_interrupt_and_events_are_not_found() {
    let (app, _directory, _executions) = fixture().await;
    for path in [
        "/api/v1/sessions/01ARZ3NDEKTSV4RRFFQ69G5FAV/interrupt",
        "/api/v1/sessions/01ARZ3NDEKTSV4RRFFQ69G5FAV/events",
    ] {
        let method = if path.ends_with("/interrupt") {
            "POST"
        } else {
            "GET"
        };
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(json_response(response).await["code"], "session_not_found");
    }
}

#[tokio::test]
async fn health_and_discovery_routes_return_contract_payloads() {
    let (app, directory, _executions) = fixture().await;
    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
    assert_eq!(health.headers()["content-type"], "application/json");
    assert!(health.headers().get("x-request-id").is_some());
    assert_eq!(json_response(health).await["status"], "ok");

    let projects = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(projects.status(), StatusCode::OK);
    assert_eq!(json_response(projects).await, json!([]));

    let session_id = session(&app, &directory).await;
    let project_id = json_response(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
    )
    .await[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let sessions = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/projects/{project_id}/sessions"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sessions.status(), StatusCode::OK);
    assert_eq!(json_response(sessions).await[0]["id"], session_id);

    let fetched = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sessions/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fetched.status(), StatusCode::OK);
    assert_eq!(json_response(fetched).await["id"], session_id);
}

#[tokio::test]
async fn route_validation_and_missing_resources_have_stable_errors() {
    let (app, _directory, _executions) = fixture().await;
    let cases = [
        (
            "/api/v1/projects/not-a-project/sessions",
            "GET",
            "invalid_project_id",
        ),
        (
            "/api/v1/sessions/not-a-session",
            "GET",
            "invalid_session_id",
        ),
        (
            "/api/v1/sessions/not-a-session/interrupt",
            "POST",
            "invalid_session_id",
        ),
        (
            "/api/v1/sessions/not-a-session/permissions",
            "GET",
            "invalid_session_id",
        ),
        (
            "/api/v1/permissions/not-a-permission/resolve",
            "POST",
            "invalid_permission_id",
        ),
    ];
    for (path, method, code) in cases {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("x-request-id", "validation-contract")
                    .header("content-type", "application/json")
                    .body(if method == "POST" {
                        Body::from(r#"{"allowed":true}"#)
                    } else {
                        Body::empty()
                    })
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(response.headers()["x-request-id"], "validation-contract");
        let body = json_response(response).await;
        assert_eq!(body["code"], code);
        assert_eq!(body["request_id"], "validation-contract");
        assert_eq!(body["retryable"], false);
    }
}

#[tokio::test]
async fn sse_replays_framed_events_and_delivers_post_commit_live_events() {
    let (app, directory, _executions) = fixture().await;
    let session_id = session(&app, &directory).await;
    let prompt = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{session_id}/prompt"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"session_id": session_id, "prompt": "hello"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let prompt_id = json_response(prompt).await["prompt_id"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_for_prompt(&app, &prompt_id, "completed").await;

    let replay = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sessions/{session_id}/events?after=0"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(replay.status(), StatusCode::OK);
    assert_eq!(replay.headers()["content-type"], "text/event-stream");
    let mut replay_body = replay.into_body();
    let first = next_sse_frame(&mut replay_body).await;
    assert!(first.starts_with("id: 1\n"));
    assert!(first.contains("event: message\n"));
    assert!(first.contains("data: {\"type\":"));

    let (live_app, live_directory, _executions) = fixture().await;
    let live_session_id = session(&live_app, &live_directory).await;
    let live = live_app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sessions/{live_session_id}/events?after=0"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let mut live_body = live.into_body();
    let live_prompt = live_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{live_session_id}/prompt"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"session_id": live_session_id, "prompt": "hello again"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(live_prompt.status(), StatusCode::ACCEPTED);
    let frame = next_sse_frame(&mut live_body).await;
    assert!(frame.contains("event: message\n"));
    assert!(frame.contains("data: {\"type\":\"session_status\""));
}
