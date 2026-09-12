# Task: Release Distribution Readiness

## Goal

Give supported-platform releases deterministic archives, SHA-256 checksums, basic archive validation, and documented redacted diagnostics checks without changing core runtime behavior.

## Scope

- Included: release packaging, checksum generation, archive validation, doctor redaction smoke checks, supported-platform metadata, and release documentation.
- Excluded: signing, package-manager publication, updater behavior, runtime diagnostics fields, and changes to the session/provider/tool runtime.

## Design

The release job builds one target per runner, packages the existing binary with license metadata, writes a SHA-256 sidecar, and verifies the archive contains the expected binary. The doctor check supplies a sentinel Copilot token through the configured environment and asserts that JSON output contains only token-presence state, not the secret. `release/platforms.json` is the machine-readable source for the support matrix.

## Implementation

- Added `scripts/package-release.sh` for target-named `tar.gz` archives and SHA-256 sidecars.
- Added `release/platforms.json` and `docs/support-matrix.md`.
- Expanded `.github/workflows/release.yml` with packaging, checksum verification, doctor redaction validation, and uploaded artifacts.
- Updated distribution, roadmap, readiness, and decision documentation.

## Verification

- `bash -n scripts/package-release.sh`: shell syntax check.
- `cargo fmt --all -- --check`: Rust formatting check.
- `cargo check --workspace --all-targets`: workspace check.
- `cargo clippy --workspace --all-targets -- -D warnings`: lint check.
- `cargo test --workspace`: workspace tests.
- `GITHUB_COPILOT_TOKEN=release-check-secret cargo run -p devfoundry -- --json doctor`: redaction smoke check; the release workflow additionally asserts the sentinel is absent.
- Local packaging is limited to the host target; cross-platform packaging is exercised by GitHub Actions.

## Risks And Follow-Up

- Windows archives use `tar.gz` for one consistent verification path; native installer/package formats are deferred.
- Checksums are not signatures and do not authenticate the publisher.
- The existing runtime doctor output remains intentionally unchanged.
