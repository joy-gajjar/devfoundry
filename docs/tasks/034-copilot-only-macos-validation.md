# Task: GitHub Copilot Only And macOS Validation

## Goal

Restrict production provider support to GitHub Copilot and verify the DevFoundry runtime on macOS Apple Silicon.

## Scope

- Included: Copilot base URL/token configuration, provider fixtures, fake-provider test-only behavior, macOS SQLite/shell/TUI compatibility, Apple Silicon CI/release metadata, and release smoke checks.
- Excluded: Anthropic, Gemini, Bedrock, other external adapters, private-key signing, and OAuth token acquisition inside DevFoundry.

## Design

Production configuration accepts only `github-copilot`, with `base_url` and `token_env`. Default values are `https://api.githubcopilot.com` and `GITHUB_COPILOT_TOKEN`. DevFoundry never persists or logs the token. OAuth/device-token acquisition is an external credential handoff.

The supported macOS release target is `aarch64-apple-darwin`. SQLite uses filesystem-native paths, Unix commands use `/bin/sh` and process groups, and TUI cleanup is idempotent.

## Implementation

- Added GitHub Copilot provider configuration and fixtures.
- Removed alternate production provider paths.
- Added macOS compatibility tests and CI target checks.
- Added Apple Silicon archive/doctor/install lifecycle validation.
- Added an opt-in `copilot-smoke --allow-network` command for one bounded real-provider request.

## Verification

Passed on the macOS ARM64 host:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- Shell/archive/checksum/install lifecycle validation
- Real-provider smoke procedure is documented, but is manual-only and is not run as part of offline
  workspace verification or CI.

## Risks And Follow-Up

- The real-provider smoke test is manual only, requires `--allow-network`, has a 30-second timeout,
  reports missing/blank/authentication failures without secrets, and reports no token, prompt,
  response text, or provider error body.
- Release signing remains an external trusted-host operation.
- Windows process-group cleanup is not yet equivalent to Unix behavior.
