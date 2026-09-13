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

#[tokio::test]
async fn resource_file_metadata_replacement_is_transactional() {
    let directory = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_path(directory.path().join("resource.db"))
        .await
        .unwrap();
    let now = Utc::now();
    let project_id = ProjectId::new();
    store
        .create_project(Project {
            id: project_id,
            root: directory.path().to_string_lossy().into(),
            name: "resource".into(),
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap();
    let manifest = devfoundry_schema::ResourceManifest {
        id: "resource-a".into(),
        version: "1.0.0".into(),
        source: "fixture".into(),
        manifest_hash: "manifest".into(),
        files: vec![],
        required_capabilities: vec![],
    };
    let files = vec![devfoundry_schema::InstalledResourceFile {
        resource_id: manifest.id.clone(),
        project_id,
        target: "README.md".into(),
        expected_hash: "expected".into(),
        installed_hash: "installed".into(),
        size: 10,
    }];
    store
        .replace_resource_files(&manifest.id, project_id, &manifest, &files)
        .await
        .unwrap();
    assert_eq!(
        store
            .list_resource_files(&manifest.id, project_id)
            .await
            .unwrap()[0]
            .installed_hash,
        "installed"
    );
}
