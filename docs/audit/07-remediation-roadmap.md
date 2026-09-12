# Remediation Roadmap

Sequenced plan to resolve the audit findings. Ordered by risk and dependency. Suggested task-record numbers continue from `061`. Every change must keep the acceptance gates green (see 06-test-coverage).

## Phase A — Security and Correctness (do first)

| Step | Findings | Suggested task | Depends on |
|---|---|---|---|
| A1 Screen sensitive paths/commands in bash/pty | C1 | 062 | — |
| A2 Parse tool_calls in JSON completion path | C2 | 063 | — |
| A3 Restore tool-call fragmentation unit test | C6 | 063 | A2 |
| A4 Fix content-filter matcher + 429 ordering | C4 | 064 | — |
| A5 Wire bounded retry for retryable classes | C3 | 064 | A4 |

Acceptance: new tests reproduce each defect before the fix and pass after; `clippy -D warnings` clean.

## Phase B — API and Persistence Reliability

| Step | Findings | Suggested task | Depends on |
|---|---|---|---|
| B1 Message-history endpoint + `list_messages` pagination | C5 | 065 | — |
| B2 Enable WAL + busy_timeout; transactional prompt admission | M4 | 066 | — |
| B3 Event schema version tag + versioned decode | M5 | 067 | B2 |
| B4 Distinguish interrupt from error status | M3 | 068 | — |
| B5 Durable permission completion; fix service leak | M1 | 069 | — |

Acceptance: crash/restart and concurrency tests cover the transaction and permission paths.

## Phase C — Runner and Provider Robustness

| Step | Findings | Suggested task | Depends on |
|---|---|---|---|
| C1 Explicit loop-limit event; configurable turn cap | M2 | 070 | — |
| C2 Persist usage; wire or remove reasoning deltas | L5 | 071 | — |
| C3 MCP `tools/list`, request cancellation, platform gate | M6 | 072 | — |
| C4 Path normalization + traversal tests | M9 | 073 | — |
| C5 `file_name()` None handling; storage unwrap consistency | M7, M8 | 073 | — |

## Phase D — TUI Product (P0 acceptance closure)

| Step | Findings | Suggested task | Depends on |
|---|---|---|---|
| D1 Structured tool cards with timing/exit code | TUI Item 2, T6 | 074 | — |
| D2 Multi-line input editor; restore draft on failure | TUI Item 6, T2 | 075 | — |
| D3 Permission scope (session) + real cancel + reason | TUI Item 3 | 076 | B5 |
| D4 Session browser delete/export/activity/badges | TUI Item 4 | 077 | B1 |
| D5 Model picker capability/context-window display | TUI Item 5 | 078 | L4 |
| D6 Single-render transcript; focus-safe input; resize in connected loop | T1, T3, T4 | 079 | — |
| D7 Permission event push (remove poll latency) | T5 | 079 | B5 |

## Phase E — TUI Differentiators (P1+)

Context inspector, diff review panel, validation panel, recovery center, transcript search, session tabs, real command palette, cost/token monitor. Sequence after Phase D. Suggested tasks 080+.

## Phase F — Cleanup and Docs Hygiene

| Step | Findings | Suggested task |
|---|---|---|
| F1 Remove `crates/opencode/`, `opencode.db`; add `*.db` to `.gitignore` | A1, A2, A3 | 081 |
| F2 Fix CLI naming in `planning/10-tui-and-cli.md` and stale task docs | 05-docs-and-naming | 081 |
| F3 Reconcile catalog drift; correct safety claim and roadmap capabilities | 05-docs-and-naming | 081 |
| F4 CLI error handling instead of `expect()` | L1 | 082 |
| F5 SSE lag resync; request_id consistency | L2, L3 | 082 |

## Non-Goals (do not implement without a product decision)

- Additional providers (Anthropic, Gemini, Bedrock, local OpenAI-compatible).
- Web UI, desktop app, VS Code extension, Slack, enterprise/identity.
- Generated public SDKs.

See 08-upstream-feature-diff for the full non-goal rationale.

## Global Acceptance Gates

```bash
cargo fmt --all
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```
