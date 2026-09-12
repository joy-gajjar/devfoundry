# Task: Large Session SSE Replay

## Problem

The TUI could appear connected but show no assistant output for older sessions with more than the maximum replay window. The server returned `413 event_replay_too_large` for an initial replay, so the client reconnect loop never received events.

## Fix

- Initial session event subscriptions without an `after` cursor now return the latest bounded event tail.
- Cursor-based oversized replay requests remain rejected so clients must advance/reconnect deliberately.
- This preserves bounded memory and allows existing sessions to display recent responses immediately.

## Verification

- Existing API/SSE tests pass.
- Manual test: reopen a session with more than 1,000 events and confirm recent assistant output is visible.
