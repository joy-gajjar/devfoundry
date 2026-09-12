# DevFoundry Granular-Inspired Workspace Implementation Plan

> For agentic workers: use subagent-driven-development or executing-plans after the user explicitly requests execution. This file is a plan, not execution authorization. Checkboxes remain unchecked until evidence passes review.

**Goal:** repair DevFoundry's execution correctness and deliver a local-first Copilot agent workspace with a reliable TUI, browser companion, coordinated workers, Git worktrees, project brain, roadmap, secrets, resources, integrations and preview.

**Architecture:** retain the Rust/Tokio core and SQLite persistence; move lifecycle ownership into a supervised application host, leaving HTTP/TUI/browser as adapters. Add versioned contracts before parallel implementation. Workers use isolated worktrees and reviewed evidence, not a shared mutable checkout.

**Tech stack:** existing Rust/SQLx/Axum/Ratatui/Crossterm/reqwest; proposed React + TypeScript + Vite browser companion, Playwright browser tests, vetted platform PTY and credential-store adapters. Audit license/MSRV/platform support before adding dependencies. Current manifest says Rust 1.85; W00 must verify actual dependency MSRV before promising it.

**Spec:** [evidence and scope](granular-workspace-evidence.md) and [proposed contracts](granular-workspace-contracts.md). All three files are the full planning package.

## 1. Scope And Delivery Promise

User decisions: **TUI + browser companion**; **Copilot-only production agents**. Include Granular-style project/session navigation, plain-English chat, boss/workers, worktree review, real terminal, preview, linked Markdown brain, task board, scoped secret vault, MCP, resources/saved prompts, usage/budget visibility and opt-in notifications. LSP completion and code-quality fixes are part of the supporting project improvements.

Not in this delivery: native desktop wrapper, external Claude/Codex/Gemini engines, model resale/billing, paid marketplace commerce, public preview hosting, automatic publishing or autonomous unrestricted shell. PDF/office conversion is deferred to an explicitly approved connector; unsupported attachments are never silently ignored. Image input is enabled only when the selected Copilot model's capability is verified.

"One go" means **one coordinated execution campaign with gated waves and resumable work packages**, not a promise that a production-grade cross-platform system finishes in one model response. A preliminary decomposition is roughly 25-40 reviewable changes within the 19 work packages below; this is not a measured effort estimate. Actual duration depends on baseline failures, provider credentials and native CI. Stop rather than fake completion when external gates cannot run.

## 2. Issues To Fix First

| Priority | Work | Evidence IDs | Owner |
|---|---|---|---|
| P0 | Permission routing/publication races | R01 | session-runner |
| P0 | Conversation history and tool-call/result correlation | R02 | llm-provider + session-runner |
| P0 | Truthful command outcomes, bounded stream/process cleanup | R03, R08 | tools-security + llm-provider |
| P0 | Atomic admission/settlement, durable tool attempts, restart truth | R04, R05 | storage-recovery + session-runner |
| P0 | Wrong-target resolution, alias/search containment, shell trust/env policy | R06, R07 | tools-security |
| P0 | Standard MCP framing and independent fixture | R09 | tools-security |
| P0/P1 | Nonblocking TUI, permission cancel, drafts, UTF-8, scroll/layout | R10-R12 | tui-cli |
| P1 | Bounded history/replay, provider JSON/classification, limits/usage | R13-R15 | protocol-api + llm-provider |
| Capability | Real LSP and native PTY | R16 | tools-security |

The earlier `docs/audit/` contains false positives and overstated guarantees. W00 reconciles them with the evidence brief. No old finding should be implemented blindly; tests must establish the actual defect before its fix.

## 3. Approach And Alternatives

1. **Recommended: stabilize core, then TUI + browser through one API.** Lowest duplication and makes all work observable/recoverable. Some UI work can use frozen fixtures while backend progresses.
2. **TUI only:** smaller delivery, but cannot deliver an integrated graphical preview/docs map. User chose broader scope.
3. **Desktop rewrite first:** expensive new lifecycle/IPC/packaging surface before fixing existing execution defects. Defer until browser/API stabilize and separately approved.

Avoid a wholesale architecture rewrite. Extract `core/host`, `core/permissions`, `tools/process`, `tools/paths`, `tui/editor`, `tui/transcript`, `tui/effects` as owned modules; root wiring is controlled by the integrator. A shared `crates/client` serves Rust consumers, while versioned JSON/OpenAPI fixtures serve the browser. Server routes call host methods rather than managing runs themselves.

