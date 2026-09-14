# Credentials, Notifications, and Diagnostics Plan

**Goal:** Provide safe operational settings without exposing secrets to the WebView or Tauri bridge.
**Architecture:** Display value-free status from existing routes and host diagnostics; Keychain/provider resolution remains Rust-owned.
**Spec:** `00-overview.plan.md`, `docs/security.md`, `.cargo/audit.toml`.

## Global Constraints

- Secret values never cross Tauri/WebView boundaries.
- Keychain and notification failures remain explicit and fail closed.

## Tasks

1. Add provider/model/keychain availability status surface.
2. Add notification status/setup/revoke surface using existing routes.
3. Add app/backend/version/platform/storage/release diagnostics.
4. Add Tauri capability tests ensuring secret values never cross the boundary.
5. Add failure states for unavailable/locked/revoked credentials and unpaired notifications.

## Acceptance

- No token/password/private key appears in renderer DTOs, logs, diagnostics, or screenshots.
- Keychain failures are explicit and fail closed.
- Notifications remain disabled until explicit pairing.
- Diagnostics identify test prerelease vs stable release accurately.
