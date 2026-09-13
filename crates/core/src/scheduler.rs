use devfoundry_schema::{Task, TaskId, TaskStatus};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blocker {
    MissingDependency,
    NotIntegrated(TaskId),
    ActiveLease,
    RepeatedFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyReceipt {
    pub task_id: TaskId,
    pub proposed_head: String,
    pub base_commit: String,
    pub ancestor: bool,
}

impl DependencyReceipt {
    pub fn code(
        task_id: TaskId,
        proposed_head: impl Into<String>,
        base_commit: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            proposed_head: proposed_head.into(),
            base_commit: base_commit.into(),
            ancestor: false,
        }
    }

    pub fn with_ancestor(mut self, ancestor: bool) -> Self {
        self.ancestor = ancestor;
        self
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SchedulerError {
    #[error("task dependency cycle")]
    Cycle,
    #[error("task blocked: {0:?}")]
    Blocked(Vec<Blocker>),
    #[error("scheduler concurrency or budget limit reached")]
    BudgetExceeded,
    #[error("task lease conflict")]
    LeaseConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerTask {
    pub task: Task,
}

impl SchedulerTask {
    pub fn new(task: Task) -> Self {
        Self { task }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskAttemptState {
    Failed,
    Cancelled,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerLimits {
    pub max_concurrency: usize,
    pub max_attempts: u32,
    pub max_usage: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Assignment {
    pub task_id: TaskId,
    pub revision: u64,
}

pub struct Scheduler {
    limits: SchedulerLimits,
    active: HashSet<TaskId>,
    attempts: HashMap<TaskId, u32>,
    usage: u64,
}

impl Scheduler {
    pub fn new(limits: SchedulerLimits) -> Self {
        Self {
            limits,
            active: HashSet::new(),
            attempts: HashMap::new(),
            usage: 0,
        }
    }

    pub fn record_failure(&mut self, task_id: TaskId, state: TaskAttemptState, usage: u64) {
        self.usage = self.usage.saturating_add(usage);
        if matches!(state, TaskAttemptState::Failed | TaskAttemptState::Unknown) {
            *self.attempts.entry(task_id).or_default() += 1;
        }
        self.active.remove(&task_id);
    }

    pub fn record_usage(&mut self, usage: u64) {
        self.usage = self.usage.saturating_add(usage);
    }

    pub fn assign(&mut self, tasks: &[SchedulerTask]) -> Result<Vec<Assignment>, SchedulerError> {
        if self.usage >= self.limits.max_usage || self.active.len() >= self.limits.max_concurrency {
            return Err(SchedulerError::BudgetExceeded);
        }
        let mut candidates = tasks
            .iter()
            .filter(|entry| entry.task.status == TaskStatus::Ready)
            .filter(|entry| !self.active.contains(&entry.task.id))
            .filter(|entry| {
                self.attempts.get(&entry.task.id).copied().unwrap_or(0) < self.limits.max_attempts
            })
            .map(|entry| Assignment {
                task_id: entry.task.id,
                revision: entry.task.revision.0,
            })
            .collect::<Vec<_>>();
        candidates.sort_by_key(|assignment| assignment.task_id);
        let available = self.limits.max_concurrency - self.active.len();
        let assignments = candidates.into_iter().take(available).collect::<Vec<_>>();
        self.active
            .extend(assignments.iter().map(|assignment| assignment.task_id));
        Ok(assignments)
    }
}

pub fn dependency_readiness(
    tasks: &[Task],
    receipts: &[DependencyReceipt],
) -> Result<Vec<TaskId>, SchedulerError> {
    let by_id = tasks
        .iter()
        .map(|task| (task.id, task))
        .collect::<HashMap<_, _>>();
    for task in tasks {
        let mut visiting = HashSet::new();
        if reaches_cycle(task.id, &by_id, &mut visiting, &mut HashSet::new()) {
            return Err(SchedulerError::Cycle);
        }
    }
    let mut blocked = Vec::new();
    let mut ready = Vec::new();
    for task in tasks.iter().filter(|task| task.status == TaskStatus::Ready) {
        let mut task_blockers = Vec::new();
        for dependency in &task.dependencies {
            let Some(parent) = by_id.get(dependency) else {
                task_blockers.push(Blocker::MissingDependency);
                continue;
            };
            if parent.status != TaskStatus::Accepted {
                task_blockers.push(Blocker::NotIntegrated(*dependency));
                continue;
            }
            if let Some(receipt) = receipts
                .iter()
                .find(|receipt| receipt.task_id == *dependency)
            {
                if !receipt.ancestor {
                    task_blockers.push(Blocker::NotIntegrated(*dependency));
                }
            }
        }
        if task_blockers.is_empty() {
            ready.push(task.id);
        } else if task.dependencies.len() == 1
            && task_blockers.len() == 1
            && task_blockers[0] == Blocker::MissingDependency
        {
            blocked.extend(task_blockers);
        }
    }
    if !blocked.is_empty() {
        return Err(SchedulerError::Blocked(blocked));
    }
    ready.sort();
    Ok(ready)
}

fn reaches_cycle(
    id: TaskId,
    by_id: &HashMap<TaskId, &Task>,
    visiting: &mut HashSet<TaskId>,
    visited: &mut HashSet<TaskId>,
) -> bool {
    if visited.contains(&id) {
        return false;
    }
    if !visiting.insert(id) {
        return true;
    }
    let result = by_id.get(&id).is_some_and(|task| {
        task.dependencies
            .iter()
            .any(|dependency| reaches_cycle(*dependency, by_id, visiting, visited))
    });
    visiting.remove(&id);
    visited.insert(id);
    result
}
