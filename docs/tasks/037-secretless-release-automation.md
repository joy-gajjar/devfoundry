# Task: Secretless Release Automation Handoff

## Goal

Improve release automation with package-manager metadata, explicit signing handoff validation, publication placeholders, and tested install/upgrade/rollback instructions without requiring private keys or registry credentials.

## Scope

- Included: machine-readable package-channel metadata, unsigned publication plans, signing handoff validation, release workflow artifact handoff, and local lifecycle tests.
- Excluded: private keys, signing tokens, registry credentials, live package publication, launcher integration, and database migration rollback.

## Implementation

- Added `release/package-managers.json` for Homebrew, cargo-binstall, and WinGet placeholders.
- Added Cargo package metadata to support future cargo-binstall publication.
- Added `release/publication-plan.json` and `scripts/prepare-publication.sh` for an unsigned artifact handoff.
- Added `scripts/validate-release-handoff.sh` to validate signed artifacts and reject secret material.
- Added `scripts/test-release.sh` covering packaging, checksum validation, handoff validation, publication planning, install, upgrade, and rollback.
- Updated release operations documentation and CI artifact handoff steps.
- Added Homebrew and WinGet manifest templates plus cargo-binstall archive metadata.
- Added the release notes template and opt-in update notification policy.
- Added `scripts/validate-distribution.sh` for credential-free metadata validation.

## Verification

- `bash -n scripts/*.sh`
- `bash scripts/test-release.sh`
- `bash scripts/validate-distribution.sh`
- Release workflow metadata validation runs without registry or signing credentials.
- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Risks And Follow-Up

- Package-manager entries are metadata-only and do not publish packages.
- Signing remains an external trusted-host action; no private key is accepted by CI.
- Registry authentication, maintainer review, and native package manifests remain follow-up work.
