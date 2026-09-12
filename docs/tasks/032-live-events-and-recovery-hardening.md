# Task: Live Events And Recovery Hardening

## Goal
Give API clients loss-aware live session events and deterministic prompt lifecycle behavior across reconnect, interruption, and restart boundaries.

## Scope
- Included: durable-event live fanout, replay/live reconnect ordering, prompt terminal status coverage, interrupt/session ownership contracts, and route contract fixtures.
- Explicitly excluded: provider adapters, macOS process behavior, TUI changes, automatic prompt resumption, and instance-wide live events.

## Design
The storage boundary publishes each durable event only after its transaction commits through a bounded in-process broadcast channel. An SSE subscriber is registered before durable replay, then filters already-replayed sequence numbers and unrelated sessions. A lagged live subscriber terminates so the client must reconnect and replay from its last acknowledged cursor; live delivery is not authoritative. Interrupt and event routes validate session ownership and return structured `404` errors for unknown sessions.

Prompt admission remains asynchronous. Clients use `GET /api/v1/prompts/{prompt_id}` and treat `completed`, `failed`, and `interrupted` as terminal. Shutdown and explicit interrupt classify cancellation as `interrupted`; restart recovery remains explicit and does not rerun work.

## Implementation
- `crates/storage/src/lib.rs`: added bounded post-commit event publication and subscriptions.
- `crates/server/src/lib.rs`: connected SSE replay to live fanout and hardened missing-session boundaries.
- `crates/storage/tests/lifecycle.rs`: added post-commit publication coverage.
- `crates/server/tests/api_lifecycle.rs`: added missing-session route contract coverage alongside existing prompt completion/failure/interruption tests.
- Route fixtures assert the SSE content type and structured prompt-status errors.

## Verification
- `cargo fmt --all -- --check`: blocked; `cargo` is not installed/on `PATH` in this environment.
- `cargo check --workspace --all-targets`: blocked; `cargo` is not installed/on `PATH` in this environment.
- `cargo clippy --workspace --all-targets -- -D warnings`: not run because the cargo toolchain is unavailable.
- `cargo test --workspace`: not run because the cargo toolchain is unavailable.
- `git diff --check`: passed.

## Risks And Follow-Up
- The channel is process-local and bounded; a lagged client must reconnect and replay durable events.
- Instance-wide live events, real external-provider kill/restart validation, and explicit prompt resume remain deferred.
- Provider and macOS process code were intentionally not changed.
