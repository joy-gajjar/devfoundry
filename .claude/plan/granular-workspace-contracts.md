# Granular Workspace: Proposed Contracts

Design for [the implementation plan](granular-workspace.md). These are proposed contracts, not existing APIs. Contract owner freezes the Rust signatures and JSON examples in W01 before dependent agents edit consumers.

## Runtime And Persistence

Reuse existing typed project/session/message/tool IDs. Add typed `RunId`, `AttemptId`, `TaskId`, `WorkspaceId`, `ArtifactId`, `TerminalId`, `SecretRef`, `Revision`, `Cursor`. Preserve the provider's opaque call ID as a string distinct from internal `ToolCallId`.

```rust
// Proposed shape; each name is owned by W01/W02.
enum RunOutcome { Completed, Failed, Cancelled, Interrupted, OutcomeUnknown }
enum TaskStatus { Draft, Ready, Assigned, Running, Blocked, Review, Accepted, Cancelled }
enum EffectDisposition { None, Known, Unknown }
struct ExecutionBudget {
    max_turns: u32, max_tool_calls: u32, max_output_bytes: u64,
    deadline_ms: u64, max_provider_attempts: u32,
}
struct SubmitPrompt {
    session_id: SessionId, idempotency_key: String, text: String,
    expected_session_revision: Revision,
    attachments: Vec<AttachmentRef>,
}
struct AttachmentRef { artifact_id: ArtifactId, sha256: String, media_type: String }
struct Admission { run_id: RunId, revision: Revision, cursor: Cursor }
struct ToolResult {
    call_id: ToolCallId, provider_call_id: String, exit_code: Option<i32>,
    outcome: RunOutcome, output: String, truncated: bool,
    effect: EffectDisposition, artifacts: Vec<ArtifactId>,
}
```

Host methods: `submit_prompt(command) -> Admission`, `cancel_run(run_id) -> acknowledgement`, `resolve_permission(request_id, expected_revision, decision) -> settlement`, `snapshot(session_id) -> snapshot_with_watermark`. All return typed application errors. Cancellation acknowledgement is not proof the process has exited.

Attachment references participate in the idempotency payload. Admission validates immutable hash, owning project, actual detected media type, count/size quota and captured model capability. Authorized text becomes bounded attributed context; supported images become explicit Copilot image-content blocks with exact wire fixtures. Reject unsupported media/capability with a typed 422 response before admission, not after silently dropping context. Artifact access uses opaque authorized references, not arbitrary remote URLs.

Storage transaction groups admission + captured session configuration + user message + event, and separately every state transition + relevant attempt/result + event. Recover interrupted runs without replaying uncertain side effects. One host owns a database; second owner fails clearly. A supervised dispatcher observes durable admitted rows, so in-memory wakeup loss cannot lose work.

Version migrations add run/attempt records, revisions, idempotency keys and payload version; subsequent migrations add workspaces/tasks/worktrees, artifact metadata and resource metadata. Do not modify `0001_initial.sql`. The storage owner assigns unique migration numbers. Legacy facts keep a version-0 decoder; never fabricate missing tool evidence. Backup and schema-compatibility check precede upgrades; executable rollback is not database rollback.

## Safe Execution Pseudocode

```text
submit(command):
  validate size, project scope, configuration revision and idempotency key
  transaction:
    return existing admission if same key and same payload
    reject same key with different payload
    reject active run for session
    persist run + user message + captured policy/model + admitted event
  wake supervised dispatcher (durable queue remains source of truth)

execute(run):
  load bounded chronological history with complete tool-call/result pairs
  for attempt within run budget:
    persist provider attempt started
    select stream next / cancellation / deadline
    validate terminal marker, byte limits and tool arguments
    for requested tool:
      persist requested call and exact provider_call_id
      request exact capability permission using request-ID broker
      atomically observe pending->allowed/denied/cancelled
      persist tool attempt started BEFORE invoking side effects
      execute with cleared environment + allowed bindings + deadline
      persist exit status/result/truncation BEFORE continuation
    if explicit final completion: settle run completed
  on exhausted budget: settle explicit failed/limit reason
  on unrecorded side effect after crash: settle outcome_unknown
  always join children and settle owned pending permissions
```

Approval waiting uses durable state plus notifications: register subscriber first, reread durable request state, wait, and reread after each notification. Routing validates run/session ownership before any mutation. Polling may reconcile state but cannot reopen a deliberately dismissed modal for the same request revision.

Retries default to at most 3 provider attempts per safe pre-output request, bounded jitter/backoff and Retry-After within overall deadline. Never automatically retry tool side effects or resume partially accepted provider output. Validate this proposed policy with fixtures before enabling it.

## Public API

Keep v1 clients functional. Introduce `/api/v2` for incompatible envelopes and state representations; remove nothing without a separate compatibility decision.

