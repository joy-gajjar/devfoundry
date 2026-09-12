# DevFoundry

Detailed preplanning is in [`docs/planning/README.md`](docs/planning/README.md). The documents there are the component-level design baseline; update the decision log when implementation changes an architectural choice.

## 1. Objective

Build DevFoundry, a Rust-native coding agent inspired by the public behavior of OpenCode:

- Interactive terminal coding agent.
- Persistent projects, sessions, messages, and events.
- Streaming LLM responses with tool calls.
- Safe filesystem and shell tools with permission checks.
- Plan and build agents.
- Local HTTP API and event stream for clients.
- Extensible provider and tool architecture.

The project should be behaviorally compatible with the useful OpenCode concepts, but it should not copy the upstream TypeScript implementation line-for-line. The Rust implementation should have a smaller, explicit core and use stable protocol boundaries.

The repository is currently empty, so the first milestone is workspace and architecture setup.

## 2. Scope Strategy

### MVP

The first usable release should support:

1. A single-project terminal workflow.
2. SQLite-backed sessions and message history.
3. One GitHub Copilot-compatible provider, including streaming and tool calls.
4. Built-in tools: `read`, `write`, `edit`, `apply_patch`, `glob`, `grep`, and `bash`.
5. Permission prompts for writes and command execution.
6. Build and plan agents.
7. Session cancellation and event streaming.
8. A stable local CLI and HTTP API.

### Post-MVP

- Additional provider adapters are intentionally out of scope; GitHub Copilot is the only supported external provider.
- MCP and LSP integrations.
- Plugin loading and third-party extension ABI.
- Remote authentication and multi-user deployment.
- Web UI and desktop application.
- Full upstream route and SDK compatibility.
- Advanced compaction, worktrees, subagents, and background tasks.

Do not start with the desktop application, all provider adapters, or a plugin ABI. Those features depend on stable session, event, tool, and protocol contracts.

## 3. Proposed Workspace

```text
devfoundry/
├── Cargo.toml
├── crates/
│   ├── schema/       # IDs, domain records, messages, events, errors
│   ├── protocol/     # HTTP DTOs, routes, SSE contracts, validation
│   ├── storage/      # SQLite schema, migrations, repositories
│   ├── llm/          # Provider trait and normalized streaming events
│   ├── tools/        # Tool trait, registry, built-in tools, permissions
│   ├── core/         # Projects, sessions, runner, context, compaction
│   ├── server/       # Axum HTTP/SSE server and request handlers
│   ├── tui/          # Ratatui terminal client
│   └── devfoundry/   # CLI entry point and application composition
├── migrations/
├── tests/
│   ├── fixtures/
│   ├── protocol/
│   └── e2e/
└── docs/
```

Dependency direction:

```text
schema → protocol → storage
schema → llm
schema → tools
schema + protocol + storage + llm + tools → core
core + protocol → server
server + core → tui
all runtime crates → devfoundry
```

`schema` must remain free of database, network, terminal, and provider dependencies. `protocol` must describe the public boundary without importing the session runner or storage implementation.

## 4. Rust Technology Choices

These are defaults, not hard requirements:

- Async runtime: `tokio`.
- HTTP and SSE: `axum`, `tower`, `tokio-stream`.
- HTTP client: `reqwest`.
- Serialization: `serde`, `serde_json`.
- Database: `sqlx` with SQLite and checked migrations.
- CLI parsing: `clap`.
- TUI: `ratatui`, `crossterm`.
- Errors: `thiserror` for library errors, `anyhow` only at application boundaries.
- IDs: UUID or ULID with newtype wrappers in `schema`.
- Logging/tracing: `tracing`, `tracing-subscriber`.
- Markdown and code display: start with plain text; add a renderer only after the TUI state model is stable.
- Process execution: `tokio::process`; PTY support can use `portable-pty` in a later phase.
- Search/globbing: `ignore`, `globset`, and a bounded file walker.

Avoid introducing an effect system or a large dependency-injection framework. Traits, explicit state ownership, cancellation tokens, and typed errors are sufficient for the first implementation.

## 5. Core Domain Model

The initial schema should define these stable concepts:

- `ProjectId`, `SessionId`, `MessageId`, `ToolCallId`, `EventId`.
- `Project`: root path, display name, VCS metadata, timestamps.
- `Session`: project, title, selected agent/model, status, timestamps.
- `Message`: user, assistant, system, tool call, or tool result.
- `Part`: text, reasoning, tool call, tool result, file reference, error.
- `Event`: session status, message delta, tool start, tool result, permission request, question request, error.
- `Agent`: name, mode, system instructions, tool policy, step limit.
- `ModelRef`: provider, model name, capabilities, context limit.
- `PermissionRequest`: operation, path/command, reason, decision.

