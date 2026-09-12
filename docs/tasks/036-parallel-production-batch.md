# Task: Parallel Production Batch

## Goal

Complete and verify independent production-hardening slices in parallel while preserving macOS and GitHub Copilot constraints.

## Completed Slices

- SQLite backup and bounded diagnostics.
- API route contract and live SSE coverage.
- Windows Job Object cleanup implementation.
- Read-only Git status tool.
- Opt-in GitHub Copilot network smoke command.
- macOS Apple Silicon release validation.

## Verification

Passed on the macOS ARM64 host:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `git diff --check`

Current suite includes 10 API lifecycle tests, 6 storage lifecycle tests, 18 tool/security tests, 15 provider tests, 8 TUI tests, and all doctests.

## Remaining External Evidence

- Real GitHub Copilot network completion/tool-call smoke with a user-provided token.
- Windows runtime CI execution for Job Object behavior.
- Release signing and package publication credentials.
- Portable session export/import.
