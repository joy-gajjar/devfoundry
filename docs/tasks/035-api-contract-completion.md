# Task: API Contract Completion

## Goal
Complete the HTTP and SSE contract coverage for the current `/health` and `/api/v1` routes so clients can rely on documented statuses, headers, payloads, errors, prompt lifecycle states, permission resolution, session discovery, interruption, replay, and live durable fanout.

## Scope
- Included: contract tests for remaining routes, validation and error responses, request IDs and content types, prompt status, interrupt, permission discovery/resolution, session discovery, SSE framing/replay, and post-commit live fanout semantics.
- Explicitly excluded: new routes, authentication, instance-wide events, provider behavior, TUI behavior, restart/resume implementation, and unrelated workspace fixes.

## Design
The existing versioned routes remain unchanged. Tests treat SQLite durable events as authoritative: SSE subscribers register before replay, replayed sequence IDs are not duplicated by live fanout, unrelated sessions are filtered, and events published after a committed write arrive on the live stream. Prompt status is polled through its durable status endpoint, while interrupt and permission routes retain structured errors for invalid or missing resources.

Applicable planning documents: `docs/planning/09-protocol-and-api.md`, `docs/planning/13-testing-and-quality.md`, `docs/planning/14-roadmap-and-gates.md`, and `docs/planning/18-documentation-workflow.md`. Ownership is the protocol/API component and `protocol-api` agent. Tests must use the router boundary rather than storage or runner shortcuts except where a deterministic fixture is required to publish an event.

## Assumptions And Open Questions
- The current route set and JSON field names are the intended MVP contract.
- Best-effort live SSE remains bounded and reconnectable rather than lossless.
- Instance-wide live events and authentication remain deferred.

## Implementation
- Added route-level contract fixtures in `crates/server/tests/api_lifecycle.rs`.
- Fixed the durable SSE live cursor so emitted events advance the duplicate filter.
- Updated the API plan, decision log, roadmap status, and pending-work entry with the resulting coverage and limitations.

## Verification
- `/Users/joy/.cargo/bin/cargo fmt --all -- --check`: blocked by pre-existing formatting drift in `crates/devfoundry/src/main.rs`, `crates/devfoundry/src/runtime.rs`, and `crates/storage/src/lib.rs`; touched API files are formatted.
- `/Users/joy/.cargo/bin/cargo check --workspace --all-targets`: passed.
- `/Users/joy/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `/Users/joy/.cargo/bin/cargo test --workspace`: passed.
- `/Users/joy/.cargo/bin/cargo test -p devfoundry-server --test api_lifecycle --test permission_api`: passed, 15 tests.
- `git diff --check`: passed.

## Risks And Follow-Up
- A bounded live subscriber can still lag and must reconnect from its last event cursor.
- Full restart recovery, instance-wide live events, authentication, and message-history routes remain outside this task.
