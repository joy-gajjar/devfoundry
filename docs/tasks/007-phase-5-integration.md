# Task: Phase 5 API Integration

## Goal

Expose the current runtime through a local HTTP server and replayable SSE session-event endpoint, with a runnable `serve` command.

## Scope

- Included: Axum server crate, health endpoint, versioned prompt admission seam, durable event replay, loopback default, SQLite startup, and CLI server composition.
- Excluded: TUI, authentication, live instance event aggregation, and complete runner dispatch from HTTP prompts.

## Design

The server uses the same storage contracts as core and exposes SSE event IDs as opaque numeric cursors. `opencode serve` defaults to `127.0.0.1:4096` and `opencode.db`. Prompt acceptance is currently a protocol seam; durable prompt inbox and runner dispatch are required before calling this a complete interactive server.

## Implementation

- Added `crates/server`.
- Added health, prompt, and session event routes.
- Added `opencode serve` runtime composition.
- Added SQLite migration startup through `SqliteStore`.

## Verification

Blocked locally because `cargo`, `rustc`, and `rustup` are unavailable. `git diff --check` passes. Required workspace formatting, compilation, clippy, test, and server smoke commands remain pending.

## Risks And Follow-Up

- The prompt route acknowledges but does not yet enqueue and run prompts.
- Add listener integration tests, API contract tests, authentication for non-loopback binding, and live event semantics.
- TUI remains intentionally deferred until the API and runner contracts are complete.
