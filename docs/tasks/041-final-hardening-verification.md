# Task: Final Hardening Verification

## Goal

Verify the final Copilot-only, macOS-compatible DevFoundry hardening batch.

## Completed

- Copilot smoke validation with bounded output and redacted auth/network failures.
- Windows Job Object CI evidence.
- MCP Content-Length framing, correlation, timeout, capability, and permission hardening.
- LSP workspace/process containment and diagnostics bounds.
- PTY cancellation/timeout/output/permission hardening.
- Secretless release/security validation.
- Version/update/release documentation.

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

## External Evidence Still Required

- A valid GitHub Copilot token for the opt-in live smoke command.
- Windows-hosted runtime execution rather than CI configuration alone.
- Real MCP/LSP server interoperability and native PTY validation.
- Release signing and package publication credentials.
