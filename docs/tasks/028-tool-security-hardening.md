# Task: Tool Security Hardening

## Goal

Prevent project tools from escaping the project root, partially replacing files, or returning unbounded command output, while making command timeout and cancellation terminate the spawned process.

## Scope

- Included: read/write path traversal and symlink checks, atomic writes, bounded command output, timeout/cancellation cleanup, and regression tests.
- Excluded: recursive search tools, secret redaction, OS-wide process-group management, and persistent permission policy.

## Design

Resolved existing targets are canonicalized and checked against the canonical project root, including final symlinks. Writes use a unique sibling temporary file, flush it, and rename it into place; failures remove the temporary file and leave the prior target unchanged. Command stdout and stderr are each read with a one-byte-over-limit cap and combined output is truncated to 64 KiB by bytes. Timeout and cancellation kill and reap the spawned child before returning.

## Implementation

- Hardened `ToolContext::resolve` and `WriteTool` in `crates/tools/src/lib.rs`.
- Added bounded command reading and child cleanup in `crates/tools/src/lib.rs`.
- Added traversal, symlink, atomic failure, output, timeout, and cancellation tests.
- Updated the Phase 2 tool task and security planning notes.

## Verification

- `cargo fmt --all -- --check`: not run; `cargo` is unavailable in the environment.
- `cargo check -p devfoundry-tools`: not run; `cargo` is unavailable in the environment.
- `cargo test -p devfoundry-tools`: not run; `cargo` is unavailable in the environment.
- `cargo clippy -p devfoundry-tools --all-targets -- -D warnings`: not run; `cargo` is unavailable in the environment.
- `git diff --check`: passed.

## Risks And Follow-Up

- Killing the shell does not guarantee cleanup of descendants it may have spawned; process-group handling remains a follow-up.
- Filesystem checks and rename are not a single kernel-level containment transaction, so hostile concurrent directory replacement remains a residual race.
- The default allow-all permission broker remains unsuitable as production policy.
