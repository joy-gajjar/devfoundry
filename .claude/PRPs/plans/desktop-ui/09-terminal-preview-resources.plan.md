# Terminal, Preview, and Resources Plan

**Goal:** Expose terminal, preview, and resource capabilities through safe desktop panels.
**Architecture:** Renderer requests go through existing Rust HTTP routes; no Tauri command grants arbitrary shell/filesystem access.
**Spec:** `00-overview.plan.md`.

## Global Constraints

- Renderer cannot spawn processes or bypass Rust permission boundaries.
- Preview/resource mutations remain bounded, transactional, and reviewable.

## Contracts

- Terminal routes under `/api/v2/projects/{project_id}/terminals` and terminal output/input routes.
- Preview routes under `/api/v2/projects/{project_id}/previews`.
- Resource inspect/preview/install/update/remove routes under `/api/v2`.

## Tasks

1. Add terminal drawer with capability, lease, output gap, truncation, input, resize, and cleanup states.
2. Add preview lifecycle panel with start/status/stop/screenshot/annotation states.
3. Add resource library with inspect/install/update/remove and transactional error states.
4. Add tests proving renderer cannot spawn processes, preview roots are bounded, and resource failures do not imply success.

## Acceptance

- Rust permission boundaries remain authoritative.
- Terminal and preview cleanup works on close/reconnect.
- Resource UI never renders installation success before authoritative response.
