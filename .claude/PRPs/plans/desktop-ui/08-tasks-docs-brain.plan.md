# Tasks, Documents, and Project Brain Plan

**Goal:** Make project tasks, Markdown documents, instruction provenance, and roadmap context first-class desktop surfaces.
**Architecture:** Reuse existing v2 task/document/roadmap contracts and preserve revision conflicts and project containment.
**Spec:** `00-overview.plan.md`.

## Global Constraints

- Revision and project-containment checks remain authoritative in Rust.
- `AGENTS.md` is context, not permission policy.

## Contracts

- `GET /api/v2/workspaces/{id}/tasks`.
- `GET/PATCH /api/v2/tasks/{id}`.
- `GET /api/v2/projects/{id}/documents`.
- Roadmap import/export routes.

## Tasks

1. Add typed task/document/roadmap client methods and loading states.
2. Build task board with status, revision, dependencies, owner, and conflict state.
3. Build document list/search/backlinks and bounded content preview.
4. Display `AGENTS.md` provenance as context, never policy authority.
5. Test stale revision conflicts, project containment, long paths, empty states, and reconnect.

## Acceptance

- Mutations preserve revision checks.
- Documents never escape the project boundary.
- The UI distinguishes instruction context from trusted security policy.
