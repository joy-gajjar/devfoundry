# Goals And Boundaries

## Mission

Create a fast, auditable, local-first coding agent that helps a developer understand, modify, and validate a codebase from a terminal. Rust is chosen for a portable binary, explicit concurrency, predictable resource use, and a narrow deployable footprint.

## Primary Users

- Individual developers working in local repositories.
- Teams that need a terminal agent with inspectable behavior.
- Automation systems that need a local HTTP interface.
- Rust developers who want a dependable foundation for extensions.

## Success Outcomes

- A user can resume a session after process restart.
- A user always knows what the agent is doing and why permission is needed.
- A coding task can inspect files, edit them, run validation, and report results.
- Tool activity is bounded, cancellable, auditable, and replayable.
- Providers can be replaced without rewriting the runner.

## Explicit Non-Goals For MVP

- Autonomous unrestricted computer control.
- Cloud-hosted multi-tenant service.
- Guaranteed exact reproduction of all upstream OpenCode routes.
- Desktop GUI, browser app, mobile app, or editor plugin.
- Arbitrary native plugin execution.
- Training or hosting foundation models.
- Silent telemetry or collection of source code.

## Product Constraints

- Local-first and useful with a single binary.
- Safe defaults even when the model is malicious or confused.
- Human approval for consequential operations.
- No provider token in logs, events, prompts, or crash reports.
- Graceful degradation when the provider, network, or tool fails.

## Definition Of Done For MVP

The MVP is complete when a clean installation can open a project, create a session, stream an answer, request and receive permission, modify a file, run a test, survive cancellation, resume after restart, and expose the same facts through the local API.

## Scope Change Rule

A feature may enter MVP only if it improves the primary workflow, has a clear owner crate, has a failure model, and does not weaken the permission boundary. Otherwise it goes into the post-MVP backlog.
