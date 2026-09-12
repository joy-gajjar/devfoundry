# Medium-Severity Risks

Reliability, consistency, and edge-case defects. These degrade behavior under load, failure, or multi-turn use, but are not immediate correctness/security breaks.

---

## M1. Permission service instance leak across turns

- Location: `crates/core/src/lib.rs:111` (`permissions` map), `:373` (per-batch overwrite), `:446` (removal)

`SessionRunner.permissions` is a `HashMap<SessionId, PermissionService>` overwritten for each tool-execution batch and only removed on run completion. `resolve_permission` iterates all services and resolves the first match. When a new `PermissionService` replaces the entry mid-run, the old service (still holding the waiting `oneshot` sender) is dropped from the map. A late API resolution updates the DB row but the waiting `authorize` future can only be released by cancellation.

- Impact: Cross-turn permission resolution can hang until cancellation in edge timing.
- Severity: Medium.
- Direction: Key pending permissions by request id independent of service instance lifetime; use durable completion signals rather than transient services.

---

## M2. Silent 4-turn tool-loop cap

- Location: `crates/core/src/lib.rs:286` (`for _turn in 0..4`)

A task needing more than four tool rounds stops silently with a partial assistant message and `Idle` status, indistinguishable from success. No loop-limit event is emitted.

- Impact: Complex tasks quietly truncate.
- Severity: Medium.
- Direction: Emit an explicit loop-limit event and make the cap configurable; add loop detection.

---

## M3. Interrupt reported as `Error`

- Location: `crates/server/src/lib.rs:640`

On interrupt, prompt status is set to `interrupted` but session status is set to `Error` because the cancelled result is not `Ok`. User interrupt is conflated with provider failure.

- Impact: A user-cancelled session is surfaced as an error.
- Severity: Medium.
- Direction: Distinguish cancellation from failure and set an `Interrupted`/`Idle` session status accordingly.

---

## M4. No WAL / busy_timeout and non-transactional prompt admission

- Location: `crates/storage/src/lib.rs:151` (`connect_with_options`), `crates/server/src/lib.rs:614` (admission sequence)

The pool sets `foreign_keys` and `max_connections(5)` but no `journal_mode=WAL` and no `busy_timeout`. With five pooled writers on default rollback-journal SQLite, concurrent writes can hit `SQLITE_BUSY`. Additionally, `admit_prompt`, `set_session_status`, and `append_event` are separate non-transactional writes; a crash between them leaves inconsistent state.

- Impact: Write contention errors and partial-state windows on crash.
- Severity: Medium.
- Direction: Enable WAL + busy_timeout; wrap prompt admission in a single transaction.

---

## M5. Unversioned event JSON payloads

- Location: `migrations/0001_initial.sql:29` (event payload column)

Event payloads are stored as opaque JSON with no schema-version tag. An `Event` enum change silently breaks replay of old rows.

- Impact: Forward/backward compatibility risk for durable replay.
- Severity: Medium.
- Direction: Add an event schema version tag and a versioned decode path.

---

## M6. MCP client: no `tools/list`, no request cancellation

- Location: `crates/tools/src/mcp.rs` (`connect`, request path)

The MCP client performs `initialize` and `tools/call` with Content-Length framing and id correlation, but never enumerates server tools via `tools/list`, and requests use only a fixed 30 s timeout with no cancellation token. The `UnsupportedPlatform` variant is defined but `connect` does not actually check the platform.

- Impact: Server-advertised tools are never discovered; requests cannot be cancelled early; non-macOS behavior is inconsistent with intent.
- Severity: Medium.
- Direction: Implement `tools/list`, plumb cancellation into requests, and enforce the platform gate.

---

## M7. `file_name().unwrap()` panic path

- Location: `crates/tools/src/lib.rs:1231`

`target.file_name().unwrap()` in the patch-backup path panics if `target` ends in `..` or is a root/empty path, with no prior guarantee that `file_name()` is `Some`.

- Impact: Potential panic on adversarial or malformed paths.
- Severity: Medium.
- Direction: Handle the `None` case with a typed error.

---

## M8. Inconsistent serialization error handling in storage

- Location: `crates/storage/src/lib.rs:113`, `:127`

`serde_json::to_string(...).unwrap()` is used for `session.status` and `message.role`, while adjacent code at `:122` handles the same pattern with `map_err`. The `unwrap()`s are a latent panic path.

- Impact: Inconsistent error handling; potential panic.
- Severity: Medium.
- Direction: Replace `unwrap()` with `map_err` to a typed storage error.

---

## M9. Path-resolution normalization gap

- Location: `crates/tools/src/lib.rs:129` (`ToolContext::resolve`)

Only the parent is canonicalized; the final candidate is `canonical_parent.join(file_name)`. The `starts_with(root)` escape check runs on a not-fully-normalized path, and symlink canonicalization only runs when the target already exists. Multi-segment relative paths containing `..` warrant a focused security review.

- Impact: Possible traversal edge cases for non-existent multi-segment targets.
- Severity: Medium (security-adjacent).
- Direction: Fully normalize the candidate before the containment check; add traversal tests.

---

## Summary

| ID | Finding | Location | Severity |
|---|---|---|---|
| M1 | Permission service instance leak | `core/src/lib.rs:373` | Medium |
| M2 | Silent 4-turn cap | `core/src/lib.rs:286` | Medium |
| M3 | Interrupt reported as Error | `server/src/lib.rs:640` | Medium |
| M4 | No WAL/busy_timeout; non-transactional admission | `storage/src/lib.rs:151`, `server/src/lib.rs:614` | Medium |
| M5 | Unversioned event JSON | `migrations/0001_initial.sql:29` | Medium |
| M6 | MCP no tools/list / cancellation / platform gate | `tools/src/mcp.rs` | Medium |
| M7 | `file_name().unwrap()` panic path | `tools/src/lib.rs:1231` | Medium |
| M8 | Storage `to_string().unwrap()` inconsistency | `storage/src/lib.rs:113,127` | Medium |
| M9 | Path normalization gap | `tools/src/lib.rs:129` | Medium |
