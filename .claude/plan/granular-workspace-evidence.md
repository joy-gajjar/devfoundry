# Granular Workspace: Evidence And Design Brief

Snapshot: 2026-09-12. Planning only. No fixes, tests, product downloads, account actions, or live-provider calls were performed for this investigation.

## Approved Scope

The user selected TUI plus an optional browser workspace, and GitHub Copilot only. Preserve the Rust agent engine. Native desktop packaging and external Claude/Codex CLI adapters are not part of this delivery. This is an independent implementation of useful workflows, not a copy of Granular branding, assets, or proprietary code.

## Research Limits

- Granular evidence is public product documentation, not verified application behavior or source access. Marketing claims about absolute secret safety or conflict-free work are not engineering guarantees.
- The requested ace-tool MCP is unavailable. The requested `~/.claude/bin/codeagent-wrapper` directory is absent. No Codex/Antigravity calls or external session IDs exist.
- Two available read-only specialists supplied backend and UX reviews. These are independent review tasks, not verified independent models.
- Local graph: `Users-joy-Projects-OpenCode-Rust`, generation `2026-09-12T17:17:44Z`. Material source paths had matching metadata. `migrations/0001_initial.sql` had a parse gap at line 30; the full file was read. Clean index coverage is best-effort, not proof of completeness. Suspect graph caller edges were not used as source evidence.
- Git reports no valid HEAD; project files are untracked. Old green test results are historical, not validation of this plan or the current tree.

## Granular Sources And Feature Mapping

All URLs below were retrieved on the snapshot date. Individual core guides display updated dates of 2026-06-29; Windows guide displays 2026-09-04.

| ID | First-party source | Documented feature | DevFoundry delivery |
|---|---|---|---|
| G01 | https://www.granular.build/docs/the-interface/ | Chat, terminal, live preview, docs; file attachments; rearrangeable panes | TUI equivalents plus responsive browser workspace; bounded text/image attachments and explicit unsupported-format messages |
| G02 | https://www.granular.build/docs/boss-and-worker-agents/ | Boss delegates to visible workers in separate worktrees | Durable task DAG, bounded workers, visible evidence, review before integration |
| G03 | https://www.granular.build/docs/branch-sessions-git/ | Branch sessions; guarded commit, push, merge, sync | Worktree ownership, Git diff/review and explicit operation approvals; no automatic push |
| G04 | https://www.granular.build/docs/digital-brain/ | Linked Markdown brain, wikilinks, roadmap board, navigable map | Adopt existing docs; lexical search/backlinks/map; revision-aware task board linked to sessions |
| G05 | https://www.granular.build/docs/the-vault/ | Encrypted credentials, project enablement, environment injection | OS credential store, scoped references and explicit per-process injection; never promise arbitrary commands cannot leak secrets |
| G06 | https://www.granular.build/docs/resource-library-skills/ | Skills/templates/brains/MCP presets; copy-only install; preserve edited files on uninstall | Manifest/hash-based library, preview installation, no hooks, safe uninstall, configured registry support |
| G07 | https://www.granular.build/docs/mcp-connectors/ | Multi-account connectors, OAuth and verified presets | Standard stdio, then remote MCP with scoped vault accounts and OAuth lifecycle |
| G08 | https://www.granular.build/docs/connect-an-llm/ | BYO Claude/Codex, model switching, hosted add-on | Intentionally different: Copilot only; capability-aware model picker, no hosted billing |
| G09 | https://www.granular.build/ | Saved prompts, screenshot/annotation examples, Telegram notifications/control, marketplace | Saved prompts/resources, preview screenshots with approval, opt-in Telegram notifications followed by narrowly scoped replies |
| G10 | https://www.granular.build/docs/granular-on-windows/ | Windows beta, PowerShell, Ctrl shortcuts, unsigned-update limitations | macOS/Linux/Windows validation matrix; do not inherit unsigned-install guidance |

### Source Contradictions

- Homepage says Windows is not available; the newer Windows guide describes a beta. Record the dated guide and conflict, not a blanket Mac-only claim.
- Resource guide calls a community marketplace planned; homepage shows a marketplace. Local resource installation is clear; creator commerce maturity was not established.
- Homepage shows broader engines while FAQ says some are upcoming. None changes the approved Copilot-only scope.
- Vault documentation relies in part on agents being instructed not to print values. Encryption at rest and output redaction do not prevent deliberate exfiltration by a credential-bearing process.

## Revalidated Local Findings

