# Task: Phase 5 Session Discovery

## Goal

Allow clients to discover projects and existing sessions through the public API so the future TUI can resume work without storage access.

## Scope

- Included: project-scoped session listing and session lookup.
- Excluded: session mutation/deletion, message pagination, authentication, and TUI.

## Design

Session listing is scoped by a validated project ID and ordered by most recently updated. Session lookup uses an opaque session ID and returns a typed not-found error. The server continues to use repository methods rather than querying SQLite directly.

## Implementation

- Added `GET /api/v1/projects/{project_id}/sessions`.
- Added `GET /api/v1/sessions/{session_id}`.
- Added storage-backed session listing.

## Verification

Blocked locally because the Rust toolchain is unavailable. `git diff --check` passes. Cargo formatting, compilation, Clippy, tests, and HTTP contract tests remain pending.

## Risks And Follow-Up

- Domain records are still exposed directly as temporary wire DTOs.
- Message history pagination is needed for a practical TUI.
- Session update, rename, delete, and fork operations remain future API work.
