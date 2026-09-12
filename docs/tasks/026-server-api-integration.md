# Task: Server API Integration Tests

## Goal

Validate the complete in-process API lifecycle from project creation through prompt execution and durable event replay.

## Scope

- Included: Axum router fixture, SQLite migration setup, project/session creation, prompt admission, fake-provider execution, and SSE event replay assertion.
- Excluded: live network listener, external provider, permission-generating tool fixture, and TUI.

## Design

The test uses the real router, storage, runner, and fake provider in one process. It does not bypass HTTP handlers. A temporary project/database isolates state. Prompt execution is awaited through a short deterministic polling delay before durable SSE replay is asserted.

## Implementation

- Added `crates/server/tests/api_lifecycle.rs`.
- Added server integration test dependencies.
- Covered project/session/prompt/event lifecycle.
- Covered invalid prompt/session payload rejection (`422` for malformed JSON/domain extraction).

## Verification

Required:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Risks And Follow-Up

- The test currently uses a short timing delay; a durable execution completion query would be more deterministic.
- Add permission and interrupt API integration coverage next.
