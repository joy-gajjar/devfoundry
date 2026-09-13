# Plan: Complete Granular-Inspired DevFoundry Workspace

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not begin this plan from an uncommitted or unapproved baseline.

**Goal:** Complete the deferred production work after commit `a9709ac`: durable boss-worker execution, authenticated browser hosting, live preview, safe resources, native credential storage, optional notifications, and cross-platform release qualification.

**Architecture:** Extend the existing Rust/Tokio/SQLite application host rather than adding a second scheduler. SQLite remains authoritative for runs, tasks, leases, evidence and revisions; the TUI and browser remain projections over versioned APIs/events. Workers execute in W08-owned worktrees, but acceptance and Git integration remain separate human-authorized operations.

**Tech Stack:** Rust 2024, Tokio, Axum, SQLx/SQLite, Ratatui, Crossterm, reqwest, existing GitHub Copilot provider, existing `apps/workspace` React/TypeScript/Vite/Playwright client, native Unix PTY, Windows ConPTY where verified, GitHub Actions release matrix.

**Spec:** `.claude/plan/granular-workspace-evidence.md`, `.claude/plan/granular-workspace-contracts.md`, `.claude/plan/granular-workspace.md`, and `.claude/PRPs/reports/granular-workspace-report.md`.

## Global Constraints

- Current branch is `feat/granular-workspace`; latest implementation baseline is commit `a9709ac`.
- Preserve local `.devfoundry.db` and `opencode.db`; never delete, reset, or migrate user databases in place during tests.
- Rust commands use `/Users/joy/.cargo/bin/cargo` when Cargo is not on `PATH`.
- Production provider remains GitHub Copilot only; do not add Claude, Codex, Gemini, Bedrock, hosted billing, or external CLI providers.
- No secret values may enter SQLite, Markdown brain files, event payloads, browser payloads, task evidence, logs, exports, or test reports.
- No worker may self-accept evidence, authorize Git integration, grant permissions, or recursively spawn unbounded workers.
- Worktrees isolate edits only; they are not an OS sandbox or credential boundary.
- Every task starts with failing tests, records RED evidence, implements the smallest passing change, and ends with targeted gates plus a task record.
- Every storage/schema/API/security/process task requires documentation under `docs/tasks/` following `docs/planning/18-documentation-workflow.md`.
- Do not claim Linux/Windows runtime support from macOS compilation; require native runner evidence.
- Do not make remote pushes, PRs, release publication, signing, or destructive cleanup without explicit authorization.

---

## Summary

### Completed prerequisites

- Durable execution lifecycle: `migrations/0002_execution_lifecycle.sql`.
- Supervised core permission/history behavior.
- Versioned v2 history/snapshot/replay API.
- Guarded worktree manager with non-mutating review and separate human integration authorization.
- Revisioned task board, dependency-cycle rejection, Markdown backlink projection and context manifests.
- Deterministic scheduler policy slice, not production execution.
- Fail-closed secret reference abstraction, but no native Keychain backend.
- Standard local MCP/LSP protocol fixtures.
- Native Unix PTY service, but no API/TUI transport integration.
- Browser workspace shell, but no authenticated Rust host/static serving/live SSE/preview/terminal/resources integration.

### Remaining completion packages

| Package | Scope | Dependency |
|---|---|---|
| W19 | Durable scheduler adapter and worker execution | W02, W03, W08, W09, W10 |
| W20 | Browser host/bootstrap/auth/reconnect integration | W06, W14, W19 API projections |
| W21 | Preview process service and browser/TUI preview | W13, W14, W20 |
| W22 | Resource/skill library and safe install/remove | W10, W11, W20 |
| W23 | Native credential backend and approved injection | W11, W03, W05, W19 |
| W24 | Terminal transport/TUI integration and worker projections | W13, W19, W20 |
| W25 | Optional notification bridge | W19, W20, W23 |
| W26 | Cross-platform and release qualification | W19-W25, release credentials for publication only |

### Parallel waves

```text
Wave A: W19 serial contract/storage adapter
Wave B: W20, W22, W23 in parallel after W19 contract freeze
Wave C: W21 and W24 in parallel after W20/W23 boundaries
Wave D: W25 after W19/W20/W23, optional
Wave E: W26 serial final qualification after all included packages
```

One integrator owns migrations, workspace manifests, shared protocol/API root wiring and final documentation reconciliation. Browser agents own `apps/workspace/**`; Rust agents do not edit browser internals. No two agents edit the same monolithic Rust root concurrently.

## User Story

As a project owner,
I want DevFoundry to run bounded Copilot workers in isolated worktrees, show their evidence in the TUI/browser, safely preview results, manage project knowledge/resources/secrets, and qualify the product across supported platforms,
so that I can complete real multi-step work without losing control of code, credentials, approvals, or recovery state.

## Problem -> Solution

The current project has reliable foundations, but the scheduler is policy-only, browser is fixture/API-only, preview and native terminal are not hosted through the API, the vault has no native backend, and release evidence does not prove runtime parity.

The completion plan adds durable leases/attempts/evidence, runs workers through the existing `SessionRunner` and `WorktreeManager`, serves the browser only behind a controlled authenticated host, adds explicit preview/resource/terminal boundaries, resolves secrets through approved OS stores, and blocks final readiness until native platform and provider evidence exists.

## Metadata

- **Complexity:** XL
- **Source PRD:** N/A; derived from the approved Granular workspace plan and implementation report
- **PRD Phase:** Completion of deferred W09/W11/W14-W18 work
- **Estimated Files:** 35-60 Rust/browser/docs/test/release files, plus 4-6 forward migrations if required
- **Execution mode:** Gated multi-agent campaign; do not attempt as one unreviewed parallel batch

---

## UX Design

### Before

```text
TUI/browser shell
        |
        +-- reads sessions/tasks/docs through v2 API
        +-- no real worker execution
        +-- no authenticated browser host
        +-- no managed preview or native terminal pane
        +-- no installed resource workflow
```

### After

