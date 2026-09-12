# Protocol And API

## API Goals

The API must allow a client to create/resume sessions, submit prompts, inspect durable state, answer permissions/questions, cancel work, select agent/model, and replay events.

## Initial Resource Groups

- Health and version.
- Projects/locations.
- Sessions.
- Messages and parts.
- Prompt admission and execution.
- Permissions and questions.
- Providers and models.
- Durable session events.
- Instance live events.

## Transport

Use JSON over HTTP for commands and queries. Use SSE for ordered event streams. WebSockets are deferred until a demonstrated need exists; SSE is easier to proxy, inspect, and reconnect.

The first server slice uses `/health`, `/api/v1/projects`, `/api/v1/projects/{project_id}/sessions`, `/api/v1/sessions/{session_id}`, `/api/v1/sessions/{session_id}/prompt`, `/api/v1/prompts/{prompt_id}`, `/api/v1/sessions/{session_id}/interrupt`, and `/api/v1/sessions/{session_id}/events`. Prompt requests are durably admitted, rejected when a session is already active, and dispatched through a process-local cancellable runner registry. The prompt status query returns the durable inbox state (`admitted`, `running`, `completed`, `failed`, or `interrupted`) and session ID, allowing clients and tests to await execution without timing sleeps.

Permission state uses `GET /api/v1/sessions/{session_id}/permissions` for pending requests. Responses use `POST /api/v1/permissions/{permission_id}/resolve` with `{ "allowed": true|false }` and wake the waiting tool execution when the request is still pending.

## Event Streams

- Durable session stream: supports `after` cursor and replay, then best-effort live fanout. Subscribers register before replay; duplicate sequence numbers are filtered, and lag closes the stream so reconnect can recover from the cursor.
- Instance live stream: best-effort, no replay guarantee; clients refresh state after disconnect.

Never combine these streams under one ambiguous schema.

## Errors

Every error has stable tag, human message, request ID, retryability, and structured fields where safe. Do not expose filesystem internals or cross-session ownership details unnecessarily.

The HTTP transport returns `code`, `message`, `request_id`, `retryable`, and optional `details`. Clients may send `x-request-id`; otherwise the server generates one and echoes it in the response header. Request bodies are limited to 1 MiB, prompts to 256 KiB, and SSE replay responses to 1,000 events. Retryability is true for server and rate-limit failures and false for ordinary validation, conflict, and not-found responses.

Loopback binds are unauthenticated by default. A non-loopback bind requires a bearer token read from the environment variable named by `DEVFOUNDRY_API_TOKEN_ENV` (default `DEVFOUNDRY_API_TOKEN`) and one or more exact, comma-separated origins in `DEVFOUNDRY_ALLOWED_ORIGINS`. The token value is never logged or included in errors. CORS preflight is allowed only for configured origins; actual requests still require the bearer token.

The listener uses graceful shutdown. Shutdown cancels active session executions through the process-local execution registry before the HTTP service exits.

## API Versioning

Start with `/api/v1`. Additive fields are preferred. Breaking changes require a new version or explicit compatibility adapter. Keep wire DTOs separate from domain structs.

## Embedded Client

An embedded client should call the same handlers through an in-memory transport rather than bypassing routing and validation. This prevents local and remote behavior from drifting.

## Contract Tests

The server contract suite covers the current route set's status codes, content types, request IDs, validation/error tags, project/session discovery, prompt terminal statuses, interruption, permission discovery/resolution, SSE framing and cursors, replay/live duplicate filtering, post-commit durable fanout, and API authentication/origin policy. Save representative fixtures and document intentional differences from upstream. Pagination beyond the event cursor, message history, and instance-wide live events remain deferred.