All persisted records need explicit versioning or migration handling. Never persist provider-specific JSON directly as the only representation of a message; preserve normalized fields and retain raw provider metadata separately when needed.

## 6. Agent Execution Architecture

The session runner is the central component:

```text
prompt
  → load session and project context
  → discover instructions and construct system context
  → select agent and model
  → send normalized messages to LLM
  → stream text/reasoning/tool-call events
  → authorize and execute tools
  → persist tool results and publish events
  → continue until final response, cancellation, error, or step limit
```

Required runner properties:

- One serialized runner per session to prevent conflicting prompts.
- Cancellation must interrupt provider requests and running tools.
- Every visible state transition is persisted before its event is considered durable.
- Tool output is bounded before it is added to model context.
- Permission decisions are attached to the request that caused them.
- Retries must not duplicate durable messages or tool settlements.
- Provider-native continuation metadata is preserved only when the originating provider/model is compatible.

The first runner can use a straightforward loop. Add compaction and queued steering prompts only after the basic loop has deterministic tests.

## 7. Implementation Phases

### Phase 0: Foundation

Deliverables:

- Rust workspace and CI.
- `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` checks.
- `schema` crate with IDs, records, serde formats, and error taxonomy.
- Configuration loader with environment and project config support.
- Basic `devfoundry --version`, `--help`, and project-root detection.

Exit gate: a clean workspace builds on macOS and Linux, and schema round-trip tests pass.

### Phase 1: Persistence and Project Context

Deliverables:

- SQLite migrations for projects, sessions, messages, parts, events, permissions, and tool outputs.
- Repository traits plus SQLite implementations.
- Session CRUD and message history.
- Project root and VCS detection.
- Ordered instruction discovery from global and project `AGENTS.md` files.
- Durable event sequence numbers.

Exit gate: a process restart preserves sessions, messages, and event ordering.

### Phase 2: Tool Runtime and Safety

Deliverables:

- `Tool` trait with typed input, output, cancellation, and permission requirements.
- Registry with deterministic tool names and descriptions.
- `read`, `write`, `edit`, `apply_patch`, `glob`, `grep`, and `bash`.
- Path containment checks relative to the project root.
- Command environment policy and timeout limits.
- Bounded output with managed temporary output files.
- Permission broker and allow/deny/ask policy.

Exit gate: traversal, symlink escape, command timeout, cancellation, oversized output, and denied-operation tests pass.

### Phase 3: LLM Abstraction

Deliverables:

- Provider-neutral request model.
- Normalized stream events for text, reasoning, tool calls, usage, finish, and errors.
- GitHub Copilot-compatible adapter.
- Environment-backed token and base URL configuration.
- Retry/backoff for retryable transport and rate-limit failures.
- Request recording fixtures with secrets removed.

Exit gate: a fake provider and a live-compatible provider can produce identical normalized events for the same fixture.

### Phase 4: Session Runner

Deliverables:

- Prompt admission and serialized session execution.
- Build and plan agents.
- Model selection and agent switching.
- Tool-call execution loop.
- Abort, timeout, step limit, and error recovery.
- Durable message/event publication.
- Initial context assembly from project instructions, agent instructions, and session metadata.

Exit gate: an end-to-end fixture can ask the agent to inspect and modify a test project, approve tools, and resume after restart.

### Phase 5: Local Server and API

Deliverables:

- Axum server with project/session/message/tool/provider routes.
- JSON request/response validation.
- SSE for durable session events and instance live events.
- Permission and question response endpoints.
- Local-only binding by default.
- OpenAPI output or equivalent machine-readable contract.

Exit gate: the CLI and a generated/manual test client can drive the same session without process-local shortcuts.

### Phase 6: TUI and CLI Product Surface

Deliverables:

- Ratatui application with prompt editor, transcript, tool progress, and status footer.
- Permission and question dialogs.
- Session creation/resume/listing.
- Agent/model switching.
- Keyboard interrupt and graceful terminal restoration.
- Non-interactive commands for scripting and CI.

Exit gate: the terminal client survives resize, cancellation, provider failure, and reconnects by refreshing state.

### Phase 7: Advanced Core

Deliverables:

- Context window accounting and compaction.
- Queued prompts and steering semantics.
- Subagents and background tasks.
- Worktree support.
- MCP client and LSP integration.
- PTY-backed terminal tool.

Exit gate: stress tests cover concurrent sessions, compaction, interrupted tools, and reconnect/replay behavior.

