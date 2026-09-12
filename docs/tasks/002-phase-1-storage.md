# Task: Phase 1 Durable Storage

## Goal

Provide SQLite-backed durable storage for projects, sessions, messages, and ordered session events, with transaction boundaries suitable for runner recovery and API replay.

## Scope

- Included: SQLite connection/migrations, project/session/message repositories, durable event replay, and atomic message/event settlement.
- Excluded: provider execution, tool execution, HTTP transport, compaction, and multi-process coordination.

## Design

SQLite is the durable source of truth. Domain IDs are encoded as text behind repository methods. Messages and events use JSON payloads initially, while event cursors wrap monotonically increasing SQLite row IDs. Message settlement and its durable event commit in one transaction.

## Implementation

- Added `migrations/0001_initial.sql`.
- Added `SqliteStore` and async repository traits.
- Added project/session/message persistence and event replay.
- Added atomic message/event settlement.

## Verification

Blocked locally because `cargo`, `rustc`, and `rustup` are unavailable. Required checks:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Static verification: `git diff --check` passes.

## Risks And Follow-Up

- Enable SQLite foreign keys on each connection before closing Phase 1.
- Add migration/restart/transaction tests once Rust tooling is available.
- Add prompt inbox, permission rows, provider attempts, and recovery markers for the runner.
