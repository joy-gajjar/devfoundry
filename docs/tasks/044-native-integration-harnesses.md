# Task: Native MCP, LSP, And PTY Harnesses

## Completed

- Added a repository-owned local MCP stdio fixture and macOS-only end-to-end test.
- Added a local LSP child fixture with diagnostics publication and cancellation coverage.
- Added PTY contract tests for local output and permission denial.
- Kept fixtures secretless, network-free, bounded, and temporary-directory based.

## Deferred

- Real external MCP/LSP server interoperability.
- Native PTY allocation, resize, input, and terminal restoration.
- Filesystem sandboxing for terminal commands.

## Verification

Run `cargo test -p devfoundry-tools --test native_integrations` and the normal
workspace format, check, clippy, and test commands. On non-macOS hosts the MCP
case is omitted because the production MCP adapter is intentionally macOS-only.