```text
Authenticated local host
  +-- Chat/session surface
  +-- Worker board: ready -> assigned -> running -> review -> accepted
  +-- Evidence drawer: diff, tests, artifacts, usage, risks
  +-- Permission/status center
  +-- Native terminal attachment with input lease
  +-- Preview pane on isolated origin
  +-- Brain/docs/resource navigation
  +-- Secret references and binding status, never secret values
```

### Interaction Changes

| Touchpoint | Before | After | Notes |
|---|---|---|---|
| Prompt | v1 prompt or browser fixture shell | Durable v2 admission with idempotency receipt | Same key must not duplicate side effects |
| Task | Task board data exists but no worker execution | Assign bounded Copilot worker with evidence | Worker cannot mark Accepted |
| Review | Worktree review API exists in tools | Review evidence then separate human integration authorization | Target HEAD is revalidated |
| Browser auth | Fixture client uses same-origin fetch | Host-issued non-secret bootstrap and authenticated session | No browser token storage |
| Preview | None | Explicitly approved managed process + isolated preview origin | Preview cannot access privileged API |
| Terminal | TUI only; native PTY service not transported | Attach/input/resize/output/close through authorized API | One input lease |
| Secrets | Fail-closed abstraction only | Native store adapter and scoped process injection | Trusted process can still exfiltrate received secret |
| Resources | No install library | Inspect, preview, install, update, remove | No hooks or automatic capability grants |

---

## Mandatory Reading

| Priority | File | Lines | Why |
|---|---|---:|---|
| P0 | `.claude/PRPs/reports/granular-workspace-report.md` | 1-149 | Exact completed/deferred state and final evidence |
| P0 | `.claude/plan/granular-workspace-contracts.md` | 1-230 | Proposed lifecycle/API/security contracts |
| P0 | `crates/core/src/scheduler.rs` | 1-210 | Existing readiness/budget policy to persist, not replace |
| P0 | `crates/core/src/workers.rs` | 1-93 | Existing brief/evidence/provider boundary |
| P0 | `crates/core/src/lib.rs` | 256-518 | SessionRunner lifecycle, permission and provider loop |
| P0 | `crates/storage/src/tasks.rs` | 1-330 | Existing revisioned task/dependency implementation |
| P0 | `crates/storage/src/lib.rs` | 1-500 | SQLite connection/migrations/repository/error conventions |
| P0 | `crates/tools/src/worktree.rs` | 1-467 | Worktree allocation/review/integration authorization |
| P0 | `crates/tools/src/terminal.rs` | 1-414 | Native PTY lifecycle and bounded output/lease behavior |
| P0 | `crates/server/src/lib.rs` | 34-238 | `ServerState`, router, security middleware and existing routes |
| P1 | `crates/server/src/routes_v2.rs` | 1-200 | Versioned DTO, bounded history and 501 admission boundary |
| P1 | `crates/server/src/routes_w10.rs` | all | Task/roadmap/document API adapter pattern |
| P1 | `crates/devfoundry/src/credentials.rs` | all | Fail-closed secret store and redacted handle boundary |
| P1 | `crates/core/src/secret_bindings.rs` | all | Value-free binding scope validation |
| P1 | `apps/workspace/src/api/client.ts` | all | Browser same-origin typed fetch/error pattern |
| P1 | `apps/workspace/src/App.tsx` | all | Browser shell, responsive tabs and status projection |
| P1 | `apps/workspace/SECURITY.md` | all | Browser CSP/bootstrap/security boundary |
| P1 | `release/platforms.json` | all | Declared platform targets; not runtime proof |
| P2 | `scripts/validate-secretless.sh` | all | Release metadata and secretless gate |
| P2 | `docs/planning/18-documentation-workflow.md` | all | Required task-record format/evidence |

## External Documentation

| Topic | Source | Key Takeaway |
|---|---|---|
| MCP stdio | https://modelcontextprotocol.io/specification/2024-11-05/basic/transports | Newline JSON-RPC; no Content-Length for MCP stdio |
| LSP transport | https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/ | LSP uses Content-Length framing and request/notification lifecycle |
| Granular workspace | https://www.granular.build/docs/the-interface/ | Chat, terminal, preview and docs are separate surfaces |
| Granular workers | https://www.granular.build/docs/boss-and-worker-agents/ | Workers use isolated worktrees and reviewable merges |
| Granular vault | https://www.granular.build/docs/the-vault/ | Secret injection is scoped, but trusted processes remain a threat boundary |
| Granular resources | https://www.granular.build/docs/resource-library-skills/ | Copy/install resources without silently running submitted code |
| Rust PTY | Platform-specific `forkpty`/ConPTY documentation | Native terminal behavior must be proven per OS; never substitute pipes silently |

## Patterns To Mirror

### NAMING_CONVENTION

// SOURCE: `crates/storage/src/tasks.rs:20-35`

```rust
#[async_trait]
pub trait TaskRepository {
    async fn create_task(&self, task: Task) -> StorageResult<Task>;
    async fn get_task(&self, id: TaskId) -> StorageResult<Option<Task>>;
    async fn list_tasks(&self, project_id: ProjectId) -> StorageResult<Vec<Task>>;
}
```

Use focused modules, `PascalCase` domain types, verb-based async repository methods, and `StorageResult<T>`.

### ERROR_HANDLING

// SOURCE: `crates/server/src/routes_w10.rs:70-86`

```rust
.map_err(|storage_error_value| match storage_error_value {
    StorageError::Domain(DomainError::Validation(message)) => {
        error(StatusCode::CONFLICT, "task_conflict", &message)
    }
    other => storage_error(other),
})?;
```

Keep handlers thin. Map typed domain/storage errors to stable HTTP codes; never leak paths, tokens, stack traces or raw provider payloads.

### LOGGING_PATTERN

The current Rust workspace has no general logging/tracing dependency or established production log macro. Do not add ad hoc `println!`/`eprintln!` output. Return redacted typed errors and persist sanitized evidence; add structured logging only as a separately reviewed observability package.

### REPOSITORY_PATTERN

// SOURCE: `crates/storage/src/tasks.rs:62-135`

```rust
let mut tx = self.pool().begin().await?;
// read expected revision, validate graph, write all related rows
tx.commit().await?;
self.get_task(id).await?.ok_or_else(|| StorageError::Domain(...))
```

