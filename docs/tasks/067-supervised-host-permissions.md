# Task: W03 Supervised Host And Correct Permissions

## Goal
Make the core session runner deterministic at permission, foreground-runner, history, cancellation, and turn-limit boundaries without changing server-facing APIs.

## Scope
- Included: `crates/core/src`, `crates/core/tests`, and this task record.
- Explicitly excluded: server, schema/protocol, storage, tools, llm, TUI, manifests, shared planning documents, commits, database destruction, and live provider tokens.

## Design
The runner retains the existing `SessionRunner::run` and `resolve_permission` entry points. Each session has one foreground drain. Permission waiters are owned by the exact request ID and are installed before the durable request/event publication; resolution also rereads durable state so an approval that wins the race is observed. Waiters are removed on success, denial, cancellation, provider failure, tool failure, and storage failure.

Durable messages are loaded in chronological order before the current prompt. Tool-call parts and compatible tool-result parts remain correlated by `ToolCallId`; provider continuation messages are derived deterministically. Cancellation, provider failure, storage failure, and provider-turn exhaustion are explicit non-success outcomes. Storage errors are hard stops and never become assistant success.

## Implementation
- RED tests added in `crates/core/tests/session_runner.rs`:
  - `permission_resolution_after_durable_publish_wakes_the_waiter`
  - `simultaneous_foreground_runs_are_rejected_per_session`
  - `durable_history_is_included_in_the_next_provider_request`
  - `exhausting_provider_turns_returns_an_explicit_limit_failure`
- The first history test is intentionally a named scaffold and will be strengthened with a request-capturing fake provider during GREEN implementation.

## Verification
- RED gate completed before production implementation:
  - `cargo test -p devfoundry-core --test session_runner` initially compiled and failed exactly two named behaviors: `exhausting_provider_turns_returns_an_explicit_limit_failure` and `simultaneous_foreground_runs_are_rejected_per_session`.
  - The first invocation using `cargo` was environment-blocked because Cargo was not on `PATH`; `$HOME/.cargo/bin/cargo` was used thereafter. A multi-filter Cargo invocation was rejected by Cargo's CLI before execution and was not treated as RED evidence.
- GREEN and targeted gates:
  - `$HOME/.cargo/bin/cargo fmt --all -- --check`: pass.
  - `$HOME/.cargo/bin/cargo test -p devfoundry-core --test session_runner`: pass, 8 passed, 0 failed.
  - `$HOME/.cargo/bin/cargo check -p devfoundry-core --all-targets`: pass.
  - `$HOME/.cargo/bin/cargo clippy -p devfoundry-core --all-targets -- -D warnings`: pass after replacing an unnecessary `filter_map` with `map`.
- Final gates:
  - `$HOME/.cargo/bin/cargo fmt --all && $HOME/.cargo/bin/cargo fmt --all -- --check`: pass.
  - `$HOME/.cargo/bin/cargo test -p devfoundry-core`: pass, 2 unit tests, 8 integration tests, 0 doc-tests failed.
  - `$HOME/.cargo/bin/cargo check -p devfoundry-core --all-targets`: pass.
  - `$HOME/.cargo/bin/cargo clippy -p devfoundry-core --all-targets -- -D warnings`: pass.

## Risks And Follow-Up
- Existing storage APIs do not expose a single atomic prompt-promotion operation to core; W03 will use the available repository methods without editing storage and will document any remaining admission limitation.
- The current core-owned history projection is deterministic and preserves tool-call/result facts as textual provider messages, but the frozen provider contract has no structured tool-message fields. A future contract revision should carry native correlated assistant/tool messages rather than the compatibility text projection.
- Permission durable reread is implemented after waiter registration and before event publication; the storage transaction itself remains owned by W02 and is not changed here.
- Explicit persisted outcome variants are constrained by the frozen schema; if a required outcome cannot be represented without a schema change, stop and report the exact contract rather than editing another owner’s files.
- No server integration was necessary, so no cross-owner contract stop was triggered.
- Supervised host extraction remains limited to core-owned modules; server integration is deferred to the integrator.
