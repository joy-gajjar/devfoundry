# Task: macOS Apple Silicon Release Validation

## Goal

Validate the DevFoundry Apple Silicon release end to end: archive contents, executable launch, checksum, install/upgrade/rollback transitions, doctor smoke behavior, and operator documentation.

## Implementation

- Tightened archive verification to require the exact target/version paths, executable mode, and `LICENSE`.
- Added an optional packaged-binary `doctor --json` launch smoke for native Unix release jobs.
- Made checksum verification independent of `realpath` and hardened rollback preconditions.
- Updated release and support documentation without adding signing secrets or signing automation.

## Verification

- `bash -n scripts/package-release.sh scripts/verify-release.sh scripts/manage-install.sh`
- Local Apple Silicon packaging, checksum, archive, executable launch, and doctor redaction smoke.
- Temporary-prefix install, upgrade, and rollback smoke.
- Available Cargo format, check, clippy, and test commands.

## Follow-up

Signing, publisher authentication, process coordination, database migration rollback, and package-manager publication remain outside this task.
