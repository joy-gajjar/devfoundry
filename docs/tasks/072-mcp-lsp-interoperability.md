# Task: MCP And LSP Interoperability

## Goal
Implement bounded, permission-gated local MCP and LSP interoperability with standard wire framing, lifecycle validation, cancellation cleanup, and independent native fixtures.

## Scope
- MCP newline-delimited JSON-RPC, initialize, bounded `tools/list`, correlation, timeout/desynchronization handling.
- LSP `Content-Length` JSON-RPC initialize, document diagnostics request, and shutdown/exit.
- Independent native fixture and regression tests.
- Excluded: remote MCP/OAuth, schema/protocol/core/storage/server/TUI/PTY/worktree changes.

## Design
MCP and LSP use distinct standard transports: MCP is newline-delimited JSON-RPC; LSP retains `Content-Length` framing. Permission approval is always obtained from the existing broker before external process admission and MCP calls. Invalid framing, invalid versions/correlation, timeout, cancellation, or EOF fails closed and prevents replay of an in-flight request. Owned child process groups are terminated and reaped.

## Implementation
- `crates/tools/src/mcp.rs`: newline framing, tools/list, bounded messages, desync state.
- `crates/tools/src/lsp.rs`: LSP transport and lifecycle requests.
- `crates/tools/src/bin/local-integration-fixture.rs`: independent MCP newline and LSP framed fixtures.
- `crates/tools/tests/native_integrations.rs`: native protocol regression tests.

## Verification
- RED: targeted tests initially failed because MCP emitted/expected Content-Length and the fixture did not accept newline JSON; the MCP framing assertions failed and the native newline fixture test timed out.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy -p devfoundry-tools --all-targets -- -D warnings`: passed.
- `cargo test -p devfoundry-tools --lib --tests`: passed, 52 unit tests and 7 native integration tests.
- `cargo test -p devfoundry-tools --test native_integrations -- --nocapture`: passed, 7 tests.
- `git diff --check`: passed.
- Debugging initially found the LSP native test hanging in `LspSession::start`: `lsp_request` incorrectly called the notification writer, so the fixture ignored initialization messages without an ID. The request path now writes an ID-bearing JSON-RPC request; the isolated regression then passed.
- Final workspace gates: `$HOME/.cargo/bin/cargo fmt --all -- --check`, `$HOME/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings`, `$HOME/.cargo/bin/cargo test --workspace`, and `git diff --check` all passed. Tools coverage was 53 unit tests and 7 native integration tests.

## Risks And Follow-Up
- Remote MCP transport/auth/OAuth remains deferred to the W11-dependent scope.
- LSP diagnostic publication and stale-version filtering require further protocol-level refinement if additional servers expose asynchronous publish notifications.
- Process-group cleanup remains platform-dependent and should be validated on native Linux and Windows runners.
- The current LSP implementation serializes request/response access and accepts asynchronous notifications while waiting, but does not yet persist a complete versioned diagnostic history for every URI.
