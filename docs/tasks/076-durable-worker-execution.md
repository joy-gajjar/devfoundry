# Task: Durable Worker Execution (W19)

## Goal

Persist W09 scheduler leases, worker attempts, bounded evidence, failure/recovery facts, and integration receipts so restart and duplicate execution cannot silently lose state or replay uncertain side effects.

## Scope

- Added forward-only scheduler execution migration and indexed durable tables.
- Added a SQLx-free `SchedulerRepository` boundary with atomic lease, attempt, evidence, settlement, and recovery operations.
- Added a core scheduler adapter and supervised worker-host boundary.
- Added worker status, assignment, cancellation-status, and evidence read projections.
- Added clean-database, duplicate-claim, stale-revision, idempotency, evidence, and recovery tests.
- Explicitly excluded browser files, W20+, database deletion/reset, automatic Git integration, and unsafe tool replay.

## Design

- `migrations/0004_scheduler_execution.sql` is forward-only; migrations 0001-0003 were not edited.
- Upgrade impact is additive tables/indexes only; existing rows remain readable and SQLite backup uses the existing store backup path before any future destructive migration. This migration contains no destructive operation and requires no database reset.
- `scheduler_leases` has a partial unique index allowing at most one `claimed` lease per task revision.
- Attempt IDs are stable identifiers. Repeating an attempt ID for the same lease/task revision returns the existing attempt; conflicting identity is rejected.
- Attempt start is committed before any future side effect boundary. Settlement writes evidence and moves the task to `review` in one transaction.
- Recovery marks started attempts `outcome_unknown` and claimed leases `recovered`; it does not replay tools or provider work.
- Evidence is a review input only. The worker host cannot set `accepted` or invoke Git integration.
- Durable repository methods expose domain values and `StorageResult`; core and server do not receive a database connection.
- Worker provider validation is Copilot-only. The execution host currently fails closed with `ExecutionUnavailable` until the existing `SessionRunner` prompt-admission lifecycle can be bridged atomically without weakening W09/W08 contracts.
- Server projections use the durable repository and do not maintain an HTTP-local lease map.

## Implementation

- `migrations/0004_scheduler_execution.sql`
- `crates/storage/src/scheduler.rs`
- `crates/storage/src/lib.rs`
- `crates/storage/tests/scheduler_recovery.rs`
- `crates/core/src/scheduler_adapter.rs`
- `crates/core/src/host.rs`
- `crates/core/src/workers.rs`
- `crates/core/src/lib.rs`
- `crates/core/tests/worker_execution.rs`
- `crates/server/src/routes_workers.rs`
- `crates/server/src/lib.rs`

## Verification

- `cargo test -p devfoundry-storage --test scheduler_recovery`: PASS, 3 passed.
- `cargo test -p devfoundry-core --test worker_execution`: PASS, 2 passed.
- `cargo test -p devfoundry-server --lib`: PASS, compiled successfully.
- `/Users/joy/.cargo/bin/cargo fmt --all -- --check`: PASS.
- `/Users/joy/.cargo/bin/cargo check --workspace --all-targets`: PASS.
- `/Users/joy/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `/Users/joy/.cargo/bin/cargo test --workspace`: PASS; all workspace unit, integration, native integration, and doc-test targets passed.
- `bash scripts/validate-secretless.sh`: PASS; secretless shell, JSON, version, workflow, and documentation validation passed.
- `git diff --check`: PASS.
- RED evidence: the new targets failed on missing W19 repository/types/methods and missing `WorkerHost`.
- GREEN evidence: the same targeted storage/core targets passed after implementation.
- Full workspace gates are recorded after execution.

## Risks And Follow-Up

- The current worker host does not execute a provider or allocate a worktree; it rejects execution before side effects. A follow-up contract must atomically connect durable attempt start to `SessionRunner` admission and the W08 worktree adapter.
- `cancel` is a durable status projection for recovered unknown attempts, not proof that an external process has exited.
- Failure fingerprint write/query APIs and integration-receipt write/query APIs remain follow-up repository surface work; no worker can mint an integration receipt in this slice.
- The new scheduler tables do not yet publish scheduler-specific live events; projections read durable rows. Existing durable session/run event cursors remain authoritative and post-commit-only.
- No commit was created per task instruction.

## Roadmap Gate

Gate 7 remains **partial**: durable lease/attempt/evidence persistence and recovery are implemented and tested, while restart-safe provider/worktree execution and complete failure-fingerprint/integration-receipt repository APIs remain open.