Use forward migrations, transaction-local validation, optimistic revision checks, and reload the committed projection. Do not edit old migration files.

### SERVICE_PATTERN

// SOURCE: `crates/core/src/tasks.rs:4-59`

```rust
pub struct TaskService {
    store: Arc<SqliteStore>,
}

impl TaskService {
    pub fn new(store: Arc<SqliteStore>) -> Self { Self { store } }
    pub async fn list(&self, project_id: ProjectId) -> Result<Vec<Task>, CoreError> { ... }
}
```

Application services wrap storage and translate errors; they do not expose SQLx or permit UI code to mutate repositories directly.

### API_PATTERN

// SOURCE: `crates/server/src/routes_v2.rs:93-135`

```rust
pub(crate) async fn history(...) -> ApiResult<Json<HistoryResponse>> {
    let session_id = parse_session_id(&session_id)?;
    ensure_session(&state, session_id).await?;
    let messages = state.store.list_messages_after(...).await.map_err(storage_error)?;
    Ok(Json(HistoryResponse { version: 2, messages, next }))
}
```

Version DTOs stay server-owned until a shared client contract is justified. Validate limits before storage calls and preserve v1 routes.

### TEST_STRUCTURE

// SOURCE: `crates/storage/tests/w10_tasks.rs:1-60`

```rust
#[tokio::test]
async fn stale_task_revision_is_rejected_without_mutation() {
    let (store, _directory, project_id) = store().await;
    let task = Task::new_for_project(project_id, "A", "first");
    store.create_task(task.clone()).await.unwrap();
    let error = store.update_task(task.id, TaskUpdate::title("changed"), 0).await;
    assert!(error.is_err());
}
```

Use temp directories, disposable SQLite databases, named behavioral tests, and exact assertions. For browser use Vitest fixtures and Playwright desktop/mobile flows under `apps/workspace/tests/`.

### BROWSER_SECURITY_PATTERN

// SOURCE: `apps/workspace/src/api/client.ts:22-36`

```ts
response = await fetcher(url, {
  method: 'GET',
  credentials: 'same-origin',
  headers: { Accept: 'application/json' },
})
```

Browser requests use same-origin credentials, no bearer token option, no browser storage, and generic retryable errors. Production host integration must own CSP headers, auth, CSRF and origin validation.

---

## Files To Change

### W19 Durable Worker Execution

| File | Action | Justification |
|---|---|---|
| `migrations/0004_scheduler_execution.sql` | CREATE | Durable leases, worker attempts, evidence, failure fingerprints and integration receipts |
| `crates/storage/src/scheduler.rs` | CREATE | Atomic lease/status/attempt/evidence repository operations |
| `crates/storage/src/lib.rs` | UPDATE | Export scheduler repository and typed errors |
| `crates/storage/tests/scheduler_recovery.rs` | CREATE | Fault injection, duplicate lease, stale revision and restart cases |
| `crates/core/src/scheduler_adapter.rs` | CREATE | Adapt persisted scheduler state to existing W09 policy |
| `crates/core/src/workers.rs` | UPDATE | Execute immutable briefs through `SessionRunner` and worktree adapter |
| `crates/core/src/host.rs` | CREATE | Supervised worker lifecycle, cancellation tree and joined cleanup |
| `crates/core/src/lib.rs` | UPDATE | Export host/adapter and integrate task execution boundary |
| `crates/core/tests/worker_execution.rs` | CREATE | Worker evidence, unknown effects, cancellation and restart behavior |
| `crates/tools/src/worktree.rs` | UPDATE | Durable lease/evidence metadata adapter only; preserve review/integration policy |
| `crates/server/src/lib.rs` | UPDATE | Add worker/task status routes only after storage/core contracts pass |
| `crates/server/src/routes_workers.rs` | CREATE | Worker status, assignment, cancel and evidence read APIs |
| `docs/tasks/076-durable-worker-execution.md` | CREATE | Required task evidence |

### W20 Browser Host And Authentication

| File | Action | Justification |
|---|---|---|
| `crates/server/src/browser.rs` | CREATE | Static asset serving and non-secret bootstrap boundary |
| `crates/server/src/auth.rs` | CREATE/UPDATE | Loopback/session auth, CSRF/origin/Host checks, browser attach policy |
| `crates/server/src/lib.rs` | UPDATE | Mount browser routes only behind explicit configuration |
| `crates/server/tests/browser_security.rs` | CREATE | CSP, origin, CSRF, auth and token non-disclosure tests |
| `apps/workspace/src/api/client.ts` | UPDATE | Snapshot/event reconnect and typed mutation errors |
| `apps/workspace/src/api/types.ts` | UPDATE | Worker/preview/resource/status DTOs |
| `apps/workspace/src/App.tsx` | UPDATE | Live status, worker board, error/reconnect UI |
| `apps/workspace/tests/api-client.test.ts` | UPDATE | Reconnect/gap/unknown-event tests |
| `docs/tasks/077-browser-host-auth.md` | CREATE | Host/security evidence |

### W21 Preview Service

| File | Action | Justification |
|---|---|---|
| `crates/core/src/previews.rs` | CREATE | Approved process/port/worktree ownership and readiness lifecycle |
| `crates/tools/src/preview.rs` | CREATE | Bounded process/port/asset serving boundary |
| `crates/server/src/routes_previews.rs` | CREATE | Start/status/stop/screenshot/annotation routes |
| `crates/server/src/lib.rs` | UPDATE | Register preview routes |
| `crates/server/tests/preview_security.rs` | CREATE | Path/origin/port/process cleanup tests |
| `apps/workspace/src/preview/` | CREATE | Preview panel, screenshot and annotation UI |
| `apps/workspace/tests/e2e/preview.spec.ts` | CREATE | Hostile preview origin and stop flows |
| `docs/tasks/078-preview-service.md` | CREATE | Threat model and verification |

### W22 Resources And Skills

