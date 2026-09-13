# Implementation Report: Granular-Inspired DevFoundry Workspace

## Summary

Executed the approved plan on local branch `feat/granular-workspace` after creating an authorized baseline commit. The implementation delivered the reliability foundation and the first Granular-inspired workspace surfaces without pretending that deferred orchestration, OS credential integration, or production browser hosting were complete.

The implementation remains Copilot-only for production provider behavior. The browser workspace is optional and fixture/API-contract driven. Existing local databases and build artifacts were preserved and ignored, not deleted.

## Assessment vs Reality

| Metric | Predicted | Actual |
|---|---|---|
| Complexity | 19 gated work packages, roughly 25-40 reviewable changes | 18 implementation checkpoints/commits across foundation, runtime, integrations and UI; remaining native-platform/release evidence is explicitly deferred |
| Confidence | High after each green gate | High for macOS/Rust and browser fixture boundaries; medium for unverified Linux/Windows runtime paths and live Copilot execution |
| Files Changed | Plan estimated a broad Rust + browser campaign | 250+ tracked project files across the baseline and feature commits; generated `target`, databases, node_modules, `dist`, test reports and build-info remain ignored |
| Production scope | TUI + browser, Copilot-only | Delivered foundations plus optional browser shell; production browser host/auth and full worker execution remain follow-up |

## Baseline

