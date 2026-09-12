# Task: W02 Transactional Lifecycle And Recovery

## Goal
Deliver storage-owned transactional lifecycle and restart recovery improvements without exposing SQLx details or changing frozen schema/protocol contracts.

## Scope
- Investigated and test-driven the W02 lifecycle/recovery acceptance boundary.
- Included storage crate and a W02 task record only.
- Explicitly excluded schema/protocol roots, core, server, tools, TUI, manifests, shared planning documents, database files, credentials, and commits.

## Design
The approved design requires atomic prompt admission/settlement, bounded message/event cursor reads, durable monotonic event sequences, idempotent attempt/tool identifiers, and restart recovery that marks interrupted work without replaying unsafe effects. Repository traits keep SQLx details inside storage; no connection is passed to tools, handlers, or UI.

Commit `42943d8` supplied the required W01 lifecycle types. W02 persists runs with `(session_id, idempotency_key)` uniqueness, attempts, execution leases, tool calls/outputs, and per-run `run_events`. `EventEnvelope.sequence` is authoritative per run. The additional storage-only structs `AdmissionInput` and `RunStatusRecord` are inputs/results needed to keep SQLx rows private while adapting to the frozen protocol types; they are not protocol replacements.

## Implementation
- Added forward-only `migrations/0002_execution_lifecycle.sql`; migration 0001 was not edited.
- Added transactional admission with same-payload replay and different-payload conflict receipts.
- Added attempt/tool start, atomic tool settlement (status + bounded output record + event), execution lease claim, run status lookup, and startup recovery to `OutcomeUnknown` for admitted/running/waiting runs and started attempts/tools.
- Added bounded keyset reads for run events and messages with count and byte limits.
- Added named lifecycle/fault tests plus bounded-query and atomic-settlement tests in `crates/storage/tests/execution_faults.rs`.

## Verification
- Initial updated-baseline RED compile/run: `"$HOME/.cargo/bin/cargo" test -p devfoundry-storage --test execution_faults`: failed before implementation with unresolved `AdmissionInput` and missing lifecycle methods, proving the tests exercised absent storage behavior.
- `"$HOME/.cargo/bin/cargo" test -p devfoundry-storage --test execution_faults`: **8 passed, 0 failed** after implementation.
- Final targeted commands and exact results are recorded in the handoff response; full storage tests, check, clippy, and format check were run after the final edit.

## Risks And Follow-Up
- The legacy `prompt_inbox` APIs remain for compatibility; new lifecycle admission uses `runs`. Existing legacy event APIs remain global-sequence based, while W02 run events are per-run monotonic. A later compatibility migration should unify or explicitly deprecate these paths.
- Fault injection is represented by transaction boundaries and failure tests, but there is no public SQL fault injector; process kill during SQLite commit still needs an external harness.
- Tool IDs are accepted as stable opaque strings at the storage boundary because W01 does not define a persisted tool-call ID type; callers must supply the same ID for retries.
- Durable run-event publication is not wired into the legacy `PublishedEvent` channel because its payload type is the older `Event`; reconnect uses durable cursor reads. A protocol-aligned fanout adapter is follow-up work.
- `pool()` remains public for existing diagnostics/tests; no new caller should use it. A future boundary cleanup should replace it with storage-owned test helpers.