| File | Action | Justification |
|---|---|---|
| `migrations/0005_resources.sql` | CREATE | Resource manifests, install hashes, ownership and revisions |
| `crates/schema/src/resources.rs` | CREATE | Manifest, capability declaration and provenance types |
| `crates/storage/src/resources.rs` | CREATE | Resource metadata/install/revision repository |
| `crates/core/src/resources.rs` | CREATE | Inspect/stage/install/update/remove orchestration |
| `crates/tools/src/archive.rs` | CREATE | Safe archive traversal, size/hash/symlink checks |
| `crates/server/src/routes_resources.rs` | CREATE | Resource lifecycle API |
| `apps/workspace/src/resources/` | CREATE | Resource browser/install review UI |
| `crates/tools/tests/resource_security.rs` | CREATE | Traversal, archive bomb, hash mismatch and no-hook tests |
| `docs/tasks/079-resource-library.md` | CREATE | Resource security/evidence record |

### W23 Native Credential Backend

| File | Action | Justification |
|---|---|---|
| `Cargo.toml` / `Cargo.lock` | UPDATE | Add only an approved, audited credential-store dependency |
| `crates/devfoundry/src/credentials.rs` | UPDATE | Implement native adapter behind existing `SecretStore` trait |
| `crates/devfoundry/tests/credentials_native.rs` | CREATE | Disposable Keychain/credential-store smoke and lock failure |
| `crates/core/src/secret_bindings.rs` | UPDATE | Process binding/lease/revocation integration |
| `crates/tools/src/process.rs` | CREATE/UPDATE | Approved environment injection with allowlist and redaction boundary |
| `docs/tasks/080-native-credential-backend.md` | CREATE | Platform/security evidence |

### W24 Terminal And Worker Projections

| File | Action | Justification |
|---|---|---|
| `crates/server/src/routes_terminals.rs` | CREATE | Terminal create/input/resize/output/close with leases |
| `crates/server/src/lib.rs` | UPDATE | Register authenticated terminal routes |
| `crates/server/tests/terminal_api.rs` | CREATE | Attach lease, gap, close and authorization tests |
| `crates/tui/src/terminal.rs` | CREATE | Native PTY byte renderer and focus/escape handling |
| `crates/tui/src/lib.rs` | UPDATE | Terminal pane and worker status projection |
| `apps/workspace/src/terminal/` | CREATE | Terminal/worker evidence views |
| `apps/workspace/tests/e2e/worker-board.spec.ts` | CREATE | Worker status/review/cancel browser flow |
| `docs/tasks/081-terminal-worker-projections.md` | CREATE | TUI/browser interaction evidence |

### W25 Optional Notifications

| File | Action | Justification |
|---|---|---|
| `crates/core/src/notifications.rs` | CREATE | Sanitized outbox, dedup, retry and pairing state |
| `crates/server/src/routes_notifications.rs` | CREATE | Opt-in setup/status/revoke; no remote shell |
| `crates/server/tests/notification_security.rs` | CREATE | Replay, wrong chat, stale revision, redaction and revoke tests |
| `apps/workspace/src/settings/notifications.tsx` | CREATE | Explicit opt-in UI and pairing state |
| `docs/tasks/082-notification-bridge.md` | CREATE | Optional feature evidence |

### W26 Qualification And Release

| File | Action | Justification |
|---|---|---|
| `.github/workflows/ci.yml` | UPDATE | Rust/browser/native matrix and artifact evidence |
| `.github/workflows/release.yml` | UPDATE | Cross-platform build/test/verify gates |
| `scripts/verify-release.sh` | UPDATE | Migration compatibility, browser asset, checksum and secretless checks |
| `docs/support-matrix.md` | UPDATE | Actual native evidence, not declared target intent |
| `docs/release-verification.md` | UPDATE | Final qualification commands/results |
| `docs/tasks/083-final-qualification.md` | CREATE | Completion evidence and blocked gates |
| `.claude/PRPs/reports/granular-workspace-completion-report.md` | CREATE | Final execution report |

## NOT Building

- Additional LLM providers or external Claude/Codex/Gemini CLI adapters.
- Hosted model billing, marketplace payments or resource execution hooks.
- Arbitrary remote shell through Telegram or browser.
- Public preview tunnels or unauthenticated preview hosting.
- Automatic Git push, merge, reset, stash or cleanup of user worktrees.
- Plaintext secret fallback, environment-variable discovery of arbitrary credentials, or secret export.
- Mobile-native application or desktop/Tauri wrapper.
- Automatic replay of uncertain side effects after crash.
- Claiming full production readiness before native platform, live-provider and release evidence exists.

---

## Step-by-Step Tasks

### Task 1: Establish completion baseline and execution queue

- **ACTION:** Create a fresh execution checkpoint from `a9709ac` and inventory generated/untracked artifacts before any new changes.
- **IMPLEMENT:** Confirm `git status --short --ignored`, preserve databases, verify `/Users/joy/.cargo/bin/cargo`, `node`, `npm`, and Playwright availability. Do not create another baseline commit unless the current branch diverged or the user authorizes it.
- **MIRROR:** Baseline workflow in `.claude/PRPs/reports/granular-workspace-report.md:18-28` and Rust gates in `docs/planning/18-documentation-workflow.md:77-86`.
- **IMPORTS:** None.
- **GOTCHA:** `.claude/PRPs/reports/` and `.claude/PRPs/plans/` are currently untracked; stage only the intended plan/report files when needed. Do not stage `node_modules`, `dist`, `test-results`, `target` or databases.
- **VALIDATE:** `git status --short --ignored`; `/Users/joy/.cargo/bin/cargo --version`; `node --version`; `npm --version`; `git diff --check`.

### Task 2: Add durable scheduler persistence

- **ACTION:** Persist the W09 scheduler policy facts so duplicate leases, stale revisions, worker attempts, evidence and restart recovery are authoritative.
- **IMPLEMENT:** Add migration `0004_scheduler_execution.sql` with `scheduler_leases`, `worker_attempts`, `worker_evidence`, `integration_receipts` and `failure_fingerprints`. Each row must carry project/task/run identity, expected revision, owner, status, created/updated times, and revision/hash fields. Add repository methods:

