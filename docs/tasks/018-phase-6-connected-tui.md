# Task: Phase 6 Connected TUI

## Goal

Connect the terminal client to the Rust API for prompt submission and durable session event consumption.

## Scope

- Included: event-to-state projection, background SSE subscription, prompt submission, and connected TUI loop.
- Excluded: automatic reconnect, project/session picker, permission modal, and server auto-start.

## Design

The TUI starts one background event task per session and applies durable events through `TuiState::apply_event`. Prompt submission remains on the UI loop and reports transport failures as system transcript entries. Rendering remains independent from transport.

## Implementation

- Added event projection for status, deltas, tool activity, and permission resolution.
- Added `run_session` connected loop.
- Added background durable SSE consumption.
- Added prompt submission through `ApiClient`.
- Added tool event state test.

## Verification

Required and currently passing before this change; rerun after formatting:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Risks And Follow-Up

- The connected loop assumes a server and existing session are already available.
- Reconnect must refresh session and pending permissions before public release.
- TUI permission modal and project/session selection are required for a complete user workflow.
