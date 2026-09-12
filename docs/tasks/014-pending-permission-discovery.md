# Task: Pending Permission Discovery

## Goal

Allow clients to recover pending approval state through an authoritative query after connection or reconnect.

## Scope

- Included: storage query for pending permissions and session-scoped API endpoint.
- Excluded: permission history pagination, approval scopes, TUI rendering, and remote authentication.

## Design

Live permission events are useful for low-latency UI updates, but a reconnecting client must query durable state. The endpoint returns pending requests ordered by creation time and validates the session before exposing them.

## Implementation

- Added `SqliteStore::list_pending_permissions`.
- Added `GET /api/v1/sessions/{session_id}/permissions`.
- Added session existence validation and typed not-found errors.

## Verification

Blocked locally because Rust tooling is unavailable. `git diff --check` passes; Cargo checks and API integration tests remain pending.

## Risks And Follow-Up

- Pending requests created by a crashed process are cancelled during next server startup, so this endpoint exposes only currently actionable requests.
- The future TUI should query this endpoint after reconnect before waiting for new SSE events.