```rust
pub trait SchedulerRepository {
    async fn claim_task(&self, task_id: TaskId, expected_revision: Revision, owner: &str) -> StorageResult<WorkerLease>;
    async fn begin_attempt(&self, lease_id: &str, attempt_id: AttemptId) -> StorageResult<WorkerAttempt>;
    async fn settle_attempt(&self, attempt_id: AttemptId, outcome: AttemptStatus, evidence: Option<&EvidenceRecord>) -> StorageResult<WorkerAttempt>;
    async fn release_task(&self, lease_id: &str, expected_revision: Revision) -> StorageResult<()>;
    async fn recover_orphaned_workers(&self, owner: &str) -> StorageResult<Vec<WorkerAttempt>>;
}
```

  Keep the exact names aligned with existing W02/W10 domain IDs. Every claim/update is a compare-and-set transaction; duplicate lease returns conflict, stale revision returns conflict, and uncertain side effects become `OutcomeUnknown` without automatic replay.
- **MIRROR:** `crates/storage/src/tasks.rs:62-135` transaction/revision pattern and `migrations/0002_execution_lifecycle.sql` row/index style.
- **IMPORTS:** `devfoundry_schema::{AttemptId, AttemptStatus, Revision, RunId, TaskId}`; `crate::{SqliteStore, StorageError, StorageResult}`; `async_trait::async_trait`.
- **GOTCHA:** Do not add a second task authority. W10 task rows remain authoritative for title/status/dependencies; scheduler tables hold execution facts and leases. Do not mutate `0001`, `0002`, or `0003`.
- **VALIDATE:** Add RED tests for duplicate lease, stale revision, crash after attempt start, review persistence and recovery. Run `cargo test -p devfoundry-storage --test scheduler_recovery`, then storage format/check/Clippy/tests.

### Task 3: Integrate scheduler policy with persistence

- **ACTION:** Adapt pure W09 readiness/limits to the durable W02/W10 repository without changing its deterministic policy.
- **IMPLEMENT:** Create `crates/core/src/scheduler_adapter.rs` with `SchedulerStore` and `WorktreeEvidence` adapters. `ready_tasks` must load task rows plus durable failure/lease/receipt facts, call `dependency_readiness`, and return stable task IDs. `assign` must claim in storage before returning `Assignment`; `record_failure`, `record_usage`, `record_review` and `record_unknown` must persist before publishing any projection.
- **MIRROR:** `crates/core/src/scheduler.rs:113-187` for policy calls; `crates/core/src/tasks.rs:4-59` for service wrapping.
- **IMPORTS:** `std::sync::Arc`; existing `Scheduler`, `SchedulerTask`, `SchedulerLimits`, `DependencyReceipt`, `TaskService`; new `SchedulerRepository`.
- **GOTCHA:** `DependencyReceipt.ancestor` is not trusted input in production. W08 must verify Git ancestry before the adapter creates a receipt. Review is non-mutating; integration requires `HumanIntegrationAuthorization`.
- **VALIDATE:** Tests for stale task revision, duplicate assignment, accepted-but-unintegrated dependency, repeated failure blocking, unknown-effect non-replay and restart recovery. Run core targeted plus workspace gates.

### Task 4: Add supervised worker execution

- **ACTION:** Execute one bounded Copilot worker through the existing `SessionRunner` and W08 worktree boundary, producing evidence only.
- **IMPLEMENT:** Add `crates/core/src/host.rs` with a supervisor owning cancellation tokens and `JoinHandle`s. Add worker execution methods:

```rust
pub async fn execute_worker(&self, lease: WorkerLease, brief: WorkerBrief) -> Result<EvidenceBundle, WorkerFailure>;
pub async fn cancel_worker(&self, attempt_id: AttemptId) -> Result<CancelAcknowledgement, CoreError>;
pub async fn recover_workers(&self) -> Result<Vec<WorkerAttempt>, CoreError>;
```

  Validate task revision and Copilot provider before worktree allocation. Allocate through a narrow `WorkerWorktree` adapter; run `SessionRunner` with the captured agent/model/policy; persist attempt start before side effects; persist evidence before task state `Review`; never set `Accepted`; never call integrate.
- **MIRROR:** `crates/core/src/lib.rs:290-518` runner lifecycle and `crates/tools/src/worktree.rs:144-328` allocation/review/integration separation.
- **IMPORTS:** `CancellationToken`; `SessionRunner`; `WorkerBrief`, `EvidenceBundle`, `WorkerOutcome`; `SchedulerRepository`; `WorktreeManager` adapter; `TaskService`.
- **GOTCHA:** Existing `SessionRunner` uses legacy prompt admission. If an atomic durable bridge cannot be implemented without changing server/storage owners, stop and add the exact contract task rather than claiming worker execution is safe.
- **VALIDATE:** RED tests for non-Copilot rejection, stale revision, child cancellation, process/provider failure, unknown side effect and evidence-before-review. Run core targeted and workspace gates.

### Task 5: Add worker API projections

- **ACTION:** Expose durable worker/task/attempt/evidence status to TUI and browser clients.
- **IMPLEMENT:** Add `crates/server/src/routes_workers.rs` with:

```text
GET  /api/v2/workspaces/{id}/workers
GET  /api/v2/tasks/{id}/attempts
POST /api/v2/tasks/{id}/assign
POST /api/v2/attempts/{id}/cancel
GET  /api/v2/attempts/{id}/evidence
POST /api/v2/tasks/{id}/review
```

  `review` records evidence acceptance only; a separate existing worktree integration operation remains required. Return stable status/error DTOs, redact command output and secrets, and require project authorization.
- **MIRROR:** `crates/server/src/routes_v2.rs:18-91` server-owned DTOs and `crates/server/src/routes_w10.rs:1-170` thin handlers/error mapping.
- **IMPORTS:** Axum `Path`, `State`, `Json`, `StatusCode`; `ServerState`; `TaskService`; `SchedulerRepository`; `storage_error`/`error`.
- **GOTCHA:** Do not add an HTTP-local lease map. All assignment/cancel/review state must use durable storage and expected revisions.
- **VALIDATE:** API tests for project isolation, stale assignment, duplicate claim, evidence-only review, cancel idempotency, and secret redaction. Run server tests and workspace gates.

### Task 6: Add authenticated browser host/bootstrap

