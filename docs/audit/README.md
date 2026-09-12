# DevFoundry Project Audit

This directory is the authoritative reference for known issues, incomplete features, and the feature difference between DevFoundry and upstream OpenCode. It is intended to be read before planning future work.

## Snapshot

- Audit date: 2026-09-06
- Workspace: `/Users/joy/Projects/OpenCode-Rust`
- Crates audited: `schema, protocol, storage, tools, llm, core, server, tui, devfoundry`
- Upstream reference: `anomalyco/opencode` `dev` branch (Bun/TypeScript monorepo, ~30 packages)
- Method: read-only source review, doc cross-reference, and upstream repository inspection.

## Documents

| File | Contents |
|---|---|
| [01-critical-bugs.md](01-critical-bugs.md) | High-severity correctness and security issues |
| [02-medium-risks.md](02-medium-risks.md) | Medium-severity reliability and correctness risks |
| [03-low-and-cleanup.md](03-low-and-cleanup.md) | Low-severity issues, dead code, stale artifacts |
| [04-tui-gaps.md](04-tui-gaps.md) | TUI feature gaps and fragilities vs the TUI roadmap |
| [05-docs-and-naming.md](05-docs-and-naming.md) | Stale documentation and naming inconsistencies |
| [06-test-coverage.md](06-test-coverage.md) | Per-crate test coverage matrix and missing tests |
| [07-remediation-roadmap.md](07-remediation-roadmap.md) | Prioritized fix plan with dependencies and gates |
| [08-upstream-feature-diff.md](08-upstream-feature-diff.md) | Verified feature difference vs upstream OpenCode |

## Severity Legend

| Severity | Meaning |
|---|---|
| High | Correctness or security defect; can cause wrong results, unsafe behavior, or data loss |
| Medium | Reliability, consistency, or edge-case defect; degraded behavior under load or failure |
| Low | Cosmetic, cleanup, dead code, or minor inconsistency |
| Gap | Feature not implemented; not necessarily a defect |

## Status Legend (feature tables)

| Status | Meaning |
|---|---|
| Implemented | Present and behaves as intended |
| Partial | Present but incomplete against its acceptance criteria |
| Stub | Placeholder or skeleton only |
| Missing | Not implemented |
| Non-goal | Intentionally excluded by product decision |

## How To Use

1. Before starting a task, check the relevant audit file for known issues in that area.
2. When a finding is fixed, update the finding's status and reference the fixing task record under `docs/tasks/`.
3. Keep this audit synchronized with `docs/PENDING_WORK.md`, `docs/TUI_FEATURE_ROADMAP.md`, and `docs/OPENCODE_COMPARISON.md`.

## Related Documents

- [../PENDING_WORK.md](../PENDING_WORK.md)
- [../TUI_FEATURE_ROADMAP.md](../TUI_FEATURE_ROADMAP.md)
- [../OPENCODE_COMPARISON.md](../OPENCODE_COMPARISON.md)
- [../tasks/061-project-audit-and-upstream-diff.md](../tasks/061-project-audit-and-upstream-diff.md)
