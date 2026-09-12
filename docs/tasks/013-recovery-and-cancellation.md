# Task: Recovery And Permission Cancellation

## Goal

Ensure interrupted process work and pending permission waits do not remain permanently active after cancellation or restart.

## Scope

- Included: cancellation-aware permission waiters, stale prompt recovery, pending permission cancellation, and startup recovery invocation.
- Excluded: automatic prompt retry, tool replay, distributed execution leases, and TUI recovery presentation.

## Design

The permission broker selects between a client resolution and the session cancellation token. Cancellation resolves the persisted request as `cancelled`, wakes no successful tool path, and returns denial semantics to the tool. On startup, admitted/running prompt inbox rows become `interrupted`; pending permission rows become `cancelled`. Recovery never reruns a non-idempotent tool automatically.

## Implementation

- Added `SqliteStore::recover_interrupted_work`.
- Added cancellation token to `PermissionService`.
- Added cancellation-aware permission waiting and persisted cancellation.
- Wired recovery into `opencode serve` startup.

## Verification

Blocked locally because `cargo`, `rustc`, and `rustup` are unavailable. Required format, check, Clippy, tests, and kill/restart integration tests remain pending. `git diff --check` must pass.

## Risks And Follow-Up

- Recovery currently marks all admitted/running prompts interrupted and requires explicit future resume.
- Cross-process leases and safe automatic retry remain future storage work.
- The API should expose interrupted prompt status and pending-history diagnostics before TUI work.