| Proposed route | Contract and authorization |
|---|---|
| `POST /api/v2/sessions/{id}/runs` | Idempotent admission, 202 receipt; 409 busy/revision/key mismatch |
| `GET /api/v2/runs/{id}` | Authoritative outcome/attempts/budget; 404 outside accessible project |
| `POST /api/v2/runs/{id}/cancel` | Idempotent cancellation acknowledgement |
| `GET /api/v2/sessions/{id}/snapshot` | Projection and watermark from one read transaction |
| `GET /api/v2/sessions/{id}/messages?cursor=&limit=` | Keyset page, stable order, byte/count limits applied before materialization |
| `GET /api/v2/sessions/{id}/events?after=` | Versioned durable envelope; no terminal byte firehose |
| `POST /api/v2/permissions/{id}/decision` | Expected revision, exact grant hash, allow-once/deny; session grant only after enforced backend support |
| `POST/GET /api/v2/workspaces/{id}/tasks` | Revision-aware DAG records; dependencies and acceptance criteria |
| `POST /api/v2/tasks/{id}/assign` | Task revision, agent profile, base commit, budgets, permitted scope |
| `POST /api/v2/tasks/{id}/review` | Evidence digest, approve/reject; cannot be self-approved by worker |
| `POST /api/v2/worktrees/{id}/integrate` | Reviewed base/head/evidence hashes and expected target HEAD; conflicts stop without mutating user checkout |
| `GET /api/v2/projects/{id}/documents` | Paginated metadata/search/backlinks; contained paths |
| `PUT /api/v2/projects/{id}/documents/{document_id}` | Content with expected hash; stale write returns 409 |
| `POST/GET /api/v2/projects/{id}/artifacts` | Bounded upload/metadata; opaque IDs; hostile formats remain untrusted |
| `GET/POST /api/v2/projects/{id}/resources` | Preview/install manifests with hash and permission requirements; no install hooks |
| `POST/GET /api/v2/projects/{id}/terminals` | Approved native terminal creation and status |
| `WS /api/v2/terminals/{id}/stream` | Authenticated input lease, resize, bounded binary output, offset/gap markers |
| `POST/GET /api/v2/projects/{id}/previews` | Approved process/port/artifact ownership and health |
| `GET/POST /api/v2/projects/{id}/connectors` | Account-scoped MCP config; no raw credentials in responses |
| `GET/POST /api/v2/projects/{id}/secret-bindings` | Opaque metadata and enablement; credential entry through trusted local setup |

W01 also freezes these action contracts before UI fixture work:

| Action route | Required semantics |
|---|---|
| `GET /api/v2/projects/{id}/documents/{document_id}` | Authorized bounded content, media type and content hash |
| `PATCH /api/v2/tasks/{id}` | Expected revision, editable fields/dependencies; reject cycles and stale writes with 409 |
| `POST /api/v2/terminals/{id}/input-lease` | Exclusive owner and expiry; reject another active owner with 409 |
| `DELETE /api/v2/terminals/{id}/input-lease` | Revoke own lease idempotently; administrator revocation audited |
| `POST /api/v2/terminals/{id}/close` | Idempotent supervised termination request; query terminal state for actual exit |
| `POST /api/v2/previews/{id}/stop` | Stop only owned process; detach does not imply stop |
| `POST /api/v2/previews/{id}/screenshots` | Explicit approval, bounded capture, artifact/hash result |
| `POST /api/v2/previews/{id}/annotations` | Text/coordinates/artifact hash; stale revision returns 409 |
| `POST /api/v2/resources/{id}/{enable,update,remove}` | Revision and installation digest, conflict-preserving operations; install never grants capability |

Every UI action must have a command/query, error example and permission rule in W01's fixtures. Resource update/remove is project scoped through its persisted ownership; inaccessible IDs return 404.

All resources are project scoped. Human commands and worker capabilities have separate principals. Browser mutations require exact-origin checks, anti-CSRF controls and authenticated sessions even on loopback. Tokens never appear in URLs, preview frames or persisted browser storage. Terminal WebSockets validate origin and authorize attachment separately from creation. Short-lived attach capabilities are passed after authenticated connection, not query strings.

Event envelope: `{cursor, version, project_id, session_id, run_id, aggregate_revision, kind, payload}`. SQLite's global sequence may have gaps for a particular session. Live broadcast only signals available data; read durable pages in DB order. Unknown kinds are surfaced without losing cursor progress; incompatible required versions trigger an upgrade error. Snapshot watermark then replay avoids racing history/live streams.

## Budgets And State Limits

Initial proposed defaults: 4 concurrent workers per workspace, 8 active provider streams per host, 100 queued tasks, 1 MiB prompt/paste, 1 MiB retained output per tool with explicit truncation, 10 MiB terminal ring per session, 20 MiB single attachment, 100 MiB upload total per workspace request batch. Defaults are configurable downward and measured before release; global quotas prevent multiplying per-session limits. Event pages default to 100 rows, maximum 500 and 2 MiB encoded response. Larger individual business events are rejected or moved to artifacts.

