use chrono::Utc;
use devfoundry_schema::{
    AgentName, AttemptId, AttemptStatus, EventEnvelope, IdempotencyResult, Message, MessagePart,
    MessageRole, ModelName, ModelRef, PartId, Project, ProjectId, ProviderName, RunStatus, Session,
    SessionId, SessionStatus,
};
use devfoundry_storage::{AdmissionInput, ProjectRepository, SessionRepository, SqliteStore};

async fn store() -> SqliteStore {
    let directory = tempfile::tempdir().unwrap();
    // Keep the directory alive for the duration of this test store.
    let path = directory.path().join("state.db");
    std::mem::forget(directory);
    SqliteStore::connect_path(path).await.unwrap()
}

async fn session(store: &SqliteStore) -> SessionId {
    let now = Utc::now();
    let project = Project {
        id: ProjectId::new(),
        root: format!("/tmp/devfoundry-{}", ProjectId::new()),
        name: "fixture".into(),
        created_at: now,
        updated_at: now,
    };
    store.create_project(project.clone()).await.unwrap();
    let session = Session {
        id: SessionId::new(),
        project_id: project.id,
        title: "fixture".into(),
        agent: AgentName("build".into()),
        model: ModelRef {
            provider: ProviderName("fake".into()),
            model: ModelName("test".into()),
        },
        status: SessionStatus::Idle,
        created_at: now,
        updated_at: now,
    };
    store.create_session(session.clone()).await.unwrap();
    session.id
}

#[tokio::test]
async fn admission_failure_leaves_no_partial_run() {
    let store = store().await;
    let input = AdmissionInput {
        session_id: SessionId::new(),
        idempotency_key: "admission-failure".into(),
        prompt: "never admitted".into(),
        expected_revision: 0,
    };

    assert!(store.admit_run(input).await.is_err());
    assert_eq!(store.count_runs().await.unwrap(), 0);
    assert_eq!(store.count_run_events().await.unwrap(), 0);
}

#[tokio::test]
async fn same_key_same_payload_returns_receipt() {
    let store = store().await;
    let session_id = session(&store).await;
    let input = AdmissionInput {
        session_id,
        idempotency_key: "same-key".into(),
        prompt: "same payload".into(),
        expected_revision: 0,
    };

    let first = store.admit_run(input.clone()).await.unwrap();
    let second = store.admit_run(input).await.unwrap();
    assert_eq!(first.run_id, second.run_id);
    assert_eq!(second.idempotency, IdempotencyResult::Existing);
    assert_eq!(store.count_runs().await.unwrap(), 1);
}

#[tokio::test]
async fn same_key_different_payload_conflicts() {
    let store = store().await;
    let session_id = session(&store).await;
    store
        .admit_run(AdmissionInput {
            session_id,
            idempotency_key: "conflict".into(),
            prompt: "first".into(),
            expected_revision: 0,
        })
        .await
        .unwrap();

    let receipt = store
        .admit_run(AdmissionInput {
            session_id,
            idempotency_key: "conflict".into(),
            prompt: "different".into(),
            expected_revision: 0,
        })
        .await
        .unwrap();
    assert_eq!(receipt.idempotency, IdempotencyResult::Conflict);
}

#[tokio::test]
async fn crash_after_tool_start_marks_unknown() {
    let store = store().await;
    let session_id = session(&store).await;
    let run = store
        .admit_run(AdmissionInput {
            session_id,
            idempotency_key: "tool-crash".into(),
            prompt: "run tool".into(),
            expected_revision: 0,
        })
        .await
        .unwrap();
    let attempt_id = AttemptId::new();
    store.start_attempt(run.run_id, attempt_id).await.unwrap();
    store
        .start_tool(run.run_id, attempt_id, "tool-1", "write")
        .await
        .unwrap();

    store.recover_interrupted_work().await.unwrap();
    let status = store.get_run_status(run.run_id).await.unwrap().unwrap();
    assert_eq!(status.status, RunStatus::OutcomeUnknown);
    assert_eq!(status.attempt_status, Some(AttemptStatus::OutcomeUnknown));
}

#[tokio::test]
async fn second_host_cannot_recover_live_owner() {
    let store = store().await;
    let session_id = session(&store).await;
    let run = store
        .admit_run(AdmissionInput {
            session_id,
            idempotency_key: "lease".into(),
            prompt: "leased".into(),
            expected_revision: 0,
        })
        .await
        .unwrap();

    assert!(
        store
            .claim_execution(run.run_id, "host-a", 60)
            .await
            .unwrap()
    );
    assert!(
        !store
            .claim_execution(run.run_id, "host-b", 60)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn bounded_events_query_uses_keyset_and_byte_limit() {
    let store = store().await;
    let session_id = session(&store).await;
    let run = store
        .admit_run(AdmissionInput {
            session_id,
            idempotency_key: "bounded-events".into(),
            prompt: "events".into(),
            expected_revision: 0,
        })
        .await
        .unwrap();

    for sequence in 2..=4 {
        store
            .append_run_event(
                run.run_id,
                EventEnvelope {
                    version: 1,
                    sequence,
                    run_id: run.run_id,
                    aggregate_revision: devfoundry_schema::Revision(sequence),
                    kind: "test.event".into(),
                    payload: serde_json::json!({"sequence": sequence}),
                },
            )
            .await
            .unwrap();
    }

    let page = store
        .list_run_events_after(run.run_id, 1, 2, 10_000)
        .await
        .unwrap();
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].sequence, 2);
    assert_eq!(page[1].sequence, 3);
}

#[tokio::test]
async fn tool_settlement_commits_output_status_and_event_atomically() {
    let store = store().await;
    let session_id = session(&store).await;
    let run = store
        .admit_run(AdmissionInput {
            session_id,
            idempotency_key: "tool-settle".into(),
            prompt: "settle".into(),
            expected_revision: 0,
        })
        .await
        .unwrap();
    let attempt_id = AttemptId::new();
    store.start_attempt(run.run_id, attempt_id).await.unwrap();
    store
        .start_tool(run.run_id, attempt_id, "tool-settle", "write")
        .await
        .unwrap();

    store
        .settle_tool(run.run_id, attempt_id, "tool-settle", "done", false)
        .await
        .unwrap();
    assert_eq!(
        store.tool_output("tool-settle").await.unwrap(),
        Some(("done".into(), false))
    );
    assert_eq!(
        store.tool_status("tool-settle").await.unwrap(),
        Some("succeeded".into())
    );
    assert!(
        store
            .list_run_events_after(run.run_id, 0, 10, 10_000)
            .await
            .unwrap()
            .len()
            >= 2
    );
}

#[tokio::test]
async fn bounded_messages_query_uses_keyset_and_byte_limit() {
    let store = store().await;
    let session_id = session(&store).await;
    for text in ["one", "two", "three"] {
        store
            .append_message(&Message {
                id: devfoundry_schema::MessageId::new(),
                session_id,
                role: MessageRole::User,
                parts: vec![MessagePart::Text {
                    id: PartId::new(),
                    text: text.into(),
                }],
                created_at: Utc::now(),
            })
            .await
            .unwrap();
    }
    let page = store
        .list_messages_after(session_id, None, 2, 10_000)
        .await
        .unwrap();
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].1.parts.len(), 1);
}
