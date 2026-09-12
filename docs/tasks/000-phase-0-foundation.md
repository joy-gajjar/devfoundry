# Task: Phase 0 Foundation

## Goal

Create the first compilable Rust foundation for the OpenCode reimplementation so later components can depend on stable shared domain types and a minimal executable.

## Scope

- Included: Cargo workspace, schema crate, domain IDs/records/errors, minimal project-directory CLI, and CI checks.
- Excluded: database, providers, tools, permissions, HTTP server, TUI, plugins, and production configuration persistence.

## Design

The workspace starts with two crates: `opencode-schema` contains transport-neutral serializable domain values, and `opencode` contains the executable entry point. IDs are ULID-backed newtypes. The CLI has `--dir`, `doctor`, version, and help behavior. CI validates formatting, compilation, linting, and tests. Full configuration loading is deferred until the Rust toolchain is available for implementation and verification.

## Implementation

- Added root `Cargo.toml` workspace.
- Added `crates/schema` with IDs, projects, sessions, messages, parts, events, statuses, and domain errors.
- Added `crates/opencode` with the initial CLI and project-directory handling.
- Added `.github/workflows/ci.yml`.

## Verification

The repository passed `git diff --check`. Rust verification is blocked because this environment does not provide `cargo`, `rustc`, or `rustup` (`command not found`). Required commands remain:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p opencode -- --help`
- `cargo run -p opencode -- doctor`

## Risks And Follow-Up

- The workspace uses the current stable Rust toolchain; the pinned minimum version should be confirmed against CI policy.
- Configuration loading and project-root discovery are intentionally minimal and need expansion before Phase 0 closes.
- Phase 1 should add protocol/storage crate boundaries before persistence implementation.

Phase 0 is not closed until the Rust commands above pass on a machine with the supported Rust toolchain.
