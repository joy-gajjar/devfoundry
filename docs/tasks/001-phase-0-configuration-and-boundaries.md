# Task: Phase 0 Configuration And Phase 1 Boundaries

## Goal

Complete the missing Phase 0 configuration/project-root foundation and establish protocol/storage crate boundaries for the next implementation phase.

## Scope

- Included: JSON configuration loading, ancestor project-root discovery, `protocol` and `storage` crate skeletons, repository traits, and tests.
- Excluded: SQLite implementation, HTTP transport, environment precedence, secrets, migrations, and session execution.

## Design

The initial config format is `opencode.json` at the discovered project root. Missing configuration is valid and produces defaults; malformed configuration is reported by `doctor`. Root discovery searches the requested directory and its ancestors for `.git` or `opencode.json`.

`opencode-protocol` owns transport-neutral request/error DTOs. `opencode-storage` owns repository traits and does not expose a database connection. The executable depends on both only to validate the initial workspace composition; business behavior remains deferred.

## Implementation

- Added `crates/protocol` with session creation, prompt, and error DTOs.
- Added `crates/storage` with project/session repository traits.
- Added CLI config parsing and ancestor root discovery.
- Added configuration and root-discovery tests.
- Added test-only JSON dependencies required by schema and protocol round-trip tests.

## Verification

Local Rust verification remains blocked because `cargo`, `rustc`, and `rustup` are unavailable. Required commands:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p opencode -- doctor`

`git diff --check` passes.

Static review also corrected test dependency declarations for `serde_json` in the schema and protocol crates.

## Risks And Follow-Up

- Configuration currently supports only JSON and a small subset of fields.
- The CLI currently prints configuration diagnostics but does not construct an application runtime.
- Phase 1 must add environment/CLI precedence, SQLite migrations, concrete repositories, and transactional event persistence.
