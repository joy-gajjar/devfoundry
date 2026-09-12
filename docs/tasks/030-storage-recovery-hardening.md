# Task: Storage And Recovery Hardening

## Goal

Make SQLite persistence enforce its declared relationships and make restart recovery observable and testable without replaying interrupted work.

## Scope

- Included: per-connection foreign-key enforcement, atomic startup recovery, restart/migration coverage, permission recovery coverage, and bounded storage diagnostics.
- Excluded: provider, TUI, and tool implementation; automatic prompt/tool replay; destructive backup migration policy.

## Design

SQLite foreign keys are enabled in the connection options so every pooled connection enforces the schema invariant. Startup recovery changes admitted/running prompts to `interrupted` and pending permissions to JSON `cancelled` in one transaction. Diagnostics expose only SQLite health and row counts; no prompt, message, target, or secret contents are exported.

## Implementation

- Hardened `SqliteStore::connect` and `recover_interrupted_work`.
- Added `SqliteStore::diagnostics` for integrity and row-count inspection.
- Added file-backed restart, migration, foreign-key, recovery, and diagnostics tests.

## Verification

- `cargo fmt --all -- --check`: blocked; `cargo` is not installed in this environment.
- `cargo check --workspace --all-targets`: blocked; `cargo` is not installed in this environment.
- `cargo clippy --workspace --all-targets -- -D warnings`: blocked; `cargo` is not installed in this environment.
- `cargo test --workspace`: blocked; `cargo` is not installed in this environment.
- `git diff --check`: passes.

## Risks And Follow-Up

- Portable session export/import remains deferred; safe whole-database backups are tracked in task 035.
- Recovery still requires explicit future resume and never automatically reruns non-idempotent work.
