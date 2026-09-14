# Chat and Transcript Plan

**Goal:** Deliver the primary chat workflow with durable replay, streaming, tool cards, and permissions.
**Architecture:** Use snapshot/history/event APIs and preserve the existing `WorkspaceError` and SSE reconciliation model.
**Spec:** `00-overview.plan.md`.

## Global Constraints

- Prompt admission, replay, permissions, and secrets remain host-controlled.
- Tool/provider output must remain bounded and value-free where sensitive.

## Contracts

- `GET /api/v2/sessions/{id}/snapshot`.
- `GET /api/v2/sessions/{id}/messages`.
- `POST /api/v2/sessions/{id}/prompt`.
- `GET /api/v2/sessions/{id}/events`.
- `GET /api/v1/sessions/{id}/permissions`.
- `POST /api/v1/permissions/{id}/resolve`.

## Tasks

1. Add typed composer state and admission/rejection behavior.
2. Render assistant/tool/system messages with bounded output and status cards.
3. Implement SSE reconnect using `reconcileSession` and durable cursors.
4. Add keyboard-accessible permission dialog and explicit risk/scope copy.
5. Add tests for replay gaps, stalled/network failure, bounded text, permissions, cancellation, and secret-free rendering.

## Acceptance

- Durable state is shown before live continuation.
- Reconnect never duplicates messages.
- Provider/tool errors are visible without token or private-path leakage.
