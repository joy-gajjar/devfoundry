# Task: Provider External-Compatible HTTP Fixtures

## Goal
Add deterministic, secret-free provider coverage against a local HTTP server for rate limits, content filtering, retry classification, and actual streamed tool calls.

## Scope
- Included: localhost HTTP/SSE fixtures exercising `GithubCopilotProvider` end to end.
- Explicitly excluded: TUI, tools, release packaging, automatic retry policy, and live credentials.

## Design
The test server binds an ephemeral loopback port and emits fixed HTTP or SSE responses. SSE bodies are written in small chunks so the provider's response-byte stream and parser are both exercised. HTTP errors assert normalized classification and retryability; content filtering remains terminal. Tool-call fragments are accumulated into one normalized `ToolCall` followed by the provider finish events.

## Implementation
- Added a local TCP fixture harness to `crates/llm/src/lib.rs` tests.
- Added external-compatible HTTP tests for 503 transient, 429 rate-limited, and content-filter responses.
- Added an end-to-end chunked SSE tool-call fixture.
- Kept the existing parser unit fixture for focused malformed/chunk-boundary coverage.

## Verification
- `cargo fmt --all -- --check`: blocked because `cargo` is unavailable in the execution environment.
- `cargo test -p devfoundry-llm`: blocked because `cargo` is unavailable in the execution environment.
- `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`: not run for the same environment blocker.
- `git diff --check`: command produced no diagnostics; the repository is currently untracked, so a normal Git diff cannot represent these files.
- Gate 3 remains open pending the repository's normal Cargo quality gates and the existing workspace issues recorded in `030-provider-transient-error-fixtures.md`.

## Risks And Follow-Up
- The fixture validates provider behavior without exercising an external vendor endpoint or `Retry-After` handling.
- Automatic retry budgets and runner persistence remain outside this task.
