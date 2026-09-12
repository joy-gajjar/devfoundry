use chrono::Utc;
use devfoundry_schema::{
    AgentName, Event, Message, MessagePart, MessageRole, ModelName, ModelRef, PermissionRequest,
    PermissionRequestId, PermissionStatus, Project, ProjectId, ProviderName, Session,
    SessionExport, SessionStatus,
};
use devfoundry_storage::{EventRepository, ProjectRepository, SessionRepository, SqliteStore};

#[tokio::test]
async fn sqlite_persists_project_and_session() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("state.db");
    let store = SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
        .await
        .unwrap();
    let now = Utc::now();
    let project = Project {
        id: ProjectId::new(),
        root: directory.path().to_string_lossy().into_owned(),
        name: "fixture".into(),
        created_at: now,
        updated_at: now,
    };
    store.create_project(project.clone()).await.unwrap();
    let session = Session {
        id: devfoundry_schema::SessionId::new(),
        project_id: project.id,
        title: "test".into(),
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
    assert_eq!(
        store.get_project(project.id).await.unwrap().unwrap().root,
        project.root
    );
    assert_eq!(
        store.get_session(session.id).await.unwrap().unwrap().title,
        "test"
    );
}

#[tokio::test]
async fn sqlite_path_with_spaces_is_supported() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("Library Support").join("state.db");
    std::fs::create_dir_all(database.parent().unwrap()).unwrap();

    let store = SqliteStore::connect_path(&database).await.unwrap();
    assert!(database.exists());
    assert!(store.diagnostics().await.is_ok());
}

#[tokio::test]
async fn restart_applies_migrations_and_recovers_durable_work() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("restart.db");
    let url = format!("sqlite://{}?mode=rwc", database.display());
    let store = SqliteStore::connect(&url).await.unwrap();
    let now = Utc::now();
    let project = Project {
        id: ProjectId::new(),
        root: directory.path().to_string_lossy().into_owned(),
        name: "restart".into(),
        created_at: now,
        updated_at: now,
    };
    store.create_project(project.clone()).await.unwrap();
    let session = Session {
        id: devfoundry_schema::SessionId::new(),
        project_id: project.id,
        title: "restart".into(),
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
    let prompt_id = store.admit_prompt(session.id, "resume me").await.unwrap();
    store.mark_prompt_running(prompt_id).await.unwrap();
    let permission = PermissionRequest {
        id: PermissionRequestId::new(),
        session_id: session.id,
        operation: "write".into(),
        target: "fixture.txt".into(),
        status: PermissionStatus::Pending,
        created_at: now,
    };
    store.create_permission_request(&permission).await.unwrap();
    store.pool().close().await;

    let restarted = SqliteStore::connect(&url).await.unwrap();
    restarted.recover_interrupted_work().await.unwrap();
    let prompt_status =
        sqlx::query_scalar::<_, String>("SELECT status FROM prompt_inbox WHERE id = ?")
            .bind(prompt_id.to_string())
            .fetch_one(restarted.pool())
            .await
            .unwrap();
    assert_eq!(prompt_status, "interrupted");
    assert_eq!(
        restarted.get_prompt_status(prompt_id).await.unwrap(),
        Some((session.id, "interrupted".into()))
    );
    assert_eq!(
        restarted
            .get_permission_request(permission.id)
            .await
            .unwrap()
            .unwrap()
            .status,
        PermissionStatus::Cancelled
    );
}

#[tokio::test]
async fn sqlite_enforces_foreign_keys_and_reports_health() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("diagnostics.db");
    let store = SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
        .await
        .unwrap();
    let error = sqlx::query(
        "INSERT INTO sessions (id, project_id, title, agent, provider, model, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(devfoundry_schema::SessionId::new().to_string())
    .bind(ProjectId::new().to_string())
    .bind("orphan")
    .bind("build")
    .bind("fake")
    .bind("test")
    .bind("\"idle\"")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(store.pool())
    .await
    .unwrap_err();
    assert!(error.to_string().contains("FOREIGN KEY"));

    let diagnostics = store.diagnostics().await.unwrap();
    assert!(diagnostics.foreign_keys);
    assert_eq!(diagnostics.integrity_check, "ok");
    assert_eq!(diagnostics.row_counts["projects"], 0);
    assert_eq!(diagnostics.row_counts["sessions"], 0);
}

#[tokio::test]
async fn sqlite_backup_is_consistent_and_does_not_overwrite() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("state.db");
    let backup = directory.path().join("backups").join("state.db");
    std::fs::create_dir_all(backup.parent().unwrap()).unwrap();
    let store = SqliteStore::connect_path(&database).await.unwrap();

    store.backup_to(&backup).await.unwrap();
    assert!(backup.exists());
    let backed_up = SqliteStore::connect_path(&backup).await.unwrap();
    assert_eq!(backed_up.diagnostics().await.unwrap().integrity_check, "ok");
    assert!(store.backup_to(&backup).await.is_err());
}