- **ACTION:** Serve the existing browser build from the Rust host only when explicitly enabled and provide a non-secret authenticated bootstrap.
- **IMPLEMENT:** Add `BrowserConfig { enabled, asset_root, session_cookie, allowed_origin }`, static fallback that never serves files outside `asset_root`, CSP response headers, authenticated bootstrap endpoint returning only API base/project/session IDs, and CSRF/origin/Host checks for browser mutations. Extend `apps/workspace/src/api/client.ts` with snapshot + event cursor reconciliation and typed mutation helpers. Add a status panel that distinguishes API reachable, SSE connected, reconnecting and stale snapshot.
- **MIRROR:** Server security middleware `crates/server/src/lib.rs:127-156,240-320`; browser same-origin client `apps/workspace/src/api/client.ts:18-43`; browser security contract `apps/workspace/SECURITY.md:1-23`.
- **IMPORTS:** Axum static response types, `Path`, `HeaderMap`, `Cookie` implementation only if already approved; browser fetch types and existing DTOs.
- **GOTCHA:** Never put tokens in query strings, `localStorage`, bootstrap JSON, preview frames or HTML. Do not enable the route by default. Keep v1 routes intact.
- **VALIDATE:** Rust tests for traversal, origin, CSRF, CSP and bootstrap redaction; browser tests for reconnect, unknown event, 401/403 and refresh reconstruction. Run Rust gates plus `npm run typecheck && npm test && npm run build && npx playwright test`.

### Task 7: Add managed preview service

- **ACTION:** Start approved preview processes for an owned worktree, report readiness, and serve only explicit artifacts on a separate origin.
- **IMPLEMENT:** Add a preview state machine `Requested -> Starting -> Ready -> Failed -> Stopped`, process/port ownership, readiness deadline, stop/reap, and artifact root containment. Add API routes:

```text
POST /api/v2/projects/{id}/previews
GET  /api/v2/previews/{id}
POST /api/v2/previews/{id}/stop
POST /api/v2/previews/{id}/screenshots
POST /api/v2/previews/{id}/annotations
```

  Require exact worktree/revision ownership, explicit `preview_start` permission, bounded ports/commands, and separate origin with no privileged API cookies.
- **MIRROR:** W05 process boundary `crates/tools/src/lib.rs:937-1034`; W13 PTY cleanup `crates/tools/src/terminal.rs:229-379`; browser security boundary `apps/workspace/SECURITY.md:8-23`.
- **IMPORTS:** Existing process-group helpers or a new bounded adapter; Axum routes; browser preview component.
- **GOTCHA:** Do not serve the project root, follow symlinks, expose arbitrary URLs, or infer public tunnel support. A browser page cannot acquire host privileges through annotations.
- **VALIDATE:** Tests for port collision, wrong worktree, failed readiness, traversal/symlink, malicious preview page, stop cleanup and annotation stale revision. Run browser hostile-origin Playwright tests.

### Task 8: Add resource/skill library

- **ACTION:** Install reusable skills/templates/brain seeds/MCP presets through inspectable manifests without running submitted code.
- **IMPLEMENT:** Add migration `0005_resources.sql`, `ResourceManifest { id, version, source, hashes, target_paths, required_capabilities }`, safe archive staging, preview diff, install/update/remove with hash-aware preservation of edited files, and resource API routes. Add browser resource review/install UI.
- **MIRROR:** W10 revision/import storage patterns `crates/storage/src/tasks.rs:62-135`; W05 path/security patterns `crates/tools/src/lib.rs:129-161`; W14 app/client patterns.
- **IMPORTS:** `sha2` only after dependency approval; `zip`/tar support only after archive-security review; existing serde/SQLx/Axum/React dependencies.
- **GOTCHA:** No install hook, shell command, credential enablement, executable filter or automatic MCP trust. Reject archive traversal, symlinks, duplicate target conflicts and oversized inputs.
- **VALIDATE:** RED security tests for traversal, archive bomb, hash mismatch, overwrite conflict, no-hook execution, edited-file-preserving uninstall. Run Rust/browser gates and `npm audit` equivalent through the lockfile workflow.

### Task 9: Enable approved native credential storage

- **ACTION:** Replace the W11 unavailable adapter with a vetted platform credential backend and scoped process injection.
- **IMPLEMENT:** Select and audit one dependency compatible with Rust 1.85 and supported targets. Add macOS Keychain adapter first; add Windows Credential Manager/Linux Secret Service only with native CI evidence. Resolve an opaque `SecretHandle` only after central permission approval, validate project/process/account binding, inject only into an approved child environment, clear afterward, and never serialize values.
- **MIRROR:** `crates/devfoundry/src/credentials.rs` fail-closed trait/fake store; `crates/core/src/secret_bindings.rs` scope validation; W05 environment clearing task record.
- **IMPORTS:** Existing `SecretStore`, `SecretBindingRequest`, `SecretHandle`, platform cfg modules and disposable test credential fixtures.
- **GOTCHA:** A trusted process receiving a secret can exfiltrate it; explicitly document this residual threat. No plaintext fallback or ambient `GITHUB_COPILOT_TOKEN` inheritance.
- **VALIDATE:** macOS native smoke with disposable credential, locked/missing/revoked/cross-project tests, export/log/browser redaction, cancellation cleanup and provider-token exclusion. Native platform failure must close access.

### Task 10: Integrate native terminal and worker projections

- **ACTION:** Expose W13 native PTY through authenticated API and render worker/terminal state in TUI/browser.
- **IMPLEMENT:** Add terminal routes:

```text
POST   /api/v2/projects/{id}/terminals
POST   /api/v2/terminals/{id}/input-lease
DELETE /api/v2/terminals/{id}/input-lease
POST   /api/v2/terminals/{id}/input
POST   /api/v2/terminals/{id}/resize
GET    /api/v2/terminals/{id}/output?after=
POST   /api/v2/terminals/{id}/close
```

  Validate owner/project/session, bounded input/output offsets, gap markers and close acknowledgement. Add a TUI terminal pane with explicit focus escape chord and a browser terminal projection that does not parse terminal bytes as Markdown.