## 4. Execution Precondition

`git rev-parse --verify HEAD` currently fails and `git status --short` shows untracked source plus local databases/build output. Worktree-based execution cannot safely begin from that state.

- W00 records file inventory and identifies secrets/runtime artifacts without printing values.
- Request explicit authorization for a reviewed initial commit or a documented isolated snapshot strategy. Do not auto-stage the repository, databases, credentials, `target`, or unrelated harness settings.
- Preserve both existing database files. Back up with SQLite's supported backup mechanism before migration tests on copies; never infer disposability from old naming.
- No commits, push, merges into the user's branch or data deletion are authorized by this planning request.

## 5. Parallel Ownership And Waves

Distinguish agents implementing DevFoundry from boss/workers implemented inside DevFoundry. The former are this execution team; the latter are product functionality in W09.

| Wave | Packages | Parallelism and gate |
|---|---|---|
| 0 | W00 then W01 | Serial baseline and contract/module ownership freeze |
| 1 | W02, W04, W05, W07 fixture stage | Storage, provider, process/path, TUI modules are separate; W07 fixture acceptance permits wave advancement, not final feature acceptance |
| 2 | W03, W06, W07 integration stage | Host and API/client owners use merged contracts; integrate W07 only after W06; all three must pass before wave 3 |
| 3 | W08, W10, W11, W12, W13 | Worktrees, brain/tasks, secrets, integrations, terminal service; at most four active workers, queue extra lanes; shared adapters serialized |
| 4 | W09, W14 | Scheduler/worker engine and browser workspace; browser consumes fixed fixtures until service integration |
| 5 | W15, W16 | Preview/annotations and safe resource library, no shared UI root edits outside integrator |
| 6 | W17 then W18 | Optional remote bridge followed by full-system release qualification |

W03 depends on W02/W04/W05. W06 depends on W02 and W01 host interface; it cannot be accepted before W03 integration. W09 depends on W08/W10/W11 plus the reliable host. W12 remote OAuth depends on W11; W13 uses W05's process supervisor. W14 depends on W06/W07 contracts and W08-W13 service fixtures. W15 depends on W13/W14. W16 depends on W10/W11/W12/W14. W17 depends on W09/W11/W14. W18 requires all included packages.

One `rust-lead` integrator owns workspace manifests, lockfiles, schema/protocol roots, migrations numbering and crate root wiring. Each package owns its submodules/tests. `architecture` reviews shared contracts; `reviewer` performs read-only security/lifecycle reviews; `test-engineer` owns independent end-to-end fixtures; `documentation` reconciles records; `release-ops` owns native CI and distribution.

At most four implementation agents concurrently, plus reviewers as resources permit. Freeze interfaces first; two agents must never edit the same monolithic `lib.rs` concurrently. Worktrees do not eliminate integration conflicts. Contract changes return to W01, update fixtures, and invalidate affected downstream approvals.

Code-dependent tasks start from an approved integration baseline that contains the prerequisite commits, not merely a branch that existed when the campaign began. Reviewed-but-unintegrated changes do not satisfy dependencies. Non-code dependencies require immutable accepted artifact receipts. The same invariant applies to product workers in W09.

## 6. Universal Task Cycle

Every package uses this checklist; below it has its own concrete cases and acceptance gate.

- [ ] Read this plan, evidence, contracts, owned files, applicable planning docs and dependencies' handoffs.
- [ ] Write named failing tests for the listed behavior; run the targeted command and record the failure or classify baseline/environment blockage.
- [ ] Implement only owned behavior and minimal extraction. Test doubles must not reproduce a known incorrect protocol.
- [ ] Run targeted tests, formatting and changed-crate lint; record exact commands/results, not inferred success.
- [ ] Update task record, relevant contract and roadmap status, recording residual risks and evidence.
- [ ] Obtain independent review, resolve findings and integrate through rust-lead. Commit only if explicitly authorized by the user at execution time.

Rollback means stop the feature, preserve evidence and user data, and return to a compatible baseline under authorization. Forward migrations are not undone by switching executables.

## 7. Work Packages

### W00: Baseline And Audit Repair

**Owner:** release-ops + documentation. **Dependencies:** none. **Files:** `docs/audit/*.md`, `docs/PENDING_WORK.md`, `docs/OPENCODE_COMPARISON.md`, `docs/TUI_FEATURE_ROADMAP.md`, `.gitignore` (create after inspection), CI manifests. **Produces:** reviewed baseline identifier, verified toolchain matrix and corrected issue register.

