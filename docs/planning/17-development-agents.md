# Development Agents

## Purpose

These project agents provide focused development roles for the DevFoundry implementation. They are stored under `.opencode/agent/` because that is the project agent discovery location.

All agents must follow `docs/planning/18-documentation-workflow.md`. Documentation is required for every step, including investigations that produce no code changes.

## Roster

| Agent | Role | Main planning documents | Edit policy |
|---|---|---|---|
| `rust-lead` | Cross-component coordination and decisions | All | Can edit |
| `architecture` | Crate boundaries and runtime design | 02, 03, 14, 15 | Can edit |
| `rust-foundation` | Workspace, schema, config, CI baseline | 00, 02, 03, 14 | Can edit |
| `storage-recovery` | SQLite, repositories, recovery | 03, 04, 14 | Can edit |
| `llm-provider` | Provider abstraction and adapters | 05, 06, 13 | Can edit |
| `tools-security` | Tools, process execution, permissions boundary | 07, 08, 13 | Can edit |
| `session-runner` | Agent loop and session lifecycle | 03, 04, 05, 06, 08 | Can edit |
| `protocol-api` | HTTP, JSON, SSE, embedded transport | 02, 09, 14 | Can edit |
| `tui-cli` | CLI and Ratatui client | 01, 09, 10, 13 | Can edit |
| `test-engineer` | Tests, fixtures, fuzzing, gates | 13, 14 | Can edit |
| `release-ops` | CI, packaging, diagnostics, operations | 08, 13, 14, 16 | Can edit |
| `documentation` | Task records, plans, decisions, verification evidence | 14, 15, 18, relevant component plan | Can edit |
| `reviewer` | Read-only correctness and security review | Relevant component plan | Read-only |

## Delegation Rules

1. Use `rust-lead` when work crosses two or more ownership areas.
2. Use `architecture` before introducing a new crate, process, persistence model, or concurrency mechanism.
3. Use `tools-security` for any filesystem, command, network, plugin, or permission change.
4. Use `test-engineer` alongside changes to storage, runner, tools, providers, protocol, or cancellation.
5. Use `reviewer` before closing a phase gate or declaring security-sensitive work complete.
6. Use `release-ops` before changing install, migration, logging, diagnostics, or CI behavior.
7. Use `documentation` for task records, plan updates, decision records, and documentation audits.
8. Agents must inspect existing work and preserve unrelated changes.
9. Agents must update planning documents when implementation invalidates a design decision.
10. Agents must document the goal, scope, design, implementation, verification, risks, and follow-up work for every task.
11. Agents must not treat chat history as the project record.

## Standard Handoff

Every agent handoff should include:

- Files changed.
- Contracts added or changed.
- Tests run and their results.
- Known risks and deferred work.
- Decision-log entry required, if any.
- Documentation files or sections updated.

## Recommended First Sequence

1. `architecture`: confirm workspace and dependency graph.
2. `rust-foundation`: create Cargo workspace and schema foundation.
3. `storage-recovery` and `llm-provider`: work in parallel after schema contracts exist.
4. `tools-security`: implement the permission and first tools.
5. `test-engineer`: establish fake provider, fixtures, and phase-gate tests.
6. `session-runner`: connect durable storage, provider, tools, and events.
7. `protocol-api`: expose the runner through the local API.
8. `tui-cli`: build the terminal client against the public API.
9. `reviewer`: audit the complete MVP path.
10. `documentation`: audit the complete project record and phase-gate evidence.
11. `release-ops`: validate packaging and operational readiness.

## Agent Quality Bar

An agent is not finished when code compiles. It is finished when its contract is documented, the task record is complete, failure behavior is tested, security boundaries are explicit, and the relevant roadmap gate can be evaluated.
