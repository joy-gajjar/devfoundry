# Documentation Workflow

Documentation is part of the DevFoundry implementation definition of done. Every development step must leave enough written context for another contributor to understand what happened, why it happened, and how it was verified.

## Required Record For Every Task

Before implementation:

- State the goal and user-visible outcome.
- Identify the planning documents and phase gate that apply.
- Record assumptions, constraints, and open questions.
- Identify the affected crate, component, and ownership agent.

During implementation:

- Keep public contracts, invariants, and behavior documented near the relevant component plan.
- Record architectural changes in `15-decision-log.md`.
- Document intentional scope cuts and deferred work.
- Add comments only where code behavior is non-obvious; put design rationale in Markdown.

After implementation:

- Record files changed and the behavior added.
- Record tests, commands, and verification results.
- Record known limitations, risks, and follow-up tasks.
- Update the relevant roadmap gate status or explain why it remains open.
- Update API, schema, security, or operations documentation when those contracts change.

## Documentation Locations

- Product and scope: `00-goals-and-boundaries.md` and `01-product-and-ux.md`.
- Architecture and ownership: `02-system-architecture.md`.
- Domain and persistence: `03-domain-model.md` and `04-storage-and-recovery.md`.
- Component behavior: the matching component document.
- Decisions: `15-decision-log.md`.
- Delivery status: `14-roadmap-and-gates.md`.
- Agent handoffs: `17-development-agents.md` or the task record.
- Release and operational behavior: `16-distribution-and-operations.md`.

## Standard Task Record

Use this structure in the task description, pull request, or a relevant Markdown record:

```markdown
# Task: <short name>

## Goal
<user-visible outcome>

## Scope
- <included work>
- <explicitly excluded work>

## Design
<contracts, invariants, and important decisions>

## Implementation
- <files/components changed>

## Verification
- `<command>`: <result>
- <manual or integration verification>

## Risks And Follow-Up
- <known limitation or next task>
```

## Rules

- Do not consider undocumented work complete.
- Do not rely on chat history as the project record.
- Do not overwrite existing documentation without preserving still-valid rationale.
- Documentation changes must be reviewed with code changes.
- If no code changes are made, document the investigation and conclusion anyway.
- If a decision is reversible, record the experiment and its result; if irreversible, record alternatives and approval.

## Minimum Evidence By Change Type

- Schema change: serialized examples, compatibility impact, round-trip tests.
- Storage change: migration notes, recovery impact, repository tests.
- Tool or permission change: threat model, denial/failure cases, security tests.
- Provider change: wire behavior, normalized event mapping, fixtures.
- Runner change: lifecycle diagram or state transition notes, cancellation/retry tests.
- API change: endpoint/event contract, error behavior, client impact, contract tests.
- TUI change: interaction description, keyboard behavior, resize/error behavior, smoke test.
- Release change: supported platforms, upgrade/rollback impact, command output.
