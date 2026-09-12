# System Architecture

## Processes

### Embedded CLI

The CLI may host core and server in one process for low latency. It must still use the same protocol handlers as a network client where practical.

### Local Server

Owns the application runtime, SQLite connection, session registry, event hub, provider clients, and tool policy. Binds loopback by default.

### TUI Client

Consumes the public client contract. It must not call storage or runner internals directly.

## Crate Responsibilities

- `schema`: serializable domain values and stable error/data vocabulary.
- `protocol`: request/response/event contracts and validation.
- `storage`: migrations and transactional repositories.
- `llm`: provider-neutral model and streaming adapters.
- `tools`: capability implementations and tool metadata.
- `core`: orchestration, context, sessions, permissions, and event publication.
- `server`: transport, authentication, routing, and SSE lifecycle.
- `tui`: terminal state and rendering.
- `devfoundry`: composition, CLI parsing, process lifecycle, and distribution entry point.

## Dependency Rules

- No upward crate dependencies.
- No global mutable singleton for session state.
- No provider type in the domain message model.
- No TUI type in core.
- No storage connection exposed to handlers or tools.
- No tool can resolve permissions by itself.

## State Ownership

- Durable truth: SQLite.
- Current execution: session runner registry in core.
- Live notifications: bounded event hub.
- UI state: client-owned projection.
- Provider stream: request-scoped adapter.

## Concurrency Model

- One actor or mutex-guarded queue per session.
- Independent sessions execute concurrently.
- Tool execution is sequential within a model turn initially.
- Event subscribers are bounded; slow clients are disconnected or told to replay.
- Every async operation accepts a cancellation token.

## Data Flow Invariant

```text
command → validate → authorize → execute → persist fact → publish event
```

Never publish a success event before its corresponding durable fact is committed.

## Architecture Alternatives

- Actor-per-session versus shared locks: choose actor-like serialized queues for predictable ordering; use locks only for small repositories.
- Embedded versus separate server: support both through one application host; do not create separate business logic.
- Event sourcing versus CRUD: use normalized CRUD plus append-only durable events; pure event sourcing adds migration and replay cost too early.
- Rust-only UI versus web UI: start with TUI; a web/desktop client can consume the same API later.
