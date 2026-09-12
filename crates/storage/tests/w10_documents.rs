use chrono::Utc;
use devfoundry_schema::{Project, ProjectId};
use devfoundry_storage::{DocumentRepository, ProjectRepository, SqliteStore};

#[tokio::test]
async fn document_index_builds_backlinks_and_rejects_symlink_escape() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("w10-docs.db");
    let store = SqliteStore::connect(&format!("sqlite://{}?mode=rwc", database.display()))
        .await
        .unwrap();
    let project = Project {
        id: ProjectId::new(),
        root: directory.path().to_string_lossy().into_owned(),
        name: "docs".into(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    store.create_project(project.clone()).await.unwrap();
    std::fs::create_dir(directory.path().join("docs")).unwrap();
    std::fs::write(directory.path().join("docs/a.md"), "See [[b]].").unwrap();
    std::fs::write(directory.path().join("docs/b.md"), "B").unwrap();
    store.rebuild_documents(project.id).await.unwrap();
    let backlinks = store.backlinks(project.id, "docs/b.md").await.unwrap();
    assert_eq!(backlinks, vec!["docs/a.md"]);

    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("secret.md"), "secret").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        outside.path().join("secret.md"),
        directory.path().join("docs/link.md"),
    )
    .unwrap();
    #[cfg(unix)]
    assert!(store.rebuild_documents(project.id).await.is_err());
}
