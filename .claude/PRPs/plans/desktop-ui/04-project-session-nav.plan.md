# Project and Session Navigation Plan

**Goal:** Replace fixture-only project/session navigation with authoritative API-backed state.
**Architecture:** Extend `createWorkspaceClient` and React state around existing v1 project/session routes and v2 snapshot reconciliation.
**Spec:** `00-overview.plan.md`.

## Global Constraints

- Use authoritative API snapshots and preserve durable state on reconnect.
- Do not erase current UI state on recoverable API errors.

## Contracts

- `GET/POST /api/v1/projects`.
- `GET/POST /api/v1/projects/{project_id}/sessions`.
- `GET/PATCH /api/v1/sessions/{session_id}`.
- `GET /api/v2/sessions/{session_id}/snapshot`.

## Tasks

1. Add typed project/session response models and client methods.
2. Replace fixture project/session selection with loading/empty/error states.
3. Preserve active selection during reconnect and restore from snapshot.
4. Add create/select/rename session flows using existing PATCH semantics.
5. Add tests for unknown project/session, network retry, durable restore, and failed update preservation.

## Acceptance

- Reopening the desktop restores the last durable project/session.
- API errors do not erase current state.
- All mutations use existing authenticated server boundaries.
