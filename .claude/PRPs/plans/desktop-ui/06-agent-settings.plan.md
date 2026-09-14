# Agent Settings and Profiles Plan

**Goal:** Expose the existing built-in agent profiles and layered configuration in the desktop UI.
**Architecture:** Keep policy validation in Rust; renderer edits only validated profile metadata and uses existing session PATCH behavior.
**Spec:** `00-overview.plan.md`, `.claude/PRPs/plans/completed/configurable-boss-worker-ui.plan.md`.

## Global Constraints

- Built-in profiles only; policy validation stays in Rust.
- No credentials or arbitrary tool policy are stored in renderer config.

## Contracts

- Profiles: `boss`, `build`, `plan`, `review`, `test`.
- Global config: `${XDG_CONFIG_HOME:-$HOME/.config}/devfoundry/config.json`.
- Project config: `<project>/devfoundry.json`.
- Session update: `PATCH /api/v1/sessions/{session_id}`.

## Tasks

1. Add typed config/profile status response without returning secrets.
2. Build Settings surface with profile, model, limits, worker concurrency, and source precedence.
3. Reject custom/unknown profiles in UI and server.
4. Add optimistic update rollback and authoritative refresh.
5. Test precedence, invalid profiles, no secret fields, and keyboard navigation.

## Acceptance

- Project overrides global values.
- `plan`, `review`, and `test` remain read-only policy profiles.
- `boss` displays delegation intent but does not imply automatic decomposition.
