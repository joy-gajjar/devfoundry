use devfoundry_core::{EvidenceBundle, WorkerBrief, WorkerFailure, WorkerOutcome};
use devfoundry_schema::{ContextManifest, ProjectId, Task, TaskId};

#[test]
fn worker_brief_is_immutable_and_contains_bounded_execution_inputs() {
    let task = Task::new_for_project(ProjectId::new(), "build", "compile the project");
    let brief = WorkerBrief::new(
        task.id,
        task.revision.0,
        "compile the project",
        "base-commit",
        vec!["crates/core".into()],
        vec!["read".into(), "bash".into()],
        ContextManifest::new(Vec::new()),
    );

    assert_eq!(brief.task_id, task.id);
    assert_eq!(brief.task_revision, 0);
    assert_eq!(brief.base_commit, "base-commit");
    assert_eq!(brief.owned_scope, vec!["crates/core"]);
}

#[test]
fn worker_outcome_is_evidence_not_acceptance() {
    let evidence = EvidenceBundle::new(TaskId::new(), "base", "head", "digest");
    let outcome = WorkerOutcome::Evidence(evidence.clone());
    assert_eq!(outcome.review_status(), "review");
    assert_eq!(evidence.proposed_head, "head");
}

#[test]
fn worker_rejects_non_copilot_provider_and_classifies_unknown_attempts() {
    assert_eq!(
        WorkerBrief::validate_provider("anthropic"),
        Err(WorkerFailure::UnsupportedProvider)
    );
    assert!(matches!(
        WorkerOutcome::unknown(TaskId::new()),
        WorkerOutcome::UnknownSideEffect(_)
    ));
}
