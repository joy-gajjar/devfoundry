# Task: TUI Markdown, Focus, And Cursor

## Goal

Make agent responses readable and make prompt/permission interaction visibly focused.

## Implementation

- Added lightweight Markdown block rendering for headings, bullets, and fenced code.
- Added role headers and spacing between transcript entries.
- Added visible prompt cursor and input prefix.
- Added explicit permission actions in the Details panel.

## Verification

- Run the full macOS workspace gates.
- Submit a response containing headings, bullets, and fenced code.
- Verify the cursor remains visible in the prompt box and approval actions show `Y allow / N deny`.
