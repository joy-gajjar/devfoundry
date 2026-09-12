# Task: Command Descendant Cleanup

## Goal

Ensure bash-tool timeout and cancellation stop the command shell and its descendants on macOS, Linux, and Windows, without changing provider, API, TUI, or release behavior.

## Scope

- Included: Unix process-group setup and bounded process-group termination in `crates/tools`.
- Included: Windows Job Object setup with kill-on-close and explicit job termination.
- Included: platform-aware descendant cleanup coverage and operational documentation.
- Excluded: shell parsing changes, PTY support, and unrelated runtime layers.

## Design

Each Unix command shell starts its own process group. Timeout or cancellation sends `SIGKILL` to that group, then waits for the direct child as before. On Windows, each command is assigned to a Job Object configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`; timeout or cancellation terminates the job before waiting for the direct child. Other non-Unix/non-Windows platforms retain Tokio's direct-child kill behavior.

## Implementation

- Added Unix process-group configuration and group termination to `crates/tools/src/lib.rs`.
- Added Windows Job Object configuration and termination to `crates/tools/src/lib.rs`.
- Added a macOS/Linux test proving a delayed descendant side effect does not occur after timeout.
- Added a Windows Job Object test proving a delayed descendant side effect does not occur after timeout.
- Added the process cleanup decision and platform support notes to the planning documentation.

## Verification

- `cargo fmt --all -- --check`: reports pre-existing formatting differences in unrelated workspace files; the touched `crates/tools/src/lib.rs` was formatted and passes a focused `rustfmt --check`.
- `cargo check -p devfoundry-tools --all-targets`: passed using `/Users/joy/.cargo/bin/cargo`.
- `cargo clippy -p devfoundry-tools --all-targets -- -D warnings`: passed using `/Users/joy/.cargo/bin/cargo`.
- `cargo test -p devfoundry-tools`: passed, 18 tests; Windows-only test is not compiled on macOS.
- `cargo check --workspace --all-targets` and `cargo test --workspace`: not run; focused gates were sufficient for this isolated change and the workspace has unrelated formatting drift.
- `git diff --check`: passed.

## Risks And Follow-Up

- The Unix implementation assumes the spawned shell successfully becomes its own process-group leader; process-group signaling is limited to that group.
- The Windows Job Object path requires a Windows build/test runner for runtime validation; this macOS/Linux workspace run cannot exercise it.
