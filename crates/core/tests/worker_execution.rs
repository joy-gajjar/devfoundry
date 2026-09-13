use devfoundry_core::{WorkerBrief, WorkerHost};
use devfoundry_schema::{ContextManifest, TaskId};

#[test]
fn worker_host_rejects_non_copilot_before_side_effects() {
    assert!(WorkerHost::validate_provider("anthropic").is_err());
    assert!(WorkerHost::validate_provider("github-copilot").is_ok());
}

#[test]
fn worker_brief_remains_immutable_and_bounded() {
    let brief = WorkerBrief::new(
        TaskId::new(),
        4,
        "objective",
        "base",
        vec!["crates/core".into()],
        vec!["read".into()],
        ContextManifest::new(Vec::new()),
    );
    assert_eq!(brief.task_revision, 4);
    assert_eq!(brief.owned_scope, vec!["crates/core"]);
}
