# Task: W13 Native PTY Service

## Goal

Provide a separately named native pseudo-terminal capability with explicit
open, input, resize, bounded offset-based output, and close semantics. The
existing `PtyTool` remains a pipe-backed command tool and is not relabeled.

## Scope

- Included: `crates/tools/src/terminal.rs`, minimal `crates/tools/src/lib.rs`
  exports, native terminal lifecycle tests, and this task record.
- Included: Unix PTY allocation, terminal resize, input lease validation,
  bounded output ring, process-group cleanup, and explicit unsupported-platform
  behavior.
- Excluded: schema, protocol, core, storage, server, TUI, LLM, worktree,
  terminal transport routes, outer TUI restoration, and OS sandboxing.

## Design

`NativePtyService` is a capability boundary independent of the legacy
pipe-backed `PtyTool`. `open` validates dimensions and arguments, resolves an
optional cwd through the existing project-root containment policy, and asks the
central `PermissionBroker` for `pty_session` authorization before spawning.
Unix targets use `forkpty`, an owned process group, and `TIOCSWINSZ`; no shell
string interpolation is used by the native path. Non-Unix targets expose the
same typed API but return `UnsupportedPlatform` rather than silently falling
back to pipes.

One `InputLease` is issued per session. Input is rejected when the lease does
not match or exceeds 64 KiB. Output is retained in a 64 KiB byte ring and is
read by monotonically increasing offsets. When the reader falls behind the
ring, `OutputGap` is returned explicitly; terminal bytes, including ANSI and
control bytes, are not converted to another format.

`close` is idempotent, kills the owned process group, waits for the child, and
marks the output stream closed. Drop performs best-effort cleanup as a safety
net if a caller abandons a session. This is a process/permission boundary, not
an OS filesystem sandbox; the existing canonicalization still has a residual
TOCTOU risk against a hostile concurrent filesystem actor.

## Security And Permission Decisions

- Tools do not self-approve; all spawn authorization goes through the broker.
- Cwd traversal, absolute paths, and symlink escapes are rejected by
  `ToolContext::resolve` before spawn.
- Native arguments are passed as an argv vector, avoiding shell interpretation.
- Input, dimensions, argument count, output retention, and output read size are
  bounded.
- The native service does not expose secrets or persist full terminal output.
- Unsupported platform behavior fails closed; no pipe-backed substitution is
  allowed.

## Implementation

- Added `crates/tools/src/terminal.rs` with public typed lifecycle objects,
  Unix implementation, and non-Unix unsupported stubs.
- Exported only the native PTY types/constants from the tools crate root.
- Added `crates/tools/tests/terminal_lifecycle.rs` covering capability,
  interactive input/output, resize, permission denial, cwd containment,
  input bounds and leases, offset gaps, bounded output, and idempotent close.

## Verification

- RED: `cargo test -p devfoundry-tools --test terminal_lifecycle
  --no-default-features` failed to compile because the new native PTY exports
  and error type did not exist.
- Targeted native lifecycle tests:
  `cargo test -p devfoundry-tools --test terminal_lifecycle -- --test-threads=1`:
  8 passed, 0 failed.
- Focused formatting:
  `cargo fmt --all -- --check`: passed.
- Focused compile:
  `cargo check -p devfoundry-tools --all-targets`: passed.
- Focused lint:
  `cargo clippy -p devfoundry-tools --all-targets -- -D warnings`: passed.
- Focused unit tests:
  `cargo test -p devfoundry-tools --lib -- lsp::tests::diagnostics_are_contained_and_bounded` initially reproduced a hang. Root cause was the test's no-argument non-LSP child combined with shutdown always sending an LSP request. `LspSession` now tracks initialization and skips protocol shutdown when initialization was not performed; the focused test passes.
- Final native integrations:
  `cargo test -p devfoundry-tools --test native_integrations -- --nocapture`:
  7 passed, 0 failed.
- Final workspace gates:
  workspace check, workspace Clippy with `-D warnings`, workspace tests, formatting, and `git diff --check` all passed.
- `git diff --check` for the four approved W13 paths: passed.
- Workspace-wide `cargo check`, workspace-wide tests, and Windows runtime
  validation were not claimed as passing in this macOS run.

## Risks And Follow-Up

- Runtime PTY evidence in this task is Unix/macOS evidence; Windows ConPTY is
  intentionally an explicit unsupported result until its dependency/API
  surface is separately verified.
- Process cleanup uses Unix process-group signaling and does not claim an OS
  sandbox.
- The bounded ring deliberately retains only recent bytes; reconnect consumers
  must handle `OutputGap` rather than expecting durable terminal history.
- Transport routes and host-owned long-lived session supervision remain W13
  follow-up integration work and are outside this crate-owned change.

## Status

Changed locally and fully verified on the macOS host. Windows runtime validation
and transport/API integration remain follow-up work. No commit or push was made
by the worker.
