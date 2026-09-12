# Task: TUI/API Reconnect And Authoritative Refresh

## Goal

Keep a connected TUI usable across completed SSE responses, transient API failures, and stale local permission/session state.

## Scope

- Included: SSE sequence-aware reconnect, durable event replay from the last applied sequence, session status persistence, periodic session refresh, and authoritative pending-permission refresh.
- Excluded: live server-side SSE fanout, exponential retry configuration, authentication, and visual redesign.

## Design

The SSE endpoint remains replay-oriented. Each frame carries its durable event sequence as the SSE `id`; the client reconnects after stream closure or transport failure with the last applied sequence and bounded exponential backoff. Events are therefore not lost when the response ends, and already-applied events are not replayed.

Prompt admission persists `running` and emits a `SessionStatus` event. Completion persists `idle` or `error` and emits the terminal status. The connected TUI periodically replaces its status and pending permission snapshot from the API; local approval state is not authoritative.

## Implementation

- Updated `crates/tui/src/client.rs` to parse SSE IDs and expose event sequences.
- Updated `crates/tui/src/lib.rs` with durable reconnect and session/permission refresh.
- Updated `crates/storage/src/lib.rs` with session status persistence.
- Updated `crates/server/src/lib.rs` with status transitions and durable status events.

## Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `git diff --check`

## Risks And Follow-Up

- The SSE handler still closes after replay; reconnect polling is the current live-update mechanism. A broadcast-backed live stream should replace this when API fanout is implemented.
- Retry state is process-local and resets when the TUI exits.
