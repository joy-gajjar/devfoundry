# Task: Copilot JSON Response Compatibility

## Goal

Fix the live Copilot smoke path when the endpoint returns a successful JSON completion instead of SSE.

## Implementation

- Detect response content type.
- Preserve bounded SSE parsing for streaming responses.
- Normalize successful JSON completion responses into text and finish events.
- Preserve secret-safe errors and smoke limits.

## Verification

- Copilot provider fixtures pass for SSE, JSON-compatible parsing, tool calls, auth, transient errors, rate limits, and content filtering.
- Full workspace format, check, Clippy, and tests pass on macOS ARM64.