Source-confirmed means the behavior follows from inspected code; reproduction tests remain required. Priorities here supersede the old audit for this plan, not by silently modifying it.

| ID | Priority | Evidence | Finding / verification target |
|---|---|---|---|
| R01 | P0 | `crates/core/src/lib.rs:135-155,174-189,231-248` | Shared DB permission settlement happens before service ownership check; request event is published before waiter registration. Test two sessions and immediate approval deterministically. |
| R02 | P0 | `crates/core/src/lib.rs:268-277,352-359,426-429`; `crates/llm/src/lib.rs:14-18` | Previous conversation is not loaded; assistant calls and tool results lack required wire correlation. Preserve opaque provider IDs and load durable history. |
| R03 | P0 | `crates/tools/src/lib.rs:983-990,1014-1033` | Exit status is discarded and output joins lie outside execution deadline. Nonzero exits must not become successful validation; descendants retaining pipes must not hang. |
| R04 | P0 | `crates/server/src/lib.rs:594-659` | Admission/status/event writes are separate; failures can strand registry ownership; terminal writes are ignored. Transactional lifecycle and joined supervision required. |
| R05 | P0 | `crates/core/src/lib.rs:352-359,410-445` | Tool facts/results are held until final assistant settlement. Crash or later provider failure can lose side-effect evidence. Add durable attempts and outcome-unknown state. |
| R06 | P0 | `crates/tools/src/lib.rs:129-161` | Resolving missing nested directories drops intermediate components and may address the wrong file. Preserve the entire missing suffix. |
| R07 | P0 | `crates/tools/src/lib.rs:665-696,731-742,477-483,943-962` | Sensitive checks use input names; search follows file aliases; approved shell is host authority with inherited environment, not a sandbox. Test symlink aliases and outside-root search. |
| R08 | P0 | `crates/llm/src/lib.rs:221-250`; `crates/core/src/lib.rs:310` | Cancellation does not independently interrupt stalled body/stream waits. Bound requests, streams, buffers, and cleanup. |
| R09 | P0 | `crates/tools/src/mcp.rs:287-337` | Content-Length framing differs from standard MCP stdio newline framing. Existing same-protocol fixture is insufficient interoperability evidence. |
| R10 | P0 | `crates/tui/src/lib.rs:1274-1292,1309-1336,1355-1360` | HTTP blocks UI loop; pending permission swallows global cancellation; rejected admission loses editable draft. |
| R11 | P1 | `crates/tui/src/client.rs:167-185` | Per-chunk lossy UTF-8 decoding corrupts split code points; first-data-line and rigid id parsing; unbounded buffer. |
| R12 | P1 | `crates/tui/src/lib.rs:174-203,350-355,412-420,641-681,739-740` | Top/bottom scroll semantics mismatch, duplicate layout work, scalar-width wrapping and code whitespace loss. |
| R13 | P1 | `crates/server/src/lib.rs:796-873`; `crates/storage/src/lib.rs:581-587,688-702` | Replay exists but bounded snapshot/history APIs and recoverable replay-gap behavior are incomplete; SQL reads precede in-memory limits. |
| R14 | P1 | `crates/llm/src/lib.rs:265-326,690-724` | JSON completions omit tool calls; content-filter substring is too broad; test has live and unreachable assertions. |
| R15 | P1 | `crates/core/src/lib.rs:286,361`; `crates/server/src/lib.rs:639-651`; `crates/storage/src/lib.rs:236-247` | Silent turn limit, discarded usage, cancellation/status ambiguity, stale restart session projections. |
| R16 | Gap | `crates/tools/src/lsp.rs:208-257`; `crates/tools/src/lib.rs:612-654` | LSP drains output and accepts injected diagnostics; PTY is pipe-backed. Real integrations are new work, not completed capabilities. |

MCP standard checked: https://modelcontextprotocol.io/specification/2024-11-05/basic/transports (stdio uses newline-delimited JSON-RPC). Choose and pin a current supported remote-transport version at implementation; do not confuse LSP Content-Length framing with MCP stdio.

## Corrections To The Previous Audit

