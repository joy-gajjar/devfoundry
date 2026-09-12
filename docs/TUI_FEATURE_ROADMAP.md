# DevFoundry TUI Feature Roadmap

This is the TUI-first product roadmap for making DevFoundry a complete, efficient terminal coding environment.

> Accuracy note: the "Current TUI Capabilities" list below is aspirational for a few items. Reconnect replays events but is not a full durable-recovery UX, tool cards do not yet show exit codes, and "recovery" is a reconnect loop, not an interrupted-work recovery center. See [audit/04-tui-gaps.md](audit/04-tui-gaps.md) for the verified gap analysis.

## Current TUI Capabilities

- Ratatui/Crossterm terminal interface.
- Project and searchable session selection.
- GitHub Copilot model and build/plan/review agent selection.
- Session resume, rename, and persisted agent/model changes.
- Prompt input and bracketed paste handling.
- Live assistant deltas through SSE.
- Thinking/loading indicator.
- Tool activity display.
- Permission approval with `y/n`.
- Markdown-like headings, bullets, and code blocks.
- Structured transcript cards with wrapped prose, fenced-code language markers, table rows, and tool gutters.
- Transcript scrolling, scrollbar, mouse wheel, arrows, `j/k` scroll mode, PageUp/PageDown, Home/End.
- Reconnect and durable event replay.
- Command palette and interrupt control.
- Session persistence and recovery.

## P0: Core TUI Product

### 1. Reliable Transcript View

Render assistant responses as structured blocks instead of one flat text stream.

- Headings with hierarchy.
- Fenced code blocks with language labels.
- Tables with horizontal scrolling.
- Links and file paths highlighted.
- Long lines wrapped without corrupting code indentation.

Acceptance: a long Markdown response remains readable in a  terminal 80 columns wide.

### 2. Structured Tool Timeline

Display each tool as an expandable activity card.

- Tool name and status.
- Start time and elapsed duration.
- Permission state.
- Bounded output preview.
- Exit code and failure reason.
- Expand/collapse with Enter.

Acceptance: a user can identify which tool is running and why without reading raw logs.

### 3. Permission Approval Modal

Replace approval-only title text with a focused modal.

- Exact operation.
- Target path or command.
- Risk category.
- Why approval is needed.
- Allow once.
- Allow for session.
- Deny.
- Cancel execution.

Acceptance: approval never depends on hidden text or ambiguous keybindings.

### 4. Proper Session Browser

Provide a searchable session browser.

- Session title.
- Last activity time.
- Status.
- Agent and model.
- Project.
- Interrupted/error badges.
- Resume, rename, delete, and export actions.

Acceptance: a user can find, rename, and resume an old session without using the API or database.

### 5. Model And Agent Picker

Provide an interactive selector for available models and agents.

- Provider/model availability.
- Context window.
- Capability flags.
- Current selection.
- Build/plan/review agent descriptions.
- Keyboard search.

Acceptance: changing model or agent is visible in the selected-session details, persisted through the existing session update API, and applies to the next turn.

### 6. Input Editor

Replace the single-line input with a capable prompt editor.

- Multi-line editing.
- Cursor movement.
- Word navigation.
- Delete-to-end and delete-word.
- History navigation.
- Paste preservation.
- Draft restoration after error or cancellation.
- Emacs-style and optional Vim-style modes.

Acceptance: large prompts can be written and edited without terminal echo problems.

## P1: Differentiating Features

### 7. Context Inspector

Show exactly what context will be sent to the model.

- Loaded instruction files.
- Active agent policy.
- Selected model.
- Included messages.
- Tool definitions.
- Estimated context usage.
- Redacted secret markers.

This makes DevFoundry more transparent than a normal agent UI.

### 8. Diff Review Panel

Show proposed and completed changes before and after approval.

- File tree.
- Added/removed line counts.
- Unified diff view.
- Hunk navigation.
- Revert file.
- Accept/reject hunk where safe.

Acceptance: users can review changes without leaving the terminal.

### 9. Test And Validation Panel

Track validation commands separately from ordinary tool output.

- Test command.
- Running status.
- Passed/failed/skipped counts.
- Failure navigation.
- Rerun last validation.
- Copy failure context into prompt.

### 10. Recovery Center

Make interrupted work understandable and recoverable.

- Interrupted prompt list.
- Pending permission list.
- Last provider attempt.
- Last tool operation.
- Resume safely.
- Retry only idempotent operations.
- Explain why unsafe replay is blocked.

### 11. Transcript Search

Search the current and historical session.

