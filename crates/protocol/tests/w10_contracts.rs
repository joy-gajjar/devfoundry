use devfoundry_protocol::{ImportRoadmapRequest, UpdateTaskRequest};
use devfoundry_schema::{Revision, TaskId};

#[test]
fn roadmap_import_and_task_update_require_revision_context() {
    let update = UpdateTaskRequest {
        expected_revision: Revision(4),
        title: Some("new title".into()),
        status: None,
        dependencies: Some(vec![TaskId::new()]),
    };
    let import = ImportRoadmapRequest {
        markdown: "- [ ] task <!-- id:01ARZ3NDEKTSV4RRFFQ69G5FAV rev:4 -->".into(),
        expected_export_hash: Some("sha256:roadmap".into()),
    };
    assert_eq!(
        serde_json::to_value(update).unwrap()["expected_revision"],
        4
    );
    assert!(
        serde_json::to_string(&import)
            .unwrap()
            .contains("expected_export_hash")
    );
}
