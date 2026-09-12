# Task: Packaging Install Upgrade Rollback Preparation

## Goal

Prepare release packaging and operator workflows so supported archives can be validated, installed, upgraded, or rolled back without requiring signing secrets or changing core runtime behavior.

## Scope

- Included: archive and checksum validation, supported-platform metadata, install/upgrade/rollback scaffolding, signing handoff documentation, and release records.
- Explicitly excluded: runtime changes, private keys, artifact signing, package publication, database migration rollback, and shell/service launcher integration.

## Design

Archives remain target-named `tar.gz` files with SHA-256 sidecars. Validation checks the sidecar and requires the expected binary and `LICENSE` before extraction. The lifecycle scaffold installs into versioned user-owned directories and swaps `current` and `previous` symlinks; rollback is explicit. Signing occurs outside repository automation on a trusted host.

## Implementation

- Extended `scripts/package-release.sh` with path validation, license validation, and checksum verification.
- Added `scripts/verify-release.sh` for reusable checksum and archive-content validation.
- Added `scripts/manage-install.sh` for secretless install, upgrade, and one-version rollback preparation.
- Expanded `release/platforms.json` with artifact policy, binary, and channel metadata.
- Added `docs/release.md` and updated support, operations, decision, and pending-work documentation.
- Extended release CI to invoke archive validation and a credential-free doctor smoke against the extracted packaged binary on every supported target.

## Verification

- `bash -n scripts/package-release.sh scripts/verify-release.sh scripts/manage-install.sh`: shell syntax check.
- Local packaging and `scripts/verify-release.sh`: archive, license, and checksum validation.
- `scripts/verify-release.sh --launch-doctor`: packaged executable smoke with no provider credentials.
- `scripts/manage-install.sh install|upgrade|rollback`: lifecycle smoke test in a temporary prefix.
- Rust workspace checks: run if the Rust toolchain is available; packaging changes do not touch runtime crates.

## Risks And Follow-Up

- Signing and publisher authentication remain incomplete by design; follow `docs/release.md` for handoff.
- The scaffold does not stop running processes, install launchers, or coordinate database migrations.
- Native installers, package managers, release notes, and cross-platform rollback tests remain open.
