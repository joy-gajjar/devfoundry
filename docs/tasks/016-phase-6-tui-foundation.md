# Task: Phase 6 TUI Foundation

## Goal

Create the first usable terminal interaction layer without coupling rendering to storage, server, or runner internals.

## Scope

- Included: Ratatui terminal setup, pure TUI state, transcript rendering, prompt editing, submit/quit controls, scrolling, and CLI launch.
- Excluded: API networking, live session event subscription, permission dialogs, model selection, and full session resume.

## Design

`TuiState` owns only client projection state. Key handling produces prompt submissions and state transitions; rendering is a pure projection. Terminal raw mode is restored on exit. The root CLI launches the TUI when no subcommand is supplied.

## Implementation

- Added `crates/tui`.
- Added transcript, prompt, status, tool, permission, and scroll state.
- Added Ratatui renderer and Crossterm input loop.
- Added prompt submission and assistant-delta reducer tests.
- Changed the default CLI path to launch the TUI.

## Verification

Required:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Risks And Follow-Up

- The TUI currently renders local state and does not yet connect to the HTTP API.
- Permission dialogs and reconnect behavior must be added before the TUI is a usable coding product.
- Terminal cleanup should gain panic/signal tests and manual pseudo-terminal verification.
