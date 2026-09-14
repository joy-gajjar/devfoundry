# DevFoundry Tauri Desktop Implementation Plan

> **For agentic workers:** Use `superpowers:subagent-driven-development` for gated task execution. Use `dispatching-parallel-agents` only for the explicitly independent work units in `01-multiagent-framework.plan.md`.

**Goal:** Build a Tauri desktop workspace over DevFoundry's existing Axum HTTP/SSE backend, exposing sessions, chat, agents, workers, tasks, documents, terminals, previews, resources, permissions, credentials, notifications, and diagnostics.

**Architecture:** Tauri is a thin Rust shell. It starts the existing DevFoundry server on loopback, loads the React/Vite workspace in a WebView, and exposes only narrow lifecycle/open-external/window-state commands. The renderer continues using the existing `/api/v1` and `/api/v2` contracts; Rust remains authoritative for storage, permissions, secrets, workers, worktrees, previews, and tools.

**Tech Stack:** Rust 1.88, Tauri, Axum, SQLx/SQLite, Ratatui, React/Vite, TypeScript, Vitest, Playwright.

**Spec:** User-approved Tauri desktop plan; current architecture `docs/architecture.md`; current UI `apps/workspace/src/App.tsx` and `apps/workspace/src/styles.css`.

## Global Constraints

- Tauri Option A: thin shell over the existing loopback HTTP/SSE server.
- Packaging priority: macOS Apple Silicon, then Windows, then Linux.
- GitHub Copilot remains the production provider boundary.
- Renderer has no arbitrary Node, filesystem, shell, or secret access.
- Tauri capabilities and CSP are strict and allowlisted.
- Worker evidence is not acceptance; no automatic merge or conflict resolution in this plan.
- Existing TUI remains functional and independently tested.
- No production-readiness claim without platform and release evidence.
- Maximum four implementation agents concurrently plus independent reviewers.

## Phase Graph

```text
01 multi-agent framework
        |
02 Tauri shell --> 03 design system --> 04 navigation --> 05 chat
                                      |                    |
                                      +--> 06 agent settings
                                      +--> 07 Boss/Workers
                                      +--> 08 tasks/docs/brain
                                      +--> 09 terminal/preview/resources
                                      +--> 10 credentials/notifications/diagnostics
                                                        |
                                                        v
                                           11 packaging/release
```

## Shared Contracts

- Backend routes: `crates/server/src/lib.rs:187-312`.
- Browser client: `apps/workspace/src/api/client.ts`.
- Browser types: `apps/workspace/src/api/types.ts`.
- TUI reducer/rendering: `crates/tui/src/lib.rs`.
- Configuration/profile validation: `crates/devfoundry/src/config.rs`.
- Worker contracts: `crates/core/src/workers.rs`, `crates/server/src/routes_workers.rs`.

## Shared Validation

```bash
/Users/joy/.cargo/bin/cargo fmt --all -- --check
/Users/joy/.cargo/bin/cargo check --workspace --all-targets
/Users/joy/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
/Users/joy/.cargo/bin/cargo test --workspace
bash scripts/validate-secretless.sh
git diff --check
```

```bash
cd apps/workspace
npm ci
npm run typecheck
npm test
npm run build
npx playwright test
```

## Non-Goals

- Automatic Boss decomposition, autonomous overnight execution, automatic worker merge, automatic conflict resolution, cloud sync, multi-user collaboration, mobile clients, unrestricted plugin execution, and secret values in the WebView.

## Execution

Implement phases in dependency order. Each phase has its own plan document,
tests, review gate, and commit. Do not execute all phases as one batch.
