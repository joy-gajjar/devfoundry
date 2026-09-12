# Task: Phase 6 API Client

## Goal

Give the TUI a typed client for our Rust HTTP/SSE server instead of allowing UI code to access storage or runner internals.

## Scope

- Included: project/session lifecycle, prompt submission, interruption, pending permissions, permission resolution, and durable SSE event parsing.
- Excluded: TUI wiring, automatic reconnect policy, live instance events, and generated SDKs.

## Design

`ApiClient` is a network-only client with typed methods over the versioned API. Durable session events are parsed from SSE data frames into schema events. Transport errors, server errors, and decode failures remain distinct client errors.

## Implementation

- Added `crates/tui/src/client.rs`.
- Added typed project/session/prompt/event/permission operations.
- Added durable SSE frame parsing.
- Kept TUI client independent of storage and runner crates.

## Verification

Required:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Risks And Follow-Up

- SSE parsing currently handles data frames but not automatic reconnect.
- API error bodies are retained as text rather than decoded into a structured client error.
- The TUI must add a background event task and reconnect refresh before production use.
