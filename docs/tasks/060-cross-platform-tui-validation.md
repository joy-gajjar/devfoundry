# Task: Cross-Platform TUI Validation

## Goal

Validate terminal behavior that can regress independently of the core UI:
resize, paste and input dispatch, exit paths, cleanup, and release support on
macOS, Linux, and Windows.

## Implementation

- Route paste, mouse, and resize events through the local TUI loop, matching the
  already-connected loop's event handling.
- Install the cleanup guard before Ratatui terminal initialization so failures
  during initialization still restore terminal state.
- Add deterministic `TestBackend` coverage for resize, multiline paste, mouse
  input, Ctrl-C/Escape exit, panic unwinding, and normal cleanup.
- Run the focused TUI test target in every OS CI job.
- Document the native-terminal release smoke and evidence required for each
  supported target in [`../tui-validation.md`](../tui-validation.md).

## Verification

```sh
cargo fmt --all -- --check
cargo test -p devfoundry-tui --all-targets
cargo clippy -p devfoundry-tui --all-targets -- -D warnings
```

The native-terminal smoke remains a release-owner step because this workspace
does not provide a stable cross-platform PTY/ConPTY harness.