Task scheduler validates a DAG, leases one task revision to one worker, and never runs more than one mutating worker in one worktree. Worker sessions cannot recursively spawn workers by default. Boss budgets include all descendants. Two repeated equivalent failures block the task; exhaustion requires human review rather than unlimited repair loops.

```text
schedule(workspace):
  ready = tasks whose dependencies are Accepted AND have integration receipts
          and whose task revision is current
  while capacity and ready:
    base = approved integration commit containing every dependency receipt
    verify ancestor relationships; block if dependency changes are not present
    atomically lease task; allocate owned worktree at base
    start Copilot worker with immutable brief and capability budget
  on worker result:
    persist evidence + changed paths + base/head + actual command outcomes
    move task to Review, never directly to Accepted
  on review approval:
    mark evidence reviewed; DO NOT mutate Git or self-authorize integration
  on separate human-authorized integrate command:
    validate target ref + expected HEAD + proposed head + evidence digest
    validate integration in owned staging worktree
    advance approved target only if unchanged; otherwise re-review
    persist integration receipt (task revision, resulting commit, evidence digest)
```

Acceptance of a report is separate from authorization to mutate the target branch. Scheduler and reviewer principals cannot mint integration permission. For non-code tasks, an approved immutable artifact receipt satisfies dependencies instead of a Git commit; the dependent brief must include that artifact hash. For code tasks, a review-approved but unintegrated/conflicted task does not satisfy dependent scheduling.

Managed Git disables hooks, fsmonitor, external diff/textconv and unapproved executable filters/helpers. Enumerate effective repository/worktree/global Git configuration before any checkout or review operation. Operations requiring an executable filter or helper become explicit trusted-process approval requests; do not silently bypass content semantics. Hostile repository fixtures cover smudge/clean filters, fsmonitor, diff helpers and hooks. Worktree allocation itself is not exempt from this policy.

## Brain, Resource And Credential Rules

- Brain documents remain Markdown with provenance and wikilinks. Indexes are rebuildable. Existing planning/audit/task files are linked, not overwritten by generated `BRAIN.md`. External and worker-authored notes are untrusted proposals until promoted.
- Task authority stays in SQLite. `roadmap.md` exports include task IDs/revisions; import detects conflicts and requires explicit confirmation. Prevent sync loops using content hashes and export revision.
- Resource manifest includes ID, version, publisher/source, file hashes, target paths, required capabilities and license. Installation stages files, rejects traversal/symlinks/oversized archives, displays diff, then copies approved content. No code execution, auto-trust or auto-credential enablement. Uninstall removes only files matching installed hashes; edited files remain.
- Use macOS Keychain, Windows Credential Manager and Linux Secret Service via a vetted Rust adapter. When unavailable/locked, fail closed for vault-dependent actions and offer manual scoped ephemeral input, never plaintext storage. Existing `GITHUB_COPILOT_TOKEN` handoff remains supported. Environment inheritance is allowlisted; the provider token is not forwarded to arbitrary worker shells.
- Secret metadata, scope and access audits are durable; values are not. Prefer trusted connector code to arbitrary shell injection. If a user approves secret-bearing shell, explain that the process can read/exfiltrate it; output masking is defense-in-depth, not containment.
- Verification separates host-controlled leakage from approved-process behavior. Assert no raw credential serialization in host records/exports/API responses, and test direct known-value redaction across output chunks. Do not claim redaction defeats encoded, transformed or out-of-band exfiltration by approved credential-bearing commands.
- Preview origin is separate from app/API, sandboxed without privileged same-origin access. Never serve a whole project root or follow symlinks for previews. Explicitly approve network-capable preview processes; public tunnels and publishing are excluded.
- Telegram is opt-in. Start with sanitized status notifications. Later responses bind to user/chat/project/request/revision, expire, are single-use and cannot disclose secrets or authorize privileged secret grants. No remote shell endpoint.

## Client Interaction Contract

Reducer receives events and emits typed effects; network/process work never blocks input/render. Model selection is captured at admission. Track API and stream connectivity separately, preserve drafts on rejected/uncertain admission and retry using the same idempotency key.

TUI focus states: editor, transcript, picker, permission, palette, terminal. Permission focus cannot intercept global cancellation. Esc dismisses only; cancel-run is separately labeled. Enter submits; Ctrl-J inserts newline; configurable alternatives support terminals without modified Enter. In editor, arrows move the cursor; transcript navigation is explicit. History anchoring uses message ID plus row, not an inverted magic offset. Byte streams decode incrementally; layout uses terminal-cell widths without altering code text.

Browser desktop layout: project/task/session rail, chat center, optional terminal below, preview/docs side panel. Narrow/mobile layout: single active surface with explicit tabs and persistent status/approval indicator; no cramped four-column miniature. Keyboard navigation, text labels independent of color, reduced animation, high contrast and readable errors are mandatory. Reuse current DevFoundry visual language rather than Granular branding.
