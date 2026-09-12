# Task: Provider Transient-Error Fixtures

## Goal
Add deterministic provider-layer coverage for retryable failures, rate limits, and content filtering without changing the TUI, tools, runner, or API contracts.

## Scope
- Included: GitHub Copilot-compatible HTTP/SSE error classification and provider fixtures.
- Explicitly excluded: automatic retry policy, API error mapping, runner behavior, TUI, and tools.

## Design
Provider errors carry a minimal classification: transient, rate-limited, content-filtered, or permanent. Transport failures and selected gateway/server statuses are retryable; HTTP 429 is rate-limited; content filtering is terminal. Malformed data, cancellation, and existing unclassified request errors retain their current behavior.

## Implementation
- Added `ProviderErrorClass` and classification helpers to `crates/llm/src/lib.rs`.
- Classified Copilot-compatible transport, HTTP, streamed transport, and `content_filter` failures.
- Added deterministic HTTP and SSE fixture tests for transient, rate-limit, and content-filter cases.

## Verification
- `cargo fmt --all -- --check`: blocked by unrelated existing formatting in `crates/core`, `crates/storage`, and `crates/tools`; the touched provider file is rustfmt-clean.
- `cargo test -p devfoundry-llm`: passed, 10 tests and 0 doctests failed.
- `cargo check --workspace --all-targets`: blocked by pre-existing missing `serde` dependency in `crates/storage/src/lib.rs:35`.
- `cargo clippy --workspace --all-targets -- -D warnings`: blocked by the same pre-existing storage compile error.
- `cargo test --workspace`: blocked by the same pre-existing storage compile error.
- `git diff --check`: passed.

## Risks And Follow-Up
- This slice exposes classification but does not perform automatic retries; retry budgets and provider attempt persistence remain runner/storage work.
- Live-provider fixtures and `Retry-After` parsing remain future work.
