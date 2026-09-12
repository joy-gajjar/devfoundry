# Task: API And Server Hardening

## Goal
Provide safer API behavior under malformed, oversized, cancelled, and shutdown-time requests while preserving the existing versioned routes.

## Scope
- Included: structured HTTP errors, request IDs, retryability, request/prompt/replay limits, execution cancellation on shutdown, graceful listener shutdown, and route contract tests.
- Explicitly excluded: TUI, tools, provider adapters, and release packaging.

## Design
Errors expose stable `code`, human-safe `message`, `request_id`, `retryable`, and optional safe `details`. The server accepts a bounded caller-supplied `x-request-id` or generates a ULID and returns it in the response header and error body. Request bodies are limited to 1 MiB, prompts to 256 KiB, and one SSE replay response to 1,000 events. `ExecutionRegistry` owns a shutdown cancellation token; graceful server shutdown cancels active runner executions before listener termination.

## Implementation
- Extended `ErrorResponse` in `crates/protocol/src/lib.rs`.
- Added request context middleware, body/prompt/replay limits, structured server errors, and shutdown propagation in `crates/server/src/lib.rs`.
- Added graceful signal shutdown in `crates/devfoundry/src/runtime.rs`.
- Added focused route contract coverage for request IDs and structured errors; prompt/body limit coverage is implemented at the transport/handler boundary but could not execute because the workspace tools crate does not compile.

## Verification
- `cargo fmt -p devfoundry-server -p devfoundry-protocol -p devfoundry -- --check`: passed.
- `cargo test -p devfoundry-protocol`: passed.
- `cargo check --workspace --all-targets`: blocked by pre-existing `crates/tools/src/lib.rs:330` compile error.
- `cargo clippy --workspace --all-targets -- -D warnings`: pending after workspace compile blocker.
- `cargo test --workspace`: pending after workspace compile blocker.

## Risks And Follow-Up
- Error body rewriting is centralized in middleware so handlers do not need repetitive request-ID plumbing; malformed body-limit responses from Tower remain framework-generated and should receive a dedicated contract fixture later.
- Runtime signal handling currently covers Ctrl-C; platform-specific termination signals remain operational follow-up work.