- **MIRROR:** W13 `NativePtyService` methods, W06 versioned routes, TUI event/reducer separation and browser API client.
- **IMPORTS:** Axum routes, binary output response/WebSocket only after origin/auth design; Ratatui terminal-byte rendering dependency selected through existing Cargo conventions.
- **GOTCHA:** Terminal attach is separate from terminal creation. One input lease only. Client disconnect does not implicitly kill the process unless requested.
- **VALIDATE:** API lease/gap/close tests, TUI focus/resize tests, browser reconnect tests, native PTY tests and descendant cleanup.

### Task 11: Add worker board and preview/resource UI projections

- **ACTION:** Make the browser/TUI show task, worker, attempt, evidence, preview and resource state without asserting completion from prose.
- **IMPLEMENT:** Browser: worker cards, dependency blockers, evidence drawer, review action, preview panel, resource install review, status/reconnect indicators. TUI: worker/task/docs/resource panels with keyboard focus, explicit permission status and evidence links. Add fixture APIs where Rust integration is unavailable, then replace with live client calls after Task 5/7/8.
- **MIRROR:** `apps/workspace/src/App.tsx` responsive shell; existing TUI `TuiState` reducer/render tests; W09 evidence-only worker outcome.
- **IMPORTS:** Existing typed DTOs, no direct storage imports.
- **GOTCHA:** A worker “done” message is not acceptance. Display evidence state and blocked reason separately. Do not show secret values or raw command output without bounds/redaction.
- **VALIDATE:** Browser Playwright worker/review/preview/resource flows, TUI reducer tests, narrow desktop/mobile snapshots and reconnect behavior.

### Task 12: Add optional notification bridge

- **ACTION:** Provide opt-in sanitized completion/blocked/failure notifications with revocable pairing; do not provide remote shell or secret approval.
- **IMPLEMENT:** Add a durable bounded outbox with dedup key, retry/backoff, user/chat/project binding, nonce and task revision. Add setup/revoke/status routes and settings UI. Notifications contain task status/deep links only. Restricted replies may acknowledge/cancel a specific current task after local pairing and revision checks; privileged approval remains local.
- **MIRROR:** Existing typed error/API patterns and W09 task revisions; browser settings/security boundary.
- **IMPORTS:** Telegram HTTP client only after dependency/audit approval; disposable fake HTTP server in tests.
- **GOTCHA:** Default disabled. Expired/replayed/wrong-chat messages must not mutate tasks. Never queue secret-bearing payloads or arbitrary command text.
- **VALIDATE:** Duplicate delivery, rate limits, wrong user/chat, replay nonce, stale revision, revoke and secret-redaction tests. Opt-in live smoke only with user-supplied disposable credentials.

### Task 13: Cross-platform qualification and release gate

