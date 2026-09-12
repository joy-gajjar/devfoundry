# Rust OpenCode Preplanning

This directory is the design baseline for a Rust implementation inspired by the public behavior of OpenCode. It is not a promise of exact internal or API compatibility with upstream.

## How To Use This Set

1. Read `00-goals-and-boundaries.md` and `01-product-and-ux.md` before adding features.
2. Read `02-system-architecture.md` and `03-domain-model.md` before creating crates or tables.
3. Read the component design document before implementing that component.
4. Record changed decisions in `15-decision-log.md`.
5. Do not mark a phase complete until its gate in `14-roadmap-and-gates.md` passes.

## Documents

| File | Area | Primary question |
|---|---|---|
| `00-goals-and-boundaries.md` | Vision and scope | What are we building and refusing to build? |
| `01-product-and-ux.md` | Product and UX | What should users experience? |
| `02-system-architecture.md` | Architecture | How do the processes and crates fit together? |
| `03-domain-model.md` | Domain | What facts and invariants exist? |
| `04-storage-and-recovery.md` | Persistence | How are facts stored and recovered? |
| `05-llm-and-providers.md` | Models | How are provider differences isolated? |
| `06-session-runner.md` | Execution | How does a prompt become safe work? |
| `07-tools-and-execution.md` | Tools | How does the agent interact with the machine? |
| `08-permissions-and-security.md` | Security | How do we fail closed? |
| `09-protocol-and-api.md` | API | How do clients control and observe the system? |
| `10-tui-and-cli.md` | Terminal UX | How does the terminal client behave? |
| `11-configuration-and-context.md` | Configuration | How are settings and instructions resolved? |
| `12-plugins-and-integrations.md` | Ecosystem | How do MCP, LSP, and extensions fit safely? |
| `13-testing-and-quality.md` | Quality | What evidence is required for correctness? |
| `14-roadmap-and-gates.md` | Delivery | What is built in what order? |
| `15-decision-log.md` | Decisions | What choices are binding and why? |
| `16-distribution-and-operations.md` | Operations | How is it packaged, upgraded, diagnosed, and supported? |
| `17-development-agents.md` | Development agents | Which specialized agents build, document, and review each component? |
| `18-documentation-workflow.md` | Documentation | What must be documented for every development step? |

## Non-Negotiable Principles

- Persist facts before publishing durable events.
- Treat model output, tool arguments, plugins, and project files as untrusted input.
- Keep permissions outside individual tools so no tool can silently bypass policy.
- Keep the core runner independent of the TUI and HTTP transport.
- Prefer a small stable MVP over premature compatibility with every upstream feature.
- Every concurrency decision must include cancellation, ordering, and recovery behavior.
- Every new capability needs a threat model and a test gate.
- Every development step must document its goal, design, implementation, verification, risks, and follow-up work.