| Prior assertion | Correct disposition |
|---|---|
| Durable replay absent | Rejected; server replay/live delivery and a contract test exist. History reconstruction still needs work. |
| Permission failure is sequential service replacement | Not established as described. R01 gives concrete cross-session routing and publication races. |
| Request ID body/header mismatch | Rejected for inspected normal middleware path: it rewrites the body ID; matching-ID test exists. |
| MCP has no ID correlation/platform gate | Rejected; `mcp.rs:340-350` checks IDs and module is platform gated. Framing/discovery/cancellation remain problems. |
| No provider HTTP integration tests | Rejected: TCP fixtures exist inside the unit-test module. Test location is not test type. |
| Zero unit tests proves poor coverage | Rejected; count is not branch or behavioral coverage. No measured coverage percentage available. |
| No SQLite busy timeout | Unverified: absence of explicit configuration does not establish dependency defaults. Inspect effective pragmas before changing them. |
| Path file_name unwrap is reachable with root input | Not demonstrated; earlier public validation rejects missing filenames. |
| No retry is necessarily a defect | It is an explicit prior decision. Add retries only after safe attempt boundaries exist. |
| Delete old DBs/rename .opencode automatically | Rejected. Preserve user data and intentional harness configuration. |
| Parser test asserts nothing | Overstated: live assertions precede an early return; restore exact fragmentation checks. |
| TUI restoration absent | Rejected broadly: cleanup callback/panic tests exist; real native-terminal signal tests remain necessary. |

## Design Decisions For This Plan

1. Retain the nine-crate architecture and extract focused modules only where ownership or testing requires it. Add a shared client crate when browser/TUI protocol reuse warrants it; do not begin with a workspace-wide rewrite.
2. One supervised Rust application host is authoritative in embedded and server modes. HTTP/TUI/browser are adapters, not competing schedulers.
3. SQLite owns execution and task state. Markdown owns human-authored brain documents. Roadmap Markdown is an explicit export/import with revisions, not a competing writable task database.
4. Browser companion is a new React/TypeScript/Vite client built separately, served locally by the Rust host. TUI and browser consume the same versioned API. Desktop wrapper is deferred.
5. Worktrees separate edits, not privilege. Arbitrary shell always requires explicit trusted-host-command permission; OS sandbox profiles are optional additional defenses, never implied by path checks.
6. Secrets use platform credential stores, with no plaintext fallback. Secret-bearing arbitrary commands are inherently trusted. Neither worker text nor brain files can grant credentials or permissions.
7. A deterministic scheduler, not an LLM assertion, enforces task dependencies, attempts, budgets, review, and completion.
8. Full scope includes local resources, optional remote connectors, and opt-in notification bridge. Hosted model resale, paid marketplace commerce, public deployment/sharing, and external CLI agents are excluded.

## Review Perspectives

Backend and UX reviewers agree on lifecycle correctness before workers, typed identity-aware events, nonblocking client effects, and worktree ownership. Backend favored a new runtime-contracts crate; this plan chooses smaller module extraction first. UX favored browser-before-desktop; user selected that option. Both reject treating worktrees as security isolation.

Backend task: `ses_f695f0f3bffefTxbXsqtyap3o0`. UX task: `ses_f695f0f20ffeG2KObQDMKLeLCY`. These are local task handles, not Codex/Antigravity sessions.

## Planning Verification And Self-Review

Adversarial reviewer `ses_f6951df85ffell6PVK7sj1wKyB` raised seven concrete plan defects. Corrections distinguish fixture versus integration gates, require dependency integration receipts, restrict executable Git helpers, separate human integration authority, connect attachments to admission/provider requests, complete lifecycle routes, and qualify vault guarantees. Follow-up review found no remaining blocking contradictions; W01 still must turn proposed contracts into executable fixtures.

The three plan files exist under `.claude/plan/`. Each was checked using `git diff --no-index --check /dev/null <file>` because ordinary `git diff --check` omits untracked content. These checks validate whitespace, not product correctness. Production tests/builds were not run and no production files were edited.

| Self-review axis | Score | Evidence and limit |
|---|---|---|
| Accuracy | 4/5 | Direct source and official documentation support findings; no runtime reproductions or Granular installation verification |
| Completeness | 4/5 | Approved TUI/browser/Copilot scope, feature mapping, security, dependencies and gates covered; requested external dual-model runtime absent |
| Clarity | 4/5 | Separate evidence/contracts/execution files; substantial scope still requires reading all three |
| Actionability | 4/5 | Nineteen owned packages, named regression cases, paths and commands; W00 baseline and W01 concrete contract freeze remain explicit prerequisites |
| Conciseness | 4/5 | Tables and package boundaries reduce repeated context; some safety invariants intentionally recur in standalone handoffs |

Overall: 4.0/5. Most useful execution-time improvements are runtime reproduction of P0 findings, executable W01 fixtures, and native platform qualification. This assessment does not certify the product as ready. A reader should treat this as a reviewed campaign blueprint, not already implemented software.
