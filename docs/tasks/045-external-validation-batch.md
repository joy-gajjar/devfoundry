# Task: External Validation Batch

## Goal

Complete the final locally verifiable external-validation seams and record the remaining credential/platform dependencies.

## Completed

- Opt-in `devfoundry update --check` with bounded anonymous metadata requests.
- Offline Copilot smoke mode and secret-safe live-token procedure.
- Native local MCP stdio fixture integration.
- Native local LSP diagnostics fixture integration.
- Native local PTY contract integration.
- Cross-platform release evidence and doctor smoke scaffolding.
- Final security, credentials, release, and vulnerability-reporting documentation.

## Verification On macOS ARM64

Passed:

- `cargo fmt --all`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `git diff --check`
- `bash -n scripts/*.sh`
- `bash scripts/validate-secretless.sh`
- `bash scripts/validate-distribution.sh`
- `bash scripts/test-release.sh`
- Native MCP/LSP/PTY tests

## External Dependencies

- Valid `GITHUB_COPILOT_TOKEN` for live provider completion.
- Windows-hosted runtime execution for Job Object behavior.
- Real third-party MCP/LSP server compatibility testing.
- Release signing keys and package-manager publisher credentials.
