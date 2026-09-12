# Task: W07 Stage A — TUI Responsive Effects

## Goal

Keep the TUI editable and predictable while prompt admission, permission polling, and durable SSE delivery are in flight. Stage A is fixture-level and preserves the public surface required by Stage B.

## Scope

- Included: pure state transitions for prompt admission/rejection, permission focus, scroll anchoring, and byte-safe SSE decoding.
- Included: documented interaction and failure behavior in the existing TUI fixtures.
- Excluded: schema, protocol, core, server, tools, W06 client integration, persistence, and production network ownership changes.

## Design

- `TuiState::begin_prompt_admission` records an optimistic transcript and retains the input draft until `confirm_prompt_admitted`; `reject_prompt_admission` removes only the optimistic rows and leaves the draft for retry.
- Legacy `push_user_prompt` retains its immediate-submit compatibility behavior for existing callers.
- Ctrl-C is handled before permission-modal focus, so it remains a global quit action. Escape dismisses the visible permission modal without resolving it; the same pending request is not reopened by the polling projection.
- Scroll position is represented as distance from the bottom and converted to a bounded top offset using rendered transcript lines and viewport height.
- `SseDecoder` accumulates bytes, validates UTF-8 only at complete frame boundaries, accepts CRLF/LF, repeated `data:` lines, `id:` with or without a space, comments, unknown fields, and a bounded frame size.

## Interaction And Failure Behavior

- Enter starts admission without clearing the draft. Success clears it; rejection shows a system error and preserves the exact draft.
- Permission modal accepts only explicit Y/N after opening; Escape cancels the modal view, not the operation. Ctrl-C exits globally even while the modal is focused.
- PageUp/PageDown, arrows, and scroll mode retain bottom/history anchoring. Resize recomputes the rendered viewport.
- Malformed or oversized SSE frames fail the stream parser rather than producing fabricated events. Split UTF-8 is not lossy.
- Existing connected-loop HTTP awaits remain a known Stage A limitation; Stage B must move admission, permission resolution, refresh, and interrupt calls behind effect tasks/channels so render/input never waits on network latency.

## Accessibility Limitations

- Keyboard-only interaction is supported for the covered controls, and essential status is not color-only.
- The current terminal renderer still uses Unicode card/border glyphs and has no screen-reader semantic tree. Plain-text fallback and richer focus announcements remain follow-up work.

## Implementation

- `crates/tui/src/lib.rs`: admission/rejection state transitions, permission focus behavior, dismissal tracking, and rendered scroll offset.
- `crates/tui/src/client.rs`: incremental bounded byte-oriented SSE decoder.
- `crates/tui/src/*` tests: reducer/effect fixtures for the behaviors above.

## Verification

- RED: initial targeted Cargo invocation was blocked because `cargo` was absent from PATH; rerunning with `$HOME/.cargo/bin/cargo` produced expected compile failures for the missing Stage A APIs (`SseDecoder`, admission confirmation, and scroll helper).
- `$HOME/.cargo/bin/cargo test -p devfoundry-tui`: 28 unit tests passed, 0 failed; doc-tests 0 run, 0 failed.
- `$HOME/.cargo/bin/cargo fmt --all -- --check`: run during implementation; initially reported pre-existing formatting changes outside the owned TUI surface plus TUI formatting. TUI changes were formatted; the workspace-wide command remains affected by unrelated `crates/llm` and `crates/tools` formatting.
- `$HOME/.cargo/bin/cargo check -p devfoundry-tui --all-targets`: passed with exit code 0 and no warnings.
- `$HOME/.cargo/bin/cargo clippy -p devfoundry-tui --all-targets -- -D warnings`: passed with exit code 0.
- `$HOME/.cargo/bin/cargo fmt --all -- --check`: final run passed with exit code 0. An earlier run exposed formatting differences in pre-existing `crates/llm` and `crates/tools` changes as well as the TUI edits; the final check was clean.
- `git diff --check`: passed with exit code 0.
- Pseudo-terminal smoke tests: not added in Stage A; the existing Ratatui `TestBackend` resize, input, cleanup, and loop fixtures remain the terminal-free smoke boundary. Native PTY coverage is deferred.

## Risks And Follow-Up

- Stage B must preserve the draft/effect contract while integrating the shared client and reconnect refresh.
- Network calls in `run_connected_loop` can still block that async loop at permission/prompt/refresh points; isolate them as cancellable effects before final W07 acceptance.
- No schema/protocol/core/server/tools files were changed.