- [ ] Reconcile every old audit claim against evidence brief; preserve rejected/unproven dispositions and historical rationale.
- [ ] Check manifest MSRV/dependencies, effective SQLite pragmas, supported targets and actual current test failures. Do not equate test counts with coverage.
- [ ] Establish approved baseline for worktrees; ignore runtime outputs without deleting them; preserve `.opencode` discovery/config.
- [ ] Run all existing Rust gates and release checks on safe fixture data. Capture logs before any feature implementation.

**Gate:** source baseline is recoverable, unknown failures classified, no private database/credential staged. **Rollback:** documentation edits only; no artifact deletion.

### W01: Contract Freeze And Module Ownership

**Owner:** architecture + rust-lead. **Depends:** W00. **Files:** `crates/schema/src/{lib.rs,execution.rs,workspace.rs}`, `crates/protocol/src/{lib.rs,v2.rs}`, `crates/core/src/lib.rs`, `crates/tools/src/lib.rs`, `crates/tui/src/lib.rs`; proposed JSON fixtures in `crates/protocol/tests/fixtures/`.

- [ ] Freeze the contracts companion's identities, error/outcome enums, permission revision, v2 envelope and host boundary.
- [ ] Map every planned UI action to command/query, authorization and error fixtures, including attachment admission, terminal lease/close, preview stop, task dependency edit and resource lifecycle. Separate evidence-review acceptance from human Git integration authorization.
- [ ] Add serialization round trips, legacy decode cases, unknown-event compatibility tests and exact JSON examples.
- [ ] Extract only module seams needed to give following workers distinct ownership; preserve behavior and v1 routes.
- [ ] Allocate unique migration filenames and files owned by each worker before dispatch.

**Run:** `cargo test -p devfoundry-schema -p devfoundry-protocol`; workspace check. **Gate:** downstream agents can build against fixtures without inventing APIs. **Rollback:** no schema migration in this package.

### W02: Transactional Lifecycle And Recovery

**Owner:** storage-recovery. **Depends:** W01. **Files:** `crates/storage/src/{lib.rs,execution.rs,events.rs}`, new forward migrations, `crates/storage/tests/{lifecycle.rs,execution_faults.rs}`. **Consumes:** run/attempt/state contracts. **Produces:** atomic admission/transition, bounded queries, snapshot watermark, durable ownership/recovery.

- [ ] Add tests `admission_failure_leaves_no_partial_run`, `same_key_same_payload_returns_receipt`, `same_key_different_payload_conflicts`, `crash_after_tool_start_marks_unknown`, `second_host_cannot_recover_live_owner`.
- [ ] Persist attempts, idempotency, policy revisions and events transactionally; keep SQLite cursor ordering authoritative.
- [ ] Add keyset pagination and size limits in SQL/read path, effective pragma tests, old-schema backup/migration/read compatibility.
- [ ] Reconcile session status and pending permissions on restart, never replay unknown effects.

**Run:** `cargo test -p devfoundry-storage`. **Gate:** injected faults leave either old or new complete state; migration copies retain original data. **Rollback:** disable writes and restore an explicitly approved consistent backup if downgrade is incompatible.

### W03: Supervised Host And Correct Permissions

**Owner:** session-runner. **Depends:** W02/W04/W05. **Files:** `crates/core/src/{host.rs,permissions.rs,runner.rs,context.rs}`, `crates/core/tests/{session_runner.rs,permission_races.rs}`, `crates/devfoundry/src/runtime.rs` through integrator. **Produces:** authoritative host command API consumed by server/client modes.

- [ ] Deterministically test two simultaneous permission services, approval before waiter, duplicate resolve/cancel, cancellation under stalled provider, per-session exclusion and shutdown with active tools.
- [ ] Implement request-ID ownership and durable-state waiting; enforce exact grants and joined task lifetimes. Session-scoped grants bind project/operation/target/policy and expire with session; no global allow-all toggle.
- [ ] Dispatcher consumes durable admissions; load chronological history and correlated tool facts; persist before side effects and before continuation.
- [ ] Emit explicit limit/cancel/failure/unknown outcomes; preserve usage/truncation. Embedded UI exit shuts host down; server-mode client disconnect does not cancel runs automatically.

**Run:** `cargo test -p devfoundry-core`. **Gate:** bounded termination and one durable terminal outcome, no stranded permission/run. **Rollback:** remain single-agent; do not activate scheduler until host gate passes.

