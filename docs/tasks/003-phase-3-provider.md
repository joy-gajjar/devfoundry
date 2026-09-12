# Task: Phase 3 Provider Abstraction

## Goal

Define a provider-neutral streaming contract and implement deterministic tests plus a GitHub Copilot-compatible provider without leaking wire types into the runner.

## Scope

- Included: normalized LLM requests/events, test-only fake provider, GitHub Copilot JSON/SSE adapter, cancellation, and error classification.
- Excluded: provider catalog, additional external adapters, token accounting, compaction, and credentials storage.

## Design

`LlmProvider::stream` returns normalized events for text, reasoning, tool calls, usage, and completion. The fake provider supports deterministic runner tests and is not a runtime option. The GitHub Copilot adapter owns HTTP headers, streaming parsing, and wire JSON.

## Implementation

- Added `crates/llm`.
- Added fake provider.
- Added GitHub Copilot-compatible streaming provider.
- Added cancellation-aware stream handling.

## Verification

- Added provider-neutral fixtures for chunked text SSE, malformed SSE, cancellation before request dispatch, OpenAI tool-definition encoding, indexed tool-call accumulation, transient HTTP failures, rate limits, and content filtering.
- `cargo fmt --all`: passed.
- `cargo test -p devfoundry-llm`: passed locally.
- Workspace `cargo check`, `cargo clippy`, and `cargo test` are blocked by an unrelated pre-existing `crates/tools` `std::io::Error` conversion error; focused provider clippy is blocked by the test-module ordering lint pending cleanup.

## Risks And Follow-Up

- SSE parsing still needs usage fixtures and live-provider response coverage.
- Tool-call continuation semantics remain owned by the runner and are not changed here.
- Credentials and provider model catalogs belong in a later slice.
