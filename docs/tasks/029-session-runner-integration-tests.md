# Task: Session Runner Integration Tests

## Goal

Make the session runner's tool-loop, durable event, completion, failure, cancellation, and recovery behavior deterministic and executable as integration tests.

## Scope

- Included: scripted provider turns, a counting fixture tool, durable tool event ordering, assistant settlement, provider failure, cancellation, and idempotent startup recovery assertions.
- Excluded: tool implementation changes, API, TUI, release work, automatic replay of non-idempotent tools, and provider-wire compatibility.

## Design

The fixture provider consumes an in-memory FIFO of normalized provider streams, so each test controls the exact turn sequence without sleeps or network activity. Tests assert that tool start and finish events precede the atomic assistant message event, while provider failure and cancellation leave no assistant settlement. Recovery is run twice and must leave an admitted/running prompt interrupted without executing a tool.

## Implementation

- Added `crates/core/tests/session_runner.rs` with deterministic runner integration fixtures.
- Added core test dependencies for the provider stream and temporary SQLite database.
- No production core, storage, tools, API, TUI, or release code changes were required.

## Verification

- `rustup run stable rustfmt --edition 2024 crates/core/tests/session_runner.rs`: passed.
- `rustup run stable cargo fmt --all -- --check`: blocked by pre-existing formatting and syntax issues in server, tools, TUI, and runtime files outside this task.
- `rustup run stable cargo check --workspace --all-targets`: blocked by the existing `crates/tools/src/lib.rs:331` closure type error; the workspace also reports existing TUI lifetime errors and a server syntax error.
- `rustup run stable cargo clippy -p devfoundry-core --all-targets -- -D warnings`: blocked by the same existing tools compile error.
- `rustup run stable cargo test -p devfoundry-core --test session_runner`: blocked by the same existing tools compile error before the new test target compiles.
- `git diff --check`: passed for the task files.

## Risks And Follow-Up

- The tests validate current explicit startup recovery; automatic prompt resume and durable provider/tool attempt IDs remain deferred storage work.
- The workspace has no committed baseline in this checkout, so final review must distinguish this task's files from pre-existing untracked workspace artifacts.
