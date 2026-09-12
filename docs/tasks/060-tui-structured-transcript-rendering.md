# Task: Structured Transcript Rendering

## Goal

Make long assistant and tool responses readable in narrow terminals without changing provider, API, or storage contracts.

## Implementation

- Render each transcript entry as a role-colored card.
- Recognize headings, unordered bullets, fenced code blocks with language labels, and Markdown tables.
- Wrap prose and table rows to the content pane; preserve code indentation and use a distinct code background.
- Mark tool body lines with a card gutter so command output is distinguishable from ordinary text.
- Calculate scrollbar bounds from rendered rows and the current viewport instead of a fixed 100-row range.

## Verification

- Unit tests cover structured Markdown rendering, code blocks, wrapping, tables, and tool output markers.
- Existing resize, input, scroll, and terminal cleanup tests remain unchanged.
- Provider/API/storage layers are not modified.
- Manual macOS terminal validation remains required for color/Unicode fallback and very large resumed histories.

## Follow-up

- Add true row virtualization or explicit page loading once message-history pagination is available, keeping the current renderer as the plain-text fallback.
