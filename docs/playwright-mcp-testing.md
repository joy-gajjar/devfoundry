# Playwright MCP Testing

Playwright MCP is an **opt-in development test integration**. It is not enabled in DevFoundry's default runtime, release builds, or CI, and it does not receive GitHub Copilot credentials.

## Local Setup

Install the Playwright MCP server using the official Playwright MCP documentation. The MCP client requires the resolved executable to be an absolute path and accepts a fixed argument vector. Do not pass a shell command string.

Example discovery:

```sh
npm root -g
command -v npx
```

If the MCP server is launched through `npx`, resolve the actual executable path first and use an explicit argv configuration in a local-only test harness. Do not put a personal path into committed project configuration.

## Security Boundary

- Keep Playwright MCP disabled by default.
- Use a dedicated test project directory.
- Approve MCP tool calls explicitly through the DevFoundry permission flow.
- Do not expose browser credentials, cookies, API keys, or private repositories.
- Do not enable remote/network MCP transport in the current client.
- Do not add the server to release or production configuration.

## Test Goals

The integration should verify initialization, exact JSON-RPC correlation, frame bounds, permission denial, bounded output, secret redaction, and clean child-process shutdown.

## Current Limitation

The current MCP implementation supports bounded local stdio JSON-RPC only. A real Playwright MCP run requires a local executable path and manual operator setup. The committed native MCP fixture tests the protocol boundary without launching Playwright.

## Recommended Manual Flow

1. Create a disposable project directory.
2. Start DevFoundry with no sensitive browser state.
3. Start the local Playwright MCP server with explicit executable/argv configuration.
4. Approve only requested `mcp_tool` operations.
5. Verify the browser process and DevFoundry exit cleanly.
6. Remove temporary browser profiles and test artifacts.
