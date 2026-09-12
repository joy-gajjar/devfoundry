# Task: W02 Transactional Lifecycle And Recovery

## Goal
Deliver storage-owned transactional lifecycle and restart recovery improvements without exposing SQLx details or changing frozen schema/protocol contracts.

## Scope
- Investigated and test-driven the W02 lifecycle/recovery acceptance boundary.
- Included storage crate and a W02 task record only.
- Explicitly excluded schema/protocol roots, core, server, tools, TUI, manifests, shared planning documents, database files, credentials, and commits.

## Design
The approved design requires atomic prompt admission/settlement, bounded message/event cursor reads, durable monotonic event sequences, idempotent attempt/tool identifiers, and restart recovery that marks interrupted work without replaying unsafe effects. Repository traits must keep SQLx and connection handles private to storage.

The current W01 contracts do not provide a run/attempt/tool state model, an ownership/lease contract, an idempotency receipt/conflict type, or a fault-injection boundary. The existing schema contains only `projects`, `sessions`, `messages`, `events`, `prompt_inbox`, and `permission_requests`; it has no `runs`, `tool_calls`, `tool_outputs`, or execution ownership table and no prompt idempotency/payload-hash columns. Creating those structures in W02 would invent an incompatible contract, so implementation is stopped at the approved safety boundary.

## Implementation
- Added temporary RED tests for the five named W02 contract-dependent cases in `crates/storage/tests/execution_faults.rs`.
- The tests deliberately checked for the minimum persisted structures needed to implement the named behaviors without inventing Rust APIs.
- Removed the temporary tests after RED verification because retaining contract-shape assertions would leave the crate with intentionally failing tests and would incorrectly define W01-owned schema semantics.
- No migration or production storage code was changed.

## Verification
- `"$HOME/.cargo/bin/cargo" test -p devfoundry-storage --test execution_faults`: **RED**, 0 passed / 5 failed. Failures were expected and specific: missing `runs`, missing `run_events`, missing `prompt_inbox.idempotency_key`, missing `prompt_inbox.payload_hash`, missing `tool_calls`, missing `tool_outputs`, and missing `execution_ownership`.
- `cargo test ...` without the explicit Cargo path: blocked because `cargo` is not on `PATH`; the repository plan already documents using `$HOME/.cargo/bin/cargo`.
- Full implementation gates were not run after the contract blocker was established; no passing claim is made for W02.

## Risks And Follow-Up
- W02 remains blocked until W01 freezes the execution/run/attempt/tool/ownership/idempotency contracts and allocates migration ownership.
- No safe migration can be authored from the current contracts without deciding persisted statuses, payload identity/hash rules, receipt shape, lease ownership/expiry, event stream scope, and recovery transitions.
- Existing storage code still has legacy non-transactional prompt admission and settlement paths; changing them safely requires the missing event and lifecycle contract.
- Existing event sequences are global SQLite autoincrement values despite the domain requirement for per-instance/session monotonic streams; changing that requires the frozen event cursor contract.
- Existing `pool()` is public for diagnostics/tests. A future repository-boundary pass should replace direct connection exposure with storage-owned diagnostics/test helpers, but that is outside this blocked W02 change and must preserve existing testability deliberately.
- Roadmap Gate 1 remains open; process-kill/interrupted-commit validation and W02 lifecycle acceptance are not complete.
