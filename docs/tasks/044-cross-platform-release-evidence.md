# Task: Cross-Platform Release Evidence And Handoff

## Goal

Make macOS Apple Silicon, Windows, and Linux release artifacts produce the same
auditable archive, checksum, doctor smoke, and external handoff evidence without
adding signing or package registry credentials to CI.

## Implementation

- Added `scripts/release-evidence.sh` as the shared archive/checksum/doctor gate.
- Added a per-target `release-evidence.json` record to the release workflow.
- Uploaded archive, checksum, evidence, and unsigned publication plan together.
- Documented trusted-host signing and release-owner package publication handoff.
- Extended the local release test to exercise the doctor evidence path.

## Verification

- `bash -n scripts/*.sh`
- `bash scripts/test-release.sh`
- `bash scripts/validate-secretless.sh`
- GitHub Actions release matrix: `macos-14`, `windows-latest`, and `ubuntu-latest`.

## Follow-up

Signing, Homebrew/WinGet/cargo-binstall publication, and registry authentication
remain external release-owner actions. No private key, token, or registry secret
is accepted by these scripts or workflows.
