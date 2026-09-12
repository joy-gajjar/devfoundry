# Critical Bugs (High Severity)

These are correctness or security defects. They should be prioritized before feature work. Each entry lists the location, impact, and a proposed remediation direction. No code has been changed by this audit.

---

## C1. Sensitive-file protection is bypassable via `bash` and `pty`

- Location: `crates/tools/src/lib.rs:594` (`BashTool::execute`), `crates/tools/src/lib.rs:612` (`PtyTool::execute`)
- Related: `is_sensitive_path` at `crates/tools/src/lib.rs:678`, enforced only in `authorize_path`

The sensitive-file check (`.env`, key files, etc.) is enforced in `authorize_path`, which only covers read/write/edit/apply_patch. `BashTool` and `PtyTool` authorize on the raw command string and never call the sensitive-path check. An approved `bash` call can therefore read protected files, for example `cat .env` or `cat id_rsa`.

- Impact: The safety guarantee stated in `docs/OPENCODE_COMPARISON.md:157` ("an allow-all broker cannot override" sensitive-file policy) does not hold for shell tools.
- Severity: High (security).
- Proposed direction: Apply a sensitive-path/command screen inside the shell tool boundary, or document the limitation explicitly and gate shell tools behind a stricter permission scope.

---

## C2. Non-streaming JSON completion silently drops tool calls

- Location: `crates/llm/src/lib.rs:265` (`parse_json_completion`)

The JSON (non-SSE) response path extracts only `/message/content` and `finish_reason`. It never reads `/message/tool_calls`. If Copilot returns a non-streamed completion containing tool calls, those calls are dropped and the turn ends as if the model produced only text.

- Impact: Silent loss of model-requested tool execution on the non-streaming path; the agent appears to stop early.
- Severity: High (correctness).
- Proposed direction: Parse `/message/tool_calls` in the JSON path and emit the same tool-call events as the SSE path.

---

## C3. Error classification exists but retry is never performed

- Location: `crates/llm/src/lib.rs:101` (`is_retryable`), `crates/core/src/lib.rs:292` (runner turn loop)

`ProviderErrorClass::Transient` and `RateLimited` are classified and `is_retryable()` exists, but no code consumes the classification. Neither the provider nor the runner retries.

- Impact: Transient failures and rate limits terminate the turn immediately even though they are marked retryable. `PENDING_WORK.md` retry items are effectively unimplemented at runtime.
- Severity: High (reliability).
- Proposed direction: Add bounded retry with backoff for retryable classes, honoring `Retry-After` (see C4/M-series).

---

## C4. Content-filter matcher is over-broad and mis-ordered

- Location: `crates/llm/src/lib.rs:305` (`classify_http_error`)

The matcher uses `body_lower.contains("filter")` to classify a response as terminal `ContentFiltered`. Any error body mentioning "filter" (for example "invalid filter parameter") is misclassified as non-retryable. The check is also ordered before the 429 handling, so a rate-limit response whose body contains the word "filter" is treated as non-retryable.

- Impact: Legitimate retryable failures are suppressed; rate limits can be misclassified.
- Severity: High (reliability).
- Proposed direction: Match on a specific provider error code/type field rather than a substring, and order the 429 check before content-filter classification.

---

## C5. No message-history endpoint or pagination

- Location: router at `crates/server/src/lib.rs:159`; `list_messages` at `crates/storage/src/lib.rs:688`

There is no `GET /sessions/{id}/messages` route, although `SqliteStore::list_messages` exists. A resumed session cannot rebuild its transcript over the API, and `list_messages` itself has no `LIMIT`/`OFFSET` (unbounded `fetch_all`).

- Impact: Resumed-session transcript backfill is impossible via the API; long sessions load unbounded rows into memory.
- Severity: High (functionality + memory risk).
- Proposed direction: Add a paginated message-history endpoint and add `LIMIT`/`OFFSET` to `list_messages`.

---

## C6. Broken/dead tool-call parser unit test

- Location: `crates/llm/src/lib.rs:690` (`tool_call_parser_accumulates_wire_fragments`)

The test is annotated `#[allow(unreachable_code)]` and contains an early `return;` that dead-codes the remaining assertions. The live assertions before the return do not match the dead assertions after it, indicating the test was edited without restoring its logic. The only real coverage of cross-record tool-call fragmentation is the external-fixture test.

- Impact: The unit-level guard for tool-call fragmentation is not actually asserting; a regression in the streaming tool-call parser could pass unit tests.
- Severity: High (test integrity; the parser underpins tool-call idempotency work).
- Proposed direction: Remove the early return, restore correct assertions, and drop the `allow` attribute.

---

## Summary

| ID | Finding | Location | Severity |
|---|---|---|---|
| C1 | bash/pty bypass sensitive-file policy | `tools/src/lib.rs:594,612` | High |
| C2 | JSON path drops tool calls | `llm/src/lib.rs:265` | High |
| C3 | Classification without retry | `llm/src/lib.rs:101`, `core/src/lib.rs:292` | High |
| C4 | Over-broad `filter` match + ordering | `llm/src/lib.rs:305` | High |
| C5 | No message-history endpoint/pagination | `server/src/lib.rs:159`, `storage/src/lib.rs:688` | High |
| C6 | Broken parser unit test | `llm/src/lib.rs:690` | High |
