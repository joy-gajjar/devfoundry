use devfoundry_core::{
    Blocker, DependencyReceipt, Scheduler, SchedulerError, SchedulerLimits, SchedulerTask,
    TaskAttemptState, dependency_readiness,
};
use devfoundry_schema::{ProjectId, Task, TaskId, TaskStatus};

fn task(title: &str, dependencies: Vec<TaskId>) -> Task {
    let mut value = Task::new_for_project(ProjectId::new(), title, title);
    value.dependencies = dependencies;
    value.status = TaskStatus::Ready;
    value
}

#[test]
fn readiness_is_deterministic_and_requires_accepted_dependencies() {
    let first = task("first", vec![]);
    let second = task("second", vec![first.id]);
    let mut tasks = vec![second.clone(), first.clone()];

    let ready = dependency_readiness(&tasks, &[]).unwrap();
    assert_eq!(ready, vec![first.id]);

    tasks[1].status = TaskStatus::Accepted;
    let ready = dependency_readiness(&tasks, &[]).unwrap();
    assert_eq!(ready, vec![second.id]);
}

#[test]
fn readiness_rejects_cycles_and_reports_missing_dependencies() {
    let first = task("first", vec![]);
    let mut second = task("second", vec![first.id]);
    second.dependencies = vec![first.id];
    let mut tasks = vec![first.clone(), second.clone()];
    tasks[0].dependencies = vec![second.id];

    assert!(matches!(
        dependency_readiness(&tasks, &[]),
        Err(SchedulerError::Cycle)
    ));

    let missing = task("missing", vec![TaskId::new()]);
    assert_eq!(
        dependency_readiness(&[missing], &[]),
        Err(SchedulerError::Blocked(vec![Blocker::MissingDependency]))
    );
}

#[test]
fn scheduler_bounds_concurrency_budget_and_repeated_failures() {
    let first = SchedulerTask::new(task("first", vec![]));
    let second = SchedulerTask::new(task("second", vec![]));
    let limits = SchedulerLimits {
        max_concurrency: 1,
        max_attempts: 2,
        max_usage: 10,
    };
    let mut scheduler = Scheduler::new(limits);

    scheduler.record_failure(first.task.id, TaskAttemptState::Failed, 1);
    scheduler.record_failure(first.task.id, TaskAttemptState::Failed, 1);

    let assignments = scheduler.assign(&[first.clone(), second.clone()]).unwrap();
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].task_id, second.task.id);

    scheduler.record_usage(9);
    assert!(scheduler.assign(&[first, second]).is_err());
}

#[test]
fn reviewed_dependency_requires_integration_receipt_and_ancestor_base() {
    let mut dependency = task("dependency", vec![]);
    dependency.status = TaskStatus::Accepted;
    let worker = task("worker", vec![dependency.id]);
    let receipt = DependencyReceipt::code(dependency.id, "review-head", "base");

    assert_eq!(
        dependency_readiness(
            &[dependency.clone(), worker.clone()],
            std::slice::from_ref(&receipt),
        ),
        Ok(Vec::new())
    );

    let integrated = receipt.with_ancestor(true);
    assert_eq!(
        dependency_readiness(&[dependency, worker.clone()], &[integrated]),
        Ok(vec![worker.id])
    );
}