- Branch: `feat/granular-workspace`
- Baseline commit: `5213c47 chore: establish DevFoundry baseline`
- Baseline excluded: `.devfoundry.db`, `opencode.db`, SQLite sidecars, `target/`, `.DS_Store`, environment files, local caches and browser dependencies/build output.
- Baseline gates passed before implementation:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace --all-targets`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `bash scripts/validate-secretless.sh`

## Tasks Completed

| Task | Commit | Status | Notes |
|---|---|---|---|
| W00 baseline and audit repair | `5213c47` | Complete | Created branch, ignore rules and authorized local baseline |
| W01 execution contract freeze | `4f54b79`, `42943d8` | Complete | Added IDs, revisions, outcomes, leases, idempotency receipts and event envelope; expanded after W02 contract blocker |
| W02 transactional lifecycle/recovery | `b52c402` | Complete | Migration 0002, durable runs/attempts/tool outputs/leases/events, bounded queries, unknown-effect recovery |
| W03 supervised core/permissions | `a7ab0f1` | Complete | Foreground exclusion, exact permission routing, waiter race handling, durable history, explicit cancellation/turn limits |
| W04 Copilot streaming/provider boundary | `995d94a` | Complete | JSON tool calls/reasoning/usage, strict classification, bounded streaming, cancellation and EOF checks |
| W05 tool/process boundary | `995d94a` | Complete | Path suffix/symlink containment, truthful exit statuses, environment clearing, bounded process cleanup |
| W06 versioned API/history | `92d6c5b` | Complete | v2 history/snapshot/replay routes; v2 idempotent admission deliberately returns 501 until host bridge exists |
| W07 TUI Stage A | `995d94a` | Complete | Draft preservation, permission focus, byte-safe SSE, scroll anchoring and non-lossy UTF-8 |
| W08 guarded worktrees | `e2c1360` | Complete | Fixed Git argv, hostile helper controls, review/integration separation, stale/dirty protections |
| W09 scheduler policy slice | `4db6fac` | Complete as bounded contract | DAG readiness, budgets, receipts, evidence-not-acceptance; not yet durable production orchestration |
| W10 brain/task board | `2e96210` | Complete as foundation | Migration 0003, revisioned tasks, cycle rejection, Markdown backlinks, context manifests and API routes |
| W11 scoped secret vault | `ee83ef2` | Complete as fail-closed boundary | Value-free metadata, fake store, redacted handles; Keychain unavailable until vetted dependency exists |
| W12 MCP/LSP interoperability | `ca9d527` | Complete as local protocol package | MCP newline JSON-RPC, tools/list, LSP framing/lifecycle, independent fixtures |
| W13 native PTY | `df6abcf`, `69e3832` | Complete on Unix/macOS | Native forkpty, input/resize/output/leases/cleanup; fixed blocking reader/reaper issue |
| W14 browser workspace | `e6da597`, `a9709ac` | Complete as optional shell | Typed v2 client, responsive UI, fixtures, Vitest, Playwright, CSP/bootstrap boundary and tracked entrypoint |
| W20 browser host/auth | `4f0a6d7` | Complete as opt-in boundary | Static host, CSP, non-secret bootstrap, Origin/Host/CSRF checks, snapshot/reconnect helpers |
| W21 preview service | `66c15dd` | Complete as bounded local boundary | Worktree/artifact containment, preview lifecycle, stop/reap, isolated browser panel |
| W22 resources | `4f0a6d7` | Partial, fail-closed | Archive security, manifests, storage/core/routes; install/remove projection remains incomplete |
| W23 native credentials | Not enabled | Blocked | Dependency audit tooling and disposable native fixtures unavailable; fail-closed adapter retained |
| W24 terminal/worker projections | `66c15dd` | Complete as process-local boundary | Terminal routes, leases/gaps, TUI/browser projections and worker-board flows |
| W25 notifications | Pending checkpoint | Complete as disabled boundary | Sanitized outbox, pairing/revoke/status routes, browser settings; no live transport |

## Validation Results

### Rust

| Level | Status | Evidence |
|---|---|---|
| Formatting | PASS | `$HOME/.cargo/bin/cargo fmt --all -- --check` |
| Static check | PASS | `$HOME/.cargo/bin/cargo check --workspace --all-targets` |
| Clippy | PASS | `$HOME/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` |
| Unit/integration tests | PASS | `$HOME/.cargo/bin/cargo test --workspace` |
| Secretless/release metadata | PASS | `bash scripts/validate-secretless.sh` |
| Diff check | PASS | `git diff --check` |

Final Rust workspace coverage included:

- `devfoundry`: 12 tests
- `devfoundry-core`: 7 unit tests, 8 session tests, 4 scheduler tests, 3 worker tests, 1 context test
- `devfoundry-llm`: 20 tests
- `devfoundry-protocol`: 6 unit tests, 1 W10 contract test
- `devfoundry-schema`: 3 unit tests, 2 W10 contract tests
- `devfoundry-server`: 19 API lifecycle tests, 5 permission tests, 1 W10 API test
- `devfoundry-storage`: 8 lifecycle tests, 8 execution tests, 2 W10 task tests, 1 W10 document test
- `devfoundry-tools`: 53 unit tests, 5 native integration tests, 7 native MCP/LSP/PTTY integrations, 8 worktree tests, 8 native PTY lifecycle tests
- W19-W25 additions: scheduler recovery, browser security, preview, terminal API, notifications, and resource security suites all passed in the workspace run.
- `devfoundry-tui`: 28 tests
- All Rust doc tests passed.

### Browser

| Check | Status | Evidence |
|---|---|---|
| Typecheck | PASS | `npm run typecheck` |
| Unit/component tests | PASS | `npm test`: 2 files, 4 tests |
| Production build | PASS | `npm run build` |
| Browser E2E | PASS | `npx playwright test`: 4 Chromium/mobile tests |
| Dependency install audit | PASS | `npm install`: 0 audit vulnerabilities reported |

## Deviations From Plan

- The repository began without a valid HEAD and all files untracked. An authorized baseline commit was created before feature execution rather than using unsafe snapshot-only parallel work.
- W02 initially stopped because the first W01 contracts lacked persisted run/attempt/lease/idempotency semantics. W01 was expanded test-first in `42943d8`, then W02 proceeded safely.
- W06 exposes v2 idempotent prompt admission as `501 idempotency_unavailable` rather than implementing an HTTP-local fake. A host admission bridge remains required.
- W09 implements deterministic scheduler policy/contracts but not durable production worker orchestration. This is deliberate and documented.
- W11 does not include a Keychain dependency or plaintext fallback; native credential resolution fails closed until a vetted adapter is approved.
- W14 does not add Rust static serving or authentication bootstrap. It is an optional standalone browser client against existing API contracts.
- W13 required an additional correction after final qualification: the native PTY reader blocked on a shared async mutex and close/reap could block. Descriptor ownership was separated and reaping bounded in `69e3832`.
- External `ccg-workflow` was unavailable; no Codex/Antigravity session IDs were fabricated. Native specialist agents were used instead.

## Issues Encountered

| Issue | Resolution |
|---|---|
| Cargo absent from PATH | Used `/Users/joy/.cargo/bin/cargo` consistently and recorded this in task records |
| W02 contract blocker | Expanded W01 lifecycle vocabulary with tests before writing migrations |
| W10 protocol test indexed a `Result` | Unwrapped `serde_json::to_value` in the test and reran compile/gates |
| W10 Clippy errors | Applied `contains` and `Default` implementations; reran workspace gates |
| W12 LSP native test hung | Found request path sent initialization as a notification without ID; sent an ID-bearing request |
| W13 LSP legacy unit test hung | Added initialization state; skip protocol shutdown for non-LSP compatibility fixture |
| W13 PTY input/close hang | Found reader thread blocking while holding async mutex; cloned descriptors and bounded nonblocking reaping |
| Browser generated artifacts | App-local `.gitignore` excludes `node_modules`, `dist`, test reports and `*.tsbuildinfo`; only `index.html` and source/config are tracked |

## Files/Subsystems Delivered

- Execution lifecycle schema and migration: `crates/schema`, `crates/protocol`, `crates/storage`, `migrations/0002_execution_lifecycle.sql`
- Core runner/permissions: `crates/core/src/lib.rs`, `scheduler.rs`, `workers.rs`, `context.rs`, `tasks.rs`, `secret_bindings.rs`
- Provider: `crates/llm/src/lib.rs`
- Tools/security: `crates/tools/src/lib.rs`, `worktree.rs`, `mcp.rs`, `lsp.rs`, `terminal.rs`
- API: `crates/server/src/routes_v2.rs`, `routes_w10.rs`, `w10.rs`
- TUI: `crates/tui/src/lib.rs`, `client.rs`
- Browser: `apps/workspace/`
- Reference records: `docs/tasks/062` through `075` plus updated plan/audit docs

## Deferred Work

- Storage-backed scheduler leases, attempts, evidence and actual `SessionRunner`/`WorktreeManager` worker execution.
- W19 worker host currently fails closed with `ExecutionUnavailable` until legacy session admission is atomically bridged to durable worker attempts/worktrees.
- Browser static serving, authenticated bootstrap, CSRF/origin host integration and live SSE adapter.
- Browser live preview, terminal panel, resource library UI, annotations and attachment upload.
- Native Windows ConPTY runtime validation and Linux/Windows process cleanup validation.
- Vetted macOS Keychain dependency and end-to-end secret injection.
- `cargo-audit`/`cargo-deny` tooling and approved disposable native credential fixtures are still required before enabling a native credential backend.
- Remote MCP/OAuth connectors.
- Full LSP asynchronous diagnostic history/version filtering.
- API host admission bridge for durable v2 idempotent prompt submission.
- Consent-based live Copilot smoke and cross-platform release signing/publication.
- W25 has no live Telegram transport by design; notification outbox/pairing is local and disabled by default.

## Residual Risks

- Worktree isolation does not provide OS-level filesystem/network/credential isolation.
- Explicitly authorized secret-bearing processes can exfiltrate received values; redaction is not sandboxing.
- Filesystem checks retain TOCTOU exposure against hostile concurrent actors.
- Existing server snapshot reads messages and event watermark through separate storage operations.
- Browser app is not production-authenticated until a host owns CSP headers, session authentication, CSRF and origin policy.
- macOS validation is strong; Windows and Linux runtime evidence remains a release gate.

## Next Steps

- [ ] Add the storage-backed scheduler adapter and fault-injection restart tests.
- [ ] Bridge `WorkerHost` to atomic legacy session admission, W08 worktree allocation and durable attempt settlement; do not remove `ExecutionUnavailable` until this gate passes.
- [ ] Integrate browser assets with an authenticated Rust host boundary.
- [ ] Add approved Keychain dependency and native credential smoke test.
- [ ] Add Windows ConPTY and cross-platform process lifecycle runners.
- [ ] Run consented live Copilot smoke and release qualification.
- [ ] Supply an explicit archive/checksum to `scripts/verify-release.sh`; its usage-only response is not release verification.
- [ ] Perform independent security/code review before any remote push or release.
