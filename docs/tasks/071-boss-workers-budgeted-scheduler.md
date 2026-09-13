# Task: W09 Boss, Workers And Budgeted Scheduler

## Goal

Provide a deterministic core scheduler and immutable worker/evidence boundary for bounded boss-worker task execution, without activating parallel provider work, subagents, queued steering, compaction, or automatic restart replay.

## Scope

- Included: pure DAG readiness, stable task ordering, concurrency/usage/attempt limits, repeated-failure suppression, dependency integration receipts, immutable worker briefs, Copilot-only validation, evidence-not-acceptance outcomes, and unknown-side-effect classification.
- Excluded: browser/TUI projections, new protocol routes, storage migrations, provider wire changes, worktree implementation changes, automatic retry after restart, Git integration, parallel tools, and subagents.

## Design

- `crates/core/src/scheduler.rs` is a deterministic policy layer. It rejects dependency cycles, requires accepted dependencies, requires an ancestor integration receipt for code dependencies, sorts ready IDs, and bounds assignments by active concurrency, usage, and attempts.
- `crates/core/src/workers.rs` owns immutable `WorkerBrief`, bounded `EvidenceBundle`, Copilot-only provider validation, and explicit unknown-side-effect outcomes. A worker outcome reports `review`, never acceptance.
- Existing W10 task records and W08 worktree types remain the integration boundary; this slice does not add persistence or mutate their modules.
- Storage failures, durable leases, task status transitions, and restart recovery still require a follow-up adapter backed by the existing public storage APIs before scheduler execution is exposed to production routes.

## Lifecycle And State Decisions

- Readiness is computed from durable task status and dependency receipts; a reviewed but unintegrated dependency remains unavailable to its worker.
- Assignment captures the task revision. A future durable adapter must reject stale revisions and duplicate leases before changing task state.
- Failure and unknown-side-effect attempts consume the attempt budget. Repeated failures stop assignment rather than silently retrying.
- Cancellation and restart replay are intentionally not simulated as successful completion; unknown effects require explicit operator recovery.
- Evidence transitions a worker to review state. Acceptance and human Git integration remain separate decisions.

## Implementation

- Added `crates/core/src/scheduler.rs`.
- Added `crates/core/src/workers.rs`.
- Re-exported the W09 core types from `crates/core/src/lib.rs`.
- Added deterministic tests in `crates/core/tests/scheduler.rs` and `crates/core/tests/workers.rs`.

## Verification

- `$HOME/.cargo/bin/cargo test -p devfoundry-core --test scheduler --test workers`: initially RED because all W09 exports were absent; after implementation, 7 passed and 0 failed.
- `$HOME/.cargo/bin/cargo test -p devfoundry-core`: passed before the final lint-only cleanup, with 15 tests and 0 failures.
- `$HOME/.cargo/bin/cargo fmt --all -- --check`: initially reported rustfmt differences; formatting was applied before final verification.
- `$HOME/.cargo/bin/cargo clippy -p devfoundry-core --all-targets -- -D warnings`: first post-implementation run found one test lint; fixed before the final run.
- Full workspace gates remain to be run after documentation changes.

## Risks And Follow-Up

- The current `Scheduler` is process-local and does not persist leases, attempts, usage, or evidence. It must not be wired as restart-safe production orchestration until a minimal storage adapter can atomically persist those facts.
- `DependencyReceipt::ancestor` is supplied by an adapter; the W09 core package does not itself execute Git ancestry checks.
- Worker execution is represented by contracts/tests only in this slice; connecting it to `SessionRunner`, `WorktreeManager`, durable task status, and permission/cancellation waits is the next implementation step.
- Follow-up: add a storage-backed scheduler adapter and fault-injection tests for duplicate lease, stale revision, crash during child attempt, restart during review, and persist-before-publish semantics.
- Follow-up: add projection/API contracts for worker cards only after the durable core adapter is accepted.
