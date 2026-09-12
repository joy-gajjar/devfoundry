# Task: Permission API Integration Tests

## Goal
Validate the permission API lifecycle through the real server router: pending discovery, approval, denial, cancellation, and invalid resolution handling.

## Scope
- Included: an in-process tool-call provider fixture and HTTP integration coverage for permission endpoints.
- Excluded: API redesign, permission policy scopes, timeouts, authentication, and TUI behavior.

## Design
The fixture emits a deterministic `write` tool call. The runner therefore creates a durable pending permission and waits for the HTTP resolution. Tests use the session-scoped pending query and resolve endpoint, then assert the resulting file or pending-state behavior. Cancellation is exercised through the existing session interrupt endpoint. Invalid and unknown permission identifiers must not mutate state.

## Implementation
- Added `crates/server/tests/permission_api.rs`.
- Added the minimal `async-trait` server test dependency for the provider fixture.
- Covered pending discovery, approval, denial, cancellation, malformed IDs, and unknown IDs.

## Verification
- `cargo fmt --all -- --check`: not run; `cargo` is unavailable in this environment.
- `cargo check --workspace --all-targets`: not run; `cargo` is unavailable in this environment.
- `cargo clippy --workspace --all-targets -- -D warnings`: not run; `cargo` is unavailable in this environment.
- `cargo test --workspace`: not run; `cargo` is unavailable in this environment.
- `git diff --check`: passed.

## Risks And Follow-Up
- The fixture polls the existing query endpoint with a short bounded delay because the API has no execution-completion wait primitive; this mirrors the existing lifecycle test limitation.
- Permission resolution remains process-local and restart recovery is outside this test scope.