- Text search.
- Search result count.
- Next/previous result.
- Search only tools, errors, files, or user messages.
- Jump to source event.

### 12. Session Tabs And Workspaces

Manage multiple sessions in one TUI process.

- Tab bar.
- Per-session event streams.
- Session activity indicators.
- New tab/resume tab.
- Project switching.
- Independent cancellation.

### 13. Command Palette

Expand the existing palette into a searchable command system.

- Commands with descriptions.
- Keybinding display.
- Fuzzy search.
- Recent commands.
- Safe command confirmation.
- User-configurable keybindings.

### 14. Cost And Token Monitor

Show provider usage when available.

- Prompt tokens.
- Completion tokens.
- Context size.
- Estimated cost when provider pricing is configured.
- Session totals.
- Warning thresholds.

## P2: Advanced Terminal Capabilities

### 15. Git Review Workflow

Provide a dedicated Git view.

- Branch and worktree.
- Changed files.
- Diff statistics.
- Read-only status/log/diff.
- Commit preview.
- Explicit approval for mutations.

### 16. Diagnostics Center

Centralize runtime health information.

- Copilot authentication.
- API connection.
- SQLite health.
- Tool availability.
- MCP/LSP process status.
- Terminal capabilities.
- Redacted export for support.

### 17. MCP Panel

Show connected MCP servers and tools.

- Server status.
- Advertised capabilities.
- Tool permission scope.
- Request/response timing.
- Stop/restart server.
- Protocol errors.

### 18. LSP Diagnostics Panel

Display language-server diagnostics.

- Errors/warnings/info.
- File and line navigation.
- Severity filters.
- Refresh diagnostics.
- Server lifecycle status.

### 19. PTY Terminal Panel

Provide an interactive terminal surface for approved PTY sessions.

- Separate terminal pane.
- Process status.
- Resize handling.
- Copy/paste.
- Explicit permission and cleanup.

### 20. Agent Plan And Timeline

Show the agent’s current plan and completed steps.

- Planned actions.
- Current action.
- Completed actions.
- Blocked action.
- User-approved changes.
- Retry state.

### 21. Bookmarks And Annotations

Allow users to mark important transcript points.

- Bookmark event.
- Add note.
- Filter bookmarks.
- Export annotated session.

### 22. Themes And Layouts

Provide polished presentation options.

- Dark/light themes.
- High contrast theme.
- Compact mode.
- Wide mode.
- Minimal mode.
- Color-blind-safe palette.
- Persisted per-user theme settings.

## P3: One Step Ahead Of OpenCode

### Evidence-First Timeline

Every assistant claim can be linked to the files, tool outputs, commands, and tests that support it.

### Explainable Permissions

Every approval explains the exact capability, target, risk, and scope. The user never approves an opaque operation.

### Recovery-First Sessions

Interrupted work is presented as a recoverable state with explicit replay safety rather than simply being marked failed.

### Context Transparency

Users can inspect instruction sources, model context, tool availability, and context limits before a request is sent.

### Responsive Large-Output UX

Long outputs are virtualized, searchable, bounded, and linked to managed files instead of freezing or flooding the terminal. The current TUI bounds its scrollbar to actual wrapped rows; true history pagination remains dependent on message-history retrieval.

### Focus-Safe Input

Text input never conflicts with navigation keys. Scroll mode, command mode, approval mode, and prompt mode are visibly distinct.

### Provider Capability Visibility

The TUI shows which models support tools, reasoning, context size, and streaming before the user selects them.

### Safe Automation Mode

Users can run repeatable tasks with a visible policy preview and deterministic permission profile instead of a hidden unrestricted mode.

## Implementation Order

1. Structured transcript and viewport virtualization.
2. Permission modal and focused approval state.
3. Input editor and session browser.
4. Model/agent picker and session update API.
5. Diff review and validation panels.
6. Context inspector and recovery center.
7. Search, tabs, command palette, and cost monitor.
8. Git, MCP, LSP, PTY, and plan timeline panels.
9. Themes, layouts, bookmarks, and accessibility polish.

## Production TUI Milestone

The TUI should not be called production-ready until:

- Long Markdown/code responses are readable.
- Multiple prompts remain isolated.
- Scroll and input modes never conflict.
- Permission approval is clear and keyboard-accessible.
- Reconnect restores authoritative state.
- Interrupted sessions can be resumed safely.
- Tool progress and errors are visible.
- Terminal state is restored after errors, signals, and panic unwinding.
- macOS Apple Silicon tests and manual terminal validation pass.
