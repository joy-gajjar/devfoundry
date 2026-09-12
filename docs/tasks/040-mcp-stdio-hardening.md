# Task: MCP Stdio Hardening

## Goal

Keep the macOS MCP stdio adapter a narrow, permission-gated local integration
without turning it into a shell runner, network client, or unrestricted process
launcher.

## Completed Contract

- Require an MCP descriptor with the `McpTool` capability.
- Require an existing absolute executable and fixed argv entries.
- Authorize server admission and every tool call as `mcp_tool`.
- Require an object-valued server `tools` capability during initialization.
- Correlate every response with the exact request ID; notifications cannot satisfy requests.
- Apply a 30-second request deadline.
- Bound headers to 8 KiB and Content-Length bodies to 256 KiB before allocation.
- Reject malformed or duplicate `Content-Length` headers.
- Do not invoke a shell, open network connections, inherit stderr, or register MCP tools in the built-in registry.

## Verification

- Unit tests cover capability validation, executable validation, permission denial,
  response correlation, malformed headers, duplicate headers, and frame bounds.
- `git diff --check`: passed.
- `cargo fmt --all -- --check`: blocked because `cargo` is unavailable on `PATH`.
- `cargo test -p devfoundry-tools`: blocked because `cargo` is unavailable on `PATH`.

## Follow-Up

- Add an integration fixture for a real MCP child once the supported macOS CI
  environment provides a deterministic executable fixture.
- Keep remote MCP transport and network permissions out of this seam until a
  separate trust and destination policy exists.