### W04: Copilot Conversation And Streaming

**Owner:** llm-provider. **Depends:** W01; retries activate only with W02/W03 attempt integration. **Files:** `crates/llm/src/{lib.rs,wire.rs,stream.rs}`, provider fixtures/tests. **Produces:** normalized JSON/SSE events and complete tool conversation messages.

- [ ] Tests for text/tool/usage JSON-SSE equivalence, opaque provider call IDs, split UTF-8, malformed/oversized arguments, EOF without finish, stalled bytes cancellation, exact filter codes versus 429 and Retry-After.
- [ ] Add exact text/image attachment wire fixtures, artifact ownership/hash checks and unsupported-model rejection; no silent attachment loss.
- [ ] Preserve assistant calls and matching tool results; parse JSON tool calls; restore dead fragmentation assertions.
- [ ] Bound body/stream/argument buffers; select deadlines/cancellation independently of network traffic.
- [ ] Implement safe pre-output retries only, capped by run budget; never repeat effects or mid-stream accepted output. Expose actual supported Copilot capabilities and unknown pricing explicitly.

**Run:** `cargo test -p devfoundry-llm`. **Gate:** independent HTTP fixture verifies exact request wire shape and cancellation. **Rollback:** retry defaults off until safe boundary tests pass.

### W05: Paths, Process Outcomes And Trust

**Owner:** tools-security. **Depends:** W01. **Files:** `crates/tools/src/{paths.rs,process.rs,patch.rs,lib.rs}`, tools tests. **Produces:** contained file operations and common process supervisor.

- [ ] Tests `missing_nested_suffix_is_preserved`, `grep_rejects_external_symlink`, `sensitive_alias_is_denied`, `exit_seven_is_failed`, `grandchild_pipe_does_not_outlive_deadline`, `provider_token_not_in_child_env`, patch restoration failure surfaces recovery instructions.
- [ ] Validate resolved targets and every search traversal; bounded streaming reads; preserve missing path suffix, reject escaping/ambiguous paths. Minimize race windows and document residual TOCTOU without claiming OS isolation.
- [ ] Preserve exit code/signal, drain over-limit streams while discarding excess, retain truncation and bounded cleanup deadline through pipe joins.
- [ ] Default-clear environment with platform essentials allowlisted. Trusted shell permission describes host authority; never use command substring filtering as a sandbox.

**Run:** `cargo test -p devfoundry-tools`. **Gate:** file/process attacks tested on supported native targets; no false validation success. **Rollback:** disable affected mutations while preserving journal/backups.

### W06: Versioned API, History And Client

**Owner:** protocol-api. **Depends:** W02, frozen host interface; accept only after W03 integration. **Files:** `crates/server/src/{lib.rs,routes_v2.rs,auth.rs}`, proposed `crates/client/`, `crates/server/tests/`, `crates/tui/src/client.rs` via integrator. **Produces:** public v2 API and shared client used by both UI paths.

- [ ] Test snapshot/replay race, reordered wakeups, lag + catch-up beyond 1,000 events, keyset boundaries, idempotent uncertain admission, cross-project authorization and v1 compatibility.
- [ ] Validate immutable attachment references in admission and idempotency; return typed capability/media rejection before persisting a run.
- [ ] Move run ownership out of HTTP handlers; serve DB-ordered bounded events and accurate errors.
- [ ] Byte-safe SSE parsing supports multiple data lines, `id:` spacing, CRLF, unknown events and bounded frames; reconnect has visible recoverable states.
- [ ] Add authenticated loopback browser bootstrap, CSRF/origin/Host validation, finite HTTP deadlines and WebSocket attach rules before browser deployment.

**Run:** `cargo test -p devfoundry-server -p devfoundry-client`. **Gate:** old clients remain usable; new clients reconstruct complete history without duplicates. **Rollback:** keep v1 path while disabling v2 feature exposure.

### W07: TUI Correctness And Editing

**Owner:** tui-cli. **Depends:** W01 fixtures. **Stage A:** fixture-tested editor/reducer acceptance in wave 1. **Stage B:** actual W06 client integration and final acceptance in wave 2. **Files:** `crates/tui/src/{editor.rs,transcript.rs,focus.rs,effects.rs,picker.rs}`, TUI reducer and native harness tests. **Produces:** responsive client projection and explicit focus/effect model.

