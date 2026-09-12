# Task: Phase 4 Session Runner

## Goal

Connect durable sessions, normalized provider streams, and assistant message settlement through a minimal runner.

## Scope

- Included: one prompt execution, user/assistant message persistence, provider streaming, cancellation propagation, and durable assistant event settlement.
- Excluded: durable prompt inbox, concurrent-run locks, full tool continuation, compaction, queued prompts, and subagents.

## Design

The first runner is deliberately sequential. It receives a loaded session, persists the user prompt, consumes normalized provider events, and commits the assistant message with its durable event atomically. Storage failure prevents success reporting.

## Implementation

- Added `crates/core`.
- Added `SessionRunner` and core error mapping.
- Connected storage, provider, and tool registry contracts.

## Verification

Blocked locally because Rust tooling is unavailable. Required workspace checks remain pending.

## Risks And Follow-Up

- Tool-call continuation is not yet wired into the runner.
- Prompt admission and session serialization are still required before API exposure.
- An integration test with `FakeProvider` is required before Phase 4 can close.
