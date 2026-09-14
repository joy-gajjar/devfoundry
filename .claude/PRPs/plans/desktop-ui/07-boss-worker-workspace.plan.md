# Boss and Worker Workspace Plan

**Goal:** Make explicit worker assignments, leases, evidence, failures, and review state visible in the desktop UI.
**Architecture:** Add a bounded durable worker projection endpoint with pagination, then bind the dashboard to existing assignment/cancel/evidence contracts. Never render worker evidence as acceptance.
**Spec:** `00-overview.plan.md`; current workers `crates/server/src/routes_workers.rs`, `apps/workspace/src/api/client.ts:65-66`.

## Global Constraints

- Worker evidence is not acceptance.
- No automatic decomposition, merge, or conflict resolution in this phase.

## New Backend Contract

- Keep `GET /api/v2/workspaces/{workspace_id}/workers` as the route.
- Return bounded rows with `id`, `task_id`, `status`, `revision`, `owner`, `attempt_id`, `worktree_label`, `evidence_count`, `failure_code`, and `updated_at`.
- Add cursor/limit bounds and never return raw private paths or secret data.

## Tasks

1. Write storage/projection tests for empty, running, review, failed, unknown, and recovered workers.
2. Implement route/repository projection using existing scheduler/attempt/evidence tables.
3. Add renderer types/client query and loading/error/reconnect states.
4. Add worker detail panel and evidence navigation.
5. Add tests ensuring labels say review/evidence rather than accepted/merged.

## Not Building

- Automatic prompt decomposition, spawn orchestration, merge, or conflict resolution.

## Acceptance

- Rows survive reconnect.
- Pagination and output bounds are enforced.
- Unknown side effects are visible and never silently retried.
