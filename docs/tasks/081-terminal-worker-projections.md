# Task: Terminal And Worker Projections

## Goal

Expose the existing native PTY through bounded authenticated API routes and project terminal/worker state in the TUI and browser without making either client a storage or runner implementation.

## Scope

- Add process-local terminal transport around `NativePtyService`.
- Enforce one explicit input lease, bounded input, bounded output and output-gap reporting.
- Add pure TUI terminal focus/output state with `Ctrl-]` focus escape.
- Add browser terminal DTOs/client access and a dedicated byte-safe terminal panel.
- Add the worker projection client boundary and a worker-board smoke flow.
- Preserve W21 preview routes, registry, UI, tests and behavior.

Explicitly excluded: storage/migrations, scheduler implementation, runner/LLM/provider changes, credentials/resources, worktree implementation, remote terminal transport, automatic process termination on client disconnect, and terminal byte interpretation as Markdown.

## Design

Terminal creation is separate from attachment and is keyed by project. The server owns a process-local registry containing the native PTY handle and lease metadata. A client must acquire the sole input lease before sending input. Resize and output reads are independently authorized against the terminal. Closing removes the registry entry and calls the PTY cleanup path; repeated close of a removed terminal returns `terminal_not_found`.

Output uses a bounded hex-encoded wire field so arbitrary PTY bytes cannot be confused with JSON or Markdown. The response includes `offset`, `next_offset`, `truncated`, and `gap`; an offset outside the native ring returns a stable conflict error. Native platform unavailability fails closed.

The TUI keeps terminal state in `TerminalPaneState`, separate from rendering. Terminal focus routes printable input to the PTY projection only while a lease is held. `Ctrl-]` exits terminal focus without closing the session. The browser uses a dedicated `<pre>` projection and never parses terminal bytes as rich text.

Worker status remains evidence-oriented. The browser client exposes worker projections, while completion prose is not treated as acceptance and Git integration remains outside this task.

## Interaction Changes

- TUI terminal focus is explicitly entered by the eventual terminal attach action; `Ctrl-]` escapes focus.
- `Esc` does not close an attached terminal; it is reserved for focus/modal handling.
- Input without the active lease is rejected and remains visible as a non-destructive error.
- Stale output offsets show an output-gap state and require refresh/re-attachment.
- Resize sends the current row/column dimensions to the host; zero dimensions fail validation.
- Browser terminal output is announced through a bounded live region and displays attachment/gap state in text, not color alone.
- Worker cards/projections must distinguish running/review/accepted; “done” text is not acceptance.

## Failure Behavior And Accessibility

- Unknown terminal IDs return `terminal_not_found` without filesystem or process-path disclosure.
- Invalid leases return `terminal_input_lease_invalid` or `terminal_input_lease_held`.
- Oversized input returns `terminal_input_too_large`.
- Unsupported native PTY platforms return `terminal_unsupported`; no pipe fallback is attempted.
- PTY output is byte-oriented. Current accessibility is limited because cursor positioning, alternate-screen semantics, terminal color semantics, and semantic screen-reader structure are not yet modeled. Essential state is also rendered as text and keyboard operation remains supported.
- Terminal restoration remains guarded by the existing TUI cleanup path for normal return, error, panic/unwind, and signal-driven shutdown paths already owned by the TUI runtime.

## Implementation

- `crates/server/src/routes_terminals.rs`: terminal registry, create/lease/input/resize/output/close routes.
- `crates/server/src/lib.rs`: additive `ServerState` registry and route registration; W21 preview wiring preserved.
- `crates/server/tests/terminal_api.rs`: validation, not-found and redaction contract tests.
- `crates/tui/src/terminal.rs`, `crates/tui/src/lib.rs`: pure terminal reducer state and focus escape.
- `crates/tui/tests/terminal.rs`: reducer tests.
- `apps/workspace/src/api/client.ts`, `src/api/types.ts`: terminal/worker DTO and client boundaries.
- `apps/workspace/src/terminal/TerminalPanel.tsx`, `src/App.tsx`: browser projection.
- `apps/workspace/tests/terminal-client.test.ts`, `tests/e2e/worker-board.spec.ts`: browser tests.

## Verification

- `cargo test -p devfoundry-tui --test terminal`: PASS, 2 tests.
- `cargo test -p devfoundry-server --test terminal_api`: PASS, 2 tests.
- `npm run typecheck`: PASS.
- `npm test -- --run tests/terminal-client.test.ts tests/app.test.tsx`: PASS, 2 files / 2 tests.
- `cargo check -p devfoundry-server -p devfoundry-tui --tests`: PASS.
- `cargo fmt --all`: PASS.
- Full Rust/browser gates are recorded in the final verification section below after execution.

## Risks And Follow-Up

- The terminal registry is process-local and is intentionally not durable; reconnect can discover a missing terminal and must not fabricate it.
- TUI/browser live terminal polling/attach orchestration remains a follow-up around these tested pure/client boundaries.
- Worker routes already present in W19 remain projection-only; task review/integration semantics are not expanded here.
- Native PTY runtime qualification beyond the current Unix implementation remains a platform gate.

## Final Verification

- `/Users/joy/.cargo/bin/cargo fmt --all -- --check`: PASS after MSRV-compatible hexadecimal validation adjustment.
- `/Users/joy/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings`: PASS after replacing `is_multiple_of` with Rust 1.85-compatible modulo validation.
- `/Users/joy/.cargo/bin/cargo test --workspace`: PASS; all workspace suites passed, including 2 terminal API tests and 2 TUI terminal tests.
- `npm run typecheck`: PASS.
- `npm test`: PASS, 4 files / 8 tests.
- `npm run build`: PASS.
- `npx playwright test`: PASS, 8 tests across desktop and mobile projects.
- `git diff --check`: PASS.
- No commit was created. Existing W21 preview files remain in the worktree and were not overwritten.