- **ACTION:** Prove supported-platform behavior and update release/support documentation based on evidence.
- **IMPLEMENT:** Add CI matrix checks for Rust workspace, browser build/E2E, migration/backup/recovery, native process/PTY, MCP/LSP, worktree helper hardening and secretless validation. Run macOS locally; use Linux and Windows runners for runtime evidence. Add migration compatibility and downgrade refusal tests. Verify archive/checksum/browser asset packaging.
- **MIRROR:** `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `scripts/validate-secretless.sh`, `scripts/verify-release.sh`, `release/platforms.json`.
- **IMPORTS:** Existing GitHub Actions/toolchain setup; no signing secrets in CI fixtures.
- **GOTCHA:** `release/platforms.json` currently declares targets as supported; update status only from actual runner evidence. Do not publish, sign, push or alter user databases without explicit authorization.
- **VALIDATE:** Full Rust/browser commands, native Linux/Windows jobs, live Copilot consented smoke, release archive/checksum, secret scan, migration backup/upgrade/downgrade and documented blocked gates.

### Task 14: Final audit/report and handoff

- **ACTION:** Reconcile all task records, comparison docs, support matrix, security docs and create the final completion report.
- **IMPLEMENT:** Update `docs/planning/14-roadmap-and-gates.md`, `docs/OPENCODE_COMPARISON.md`, `docs/PENDING_WORK.md`, `docs/TUI_FEATURE_ROADMAP.md`, `docs/support-matrix.md`, `docs/security.md`, and create `docs/tasks/083-final-qualification.md`. Write `.claude/PRPs/reports/granular-workspace-completion-report.md` with exact commands/results, deferred items and residual risks.
- **MIRROR:** `docs/planning/18-documentation-workflow.md:21-27,40-66` and the existing `.claude/PRPs/reports/granular-workspace-report.md` structure.
- **IMPORTS:** None beyond existing Markdown conventions.
- **GOTCHA:** Do not replace historical audit findings with blanket “complete.” Distinguish implemented, partial, blocked, unsupported and unverified.
- **VALIDATE:** `git diff --check`, inspect all changed files, run final gates, check ignored/generated artifacts, and verify every acceptance row has evidence or an explicit blocker.

---

## Testing Strategy

### Unit and Integration Tests

| Test | Input | Expected Output | Edge Case? |
|---|---|---|---|
| Duplicate scheduler lease | Same task/revision/owner twice | One lease, second conflict | Yes |
| Stale worker revision | Old task revision | 409/conflict; no task mutation | Yes |
| Crash after worker side effect | Attempt started, process interrupted | `OutcomeUnknown`; no automatic replay | Yes |
| Evidence review | Worker evidence | `Review`, never `Accepted` | No |
| Human integration | Wrong target HEAD/digest | Conflict; primary checkout unchanged | Yes |
| Browser bootstrap | Token-shaped config | Response contains no secret | Yes |
| Preview traversal | `../`/symlink asset | Rejected before spawn/serve | Yes |
| Preview malicious page | Page tries privileged API | Separate origin/CSP blocks access | Yes |
| Resource archive | Traversal/symlink/oversized archive | Rejected, no files mutated | Yes |
| Resource uninstall | Edited installed file | Preserve edited file | Yes |
| Credential scope | Wrong project/process/account | Fail closed; no backend value lookup | Yes |
| Terminal lease | Two clients input | One owner; second rejected | Yes |
| Terminal gap | Offset older than ring | Explicit gap result | Yes |
| Notification replay | Same nonce or stale revision | No mutation | Yes |
| Migration compatibility | Existing database copy | New migration preserves old rows | No |
| Windows/Linux process | Native CI runner | Platform-specific cleanup evidence | Yes |

### Edge Cases Checklist

- [ ] Empty task/worker objective
- [ ] Maximum task/evidence/output sizes
- [ ] Duplicate idempotency key with changed payload
- [ ] Concurrent lease and stale revision
- [ ] Dependency accepted but not integrated
- [ ] Worker process exits nonzero
- [ ] Worker output pipe/PTY remains open after child exit
- [ ] Provider cancellation while response body stalls
- [ ] Browser reconnect after replay cursor gap
- [ ] Browser auth/origin/CSRF failure
- [ ] Preview process port collision and readiness timeout
- [ ] Symlink/traversal in resources, documents, previews and worktrees
- [ ] Locked or unavailable credential backend
- [ ] Secret-shaped error/output/export content
- [ ] Notification wrong chat, replayed nonce and revoked binding
- [ ] Native Windows ConPTY unavailable
- [ ] Migration downgrade attempted with incompatible schema

## Validation Commands

### Rust Static Analysis

```bash
/Users/joy/.cargo/bin/cargo fmt --all -- --check
/Users/joy/.cargo/bin/cargo check --workspace --all-targets
/Users/joy/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
```

EXPECT: zero formatting, compile or Clippy errors.

### Rust Tests

```bash
/Users/joy/.cargo/bin/cargo test --workspace
```

EXPECT: all existing and new suites pass, including storage recovery, scheduler, API, tools, native PTY, MCP/LSP and TUI tests.

### Browser Tests

```bash
cd apps/workspace
npm ci
npm run typecheck
npm test
npm run build
npx playwright test
cd ../..
```

EXPECT: typecheck, Vitest, Vite build and desktop/mobile Playwright flows pass. Generated `node_modules`, `dist`, `test-results` and `*.tsbuildinfo` remain ignored.

### Release/Security

```bash
bash scripts/validate-secretless.sh
bash scripts/verify-release.sh
git diff --check
git status --short --ignored
```

EXPECT: metadata is valid, no secret material is detected, artifacts are bounded, and only intended files are tracked.

### Native Platform Validation

- [ ] macOS Apple Silicon: Rust gates, native Unix PTY, Keychain if approved, Copilot smoke.
- [ ] Linux x64: Rust gates, process groups, PTY/terminal, MCP/LSP, worktrees, release archive.
- [ ] Windows x64: Rust gates, Job Objects, ConPTY if enabled, PowerShell/process cleanup, worktrees, release archive.
- [ ] All platforms: migration compatibility and no-secret release checks.

## Acceptance Criteria

- [ ] A worker can be durably leased exactly once for a task revision.
- [ ] A worker can execute in an owned worktree and produce bounded evidence without self-accepting or integrating.
- [ ] Unknown side effects are persisted and never automatically replayed.
- [ ] Browser host serves only approved assets with authenticated non-secret bootstrap and correct CSP/origin/CSRF controls.
- [ ] Browser reconnect reconstructs task/session state from snapshot plus durable events.
- [ ] Preview processes are explicitly approved, isolated from privileged browser APIs, and cleaned up.
- [ ] Resource installs reject traversal/symlink/archive abuse and never execute hooks.
- [ ] Credential resolution fails closed when unavailable and never serializes values.
- [ ] Native terminal API supports input lease, resize, bounded output, gap reporting and cleanup.
- [ ] MCP/LSP/PTY native tests pass on every claimed platform.
- [ ] Optional notifications are disabled by default and cannot authorize privileged actions through replayed messages.
- [ ] Full Rust/browser/release validation passes, or every blocked gate is documented with owner and next action.

## Completion Checklist

- [ ] Code follows existing module/repository/service/API/browser patterns.
- [ ] Error handling is typed, stable and redacted.
- [ ] No ad hoc logging was introduced; no raw secrets are logged.
- [ ] Tests cover failure, cancellation, concurrency and security boundaries.
- [ ] Forward migrations only; old migrations unchanged.
- [ ] Browser uses same-origin credentials and no token storage.
- [ ] Git integration remains separately human-authorized.
- [ ] Cross-platform status matches actual runner evidence.
- [ ] Task records and roadmap gates updated.
- [ ] Final report separates complete, partial, blocked, unsupported and unverified.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Worker side effect occurs before evidence commit | Medium | Critical | Persist attempt before side effect; settle unknown; never auto-replay |
| Browser host becomes a privileged origin | Medium | Critical | Separate preview origin, CSP, auth/CSRF/Host checks, no token bootstrap |
| Trusted command exfiltrates secret | Medium | Critical | Explicit consent, allowlisted injection, redaction, documented non-sandbox boundary |
| Git hooks/helpers execute during worker operations | Medium | High | W08 fixed Git env/options and hostile fixtures; require explicit exception for helpers |
| Preview serves project/secret files | Medium | High | Allowlisted artifact root, symlink rejection, separate origin and tests |
| Resource package executes code on install | Medium | High | Copy-only staged install, no hooks, manifest review and archive security tests |
| Windows/Linux runtime diverges from macOS | High | High | Native CI gates; do not mark support from compilation alone |
| Full scope exceeds one execution context | High | Medium | Wave gates, isolated worktrees, durable task records and explicit stop-loss |
| API snapshot/event race | Medium | Medium | Add storage-owned snapshot watermark before claiming atomic reconstruction |
| External credentials unavailable | High | Medium | Fail closed, retain disabled state and document blocker |

## Notes

- The completed report is authoritative for already-delivered packages; this plan covers only deferred completion work.
- Existing v2 prompt admission returns `501 idempotency_unavailable` until W19 connects the durable host admission bridge.
- W09 scheduler types are policy contracts. W19 must not expose them as restart-safe production behavior until storage-backed leases/evidence pass fault-injection tests.
- W11 secret bindings are metadata-only until W23 adds a vetted native backend.
- W14 browser is standalone until W20 owns authenticated hosting; do not infer production security from fixture tests.
- If any package requires changing a frozen contract, stop, add a contract task and update dependency receipts before continuing downstream work.
