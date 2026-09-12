# Task: Project Audit and Upstream Feature Difference

## Goal

Produce durable reference documentation for known issues, incomplete features, and the feature difference between DevFoundry and upstream OpenCode, so future work can be planned deterministically. This also records the previously-undocumented audit and roadmap work.

## Implementation

- Created the `docs/audit/` reference tree:
  - `README.md` — index, methodology, severity/status legends, snapshot metadata.
  - `01-critical-bugs.md` — high-severity correctness/security findings (bash/pty sensitive-file bypass, JSON tool-call drop, classification-without-retry, over-broad content-filter match, missing message-history endpoint, broken parser test).
  - `02-medium-risks.md` — medium-severity reliability/consistency findings.
  - `03-low-and-cleanup.md` — low-severity issues, dead code, stale artifacts.
  - `04-tui-gaps.md` — TUI roadmap gap analysis and fragilities.
  - `05-docs-and-naming.md` — stale naming, task-number duplication, doc drift.
  - `06-test-coverage.md` — per-crate coverage matrix and missing tests.
  - `07-remediation-roadmap.md` — sequenced fix plan with suggested task numbers and gates.
  - `08-upstream-feature-diff.md` — verified difference vs upstream OpenCode (~30 packages).
- Updated `docs/OPENCODE_COMPARISON.md`, `docs/PENDING_WORK.md`, and `docs/TUI_FEATURE_ROADMAP.md` to link the audit and correct inflated claims.

## Method

- Read-only source review of all nine crates.
- Cross-reference of `docs/PENDING_WORK.md` and `docs/OPENCODE_COMPARISON.md`.
- Upstream inspection of `anomalyco/opencode` `dev` branch README and `packages/` tree.

## Scope

Documentation only. No production code was changed. Findings are documented for later remediation per `docs/audit/07-remediation-roadmap.md`.

## Verification

- All new files render as valid Markdown.
- `git diff --check` passes.
