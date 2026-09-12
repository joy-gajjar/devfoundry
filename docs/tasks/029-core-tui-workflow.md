# Task: Core TUI Workflow

## Goal

Let a user choose a project, resume or create a session, see the session's configured agent/model, reconnect to durable events, and use documented keyboard commands without leaving the terminal in raw mode.

## Scope

- Included: project and session selection, session resume through the existing event replay contract, new-session creation, agent/model display, command palette, interrupt/refresh/quit bindings, reconnect state, and drop-based terminal cleanup coverage.
- Excluded: tools, core runner behavior, API/server internals, release work, message-history retrieval, and model/agent catalogs or mutation endpoints that do not exist in the current contracts.

## Design

The TUI remains an API client and never accesses storage directly. Startup presents projects and then sessions; Enter resumes the selected durable session and `n` creates a new one. The selected session's stored agent and provider/model are displayed in the status bar. `Ctrl-P` opens a command palette: `q` quits, `i` interrupts the current session, `r` refreshes its authoritative status, and `Esc` closes the palette. Session SSE reconnect continues from the last applied sequence, while periodic status and permission queries remain authoritative.

Terminal restoration is owned by a `Drop` guard so early returns and unwinding after terminal initialization restore Ratatui/Crossterm state.

## Implementation

- Updated `crates/tui/src/lib.rs` with project/session pickers, workflow state, metadata rendering, command palette handling, interrupt/refresh commands, and cleanup guard.
- Updated `crates/devfoundry/src/runtime.rs` to hand startup selection to the TUI instead of silently choosing the newest session.
- Added reducer and cleanup-oriented tests in `crates/tui/src/lib.rs`.

## Verification

- `cargo fmt --all -- --check`: not run; Cargo toolchain is unavailable in the environment.
- `cargo check --workspace --all-targets`: not run; Cargo toolchain is unavailable in the environment.
- `cargo clippy --workspace --all-targets -- -D warnings`: not run; Cargo toolchain is unavailable in the environment.
- `cargo test --workspace`: not run; Cargo toolchain is unavailable in the environment.
- `git diff --check`: passed.

## Risks And Follow-Up

- Existing API contracts expose stored session agent/model values but no catalog or update endpoints, so this task displays metadata rather than adding selection controls that would invent a contract.
- Existing API contracts expose no message-history route; resumed sessions receive future/replayed events but do not backfill old transcript messages.
- Pseudo-terminal resize, signal, and panic tests remain follow-up work; cleanup is guarded in-process and covered by a focused drop test.
