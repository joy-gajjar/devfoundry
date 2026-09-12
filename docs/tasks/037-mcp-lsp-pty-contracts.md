# Task: MCP, LSP, And PTY Integration Contracts

## Goal

Create narrow, permission-first seams for future MCP, LSP, and PTY work without adding an unsupported provider, transport, or unrestricted process capability.

## Scope

- Included: provider-neutral capability descriptors, request validation, permission operation names, a minimal LSP lifecycle/diagnostics seam, and a bounded macOS-only MCP stdio client.
- Excluded: MCP network clients, LSP JSON-RPC transport, PTY allocation, tool registration, and provider credentials.

## Contract

`devfoundry_tools::IntegrationRequest` represents one requested external capability. MCP tool calls, LSP diagnostics, and PTY sessions are separate capability classes and therefore separate permission operation classes (`mcp_tool`, `lsp_diagnostics`, and `pty_session`). Adapters must validate the request and call the existing `PermissionBroker` before external work. Targets are labels only; this contract does not interpret them as commands, URLs, paths, or executable arguments.

`IntegrationDescriptor` must declare at least one capability, and capabilities must belong to the descriptor's integration kind. Control characters, empty values, and oversized labels/targets are rejected before authorization.

The LSP seam is `devfoundry_tools::LspProcessConfig` and `LspSession`. Configuration requires an existing absolute executable and workspace directory; arguments are passed as fixed argv entries through `tokio::process::Command`, never through a shell. Session startup requires `lsp_diagnostics`, runs in the canonical workspace root, drains child output without retaining it, and terminates the entire Unix process group on cancellation or shutdown. Diagnostics are read-only, workspace-contained, control-character-free, and bounded to 1,000 entries and 64 KiB per message. No environment map, network permission, arbitrary command string, or file mutation is exposed.

## Security Tests

- Reject MCP/LSP/PTY capability-kind mismatches.
- Reject empty capability declarations.
- Reject control characters in untrusted labels and targets.
- Keep each integration's permission operation distinct.
- Reject relative LSP executables and workspace escapes.
- Stop an LSP child on cancellation and bound diagnostic storage.
- Stop LSP descendants and drain noisy stdout/stderr without unbounded retention.

## Verification

- `git diff --check`: passed.
- `cargo fmt --all -- --check`: blocked locally because `cargo` is unavailable on `PATH`.
- `cargo check --workspace --all-targets`: blocked locally because `cargo` is unavailable on `PATH`.
- `cargo test --workspace`: blocked locally because `cargo` is unavailable on `PATH`.
- `cargo clippy --workspace --all-targets -- -D warnings`: blocked locally because `cargo` is unavailable on `PATH`.

## MCP Stdio Seam

`devfoundry_tools::McpClient` accepts one existing absolute executable and fixed argv vector, never invokes a shell, does not expose a network client or network permission, and authorizes server admission and each `tools/call` as `mcp_tool`. Initialization requires an object-valued MCP `tools` capability. Requests use bounded Content-Length frames (256 KiB body, 8 KiB headers), a 30-second deadline, and exact response-ID correlation; notifications cannot satisfy a request. It is not registered as a built-in tool and does not expose arbitrary process or network execution. Tests cover capability and executable validation, control-character rejection, response correlation, timeout/framing bounds, and permission boundaries.

## Follow-Up

- Extend the MCP seam only after tool metadata, output limits, timeout, and remote trust configuration are specified.
- Add PTY execution only with explicit terminal restoration, process-group cleanup, output limits, and approval semantics.
- Add reviewed LSP JSON-RPC framing, request cancellation, and protocol message limits.
