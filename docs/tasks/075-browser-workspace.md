# Task: W14 Browser Workspace

## Goal

Provide an optional local browser companion for the existing v2 history,
snapshot, task, and document routes without changing Rust core, storage,
schema, protocol, LLM, tools, worktree, or TUI code.

## Scope

- Included: `apps/workspace`, a React + TypeScript + Vite client, typed API
  client, fixture-backed UI and contract tests, responsive navigation shell,
  reconnect/error states, CSP/bootstrap boundary documentation, and this task
  record.
- Included surfaces: chat/session navigation, tasks, documents, status/usage,
  keyboard focus, desktop/mobile layout, reduced motion, and value-free
  credential boundaries.
- Excluded: prompt admission implementation, terminal/preview execution,
  server static serving, token handling, Markdown rendering, uploads, and TUI
  changes. The existing v2 prompt route remains an explicit `501` boundary.

## Design

The client calls the existing routes with a typed fetch wrapper:

- `GET /api/v2/sessions/{id}/snapshot`
- `GET /api/v2/sessions/{id}/messages?limit=100&after=...`
- `GET /api/v2/workspaces/{id}/tasks`
- `GET /api/v2/projects/{id}/documents`

Fixtures mirror the current v2 DTO shapes and keep UI tests independent of a
running Rust host. Network failures become a reconnectable typed error, HTTP
failures become generic retry-aware errors, and malformed JSON is rejected.
The browser sends same-origin credentials but never accepts a token option,
adds an Authorization header, writes browser storage, or displays secrets.

The desktop shell uses project/session navigation, a central surface, and a
status inspector. At narrow widths it switches to explicit Chat/Tasks/Docs
tabs instead of compressing the desktop columns. Focus rings, semantic roles,
textual status labels, and reduced-motion styling are included.

## Integration Seam

No static-serving route was added because the server does not yet own a safe
browser bootstrap/authentication boundary. A future host should serve the
built assets from a controlled origin, emit a response-header CSP, and provide
only non-secret API/project/session configuration through a same-origin
bootstrap channel. Authentication, CSRF/origin validation, and authorization
remain server responsibilities. See `apps/workspace/SECURITY.md`.

## Verification

RED:

- `npm test` initially failed because Vitest was not installed after the
  lockfile-only setup (`sh: vitest: command not found`).
- After `npm install`, the named suites failed at missing implementation
  imports (`../src/api/client` and `../src/App`), with 0 tests executed.

GREEN evidence:

- `npm run typecheck` — passed.
- `npm test` — passed: 2 test files, 4 tests, 0 failures.
- `npm run build` — passed: Vite production bundle generated in `dist/`.
- `npx playwright test` — passed: 4 tests across Chromium and mobile Chromium
  in 2.7 seconds.

The Playwright package and browser executable were available locally. The
workspace package installed with `npm install`; the generated `package-lock`
is included for reproducible `npm ci` setup. `npm install` reported 0 audit
vulnerabilities.

## Residual Risks

- Snapshot/event reconstruction is not live-wired in this UI slice; the API
  client and fixture contract establish the boundary for a later SSE adapter.
- The current server snapshot is documented as potentially non-atomic in W06.
- Authentication/bootstrap integration is intentionally blocked until a host
  owns CSP headers, session authentication, CSRF, and origin policy.
- No commit was made.
