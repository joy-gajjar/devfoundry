# Task: TUI Session Browser Workflow

## Goal

Make existing sessions practical to find and resume while keeping the model and agent choice visible.

## Workflow

- Press `/` in the session list to enter a case-insensitive filter over title, agent, provider, and model.
- Press `Enter` to resume the selected session.
- Press `r` to rename the selected session.
- Press `a` or `m` to persist a new agent or model on the selected session.
- Press `n` to create a session after choosing an agent and model.
- The right-hand details pane shows the selected session's title, agent, model, and status.

The workflow uses the existing `PATCH /api/v1/sessions/{session_id}` contract. No new API endpoint or platform-specific terminal behavior is required.

## macOS

The picker uses the existing Crossterm event loop and terminal cleanup path, so the macOS raw-mode and terminal restoration behavior remains unchanged.
