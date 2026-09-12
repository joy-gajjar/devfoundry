# Task: Copilot Response Compatibility

## Goal

Allow the GitHub Copilot provider to handle both streaming SSE responses and successful JSON completion responses without exposing response bodies or credentials.

## Implementation

- Detects response `Content-Type` safely.
- Parses `text/event-stream` through the existing bounded SSE parser.
- Parses successful JSON completions through a normalized completion adapter.
- Preserves bounded smoke limits and secret-safe provider errors.

## Verification

- Added/retained provider fixture coverage for SSE, tool calls, malformed data, authentication, rate limits, and content filtering.
- Full workspace gates required after formatting.

## Follow-Up

Run `devfoundry copilot-smoke --allow-network` with the user-provided token to validate the live Copilot endpoint/model combination.
