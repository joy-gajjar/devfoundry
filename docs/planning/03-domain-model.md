# Domain Model

## Identity Types

Use newtype IDs, not bare strings, for `ProjectId`, `SessionId`, `MessageId`, `PartId`, `ToolCallId`, `PermissionRequestId`, and `EventSequence`. IDs are opaque at API boundaries.

## Aggregate Roots

### Project

Owns root path, display metadata, VCS information, configuration location, and known sessions.

### Session

Owns title, selected agent/model, lifecycle status, message sequence, prompt queue, and execution metadata.

### Message

Owns role, parts, timestamps, provider provenance, and durable state. Parts include text, reasoning, tool call, tool result, error, and file reference.

## Status State Machines

Session: `idle → running → idle`, with `waiting_permission`, `waiting_question`, `cancelling`, and `error` transitions. Terminal states must be explicit; do not infer status from missing rows.

Permission: `pending → approved | denied | expired | cancelled`.

Tool call: `requested → authorized → running → succeeded | failed | cancelled`.

## Invariants

- A message belongs to exactly one session.
- A tool result references one tool call.
- Event sequence is monotonic per instance/session stream.
- A permission response can settle a request only once.
- A session cannot have two active foreground runners.
- IDs and timestamps are generated server-side.
- Raw provider payloads are optional diagnostics, never the domain source of truth.

## Context Epoch

Context assembly should distinguish durable conversational history from the current provider system context. Compaction starts a new epoch; old audit messages remain durable but may not be included in the next provider request.

## Error Taxonomy

Use stable tagged errors: validation, not found, conflict, permission denied, cancellation, provider, tool, storage, protocol, and internal. Error text is for humans; tags and fields are for clients.

## Versioning

Every public enum and persisted record must have an evolution strategy. Unknown event kinds should be safely ignored by clients that do not understand them.
