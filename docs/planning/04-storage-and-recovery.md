# Storage And Recovery

## Storage Choice

SQLite is the MVP source of durable truth. It provides local installation simplicity, transactions, indexes, and a path to a future server deployment. Repository traits keep the core independent of SQLx details.

The first migration uses text IDs, UTC timestamps, and JSON message/event payloads behind repository methods. These are storage details and must not leak into protocol DTOs.

## Initial Tables

- `projects`
- `sessions`
- `messages`
- `message_parts`
- `tool_calls`
- `tool_outputs`
- `events`
- `permission_requests`
- `prompt_inbox`
- `provider_attempts`
- `schema_migrations`

## Transaction Boundaries

- Prompt admission: insert prompt and admission event atomically.
- Message completion: persist message/parts and completion event atomically.
- Tool settlement: persist bounded result and tool status atomically.
- Permission response: update request and emit response event atomically.
- Event sequence assignment occurs inside the same transaction as the fact.

## Recovery

On startup:

1. Apply migrations.
2. Mark stale `running` executions as interrupted or recoverable.
3. Leave admitted prompts durable.
4. Rebuild active session indexes from storage.
5. Do not automatically rerun non-idempotent tools.
6. Offer explicit resume with a new provider attempt.

Startup recovery marks `admitted` and `running` prompt rows as `interrupted`, and `pending` permission rows as `cancelled`, atomically. SQLite foreign keys are enabled for every pooled connection. It does not automatically rerun work.

## Idempotency

Provider attempts and tool calls require stable attempt IDs. Retrying a transport request must not duplicate a durable assistant message. A tool call may be replayed only when its operation is explicitly classified idempotent.

## Event Retention

Keep durable session events long enough for reconnect/replay. Add bounded cleanup only with a documented policy. Live-only events may be dropped on disconnect and must never be treated as authoritative.

The initial event cursor wraps a monotonic SQLite sequence. Clients treat it as opaque.

The process-local live session channel is bounded and publishes only after the durable event transaction commits. A lagged subscriber is closed rather than silently claiming continuity; clients reconnect with the last durable sequence.

## Migration Rules

- Every migration is forward-only and tested from a clean database.
- Backups are created before destructive migrations.
- Do not rename a field without a compatibility read path or migration.
- Test interrupted migration behavior where feasible.
- Enable and verify SQLite foreign keys on every pooled connection.
- Keep diagnostics bounded and redacted; do not export prompt, message, tool, or secret contents.

## Recovery Acceptance Tests

- Kill the process during provider streaming, tool execution, and database commit.
- Restart and verify no impossible state is visible.
- Reconnect with an event cursor and receive all durable events after it.
- Resume a prompt without duplicating prior durable output.