- [ ] Tests for rejected/uncertain draft preservation, global Ctrl-C during permission, dismiss not reopened by poll, stalled HTTP with live editing, split UTF-8, visible picker selection and bottom/history anchoring.
- [ ] Build multiline grapheme-aware editor, history, undo, searchable palette, transcript search, tabs with independent drafts, identity-grouped tool cards and exact scrollable permission details.
- [ ] Cache layout once per width/content revision; preserve code whitespace, terminal-cell width and reader position during streaming.
- [ ] Implement documented print/JSON/plan modes through public client contracts; terminal controls must not contaminate machine-readable output.

**Run:** `cargo test -p devfoundry-tui`; CLI integration tests. **Gate:** 40/80/120-column, 100 KiB paste and 10k-event history scenarios; target input-to-render p95 below 100 ms on recorded reference machine without network blocking. **Rollback:** retain simple transcript mode behind a temporary feature gate, remove it after migration only if no shipped dependency.

### W08: Worktree Ownership And Git Review

**Owner:** tools-security. **Depends:** W03/W05/W06. **Files:** `crates/tools/src/worktree.rs`, owned core workspace adapter, `crates/tools/tests/worktree_lifecycle.rs`. **Produces:** allocate/status/review/integrate/reconcile operations with base/head hashes.

- [ ] Test dirty primary checkout, concurrent Git metadata operations, cancellation, unknown worktree, restart orphan reconciliation, stale target HEAD and merge conflict.
- [ ] Test hostile Git hooks, smudge/clean filters, fsmonitor and diff helpers. Disable executable Git integrations by default; operations needing them require explicit trusted-process approval before checkout or review.
- [ ] One owned worktree/branch per mutating worker; retain base commit and lease, serialize Git administrative writes.
- [ ] Review in owned staging worktree; require exact evidence/base/head approval before target advancement. Commit/push/sync remain explicit user operations; never reset or stash unrelated user changes automatically.
- [ ] Keep review acceptance non-mutating; a separate human integration authorization binds target ref, expected HEAD, proposed head and evidence digest. Record resulting integration commit for dependent tasks.

**Run:** fixture-repository worktree tests. **Gate:** conflicting work cannot silently alter primary checkout and dirty work is preserved. **Rollback:** release process leases but retain worktrees until user-authorized cleanup.

### W09: Boss, Workers And Budgeted Scheduler

**Owner:** session-runner. **Depends:** W08/W10/W11 and host gates. **Files:** `crates/core/src/{scheduler.rs,workers.rs}`, scheduler tests, protocol routes through owner. **Produces:** parent/child sessions and deterministic task execution.

- [ ] Test DAG cycles, unmet dependencies, duplicate leases, stale revisions, cancellation tree, max concurrency/budget, repeated failure, child attempt unknown and restart during review.
- [ ] Test that a reviewed but unintegrated dependency remains blocked; verify each dependency's integration commit is an ancestor of the worker base. Accepted non-code artifacts require immutable receipt/hash in the brief.
- [ ] Boss proposes decomposition; scheduler enforces dependencies. Immutable worker brief includes objective, acceptance, base commit, owned scope, tools, context and budget.
- [ ] Worker returns evidence bundle, not acceptance. Reviewer moves Review to Accepted only after tests/diff checks; worker cannot approve itself or spawn unlimited descendants.
- [ ] Expose worker cards, blockers, elapsed time, usage and individual chat/cancel controls via shared projections.

**Run:** `cargo test -p devfoundry-core scheduler`; integration scenario with at least three workers, one failure and one conflict. **Gate:** no false done state or budget bypass. **Rollback:** disable new assignments; finish/cancel supervised active runs and retain evidence.

### W10: Project Brain, Context And Roadmap

**Owner:** general (knowledge lane), coordinated with storage-recovery. **Depends:** W02/W03/W06. **Files:** proposed `crates/core/src/{knowledge.rs,tasks.rs}`, storage document/task modules, future `docs/brain/README.md` only if approved/adopted. **Produces:** linked documents, search/context manifest and revisioned task records.

- [ ] Test wikilinks/backlinks, broken links, external/symlink traversal, conflicting file edit, malformed task import, DAG cycles and generated export feedback loop.
- [ ] Adopt existing docs with explicit front-door links; index lexical search/map as rebuildable projection. Preserve human text; worker notes are proposals with provenance.
- [ ] SQLite task board links each card to sessions/evidence; revision-aware export/import keeps Markdown portable without split authority.
- [ ] Context manifest records instructions/history/resources actually supplied; bound budget, compact only complete old tool groups with provenance; test that current instructions and unresolved calls survive compaction.

