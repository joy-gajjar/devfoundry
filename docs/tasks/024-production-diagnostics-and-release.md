# Task: Production Diagnostics And Release Checks

## Goal

Add machine-readable diagnostics and a cross-platform release workflow for DevFoundry.

## Scope

- Included: `--json` diagnostics, provider key-presence reporting, release CI for macOS/Linux/Windows, and production verification documentation.
- Excluded: signing credentials, package-manager publication, updater implementation, and telemetry.

## Design

`devfoundry --json doctor` emits redacted diagnostic data suitable for support tooling. Release CI builds the DevFoundry binary for primary platforms and runs workspace tests. Secrets are never emitted.

## Implementation

- Added global `--json` diagnostics mode.
- Added release workflow for three target platforms; archive packaging and checksums are tracked in task 030.
- Added production diagnostics/release documentation.

## Verification

Run:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p devfoundry -- --json doctor`

## Risks And Follow-Up

- Release artifacts are not signed or published automatically yet.
- Provider networking, tool safety, and TUI behavior still need end-to-end/manual production validation.
