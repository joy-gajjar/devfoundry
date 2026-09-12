# Task: Application Bootstrap And TUI Polish

## Goal

Make the default binary launch a complete local session flow and improve the Ratatui presentation of runtime state.

## Scope

- Included: loopback server auto-start, project/session create-or-reuse, connected TUI launch, status header, active tool indicator, and pending permission title.
- Excluded: persistent external server reuse, reconnect, interactive approval keybindings, and model/provider selection UI.

## Design

The default DevFoundry command owns a Tokio runtime, starts the Rust server on an ephemeral loopback port, uses the API client to find or create the current project/session, and hands the session to the connected TUI. The server task is aborted when the TUI exits. TUI rendering exposes status, tool, and approval state without bypassing the API.

## Implementation

- Added runtime `start_server` and `interactive` composition.
- Added automatic project/session bootstrap.
- Fixed CLI async runtime launch path.
- Added Ratatui status header and approval-aware transcript title.
- Added `ApiClient::list_sessions`.
- Normalized relative SQLite database paths to writable absolute URLs.

## Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Risks And Follow-Up

- The default runtime currently uses the fake provider response; provider configuration must select a real provider before release.
- Permission approval keybindings/modal are still required.
- Reconnect and external server reuse remain future work.
