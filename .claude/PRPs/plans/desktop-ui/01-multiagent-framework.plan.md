# Multi-Agent Execution Framework Plan

> **For agentic workers:** Use `superpowers:subagent-driven-development` as the primary execution skill.

**Goal:** Establish safe parallel ownership and merge gates for the Tauri desktop program.
**Architecture:** Freeze shared contracts first, then dispatch disjoint worktrees. An Integrator owns shared manifests, route wiring, migrations, and final reconciliation. Reviewers are independent and workers cannot self-accept.
**Tech Stack:** Git worktrees, Superpowers subagent-driven development, existing Rust/browser CI.
**Spec:** `00-overview.plan.md`, `.claude/plan/granular-workspace.md`, `.claude/PRPs/plans/completed/granular-workspace-completion.plan.md`.

## Global Constraints

- Maximum four implementation agents concurrently plus reviewers.
- Interfaces freeze before dependent work; workers cannot self-accept.
- No secret values, automatic merge, or unreviewed cross-file ownership changes.

## Ownership

| Role | Owns |
|---|---|
| Contract Owner | Frozen Rust/TypeScript/Tauri contracts and compatibility decisions |
| Shell Agent | `apps/desktop/src-tauri/**`, desktop bootstrap |
| Renderer Agent A | `apps/workspace/src/agents/**`, `workers/**`, chat/shared state |
| Renderer Agent B | `tasks/**`, `documents/**`, `settings/**`, resources |
| Rust API Agent | `crates/server/src/routes_*.rs`, `crates/protocol/**` |
| Rust Core/Storage Agent | `crates/core/**`, `crates/storage/**`, migrations |
| Integrator | `Cargo.toml`, workspace manifests, `crates/server/src/lib.rs`, docs, CI |
| Reviewer | Read-only correctness/security/release review |

## Rules

- Maximum four implementation agents concurrently plus reviewers.
- Every agent uses an isolated Git worktree.
- No two agents edit `crates/server/src/lib.rs`, `crates/tui/src/lib.rs`, or `apps/workspace/src/App.tsx` concurrently.
- Freeze interfaces before dependent implementation begins.
- Contract changes return to Contract Owner and invalidate downstream approvals.
- Each task must pass its narrow tests before merge-queue entry.
- Integrator runs the full validation matrix after each merge batch.

## Gates

1. Contract Owner publishes interface list and ownership table.
2. Each worker submits tests, diff summary, and known limitations.
3. Independent reviewer checks security, scope, and regressions.
4. Integrator merges only green work into the feature branch.
5. Final reviewer checks full diff and release evidence.

## Dispatch

- Use `subagent-driven-development` for dependent tasks and review checkpoints.
- Use `dispatching-parallel-agents` only for tasks with disjoint files and frozen interfaces.
- Never launch all phases in parallel.

## Validation

Run shared Rust/browser/secretless gates from `00-overview.plan.md` plus the
phase-specific commands. Record worktree, agent, commit, tests, and reviewer in
the phase report.