### Phase 8: Extensibility and Distribution

Deliverables:

- In-process Rust plugin traits for tools, providers, agents, and event hooks.
- Versioned plugin contract.
- Sandboxed/external plugins only if a real use case requires them.
- Cross-platform release builds and install scripts.
- Homebrew, cargo-binstall, and archive packaging.
- Optional desktop sidecar application.

Exit gate: plugins cannot bypass permission policy or corrupt the session store, and release artifacts are reproducible.

## 8. API Compatibility Plan

Treat upstream behavior as a reference, not as an implementation dependency.

Preserve these concepts first:

- Session lifecycle and prompt submission.
- Normalized message parts.
- Durable session event replay.
- Permission and question workflows.
- Tool names and basic semantics.
- Provider/model selection.

Only promise exact route, payload, error, and SDK compatibility after the Rust protocol is covered by contract tests. At that point:

1. Capture upstream request/response fixtures.
2. Define Rust protocol DTOs from the fixtures.
3. Compare status codes, headers, JSON shape, SSE framing, and error tags.
4. Run the same fixtures against both implementations.

Do not couple the Rust storage schema to upstream internal tables. Compatibility belongs at the protocol boundary.

## 9. Testing Strategy

- Unit tests: schema validation, path policy, patch application, truncation, cursor handling, and provider parsing.
- Repository tests: migrations, transactions, restart behavior, ordering, and idempotency.
- Tool tests: temporary projects with hostile paths, symlinks, large files, and failing commands.
- Provider tests: recorded streaming fixtures, malformed chunks, rate limits, retries, and cancellation.
- Runner tests: fake LLM scripts for final answers, tool loops, denied permissions, aborts, compaction, and recovery.
- Protocol tests: route contract, error encoding, SSE replay, and authorization.
- TUI smoke tests: state transitions independent of terminal rendering, plus a small number of PTY tests.
- End-to-end tests: start server, create project/session, prompt, approve tool, verify file and event history.
- Fuzzing: JSON decoding, patch parsing, path normalization, provider stream framing, and event replay.

Every bug involving persistence, permissions, tool execution, or provider parsing should add a regression test before being fixed.

## 10. Security Requirements

- Never execute a command without an explicit policy decision or configured safe mode.
- Resolve and validate paths after symlink resolution where possible.
- Keep all project-scoped filesystem operations inside an approved root.
- Redact tokens and authorization headers from logs, fixtures, and error messages.
- Apply timeouts and output limits to every external process and network request.
- Bind the local server to loopback by default.
- Require explicit configuration before listening on non-loopback interfaces.
- Treat model output as untrusted input.
- Keep plugin capabilities narrower than core capabilities.
- Make durable event and permission records auditable.

## 11. Performance Targets

Initial targets, to be measured rather than assumed:

- CLI startup under 300 ms without database migration work.
- First visible streamed token forwarded without unnecessary buffering.
- No unbounded memory growth from tool output or event subscribers.
- SQLite writes batched where safe, while preserving durable ordering.
- One session runner must not block unrelated sessions.
- TUI remains responsive while tools and provider requests run.

Use tracing spans and representative fixtures before optimizing. Do not optimize the runner around speculative parallel tool execution; deterministic serialized tool execution is the safer default.

## 12. First Backlog

1. Create the Cargo workspace and crate skeleton.
2. Add CI for formatting, clippy, tests, and release compilation.
3. Implement schema IDs and the first session/message/event records.
4. Add SQLite migrations and repository tests.
5. Implement project-root discovery and config loading.
6. Implement a fake LLM provider and normalized stream event model.
7. Implement `read` and `bash` behind the permission broker.
8. Implement the minimal session runner.
9. Add a non-interactive `prompt` command for end-to-end validation.
10. Add the first local HTTP endpoint and SSE event stream.
11. Add versioned, secret-free session export/import commands.

The first meaningful demo should be:

```text
devfoundry --dir ./demo
> Inspect this project, explain the failing test, and fix it.
```

The agent must read files, ask before changing them, apply the change, run the test, stream progress, and leave a durable session that can be resumed.

## 13. Success Definition

The Rust implementation is ready for a public MVP when a new user can install one binary, open a project, run a multi-step coding task with at least one tool call, approve safe operations, interrupt and resume the session, and inspect the same activity through the local API. Provider, TUI, and persistence behavior must be covered by automated tests, and unsafe filesystem or command operations must fail closed.

## References

- Upstream repository: https://github.com/anomalyco/opencode
- Upstream README: https://github.com/anomalyco/opencode/blob/dev/README.md
- This plan is an independent Rust implementation plan and is not an official upstream roadmap.
