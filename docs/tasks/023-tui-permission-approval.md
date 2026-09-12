# Task: TUI Permission Approval

## Goal

Make pending tool permissions actionable from the DevFoundry TUI.

## Scope

- Included: pending permission polling, reconnect-safe discovery, approval status, and `y`/`n` resolution controls.
- Excluded: remembered approval scopes, permission history, and policy configuration UI.

## Design

The connected TUI polls the authoritative pending-permissions endpoint after startup and during the session. A pending request opens a focused modal showing the exact operation, target, derived risk category, and the current once/exact-request scope. The modal has visible focus and accepts only `y` to allow, `n` to deny, or Escape to cancel/close without resolving. Other keys cannot reach prompt or quit handling while a request is pending. Resolution uses the API; the TUI does not mutate storage directly.

## Implementation

- Added periodic pending permission discovery.
- Added `y`/`n` approval controls.
- Added a focused approval modal with operation, target, risk, scope, and fail-closed key handling.
- Added waiting/running status transitions.
- Preserved API-driven permission resolution.

## Verification

Required:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Risks And Follow-Up

- Approval scope is currently one exact request.
- Risk is displayed conservatively from the operation name because the current permission contract does not persist a separate risk field.
- The current provider/tool fixture does not automatically generate a permission request, so manual or integration testing needs a scripted tool-call provider.
