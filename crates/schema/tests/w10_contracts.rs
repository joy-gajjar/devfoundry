use devfoundry_schema::{ContextManifest, ContextSource, DocumentMetadata, Task, TaskStatus};

#[test]
fn task_contract_preserves_revision_dependencies_and_provenance() {
    let task = Task::new("Index the brain", "Build the searchable document map");
    assert_eq!(task.status, TaskStatus::Draft);
    assert_eq!(task.revision.0, 0);
    assert!(task.dependencies.is_empty());

    let source = ContextSource::new("AGENTS.md", "sha256:instructions", "project_instruction");
    let manifest = ContextManifest::new(vec![source.clone()]);
    assert_eq!(manifest.sources, vec![source]);
}

#[test]
fn document_metadata_round_trips_wikilink_targets() {
    let document = DocumentMetadata::new("docs/brain.md", "sha256:document");
    let encoded = serde_json::to_string(&document).unwrap();
    let decoded: DocumentMetadata = serde_json::from_str(&encoded).unwrap();
    assert_eq!(document, decoded);
}
