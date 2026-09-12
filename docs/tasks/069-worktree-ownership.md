# Task: W08 Worktree Ownership And Git Review

## Goal
Provide safe, reviewable Git worktree allocation for mutating workers without allowing repository-controlled Git integrations or implicit primary-checkout mutation.

## Scope
- Included: `WorktreeManager` allocation, status, review, reconcile, and separately authorized integration contracts; owned metadata; fixed-argv Git execution; dirty-primary and exact-target checks; conflict/stale reporting; hostile Git integration fixtures.
- Explicitly excluded: schema/protocol/core/server/storage/LLM/TUI changes, automatic cleanup, reset/stash, push/sync, commit creation in the product repository, and automatic human approval.

## Design
Each allocation requires the central `PermissionBroker` operation `worktree_allocate`, a clean primary checkout, and an exact target HEAD when supplied. The manager creates one named branch and one owned worktree under an external metadata root, records worker ID, branch, target ref, expected target HEAD, base commit, lease, and worktree path, and never stashes or resets primary changes. Metadata roots inside the primary repository are rejected so bookkeeping cannot dirty the checkout.

Git is invoked with a fixed argument vector and a cleared environment. System/global configuration is disabled; optional locks, hooks, fsmonitor, attributes, external diff, and text conversion are disabled for administrative, checkout, and review commands. No shell parsing is used.

Review is non-integrating and returns base/head/diff/evidence digest. Integration requires both the central `worktree_integrate` permission and a separate `HumanIntegrationAuthorization` matching target ref, expected target HEAD, current owned head, and evidence digest. A changed target produces a stale/conflict failure and leaves both checkouts intact. Reconciliation reports owned metadata and preserves missing/stale worktrees for explicit cleanup.

## Threat Model And Permission Rules
- Model arguments and repository configuration are hostile input.
- A worktree tool never self-approves; every operation calls the central broker.
- Dirty primary state is a hard refusal; unrelated user changes are never reset, stashed, or overwritten.
- Review acceptance does not authorize integration.
- Git helpers, hooks, fsmonitor, external diff/textconv, and executable filters are disabled by default; trusted-process exceptions are not implemented in W08 and remain denied/deferred.
- Cancellation is checked while waiting for each Git child; no destructive cancellation cleanup is performed.

## Implementation
- Added `crates/tools/src/worktree.rs`.
- Added the unavoidable `worktree` module export in `crates/tools/src/lib.rs`.
- Added `crates/tools/tests/worktree_lifecycle.rs` with temporary hostile-repository fixtures.

## Verification
- `cargo test -p devfoundry-tools --test worktree_lifecycle`: initial command blocked because `cargo` was not on PATH.
- `$HOME/.cargo/bin/cargo test -p devfoundry-tools --test worktree_lifecycle`: RED as expected before implementation (`E0583`, missing `crates/tools/src/worktree.rs`).
- `$HOME/.cargo/bin/cargo fmt --all -- --check`: passed.
- `$HOME/.cargo/bin/cargo test -p devfoundry-tools --test worktree_lifecycle`: passed, 8 tests.
- Named passing tests: dirty primary preservation, exact expected target HEAD, hostile Git integrations disabled, stale target conflict preservation, separate human integration authorization, cancelled allocation, unknown worktree status, and restart/orphan reconciliation.
- `$HOME/.cargo/bin/cargo check -p devfoundry-tools --all-targets`: passed.
- `$HOME/.cargo/bin/cargo clippy -p devfoundry-tools --all-targets -- -D warnings`: passed.
- `$HOME/.cargo/bin/cargo test -p devfoundry-tools`: passed, 50 unit tests, 5 native integration tests, 8 worktree lifecycle tests, and 0 doc-tests failed.
- `git diff --check`: passed.

## Risks And Follow-Up
- W08 currently records metadata in JSON but does not add durable storage integration; restart reconciliation is bounded to the owned metadata directory and must be exercised by a later host/storage adapter.
- Worktree and metadata paths are intentionally retained; W08 has no cleanup operation.
- `reconcile` preserves stale/missing worktree metadata but does not delete worktrees by design.
- Git command timeout enforcement is not yet shared with the process supervisor; cancellation is supported, while deadline enforcement remains a follow-up.
- The digest is an in-process deterministic evidence identifier, not a cryptographic hash; a cryptographic evidence contract belongs with the versioned protocol/storage owners.
- Platform-specific Git configuration paths and executable-filter fixtures need native Linux/Windows validation before the advanced-runtime gate can close.
