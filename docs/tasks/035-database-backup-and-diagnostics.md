# Task: Database Backup And Diagnostics

## Goal

Provide safe local SQLite backups and bounded diagnostics without changing provider, TUI, or API behavior.

## Implementation

- Added `devfoundry database backup --database PATH --output PATH`.
- Backups use SQLite `VACUUM INTO`, so live database files and WAL state are not copied directly.
- Backups are written to a temporary sibling and published with a same-filesystem link; existing destinations are never overwritten.
- Added `devfoundry [--json] database diagnose --database PATH` with foreign-key, integrity, and row-count diagnostics only.

## macOS Contract

Filesystem paths are passed through SQLx filename options and backup paths are handled with native `PathBuf` operations, including paths containing spaces.

## Verification

Run:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Portable session export/import remains separate because the current storage contract stores provider and tool payloads as durable session facts rather than a versioned interchange format.
