# Session Runner

## Responsibility

The runner turns an admitted prompt into a sequence of provider turns and authorized tool executions. It owns ordering, continuation, cancellation, step limits, and context assembly, but not transport or rendering.

## Lifecycle

1. Load session and verify ownership/project.
2. Prepare complete initial system context.
3. Promote one eligible prompt from the durable inbox.
4. Persist provider attempt start.
5. Stream provider events.
6. Persist assistant parts and tool calls.
7. Resolve permissions and execute tools.
8. Persist bounded tool results.
9. Continue or settle the turn.
10. Publish final state and release the session lease.

## Serialization

One foreground drain per session. New prompts are admitted immediately but are promoted at a safe boundary. The MVP may reject concurrent prompts with a conflict; queued steering is a later feature.

## Cancellation

Cancellation propagates from API/TUI to provider request, tool process, event stream, and runner. A cancellation result must distinguish user cancellation from provider failure and tool termination.

## Tool Loop Safety

- Maximum provider turns per prompt.
- Maximum tool calls per turn.
- Maximum wall-clock duration.
- Maximum aggregate tool output.
- Loop detection for repeated equivalent calls.
- Explicit stop when the provider emits an invalid tool call.

The current implementation has a bounded sequential continuation loop of four provider turns. It projects registered tool definitions, executes returned calls through the registry, persists tool start/finish events, and feeds result text into the next provider turn. Persistent user approval remains a required security follow-up.

## Context Assembly

The current slice composes a minimal system message from the baseline safety statement, session agent/model metadata, and `AGENTS.md` files discovered from the project root to the requested directory in outermost-to-innermost order. Durable conversation and tool results follow as runner messages. Dynamic context changes are admitted only at provider-turn boundaries. Configured instruction-file precedence and budget accounting remain deferred.

## Compaction

Implement after the simple loop. Compaction must preserve a durable summary, start a new context epoch, retain audit history, and never delete facts required to understand tool settlements or permission decisions.

## Runner Failure Matrix

- Provider fails before output: prompt remains retryable.
- Provider fails after tool call: tool settlement remains durable; retry must use recorded facts.
- Tool denied: record denial and let the model decide whether to continue.
- Tool crashes: record failure and continue only within policy.
- Storage fails: stop execution; never report success.
- Process dies: recover from durable state, not in-memory assumptions.
