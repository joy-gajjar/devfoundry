# Product And UX

## Core User Journey

1. User launches in a repository.
2. Application discovers the project root and loads visible configuration.
3. User creates or resumes a session.
4. User enters a natural-language task.
5. Agent explains intent briefly, then streams work and tool status.
6. Permission requests show the exact operation, target, and reason.
7. User approves, denies, or approves once for a bounded scope.
8. Agent reports changes, validation, unresolved issues, and next steps.
9. User can inspect history or continue later.

## Interaction Modes

- `interactive`: TUI with prompts and approval dialogs.
- `print`: one request, streamed or final text, suitable for scripts.
- `json`: machine-readable events and final result.
- `server`: local API without owning a terminal.
- `plan`: read-only analysis unless a future explicit exception is approved.

## Agent Modes

### Build

Can use approved read, write, edit, patch, and command tools. Must still ask for protected actions.

### Plan

Read-only by default. It may inspect files and run explicitly safe read-only commands, then produce a plan without changing the repository.

### Future Review

A review mode should analyze a diff and report findings without editing. It is separate from plan because its output is structured around defects.

## Permission UX

Every request should show:

- Operation type.
- Exact path, command, or network target.
- Why the agent requested it.
- Scope of the approval.
- Risk warning when the operation is destructive, external, or broad.

Never make approval depend on reading a large hidden prompt. The user should be able to deny without losing the session.

## TUI Information Hierarchy

- Main transcript: user and assistant content.
- Activity line: current tool and elapsed time.
- Side/status area: project, session, model, agent, token estimate, connection state.
- Modal layer: permission, question, error, and confirmation flows.
- Command palette: session, model, agent, theme, and diagnostic commands.

## UX Failure Cases

- Provider failure: preserve the prompt and explain whether retry is safe.
- Tool failure: show the command/path and bounded output; allow retry or continue.
- Lost terminal connection: server continues only when explicitly configured; otherwise cancel safely.
- Permission timeout: deny, persist the reason, and keep the session resumable.
- Huge output: show a preview and a managed file path.
- Ambiguous edits: ask a question rather than guessing.

## Accessibility And Portability

- Work with keyboard only.
- Do not encode meaning only through color.
- Support narrow terminals and resize events.
- Keep output usable in plain print mode.
- Avoid requiring Unicode glyphs for essential status.

## UX Acceptance Tests

- A new user can understand a permission request in under one screen.
- A denied write leaves no partial file mutation.
- Restarting returns the user to the last durable session state.
- Non-interactive output never contains terminal control sequences.
