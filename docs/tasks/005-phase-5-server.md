# Task: Phase 5 Local API

## Goal

Expose the runtime through a local HTTP/SSE boundary that clients can use without reaching into core or storage internals.

## Scope

- Included: Axum router, health endpoint, prompt admission placeholder, durable session event replay over SSE, typed protocol errors, and loopback-ready server state.
- Excluded: listener lifecycle in the binary, authentication, live instance-wide events, full prompt-to-runner dispatch, and TUI.

## Design

The server owns transport mapping only. Durable session events are replayed with an opaque `after` cursor and SSE IDs. The prompt route establishes the public contract but remains an admission seam until the durable inbox and session registry are implemented.

## Implementation

- Added `crates/server`.
- Added health endpoint.
- Added versioned prompt route.
- Added durable session SSE replay route.
- Added typed invalid-ID and storage errors.

## Verification

Blocked locally because `cargo`, `rustc`, and `rustup` are unavailable. Required workspace checks remain pending:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Static verification: `git diff --check` passes.

## Risks And Follow-Up

- Prompt route currently acknowledges admission but does not schedule the runner.
- Server listener and loopback binding belong in application composition.
- Add SSE reconnect/live event tests and route contract tests before Phase 5 closes.
