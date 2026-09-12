# Task: TUI Multi-Message Transcript And Scroll

## Problem

Assistant deltas were appended to the most recent assistant entry anywhere in the transcript. After a second prompt, the response could be merged into the first response. Scrolling only supported PageUp/PageDown and did not explicitly follow new output.

## Fix

- Each submitted prompt creates its own assistant transcript entry.
- Streaming deltas append only to the current trailing assistant entry.
- New prompts and live deltas return the viewport to the newest content.
- Up/Down and PageUp/PageDown scrolling are supported.

## Verification

- Add regression coverage for multiple prompt/assistant grouping and scroll key behavior.
- Run the full macOS workspace gates.
