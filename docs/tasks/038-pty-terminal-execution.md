# Task: Safe Terminal Execution Boundary

## Goal

Provide a permission-aware terminal capability without weakening process
cleanup, timeout, cancellation, or output limits.

## Implementation

`devfoundry_tools::PtyTool` is registered as `pty`, validates its command via
the existing `IntegrationRequest` contract, and requests the distinct
`pty_session` permission operation. It uses the bounded process boundary:
Unix process groups on macOS and Linux, and a Windows Job Object on Windows.

This release deliberately uses pipes rather than allocating a pseudo-terminal.
It canonicalizes the project working directory, checks cancellation before
spawn, bounds combined output to 64 KiB, and uses the shared Unix process-group
or Windows Job Object cleanup boundary. It is a safe non-interactive command
abstraction, not a filesystem sandbox: a command can still change directory or
access paths outside the project. Resize, terminal-mode restoration, and
interactive input semantics remain unspecified.

## Tests

- Permission denial uses `pty_session` and prevents spawn.
- Control characters are rejected by the integration contract.
- Timeout and cancellation use the shared process cleanup boundary.
- Standard tool discovery includes `pty`.
- No process is spawned when cancellation is already requested.

## Verification

Run `cargo fmt --all -- --check`, `cargo check -p devfoundry-tools --all-targets`,
`cargo clippy -p devfoundry-tools --all-targets -- -D warnings`, and
`cargo test -p devfoundry-tools`.

## Deferred Limitations

- Native PTY allocation and terminal restoration are deferred until the
  cross-platform interactive contract is defined.
- Filesystem confinement beyond the canonical working directory is deferred;
  permission approval is not a sandbox.
