# Task: API Completion And Restart Boundaries

## Goal
Expose durable prompt execution status so API clients can observe completion, failure, interruption, and recovery without timing sleeps, and cover process shutdown/restart boundaries.

## Scope
- Included: prompt status query, durable cancellation classification, API lifecycle contract tests, shutdown cancellation tests, and restart persistence assertions.
- Explicitly excluded: provider, TUI, and release changes; automatic replay of interrupted prompts.

## Design
`POST /api/v1/sessions/{session_id}/prompt` remains asynchronous and returns `prompt_id`. Clients query `GET /api/v1/prompts/{prompt_id}` until its durable inbox status is terminal. Provider errors settle as `failed`; cancellation caused by the interrupt endpoint or execution-registry shutdown settles as `interrupted`. Existing recovery marks admitted/running work interrupted and remains idempotent; it does not rerun prompts.

## Implementation
- Added `PromptStatusResponse` and `GET /api/v1/prompts/{prompt_id}`.
- Added storage lookup for prompt status and session ownership.
- Classified cancelled API executions as `interrupted` instead of `failed`.
- Extended API boundary fixtures for prompt completion, provider failure, interrupt, and registry shutdown.
- Existing storage restart test verifies interrupted durable work survives reconnect and recovery, including the status query's persisted result.

## Verification
- `cargo fmt --all -- --check`: blocked, `cargo` is not installed/on `PATH` in this environment.
- `cargo check --workspace --all-targets`: blocked, `cargo` is not installed/on `PATH` in this environment.
- `cargo clippy --workspace --all-targets -- -D warnings`: not run because the cargo toolchain is unavailable.
- `cargo test --workspace`: not run because the cargo toolchain is unavailable.
- `git diff --check`: passed.

## Risks And Follow-Up
- Prompt status is durable, but live event fanout and automatic resumption after process restart remain deferred.
- The API integration fixture uses a deterministic in-process provider; real-provider end-to-end coverage remains separate.
