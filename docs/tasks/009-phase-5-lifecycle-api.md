# Task: Phase 5 Project And Session Lifecycle API

## Goal

Allow a clean client to create a project and session through the API before submitting prompts.

## Scope

- Included: project create/list, session create, root validation, default build agent/model, and storage-backed JSON responses.
- Excluded: session update/delete/list, authentication, provider catalog, and TUI.

## Design

Project roots are canonicalized and must be directories. Session creation verifies both path and body project IDs, then applies explicit defaults (`build` and `github-copilot/default`) when omitted. Wire responses use domain records temporarily; dedicated DTO projections can be added when API compatibility stabilizes.

## Implementation

- Added `CreateProjectRequest` and session lifecycle contracts.
- Added `GET/POST /api/v1/projects`.
- Added `POST /api/v1/projects/{project_id}/sessions`.
- Added storage-backed project listing.
- Added direct `chrono` dependency required by timestamped API records.

## Verification

Blocked locally because the Rust toolchain is unavailable. Required workspace format, check, clippy, test, and HTTP integration commands remain pending. `git diff --check` passes.

## Risks And Follow-Up

- Duplicate project roots currently rely on SQLite constraint errors rather than a dedicated conflict response.
- Session list/get/update/delete routes remain to be added.
- API DTOs should stop exposing internal record layout before public compatibility is promised.
