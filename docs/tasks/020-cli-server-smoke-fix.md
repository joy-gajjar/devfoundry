# Task: CLI And Server Smoke Fix

## Goal

Correct the documented CLI version invocation and make relative SQLite database paths reliably open during server startup.

## Scope

- Included: Cargo argument clarification and absolute SQLite path normalization.
- Excluded: database migration redesign and remote database support.

## Design

Cargo consumes arguments before the binary, so binary flags require the `--` separator. Relative database paths are resolved against the current directory, parent directories are created if needed, and the resulting SQLite URL is absolute with SQLite read-write-create mode enabled.

## Implementation

- Updated runtime database URL normalization.
- Documented the correct version command.
- Added SQLite `mode=rwc` for first-run database creation.

## Verification

Run:

- `cargo run -p opencode -- --version`
- `cargo run -p opencode -- doctor`
- `cargo run -p opencode -- serve`
- `curl http://127.0.0.1:4096/health`

Verified with a fresh database and port:

- `cargo build -p opencode`
- `target/debug/opencode serve --bind 127.0.0.1:4109 --database /tmp/opencode-rust-smoke.db`
- `curl http://127.0.0.1:4109/health`
- Response: `{"status":"ok","version":"0.1.0"}`

## Risks And Follow-Up

- The default provider is still the fake provider.
- Database file permissions still depend on the selected directory being writable.
