# Tauri Shell Plan

> **For agentic workers:** Use `subagent-driven-development`; shell changes require an independent security review.

**Goal:** Create a secure Tauri shell that supervises the existing DevFoundry server and loads the React workspace.
**Architecture:** The Tauri Rust process starts the existing binary/server on loopback, waits for `/health`, passes only the validated base URL to the WebView, and shuts the child down on app exit. No desktop command replaces HTTP/SSE.
**Tech Stack:** Tauri, Rust, React/Vite, existing Axum health route.
**Spec:** `00-overview.plan.md`.

## Global Constraints

- Tauri is a thin shell over existing HTTP/SSE; renderer has no arbitrary native access.
- Backend remains authoritative; no secrets cross the WebView boundary.

## Mandatory Reading

- `crates/server/src/lib.rs:180-320` for router and health route.
- `crates/devfoundry/src/main.rs:210-222` for server startup.
- `apps/workspace/src/api/client.ts` for base URL use.
- Tauri official security guide: `https://v2.tauri.app/security/`.

## Files

- Create `apps/desktop/src-tauri/Cargo.toml`, `src/main.rs`, `tauri.conf.json`, `capabilities/default.json`.
- Create `apps/desktop/package.json`, `src/main.tsx`, `vite.config.ts`, `tsconfig.json`.
- Create `apps/desktop/tests/shell.test.ts` and Playwright shell smoke tests.
- Modify workspace manifests only through Integrator.

## Tasks

1. Add a failing backend-lifecycle test for single start, health timeout, and graceful shutdown.
2. Add minimal Tauri app and process supervisor using validated argv, bounded startup timeout, and child cleanup.
3. Add narrow preload/bridge only if required by Tauri; expose health/base-url/open-external, never arbitrary shell or filesystem.
4. Add strict CSP and capability allowlist; deny untrusted navigation and new windows.
5. Add WebView boot screen for starting/ready/error states.
6. Run typecheck, Tauri build, shell tests, Rust checks, and security review.

## Acceptance

- One backend child per app instance.
- Health timeout is bounded and user-visible.
- Closing the app cleans up the backend.
- Renderer cannot access Node, shell, arbitrary filesystem, or secrets.
