# Task: W20 Browser Host And Authentication

## Goal

Serve the existing workspace browser from the Rust host only when explicitly
enabled, with a non-secret bootstrap and a controlled browser request boundary.
The browser client must retain a durable session snapshot/event reconnect seam.

## Scope

- Included: opt-in Rust static browser hosting, asset-root containment,
  response CSP, non-secret bootstrap, origin/Host/CSRF checks, browser client
  snapshot reconciliation and unknown-event filtering, server/browser contract
  tests, and this task record.
- Excluded: storage, core, tools, LLM, worktree, PTY, resources, credentials,
  preview services, new providers, and v1/v2 route redesign.

## Design

`BrowserConfig` is disabled by default. `router_with_browser` mounts the
browser routes only when enabled and requires an explicit asset root. Static
requests are canonicalized and accepted only when the resolved file remains
under that root. The SPA root is served from `index.html`; unsafe file paths
return `404`.

The bootstrap endpoint is `GET /api/v2/browser/bootstrap` and returns only a
version and relative API base. It does not return bearer tokens, CSRF values,
filesystem paths, provider credentials, or authorization headers. Existing
`/api/v1/**` and `/api/v2/**` routes remain mounted.

Configured browser origins are checked exactly. Browser state-changing requests
with an Origin require `x-csrf-token`; configured origin authority is also used
for Host validation. API bearer authentication remains server-side and tokens
are not copied into browser configuration or errors. Static HTML receives a
strict response-header CSP.

The browser client retains same-origin credentials and no token option. Its
reconciliation helper refetches a snapshot after replay failure and accepts
only known durable event kinds; unknown event kinds are ignored.

## Endpoint And Event Contract Changes

- Added opt-in `GET /api/v2/browser/bootstrap`.
- Added opt-in `/` and safe static asset serving from the configured asset root.
- Added `BrowserConfig`/`router_with_browser` server composition seam.
- Added browser-side bootstrap, reconciliation, and event-boundary helpers.
- Durable session SSE routes and existing v1/v2 API routes are unchanged.
- Instance-wide live events remain outside W20 and retain best-effort semantics.

## Compatibility Impact

Browser hosting is disabled unless explicitly configured, so existing server
callers and loopback API users are unaffected. Existing browser clients can
continue using the same-origin API routes. The new bootstrap response is
additive and intentionally omits credentials. Browser mutations that include
an Origin now require a CSRF header; non-browser requests without Origin retain
their existing route behavior.

## Implementation

- `crates/server/src/browser.rs`: browser configuration, static serving, CSP,
  and bootstrap DTO/handler.
- `crates/server/src/lib.rs`: opt-in browser router and Host/CSRF boundary.
- `crates/server/tests/browser_security.rs`: static, traversal, CSP,
  bootstrap, origin, Host, and CSRF contracts.
- `apps/workspace/src/api/client.ts`: snapshot reconciliation and event filter.
- `apps/workspace/src/api/types.ts`: browser DTO/event types.
- `apps/workspace/tests/api-client.test.ts`: reconnect/gap and unknown-event
  tests.

## Verification

- `cargo test -p devfoundry-server --test browser_security --test api_lifecycle`:
  passed, 22 tests.
- `cargo fmt --all` followed by `cargo fmt --all -- --check`: passed.
- `cd apps/workspace && npm run typecheck`: passed.
- `cd apps/workspace && npm test`: passed, 2 files / 6 tests.
- `cd apps/workspace && npm run build`: passed.
- Initial RED runs intentionally failed on missing `BrowserConfig`, browser
  router, reconciliation helper, and event filter before implementation.

Final integrated verification after W22/W23 changes landed:

- Workspace check, workspace Clippy with `-D warnings`, workspace tests and
  `git diff --check` all passed.
- Browser typecheck, Vitest, Vite build and Playwright checks remain green.

## Risks And Follow-Up

- CSRF is currently presence-based at the transport boundary; a future
  host-issued session/CSRF token mechanism should bind the value to the browser
  session before remote browser exposure.
- Static serving is opt-in but asset-root configuration uses a panic for an
  invalid enabled configuration; application configuration should validate this
  before router construction.
- Snapshot/event atomicity remains the existing v2 limitation. Reconnect uses a
  fresh snapshot when replay continuity fails.
- No instance-live event API was added; that remains a separate best-effort
  contract.
- Phase 5 Gate 5 remains open until full workspace verification,
  authentication hardening, restart recovery, and instance-wide event work are
  complete.