**Run:** knowledge/task unit tests and storage integration tests. **Gate:** external edits produce conflicts rather than silent overwrites; untrusted memory cannot grant tools. **Rollback:** rebuild index or disable generated writes; original Markdown remains intact.

### W11: Scoped Secret Vault

**Owner:** tools-security (credential lane). **Depends:** W03/W05/W06. **Files:** proposed `crates/devfoundry/src/credentials.rs`, `crates/core/src/secret_bindings.rs`, platform credential adapter and tests. **Produces:** secret reference resolution only for approved project/process/account binding.

- [ ] Test locked/missing key store, disabled entry, cross-project request, revoked scope, inherited-provider-token exclusion, export/log/terminal redaction and crash cleanup.
- [ ] Add trusted local setup and OS store adapters; preserve env-token handoff. No plaintext DB/config fallback.
- [ ] Prefer trusted connector adapters; human consent is required for arbitrary secret-bearing commands. Audits contain references, not values; no automatic rotation without support.

**Run:** fake-store contract tests plus native key-store smoke with disposable credentials. **Gate:** no host-controlled raw credential serialization in event fixtures, exports, logs or browser responses; known-value output redaction tested across chunks; platform failure closes access. This does not guarantee protection against transformed or out-of-band exfiltration by explicitly trusted secret-bearing commands. **Rollback:** disable bindings; keep user-controlled vault entries, do not delete secrets automatically.

### W12: MCP And LSP Interoperability

**Owner:** tools-security (integration lane). **Depends:** W03/W05/W06; remote accounts require W11. **Files:** `crates/tools/src/{mcp.rs,lsp.rs}`, independently implemented fixtures and native integration tests. **Produces:** standard tools discovery/call and versioned diagnostics.

- [ ] Replace MCP framing with newline JSON-RPC; test using a fixture that rejects Content-Length. Add initialization, bounded paginated tools/list, ID correlation, cancellation, timeouts and process restart on desynchronization.
- [ ] Add remote MCP using a pinned supported transport/auth specification, account isolation, OAuth state/PKCE, revocation and destination/redirect validation; no ambient token forwarding.
- [ ] Implement real LSP initialize/document/diagnostics/shutdown using LSP framing; stale diagnostic versions ignored, workspace edits require normal permissions.

**Run:** native integration tests on all supported OSes plus opt-in real-server checks. **Gate:** at least one independently maintained MCP and LSP server interoperates; fixture-only mode clearly labeled if unavailable. **Rollback:** disable connector/kill owned process, preserve config and error evidence.

### W13: Native PTY Service

**Owner:** tools-security (terminal lane), scheduled separately from overlapping process edits. **Depends:** W05/W06. **Files:** proposed `crates/tools/src/terminal.rs`, terminal transport routes, `crates/tools/tests/terminal_lifecycle.rs`. **Produces:** native open/input/resize/output/close, one input lease and bounded offset stream.

- [ ] Test isatty, interactive stdin, resize dimension, detach/reconnect gap, denied spawn, slow reader, terminal escapes and whole-process cleanup.
- [ ] Use vetted Unix PTY/Windows ConPTY implementation with platform capability reporting. Separate terminal byte ring from business events; child lifetime belongs to host, outer terminal restoration to TUI.
- [ ] Provide explicit escape chord from terminal focus; no hidden command injection or ANSI-to-Markdown conversion.

**Run:** native PTY harness on macOS/Linux/Windows. **Gate:** real TTY behavior proven, not pipe fixture equivalence. **Rollback:** close owned terminals and report unsupported feature, never silently substitute pipes as native PTY.

### W14: Browser Workspace And TUI Workspace Panels

**Owner:** general (browser) + tui-cli on disjoint files. **Depends:** W06/W07 and W08-W13 frozen service fixtures. **Files:** proposed `apps/workspace/`, `crates/tui/src/workspace.rs`, static serving route through protocol owner. **Produces:** four-surface workspace and unified project/agent/task navigation.

- [ ] Create browser contract tests from v2 fixtures and Playwright flows for session, admission, approval, worker board, docs map and reconnect.
- [ ] Implement chat/terminal/preview/docs layout; task cards open sessions; search, usage, status, keyboard navigation, accessible focus and narrow/mobile tabs.
- [ ] Add TUI worker/task/docs/resource panels without browser dependency. Never render success merely from an agent message.
- [ ] Use authenticated bootstrap and CSP; sanitize Markdown, treat attachments as untrusted, feature-test image support and bound uploads; prohibit raw token display.
- [ ] Test that admitted text/image artifact hashes appear in captured provider request fixtures and unsupported media is rejected visibly with the draft preserved.

