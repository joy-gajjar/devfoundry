# Low-Severity Issues and Cleanup

Cosmetic issues, dead code, and stale artifacts. Low risk but worth resolving for hygiene and to prevent confusion.

---

## Low-Severity Issues

### L1. CLI entrypoint uses `expect()` panics

- Location: `crates/devfoundry/src/main.rs:113,125,199,215,225,259,313`

Repeated `.expect("runtime initialization failed")` and `.expect("current directory is unavailable")` produce raw panics instead of clean CLI error messages and exit codes. Acceptable at a binary entrypoint but not ideal user-facing behavior.

- Direction: Map to a top-level error handler that prints a clean message and returns a non-zero exit code.

### L2. SSE live stream terminates on lag

- Location: `crates/server/src/lib.rs:867`

The events handler breaks the loop on `RecvError::Lagged`; a slow client silently loses the live tail. The broadcast channel is only 256 (`crates/storage/src/lib.rs:157`).

- Direction: On lag, emit a resync signal instead of terminating, and consider a larger or adaptive channel.

### L3. `request_id` body/header mismatch

- Location: `crates/server/src/lib.rs:127` (middleware), `:776` (`error`)

The middleware injects a `request_id` into error bodies, but `error()` generates a different random `request_id` than the `x-request-id` header, so the two can disagree on the same response.

- Direction: Thread a single request id through both the header and the body.

### L4. Hardcoded model catalog and `"default"` session model

- Location: `crates/server/src/lib.rs:270` (catalog), `:391` (default model)

The model catalog is hardcoded and unrelated to `session.model`. The default session model is the literal `"default"`, which is not present in the catalog.

- Direction: Derive the catalog from provider metadata; use a real default model id.

### L5. `Usage`/`Finished`/`ReasoningDelta` events discarded

- Location: `crates/core/src/lib.rs:361` (usage/finished), `crates/llm/src/lib.rs:40` (`ReasoningDelta` unused)

Token usage is never persisted (blocks context budgeting), and `ReasoningDelta` is never produced by either parser, making the enum variant and its handler dead.

- Direction: Persist usage; either wire reasoning deltas or remove the dead variant.

---

## Dead Code

### D1. Redundant unused-binding suppression

- Location: `crates/core/src/lib.rs:447`

`let _ = (&self.tools, &root);` is leftover suppression; both values are used elsewhere, so the line is redundant and misleading.

### D2. Stub helper ignores its argument

- Location: `crates/core/src/lib.rs:452`

`session_status_running()` ignores `_session` and always returns `SessionStatus::Running`.

### D3. Suppressed dead field on MCP response

- Location: `crates/tools/src/mcp.rs:76`

`#[allow(dead_code)]` on `RpcResponse.id`: the response id is deserialized but the correlation smell is suppressed (see M6).

---

## Stale Artifacts

### A1. Empty legacy crate directory

- Location: `crates/opencode/`

Untracked, empty, not a workspace member (`Cargo.toml:3-13`). Should be removed.

### A2. Stale root database file

- Location: `opencode.db` (repo root)

An 86 KB SQLite file superseded by `.devfoundry.db`. Should be removed.

### A3. `.gitignore` does not ignore database files

- Location: `.gitignore`

Neither `*.db`, `opencode.db`, nor `.devfoundry.db` is ignored. The 2.4 MB `.devfoundry.db` and 86 KB `opencode.db` risk being committed.

- Direction: Add `*.db` (and any DB sidecar patterns) to `.gitignore`.

---

## Summary

| ID | Finding | Location |
|---|---|---|
| L1 | CLI `expect()` panics | `devfoundry/src/main.rs` |
| L2 | SSE terminates on lag | `server/src/lib.rs:867` |
| L3 | request_id body/header mismatch | `server/src/lib.rs:127,776` |
| L4 | Hardcoded catalog + `default` model | `server/src/lib.rs:270,391` |
| L5 | Usage/reasoning events discarded | `core/src/lib.rs:361`, `llm/src/lib.rs:40` |
| D1 | Redundant suppression | `core/src/lib.rs:447` |
| D2 | Stub status helper | `core/src/lib.rs:452` |
| D3 | Suppressed dead MCP field | `tools/src/mcp.rs:76` |
| A1 | Empty `crates/opencode/` | repo |
| A2 | Stale `opencode.db` | repo |
| A3 | `.gitignore` missing `*.db` | repo |
