# Testing And Quality

## Test Pyramid

- Unit: pure schema, policy, parsers, reducers, truncation, patching.
- Component: storage, provider adapters, tool implementations, event hub.
- Contract: HTTP/SSE and serialized records.
- Integration: runner with fake provider and temporary projects.
- End-to-end: installed binary, real local server, controlled provider fixture.
- Manual: TUI usability and destructive-operation review.

## Deterministic Fake Provider

Build a scripted provider that emits text, tool calls, malformed output, delays, errors, and cancellation checkpoints. Most runner tests must use it, not a live provider.

## Required Scenario Matrix

- Simple final answer.
- Read then answer.
- Write after approval.
- Write denial and recovery.
- Tool failure and continuation.
- Provider retry and non-retryable error.
- Cancellation during provider and tool execution.
- Process restart with admitted prompt.
- Event replay after reconnect.
- Context overflow and compaction.
- Two sessions executing concurrently.

## Quality Gates

Every change must pass format, clippy, unit/component tests, and relevant integration tests. Security-sensitive changes require threat-model review and regression coverage.

## Fuzzing And Property Tests

Fuzz JSON, SSE chunks, patch syntax, path normalization, config parsing, cursor decoding, and event replay. Properties include no path escape, monotonic event sequence, atomic failed edits, and idempotent permission settlement.

## Observability Tests

Verify spans contain correlation IDs but not secrets or source contents. Test bounded queues and slow subscribers to ensure memory cannot grow without limit.
