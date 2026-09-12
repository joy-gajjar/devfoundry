# Task: Phase 5 Prompt Dispatch

## Goal

Make the Phase 5 prompt endpoint perform durable admission and start one cancellable session execution instead of returning an acknowledgement only.

## Scope

- Included: prompt inbox table, admission/status updates, session execution registry, runner dispatch, duplicate-run conflict, interrupt endpoint, and session/project validation.
- Excluded: queued steering, restart recovery worker, persistent approval UI, live event fanout, and TUI.

## Design

The API validates the path/body session ID, rejects empty prompts, verifies the session and project, claims the session in a process-local execution registry, then persists an `admitted` inbox row. If admission fails, the registry claim is released. The runner runs in a spawned task with a cancellation token. Completion or failure settles the inbox row and releases the registry entry. A second prompt for an active session receives `409 session_busy` without leaving an orphaned inbox row.

## Implementation

- Added `prompt_inbox` migration table and indexes.
- Added storage admission and prompt status methods.
- Added server execution registry with per-session cancellation tokens.
- Connected prompt route to `SessionRunner`.
- Added `interrupt` endpoint.
- Added validation and typed API errors.

## Verification

Blocked locally because `cargo`, `rustc`, and `rustup` are unavailable. Required workspace formatting, compilation, clippy, tests, and API smoke tests remain pending. `git diff --check` passes.

## Risks And Follow-Up

- Process restart recovery must claim admitted/running prompts safely in a future storage phase.
- The in-memory registry prevents duplicates only within one server process.
- Durable live event publication and full tool continuation remain incomplete.
- The server runtime must normalize SQLite database URLs before smoke testing.
