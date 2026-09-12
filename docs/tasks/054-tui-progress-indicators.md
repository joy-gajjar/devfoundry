# Task: TUI Progress Indicators

## Goal

Make model thinking and tool approval state visible during a running request.

## Implementation

- Empty trailing assistant blocks now render `thinking | / - \\` while the session is running.
- Permission titles explicitly show `APPROVAL REQUIRED`, operation, target, and `y/n` controls.
- Tool activity remains shown in the status header and transcript.

## Verification

- Run the full workspace gates.
- Submit a prompt and observe the thinking spinner before deltas arrive.
- Trigger a tool permission and verify the approval banner.
