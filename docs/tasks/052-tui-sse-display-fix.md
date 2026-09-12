# Task: TUI SSE Display Fix

## Problem

The TUI could remain in `working` state with empty assistant blocks even though the server runner was publishing events. The client SSE parser only accepted one exact `data: ` spelling and assumed LF-only frame boundaries.

## Fix

- Accept `data:` with optional whitespace.
- Preserve SSE event IDs for sequence-aware reconnect.
- Keep durable replay/live fanout as the source of assistant deltas and errors.

## Verification

Run the full workspace gates and manually submit a prompt from the TUI. Assistant deltas, tool events, status changes, and errors must appear in the transcript.
