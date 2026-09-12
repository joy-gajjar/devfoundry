# Task: Sensitive Files And Patch Rollback

## Goal

Prevent built-in tools from exposing or mutating common credential files by default, and ensure a multi-file patch failure leaves every target unchanged.

## Scope

- Included: default sensitive-path policy, sensitive-file denial before broker approval, search exclusion, staged multi-file patch replacement, rollback tests, and documentation.
- Excluded: provider, TUI, release, persistent policy configuration, and OS keychain integration.

## Design

The default broker denies path components commonly used for credentials (`.env`, credentials, secrets, private-key extensions, and identity-key names) while allowing ordinary project paths. Read, write, edit, patch, glob, and grep apply the same path classification; an allow-all broker cannot override the built-in sensitive-path boundary. A patch validates all changes, stages replacement content, moves existing targets to unique sibling backups, installs staged files, and restores backups on any failure.

## Implementation

- Updated `crates/tools/src/lib.rs` with `DefaultPermissions`, sensitive-path checks, search filtering, duplicate/empty patch rejection, and rollback-safe patch installation.
- Added tests for default denial, broker-independent secret denial, invalid patches, replacement failure, delete failure, and no partial mutation.
- Updated tool/security planning and roadmap notes.

## Verification

- `cargo fmt --all -- --check`: blocked; Cargo and rustc are unavailable in the environment.
- `cargo check -p devfoundry-tools`: blocked; Cargo and rustc are unavailable in the environment.
- `cargo test -p devfoundry-tools`: blocked; Cargo and rustc are unavailable in the environment.
- `cargo clippy -p devfoundry-tools --all-targets -- -D warnings`: blocked; Cargo and rustc are unavailable in the environment.
- `git diff --check`: passed.

## Risks And Follow-Up

- The sensitive-file list is conservative and filename-based; configurable patterns and OS keychain integration remain future work.
- Backup-and-rename rollback is not a kernel transaction and cannot guarantee recovery from simultaneous hostile filesystem changes.
