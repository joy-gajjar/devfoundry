# Plugins And Integrations

## Sequencing

1. Built-in Rust traits for internal development.
2. In-process trusted extensions with a versioned trait facade.
3. External process protocol for untrusted or cross-language tools.
4. Dynamic/native loading only if justified by real demand.

## Plugin Capabilities

Separate capabilities for tools, providers, agents, commands, context sources, event observers, MCP, and LSP. A plugin must declare capabilities and receive only the handles it needs.

## Plugin Rules

- Plugins cannot mutate storage directly.
- Plugins cannot publish fake durable events.
- Plugin tool calls pass through standard permissions and output limits.
- Plugin failures are isolated from the core runner.
- Plugin version and compatibility are explicit.

## MCP

The initial MCP seam is a macOS-only, stdio JSON-RPC client. It accepts one existing absolute executable plus fixed argv, requires the server `tools` capability, and authorizes server admission and each tool call through `mcp_tool`. It uses bounded headers and bodies, a request deadline, and exact response-ID correlation. It is not a shell runner, network client, or built-in tool registration path. Future tool metadata, output, and trust mapping must preserve these boundaries.

MCP servers remain separate trust domains and are not registered into the standard local tool registry by this seam.

## LSP

An LSP manager owns process lifecycle, workspace root, initialization, diagnostics, and shutdown. LSP results are read-only context unless a separate edit tool is explicitly invoked.

The current code only provides a permission-classified diagnostics contract. It does not spawn or manage language servers.

## PTY

PTY execution remains a separate capability from non-interactive shell execution. It requires terminal restoration, process-group cleanup, bounded output, cancellation, and explicit permission before allocation. The current code does not allocate PTYs.

## Git And Worktrees

Git operations should be a separate capability with previewable commands and safety checks. Worktrees require session ownership, cleanup policy, and recovery after interruption.

## Desktop

Desktop is a shell around the server/TUI capabilities, not a second product core. Defer until API, lifecycle, and packaging are stable.
