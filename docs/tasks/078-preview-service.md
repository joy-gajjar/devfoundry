# Task: W21 Preview Service

## Goal
Provide an explicitly approved, bounded preview process for an owned worktree and expose only value-free lifecycle metadata to the authenticated workspace browser.

## Scope
- Included: preview lifecycle state, broker-gated process start, canonical worktree/artifact containment, sanitized process environment, loopback preview URL metadata, stop/reap, status and annotation routes, isolated browser preview panel, and regression tests.
- Excluded: project-root static serving, public tunnels, arbitrary URL proxying, privileged preview API access, credentials, scheduler/worker changes, resource installation, and TUI changes.

## Design
- States are `Requested -> Starting -> Ready | Failed -> Stopped`; stopped previews are terminal.
- `preview_start` is authorized through the central `PermissionBroker` before process creation. Preview code has no approval path of its own.
- Worktree and artifact roots are canonicalized and must be existing directories; the artifact root must be contained by the owned worktree. Invalid, missing, traversal, or ambiguous paths fail closed.
- Processes run with an explicit executable/argv, cleared environment, project working directory, bounded `PORT`, and Unix process-group cleanup. Stop is idempotent and waits for reaping.
- Preview URLs are accepted only as loopback HTTP URLs. No host credentials, browser bootstrap values, or privileged cookies are sent to the preview surface. Annotations require the current revision and are size bounded.
- Screenshots currently return a managed-unavailable response rather than exposing arbitrary filesystem or process output; capture/storage remains a follow-up interface.

## Implementation
- `crates/tools/src/preview.rs`: bounded process and path boundary.
- `crates/core/src/previews.rs`: lifecycle and in-memory ownership registry.
- `crates/server/src/routes_previews.rs`: start/status/stop/screenshot/annotation routes.
- `crates/server/src/lib.rs`: narrow state and route registration.
- `apps/workspace/src/preview/PreviewPanel.tsx`: isolated preview presentation.
- `apps/workspace/src/App.tsx`, `apps/workspace/src/styles.css`: preview-only browser surface integration.
- `crates/tools/tests/preview_security.rs`, `crates/core/tests/previews.rs`, `crates/server/tests/preview_security.rs`, `apps/workspace/tests/preview-panel.test.tsx`, `apps/workspace/tests/e2e/preview.spec.ts`: regression coverage.

## Verification
- `cargo test -p devfoundry-tools --test preview_security`: 3 passed.
- `cargo test -p devfoundry-core --test previews`: 2 passed.
- `cargo check --workspace --all-targets`: passed.
- `npm test -- --run tests/preview-panel.test.tsx tests/app.test.tsx`: 2 files, 2 tests passed.
- `npm run typecheck`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed; all workspace suites green, including 2 core preview, 3 tools preview, and 2 server preview tests.
- `npm run typecheck`: passed.
- `npm test`: 3 files, 7 tests passed.
- `npm run build`: passed.
- `npm run test:e2e`: 6 tests passed across Chromium and mobile-Chrome projects.
- `git diff --check`: passed.

## Risks And Follow-Up
- Preview registry is currently process-local and not durable; restart recovery is not implemented in W21.
- Readiness probing and port reservation are represented by bounded inputs but require a follow-up adapter for real preview protocols.
- Screenshot capture is intentionally unavailable until a retention-safe managed artifact store is connected.
- The preview process remains a trusted local process boundary, not an OS sandbox; it can exfiltrate data available to its environment.
- Linux/Windows descendant cleanup requires native CI evidence and is not claimed from macOS validation.
- No commit was created; changes remain local on `feat/granular-workspace`.