**Run:** `npm ci`, `npm run typecheck`, `npm test`, `npm run build`, `npx playwright test` in `apps/workspace` (scripts created in this package). **Gate:** 1440px and 390px flows pass; keyboard-only and reduced-motion tests; browser refresh reconstructs state. **Rollback:** browser disabled, TUI still works.

### W15: Isolated Live Preview And Annotations

**Owner:** general (preview), security-reviewed. **Depends:** W13/W14. **Files:** proposed `crates/core/src/previews.rs`, dedicated preview-serving routes, `apps/workspace/src/preview/`. **Produces:** managed preview lifecycle, screenshots and revision-bound annotations.

- [ ] Tests for port collision, wrong worktree, failed readiness, path traversal, symlink file, malicious HTML/API call, unknown URL redirect and stop/cancel cleanup.
- [ ] Approved local process has owned port, health/deadline and stop action; generated artifacts served from explicit allowlisted directory on isolated origin, never entire project.
- [ ] Browser sandbox cannot access host/API credentials. Screenshot capture is opt-in, bounded and tied to artifact/hash. User annotations become explicit prompt context with captured revision, not arbitrary privileged page commands.

**Run:** preview lifecycle tests + Playwright hostile-page/annotation tests. **Gate:** untrusted preview cannot invoke host controls. **Rollback:** disable preview origin/process, retain artifacts.

### W16: Resources, Skills And Saved Prompts

**Owner:** general (resources). **Depends:** W10/W11/W12/W14. **Files:** proposed `crates/core/src/resources.rs`, resource storage adapter, `apps/workspace/src/resources/`, TUI resources module. **Produces:** manifest-based local library and explicit registry import.

- [ ] Test traversal, archive bombs, symlinks, hash mismatch, overwrite conflicts, no-hook execution and edited-file-preserving uninstall.
- [ ] Implement inspect/preview/install/enable/update/remove for templates, prompts, brain seeds, skills and MCP presets. Capabilities remain separately approved; installation never executes code or trusts text.
- [ ] Save reusable prompts with variables and project scope; registry items require origin/license/hash and explicit download consent. Marketplace publishing/payment services are excluded.

**Run:** resource unit/security tests plus browser install/uninstall flow. **Gate:** edited files and secrets survive uninstall; no package can auto-enable capabilities. **Rollback:** disable metadata entry; remove only unchanged owned files with user approval.

### W17: Optional Telegram Notifications And Restricted Replies

**Owner:** general (notifications), security-reviewed. **Depends:** W09/W11/W14. **Files:** proposed `crates/core/src/notifications.rs`, opt-in Telegram adapter, notification UI/settings. **Produces:** sanitized outbox and revocable account/chat binding.

- [ ] Test duplicate delivery, rate limit/backoff, wrong chat/user, expired/replayed response, stale task revision, revoked binding and secret-bearing message suppression.
- [ ] Default off; notify completion/block/failure with deep links requiring local authentication. Durable outbox has bounded retries and dedup keys.
- [ ] Add restricted cancel/acknowledge replies only after notification tests pass; require explicit pairing and nonce/revision. Privileged approvals and credential grants remain local. No remote shell.

**Run:** fake Telegram HTTP fixtures; opt-in disposable-account smoke only with credentials supplied at execution. **Gate:** unauthenticated/replayed messages cannot mutate tasks. **Rollback:** disable adapter and revoke pairing; no queued sensitive payload retained.

### W18: End-to-End, Documentation And Release Gate

**Owner:** test-engineer + release-ops + documentation; independent reviewer. **Depends:** every included package.

- [ ] Full scenario: open project, create brain/task, boss delegates three workers, one waits for permission, one fails a command, one produces preview, review evidence, resolve integration conflict, restart host and reconnect both clients without duplicating effects.
- [ ] Run migration/backup/recovery, native terminal/key store, real Copilot consented smoke, independent MCP/LSP interoperability, resource malicious input, browser isolation, notification denial and budget exhaustion tests.
- [ ] Update original audit/comparison/TUI roadmap, API docs, support matrix, security boundaries, per-task records and decision log. Do not overwrite historical evidence with blanket "complete" claims.
- [ ] Build/test/package on macOS Apple Silicon, Linux x64 and Windows x64; verify artifacts/checksums and browser assets. Signing/publication remain blocked until credentials and explicit release authorization exist.

