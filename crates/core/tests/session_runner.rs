use async_trait::async_trait;
use chrono::Utc;
use devfoundry_core::{CoreError, SessionRunner};
use devfoundry_llm::{LlmError, LlmEvent, LlmProvider, LlmRequest, LlmStream};
use devfoundry_schema::{
    AgentName, Event, MessageRole, ModelName, ModelRef, Project, ProjectId, ProviderName, Session,
    SessionStatus,
};
use devfoundry_storage::{EventRepository, ProjectRepository, SessionRepository, SqliteStore};
use devfoundry_tools::{Tool, ToolContext, ToolError, ToolOutput, ToolRegistry, ToolRequest};
use futures_util::stream;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
enum Script {
    Events(Vec<Result<LlmEvent, LlmError>>),
    WaitForCancellation,
}

struct ScriptedProvider {
    scripts: Mutex<VecDeque<Script>>,
}

#[async_trait]
impl LlmProvider for ScriptedProvider {
    async fn stream(
        &self,
        _request: LlmRequest,
        cancellation: CancellationToken,
    ) -> Result<LlmStream, LlmError> {
        let script = self.scripts.lock().unwrap().pop_front().unwrap();
        match script {
            Script::Events(events) => Ok(Box::pin(stream::iter(events))),
            Script::WaitForCancellation => Ok(Box::pin(async_stream::stream! {
                cancellation.cancelled().await;
                yield Err(LlmError::Cancelled);
            })),
        }
    }
}

struct CountingTool {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[async_trait]
impl Tool for CountingTool {
    fn name(&self) -> &'static str {
        "fixture"
    }
    fn description(&self) -> &'static str {
        "A deterministic fixture tool."
    }

    async fn execute(
        &self,
        _request: ToolRequest,
        _context: ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(ToolOutput {
            text: "fixture output".into(),
            truncated: false,
        })
    }
}

async fn fixture(
    scripts: Vec<Script>,
) -> (
    Arc<SqliteStore>,
    Session,
    SessionRunner,
    tempfile::TempDir,
    Arc<std::sync::atomic::AtomicUsize>,
) {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("runner.db");
    let store = Arc::new(
        SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
            .await
            .unwrap(),
    );
    let now = Utc::now();
    let project = Project {
        id: ProjectId::new(),
        root: directory.path().to_string_lossy().into_owned(),
        name: "runner fixture".into(),
        created_at: now,
        updated_at: now,
    };
    store.create_project(project.clone()).await.unwrap();
    let session = Session {
        id: devfoundry_schema::SessionId::new(),
        project_id: project.id,
        title: "runner fixture".into(),
        agent: AgentName("build".into()),
        model: ModelRef {
            provider: ProviderName("fixture".into()),
            model: ModelName("test".into()),
        },
        status: SessionStatus::Idle,
        created_at: now,
        updated_at: now,
    };
    store.create_session(session.clone()).await.unwrap();
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut tools = ToolRegistry::default();
    tools.register(Arc::new(CountingTool {
        calls: calls.clone(),
    }));
    let provider = Arc::new(ScriptedProvider {
        scripts: Mutex::new(scripts.into()),
    });
    let runner = SessionRunner::new(store.clone(), provider, Arc::new(tools));
    (store, session, runner, directory, calls)
}

fn finished(reason: &str) -> Result<LlmEvent, LlmError> {
    Ok(LlmEvent::Finished {
        reason: reason.into(),
    })
}

#[tokio::test]
async fn scripted_tool_call_persists_tool_events_and_assistant_completion() {
    let (store, session, runner, directory, calls) = fixture(vec![
        Script::Events(vec![
            Ok(LlmEvent::ToolCall {
                id: "fixture-call".into(),
                name: "fixture".into(),
                arguments: "{}".into(),
            }),
            finished("tool_calls"),
        ]),
        Script::Events(vec![
            Ok(LlmEvent::TextDelta {
                text: "completed".into(),
            }),
            finished("stop"),
        ]),
    ])
    .await;

    let assistant = runner
        .run(
            session.clone(),
            "use the fixture".into(),
            directory.path().into(),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(assistant.role, MessageRole::Assistant);
    assert!(assistant.parts.iter().any(|part| matches!(part, devfoundry_schema::MessagePart::ToolResult { output, .. } if output == "fixture output")));
    assert!(assistant.parts.iter().any(|part| matches!(part, devfoundry_schema::MessagePart::Text { text, .. } if text == "completed")));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);

    let events = store.list_events_after(session.id, None).await.unwrap();
    assert!(matches!(events[0].1, Event::ToolStarted { .. }));
    assert!(matches!(
        events[1].1,
        Event::ToolFinished { success: true, .. }
    ));
    assert!(events.iter().any(|(_, event)| matches!(event, Event::MessageCreated { message_id, .. } if *message_id == assistant.id)));
}

#[tokio::test]
async fn provider_failure_does_not_settle_an_assistant_message() {
    let (store, session, runner, directory, _calls) = fixture(vec![Script::Events(vec![
        Ok(LlmEvent::TextDelta {
            text: "partial".into(),
        }),
        Err(LlmError::Request("fixture outage".into())),
    ])])
    .await;

    let result = runner
        .run(
            session.clone(),
            "fail".into(),
            directory.path().into(),
            CancellationToken::new(),
        )
        .await;
    assert!(
        matches!(result, Err(CoreError::Provider(LlmError::Request(message))) if message == "fixture outage")
    );
    assert_eq!(
        store
            .list_messages(session.id)
            .await
            .unwrap()
            .iter()
            .filter(|message| message.role == MessageRole::Assistant)
            .count(),
        0
    );
    assert!(
        !store
            .list_events_after(session.id, None)
            .await
            .unwrap()
            .iter()
            .any(|(_, event)| matches!(event, Event::MessageCreated { .. }))
    );
}

#[tokio::test]
async fn cancellation_stops_stream_without_assistant_completion() {
    let (store, session, runner, directory, _calls) =
        fixture(vec![Script::WaitForCancellation]).await;
    let session_id = session.id;
    let cancellation = CancellationToken::new();
    let task = tokio::spawn({
        let runner = runner;
        let root = directory.path().to_path_buf();
        let cancellation = cancellation.clone();
        async move {
            runner
                .run(session, "cancel".into(), root, cancellation)
                .await
        }
    });
    tokio::task::yield_now().await;
    cancellation.cancel();
    let result = tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        result,
        Err(CoreError::Provider(LlmError::Cancelled))
    ));
    assert_eq!(
        store
            .list_messages(session_id)
            .await
            .unwrap()
            .iter()
            .filter(|message| message.role == MessageRole::Assistant)
            .count(),
        0
    );
}

#[tokio::test]
async fn recovery_is_idempotent_and_does_not_rerun_admitted_work() {
    let (store, session, _runner, _directory, calls) = fixture(Vec::new()).await;
    let prompt_id = store.admit_prompt(session.id, "recover me").await.unwrap();
    store.mark_prompt_running(prompt_id).await.unwrap();
    store.recover_interrupted_work().await.unwrap();
    store.recover_interrupted_work().await.unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM prompt_inbox WHERE id = ?")
        .bind(prompt_id.to_string())
        .fetch_one(store.pool())
        .await
        .unwrap();
    assert_eq!(status, "interrupted");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert!(
        store
            .list_events_after(session.id, None)
            .await
            .unwrap()
            .is_empty()
    );
}
