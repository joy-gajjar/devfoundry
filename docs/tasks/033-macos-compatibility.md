# Task: macOS Compatibility Audit

## Goal

Keep the macOS developer and release path reliable without changing provider or API semantics.

## Scope

- Unix process-group cleanup and POSIX shell execution.
- TTY restoration and SIGINT/SIGTERM server shutdown.
- SQLite filesystem paths containing spaces and macOS directory names.
- Project-local configuration and database conventions.
- `aarch64-apple-darwin` release and CI smoke coverage.

## Implementation

- Shell commands use `/bin/sh` on Unix and retain the existing command string and permission boundary.
- SQLite filesystem paths use SQLx filename options instead of raw URL construction.
- Server shutdown responds to both Ctrl-C and SIGTERM on Unix.
- Terminal cleanup is guarded against repeated restoration.
- Added a SQLite path-with-spaces test and a macOS CI smoke job.

## Platform Contract

Configuration remains `devfoundry.json` discovered from the project root. Interactive state remains `.devfoundry.db` in that root. This avoids silently moving existing projects into a user directory; configurable data directories remain separate follow-up work.

The supported release target is `aarch64-apple-darwin` on `macos-14`.

## Verification

Run on macOS:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --target aarch64-apple-darwin
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --target aarch64-apple-darwin -p devfoundry
```
