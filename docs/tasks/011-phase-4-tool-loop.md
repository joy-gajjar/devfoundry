# Task: Phase 4 Tool Loop

## Goal

Execute provider-requested tool calls through the shared registry and feed bounded tool results into a follow-up provider turn.

## Scope

- Included: tool definitions in provider requests, up to four sequential continuation turns, tool request decoding, tool start/finish events, tool result message parts, cancellation propagation, and provider follow-up messages.
- Excluded: persistent user approval flow, parallel tools, provider-native tool-call wire compatibility, compaction, and subagents.

## Design

The runner advertises the registry's tools using provider-neutral definitions. Each provider turn may return tool calls. Calls execute sequentially through `ToolRegistry`, sharing the session cancellation token and project root. Results become tool message context and the runner continues up to four turns. Tool start and finish events are durable, while the assistant settlement remains atomic with its final message.

## Implementation

- Added tool definition projection from the registry.
- Added bounded sequential tool continuation loop.
- Added tool request JSON parsing and result parts.
- Added durable tool start/finish events.
- Added provider follow-up messages containing tool output.

## Verification

Blocked locally because `cargo`, `rustc`, and `rustup` are unavailable. Required workspace formatting, compilation, Clippy, tests, and fake-provider tool-loop integration tests remain pending. `git diff --check` should pass.

## Risks And Follow-Up

- The current runner uses `AllowAllPermissions`; production execution must connect a user-facing permission broker before mutating tools are registered.
- GitHub Copilot-compatible tool-call wire encoding and streamed argument deltas are incomplete.
- Tool results need explicit generic output bounding and managed files.
- Runtime composition now wires the initial `read` and `write` tools. The permission broker is still development-only and must be replaced before public use.
