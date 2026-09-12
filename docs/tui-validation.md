# TUI Validation

The TUI keeps the existing Ratatui/Crossterm interaction model. Validation is
split between deterministic tests and runner-level smoke checks so terminal
state is not changed by ordinary unit tests.

## Automated Contract

`cargo test -p devfoundry-tui --all-targets` covers:

- rendering after a terminal resize event;
- pasted text, including newlines, entering the prompt as one input value;
- mouse scroll events without panics;
- Ctrl-C and Escape exit paths;
- terminal restoration during panic unwinding and normal drop;
- rendering through Ratatui's in-memory `TestBackend`.

The same focused test command runs on the Linux, macOS, and Windows CI jobs.
The tests do not enable raw mode, require a real TTY, or depend on terminal
dimensions supplied by the host.

## Manual Release Smoke

Before publishing an artifact on each supported operating system, run the
packaged binary from a real terminal and record the result with the release
evidence:

1. Start the binary in an empty project directory.
2. Paste multiline text and verify it remains a single prompt.
3. Resize the terminal smaller and larger while the prompt and transcript are visible.
4. Press Ctrl-C, then launch the binary again in the same terminal. The shell prompt and input echo must remain normal.
5. Repeat the exit check with Escape and, where applicable, close the terminal window during an active session.

Use a native terminal on macOS and Windows rather than relying only on a CI
shell. On Linux, exercise at least one common terminal emulator and one SSH
session when those are part of the release support claim.

## Evidence And Limits

CI proves the event handling and cleanup contract on all three supported OS
runners. The manual smoke proves integration with the host terminal and should
include OS, terminal name/version, artifact target, and pass/fail notes.

External PTY automation is intentionally not part of the unit suite: PTY APIs,
ConPTY, and terminal emulator behavior differ across operating systems. Add a
platform-specific harness only when its launch and artifact-capture contract is
stable enough to keep as a release gate.