**Gate:** all acceptance criteria have actual evidence or are explicitly blocked; no production-ready label if required platform/provider/security checks are absent. **Rollback:** compatibility-checked binary rollback plus separately approved DB restore procedure; never auto-delete workspace data.

## 8. Acceptance Matrix

| Feature | Minimum evidence |
|---|---|
| Reliable core | Race barriers, storage fault injection, request replay, unknown-side-effect recovery |
| Copilot loop | Realistic HTTP fixtures + consented live text/tool continuation |
| Parallel workers | Isolated worktrees, enforced dependency/budget, independent evidence review, conflict preservation |
| TUI/browser | Nonblocking input, accurate identities, reconnect/history, long output and narrow-layout tests |
| Vault | Native store tests, project scope, no ambient inheritance, no plaintext fallback |
| Brain/resources | Provenance, revision conflicts, malicious files, safe uninstall, no auto-run |
| PTY/MCP/LSP | Real native and independent protocol interoperability, cancellation/reap |
| Preview | Cross-origin/CSRF/path attack cases and opt-in screenshot policy |
| Release | Native OS matrix, backup/migration/downgrade checks, artifact validation |

Rust gates at each integration boundary:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Cargo was historically outside PATH; use `$HOME/.cargo/bin/cargo` where needed. `git diff --check` does not cover untracked files, so explicitly validate generated/added docs and staged intended changes. No tests were run for this planning-only task.

## 9. Agent Handoff Template

```text
Task: Wxx, objective and acceptance test names from this plan
Mode: implement only after explicit user execution request
Baseline: approved commit/snapshot ID; own isolated worktree
Read: all three plan files, relevant existing component docs, prerequisite handoffs
Own: exact modules/tests listed in package; no edits to other owners' roots
Inputs: frozen versioned contracts, fixtures and dependency revisions
Constraints: Copilot only; no secrets in output; no implicit Git mutation/publication
Required: failing regression evidence -> minimal code -> passing targeted tests
Return: changed paths, interface changes, commands/results, evidence IDs, residual risks
Stop: contract mismatch, missing baseline/credential, ownership conflict, failed prerequisite
Reviewer: separate agent; worker cannot self-accept
```

## 10. Risks And Stop-Loss

| Risk | Mitigation / stop condition |
|---|---|
| Prior audit misinformation | Revalidate and classify; do not implement unsupported fixes |
| Shared file conflicts | Serial contract/root edits, module ownership, integration queue |
| Unborn/untracked repository | No parallel mutations until approved recoverable baseline |
| Premature retries/merges | Durable attempts and evidence-bound human approval |
| Secret leakage through shell | Cleared env, exact grants, trusted-process disclosure, optional OS sandbox |
| Two writable roadmap authorities | SQL authority + revisioned Markdown export/import |
| UI preview privilege escalation | Separate origin, auth/CSRF/Host validation, bounded allowlisted artifacts |
| Unlimited agents/output/context | Enforced parent budgets, bounded queues, explicit failures |
| Missing native credentials/CI | Mark blocked and retain safe disabled state; no fabricated readiness |
| Too much scope for one context | Checkpoint work packages and resume from durable handoffs, not chat memory |

Do not progress past a failed wave gate. Failures are triaged to defect/environment/scope before retry. Do not repeatedly run the same failed command without new evidence. No autonomous repair loop may exceed the configured task budget.

## 11. Runtime And Session Handoff

- `CODEX_SESSION: unavailable` (ccg-workflow wrapper absent; no call made).
- `ANTIGRAVITY_SESSION: unavailable` (ccg-workflow wrapper absent; no call made).
- Local backend reviewer: `ses_f695f0f3bffefTxbXsqtyap3o0`.
- Local UX reviewer: `ses_f695f0f20ffeG2KObQDMKLeLCY`.
- Adversarial plan review: `ses_f6951df85ffell6PVK7sj1wKyB`; corrected cross-wave TUI gating, dependency baselines, Git helper policy, human integration authority, attachment contracts, UI lifecycle routes and bounded vault guarantees.

The plan can be executed by available native agents without pretending to resume external sessions. `/ccg:execute` requires separately provisioning and verifying the external runtime; it cannot resume IDs that were never created. Do not initialize external tooling automatically during this plan-only request.
