# Task: Production Readiness Assessment

## Goal

Record the verified state of DevFoundry and prevent a green unit-test suite from being mistaken for production readiness.

## Verified

- Rust workspace compiles.
- Formatting and Clippy pass with warnings denied.
- Unit tests and doctests pass.
- SQLite project/session lifecycle integration test passes.
- In-process HTTP project/session/prompt/SSE lifecycle test is now included.
- Invalid prompt/session validation is covered by the API integration suite.
- Permission discovery, approval, denial, cancellation, and invalid-resolution API tests pass.
- Provider streaming, tool-call, malformed-SSE, and cancellation fixtures pass.
- Tool traversal, symlink, atomic write, output, timeout, and cancellation tests pass.
- Provider configuration and redacted JSON diagnostics work.
- TUI state, connected event loop, bootstrap, and permission controls compile.
- Release CI is defined for macOS Apple Silicon, Linux x64, and Windows x64.
- GitHub Copilot is the only supported production provider; fake mode is test-only.
- macOS Apple Silicon path, SQLite spaces, Unix process cleanup, and terminal lifecycle tests pass.
- Core tools, runner recovery, context/agent policy, storage recovery, provider fixtures, API hardening, and TUI workflow suites pass.
- API prompt-status completion, storage recovery, sensitive-file policy, process-group cleanup, and release archive validation are covered.
- Backup/diagnostics, API contract expansion, read-only Git status, and Copilot smoke command scaffolding are covered.
- Session export/import, MCP/LSP/PTY contract validation, Windows CI scaffolding, and release lifecycle automation are covered.
- MCP/LSP/PTY hardening, Copilot smoke validation, Windows CI evidence, and secretless release validation pass.

## Not Yet Production-Ready

- Real GitHub Copilot network completion/tool-call validation with a user-provided token.
- Real MCP/LSP server interoperability and full PTY transport validation.
- Hostile concurrent filesystem rollback edge cases.
- Windows job-object descendant cleanup validation.
- Release artifact signing, package publication, and rollback testing. CI now generates and verifies archive checksums.
- Manual pseudo-terminal usability and signal/panic cleanup validation.

## Decision

Continue implementation. Do not label the application production-ready until every item above has evidence recorded in a task document and the relevant roadmap gate is updated.
