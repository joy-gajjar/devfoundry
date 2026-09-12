# TUI And CLI

## CLI Commands

- `opencode`: interactive session.
- `opencode run <prompt>`: non-interactive request.
- `opencode session list|resume|show`.
- `opencode serve`: local API server.
- `opencode config`: inspect effective non-secret configuration.
- `opencode doctor`: environment, provider, permissions, and database diagnostics.

## TUI State

Keep a pure client state model separate from rendering. State includes connection, project, session, transcript, pending approvals, active tool, input buffer, scroll position, and error notices.

The initial DevFoundry TUI implements this state model with Ratatui/Crossterm, transcript rendering, prompt editing, submit, scrolling, and quit controls. Network event and permission synchronization remain the next integration slice.

The connected slice subscribes to durable session SSE events, submits prompts through `ApiClient`, lets the user select a project/session, resumes an existing session, and displays its configured agent/model. Reconnect continues from the last durable sequence and refreshes authoritative status and permissions.

The complete TUI-first roadmap is in [`../TUI_FEATURE_ROADMAP.md`](../TUI_FEATURE_ROADMAP.md). It is the authoritative product backlog for transcript rendering, input focus, permissions, model selection, recovery, diagnostics, Git, MCP, LSP, PTY, and accessibility improvements.

## Rendering Rules

- Render streaming deltas without corrupting the input editor.
- Render transcript entries as bounded cards with heading, bullet, fenced-code, table, and tool-output structure.
- Wrap prose and table rows to the content pane while preserving code indentation and language markers.
- Base scrollbar range on rendered transcript and viewport rather than a fixed line count; keep pagination/true row virtualization as a follow-up for very large resumed histories.
- Show tool progress and completion distinctly.
- Preserve plain text fallback for terminals with limited capabilities.
- Restore terminal mode on panic, error, and signal.

## Input And Keybindings

Essential shortcuts: `Enter` submits, `Esc` quits or closes the palette, `Ctrl-C` quits, `Ctrl-P` opens the command palette, `PageUp/PageDown` scroll, `y/n` resolve the exact visible permission, and palette commands `q/i/r` quit, interrupt, or refresh. No shortcut silently approves a broad action.

## Reconnection

On event-stream loss, show disconnected state, reconnect with the last durable sequence, then refresh authoritative session status. Live-only events may be missing and must not be fabricated.

## Non-Interactive Contract

Exit codes distinguish success, provider failure, denied operation, validation error, cancellation, and internal failure. JSON mode emits stable event objects and never terminal formatting.

## TUI Tests

Test reducer/state transitions without a terminal, then smoke-test rendering, resize, interruption, modal focus, reconnect, and terminal restoration in a pseudo-terminal.