#[tokio::test]
async fn committed_events_are_published_after_commit() {
    let directory = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_path(directory.path().join("events.db"))
        .await
        .unwrap();
    let now = Utc::now();
    let project = Project {
        id: ProjectId::new(),
        root: directory.path().to_string_lossy().into_owned(),
        name: "events".into(),
        created_at: now,
        updated_at: now,
    };
    store.create_project(project.clone()).await.unwrap();
    let session = Session {
        id: devfoundry_schema::SessionId::new(),
        project_id: project.id,
        title: "events".into(),
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
    let mut receiver = store.subscribe_events();
    let event = Event::SessionStatus {
        session_id: session.id,
        status: SessionStatus::Running,
    };
    let sequence = store.append_event(session.id, &event).await.unwrap();
    let published = receiver.recv().await.unwrap();
    assert_eq!(published.session_id, session.id);
    assert_eq!(published.sequence, sequence);
    assert_eq!(published.event, event);
}

#[tokio::test]
async fn session_export_import_round_trip_excludes_runtime_records() {
    let directory = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_path(directory.path().join("state.db"))
        .await
        .unwrap();
    let now = Utc::now();
    let project = Project {
        id: ProjectId::new(),
        root: directory.path().to_string_lossy().into_owned(),
        name: "portable".into(),
        created_at: now,
        updated_at: now,
    };
    store.create_project(project.clone()).await.unwrap();
    let session = Session {
        id: devfoundry_schema::SessionId::new(),
        project_id: project.id,
        title: "portable session".into(),
        agent: AgentName("build".into()),
        model: ModelRef {
            provider: ProviderName("fake".into()),
            model: ModelName("test".into()),
        },
        status: SessionStatus::Running,
        created_at: now,
        updated_at: now,
    };
    store.create_session(session.clone()).await.unwrap();
    let message = Message {
        id: devfoundry_schema::MessageId::new(),
        session_id: session.id,
        role: MessageRole::User,
        parts: vec![MessagePart::Text {
            id: devfoundry_schema::PartId::new(),
            text: "hello".into(),
        }],
        created_at: now,
    };
    store.append_message(&message).await.unwrap();

    let export = store.export_session(session.id).await.unwrap();
    assert_eq!(export.version, SessionExport::VERSION);
    assert_eq!(export.messages, vec![message.clone()]);
    assert_eq!(export.session.title, session.title);
    assert!(
        serde_json::to_string(&export)
            .unwrap()
            .contains("devfoundry.session")
    );

    let imported = store.import_session(project.id, &export).await.unwrap();
    assert_ne!(imported.id, session.id);

    let imported_project = Project {
        id: ProjectId::new(),
        root: directory
            .path()
            .join("other")
            .to_string_lossy()
            .into_owned(),
        name: "other".into(),
        created_at: now,
        updated_at: now,
    };
    std::fs::create_dir_all(&imported_project.root).unwrap();
    store
        .create_project(imported_project.clone())
        .await
        .unwrap();
    let imported = store
        .import_session(imported_project.id, &export)
        .await
        .unwrap();
    assert_eq!(imported.status, SessionStatus::Idle);
    let imported_messages = store.list_messages(imported.id).await.unwrap();
    assert_eq!(imported_messages.len(), 1);
    assert_eq!(imported_messages[0].parts, message.parts);
    assert_eq!(imported_messages[0].session_id, imported.id);
    assert_eq!(store.diagnostics().await.unwrap().row_counts["events"], 0);
}

#[tokio::test]
async fn session_import_rejects_unknown_version_and_cross_session_messages() {
    let directory = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_path(directory.path().join("state.db"))
        .await
        .unwrap();
    let now = Utc::now();
    let project = Project {
        id: ProjectId::new(),
        root: directory.path().to_string_lossy().into_owned(),
        name: "portable".into(),
        created_at: now,
        updated_at: now,
    };
    store.create_project(project.clone()).await.unwrap();
    let session = Session {
        id: devfoundry_schema::SessionId::new(),
        project_id: project.id,
        title: "portable".into(),
        agent: AgentName("build".into()),
        model: ModelRef {
            provider: ProviderName("fake".into()),
            model: ModelName("test".into()),
        },
        status: SessionStatus::Idle,
        created_at: now,
        updated_at: now,
    };
    let mut export = SessionExport::new(&session, Vec::new());
    export.version = 99;
    assert!(store.import_session(project.id, &export).await.is_err());
}
