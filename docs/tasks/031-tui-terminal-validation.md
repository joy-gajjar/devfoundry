# Task: TUI Terminal Validation

## Goal

Validate that the interactive TUI renders after terminal size changes, exits on the interrupt key path, restores terminal state during panic unwinding, and can be driven by a deterministic interactive event source.

## Scope

- Included: in-memory Ratatui resize coverage, Ctrl-C interrupt coverage, panic/drop cleanup coverage, and a test-only event-source seam for the local interactive loop.
- Explicitly excluded: provider, tools, API/client internals, process-global signal handlers, and a platform-specific external pseudo-terminal harness.

## Design

The production loop still reads Crossterm events and uses the existing Ratatui cleanup callback. Tests use Ratatui's `TestBackend` and inject a finite event source, so they do not depend on terminal dimensions, wall-clock polling, or a real process terminal. Cleanup accepts a restoration callback only to make panic unwinding observable in-process; production passes `ratatui::restore` unchanged.

The Ctrl-C test covers the user-visible interrupt/exit behavior represented by the existing `Ctrl-C` key binding. OS-level SIGINT delivery and external PTY launch remain deferred because they require platform-specific process orchestration and would be brittle in the workspace unit suite.

## Implementation

- Updated `crates/tui/src/lib.rs` with a generic event-driven loop seam and callback-based cleanup guard.
- Added resize, interrupt-loop, and panic-unwind cleanup tests using only in-memory test infrastructure.

## Verification

- `/Users/joy/.cargo/bin/cargo fmt --all -- --check`: blocked by pre-existing formatting differences in `crates/tools/src/lib.rs`.
- `/Users/joy/.cargo/bin/rustfmt --edition 2024 --check crates/tui/src/lib.rs`: passed.
- `/Users/joy/.cargo/bin/cargo check -p devfoundry-tui --all-targets`: passed.
- `/Users/joy/.cargo/bin/cargo clippy -p devfoundry-tui --all-targets -- -D warnings`: passed.
- `/Users/joy/.cargo/bin/cargo test -p devfoundry-tui`: passed, 8 unit tests and 0 doctests failed.
- `/Users/joy/.cargo/bin/cargo check --workspace --all-targets`: blocked by pre-existing errors in `crates/storage/src/lib.rs` and `crates/llm/src/lib.rs`.
- `/Users/joy/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings`: blocked by pre-existing errors in `crates/storage/src/lib.rs` and `crates/llm/src/lib.rs`.
- `/Users/joy/.cargo/bin/cargo test --workspace`: blocked by the same pre-existing storage and LLM test-compilation errors.
- `git diff --check`: passed.

## Risks And Follow-Up

- This validates the TUI's deterministic lifecycle behavior but does not prove terminal restoration across an OS-level SIGINT or an external PTY process.
- Follow-up: add a platform-gated PTY smoke harness when the launch contract and supported PTY dependencies are stable.
