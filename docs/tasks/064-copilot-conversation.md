# Task: W04 Copilot Conversation And Streaming

## Goal
Provide a Copilot-only provider adapter that exposes provider-neutral requests and normalized stream events while preserving complete text, reasoning, tool-call, usage, cancellation, and typed-error behavior.

## Scope
- Included: `crates/llm/src/lib.rs` and its unit tests; Copilot-compatible JSON and SSE parsing, bounded parser state, exact HTTP classification, cancellation-aware stream reads, and capability metadata.
- Explicitly excluded: schema, protocol, core, server, tools, TUI, other providers, provider-side retries, and persisted domain-message changes.

## Design
- The existing `LlmProvider::stream(LlmRequest, CancellationToken)` trait remains unchanged.
- Copilot wire JSON remains private to the adapter. Core-facing events remain `TextDelta`, `ReasoningDelta`, `ToolCall`, `Usage`, and `Finished`.
- JSON completions map reasoning content, complete tool calls with opaque IDs, usage, text, and finish reason. SSE tool-call fragments accumulate by provider index and are emitted only at finish/DONE.
- SSE parser state is byte-buffered so a UTF-8 sequence or JSON record split across network chunks is not decoded prematurely. Response/frame state is bounded to 1 MiB and tool arguments to 128 KiB.
- EOF without a finish event is an invalid provider stream. Completed tool arguments must be valid JSON; malformed or oversized values become `InvalidData` and are never retried.
- HTTP classification is exact: 401/403 authentication, explicit content-filter signals content-filtered, 429 rate-limited, 408/5xx selected transient, and other failures permanent. The provider does not retry; `LlmError::is_retryable` remains the retry-policy metadata boundary.
- Cancellation is selected independently while awaiting each response body chunk and returns `Cancelled` without logging credentials, prompts, tokens, or raw source content.
- `ModelCapabilities` reports model capability facts and leaves pricing unknown unless explicitly known. No provider wire type is exposed to core or persisted messages.

## Implementation
- `crates/llm/src/lib.rs`: normalized capability metadata, JSON tool/reasoning/usage mapping, strict content-filter matching, byte-bounded SSE parsing, completed-argument validation, EOF detection, and chunk-read cancellation.
- `docs/tasks/064-copilot-conversation.md`: task decisions, fixtures, evidence, and follow-up record.

## Fixtures And Tests
- Existing text SSE fixture with network chunk boundaries.
- Existing fragmented tool-call SSE fixture preserving opaque `call_` ID and reconstructed arguments.
- JSON completion fixture for reasoning, tool call, usage, and finish mapping.
- Malformed SSE JSON, malformed completed tool arguments, oversized arguments, content filtering, authentication, transient HTTP, rate limiting, pre-request cancellation, and EOF-without-finish fixtures.
- Existing request-wire fixture verifies OpenAI-compatible function tool encoding and bearer authentication behavior.

## Verification
- `cargo` was not on PATH; the repository toolchain was invoked as `$HOME/.cargo/bin/cargo`.
- RED: `$HOME/.cargo/bin/cargo test -p devfoundry-llm` reached 20 tests with 4 expected new-test failures before implementation: missing JSON mapping, permissive filter classification, accepted EOF without finish, and unbounded tool arguments. One initial JSON fixture delimiter was corrected and re-run to confirm the intended production failure.
- GREEN: `$HOME/.cargo/bin/cargo test -p devfoundry-llm`: passed, 20 unit tests and 0 doctests, 0 failures.
- `$HOME/.cargo/bin/cargo fmt --all`: passed.
- `$HOME/.cargo/bin/cargo fmt --all -- --check`: passed as part of final verification.
- `$HOME/.cargo/bin/cargo clippy -p devfoundry-llm --all-targets -- -D warnings`: passed with no warnings.

## Risks And Follow-Up
- Retry orchestration remains outside this provider crate and is intentionally not activated here; retries must occur only before output and before any uncertain tool settlement, under W02/W03 attempt integration.
- `LlmError::Classified` was not structurally expanded with `Retry-After` because existing public construction sites outside the owned crate must remain source-compatible. Retry-After parsing/propagation is deferred to a coordinated contract change.
- The current public event shape retains `ToolCall` rather than introducing a separate `ToolCallComplete` variant to avoid incompatible consumers; emission is completion-bound.
- Capability metadata is descriptive only; model catalog discovery and attachment wire support remain deferred until a frozen cross-crate contract exists.
- A stalled-response cancellation integration fixture was not added because the existing single-connection fixture server cannot safely observe client disconnect while blocked without changing its test harness; cancellation is implemented with a `select!` around each body-chunk await and remains a follow-up fixture.
