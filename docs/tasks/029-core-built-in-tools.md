# Task: Core Built-In Tools

## Goal
Complete the core project tools so users can safely edit, patch, search, and run commands from the standard tool registry.

## Scope
- Included: stale-aware `edit`, constrained `apply_patch`, bounded project-scoped `glob` and `grep`, and explicit standard `bash` registration.
- Excluded: provider/API/TUI/release work, persistent policy storage, and process-group cleanup.

## Design
All mutating tools resolve paths through the existing project-root and symlink checks and use atomic file replacement. Edit requires an exact old string and rejects missing or ambiguous matches unless `replace_all` is requested. Patch parsing and content application complete before mutation, so malformed or stale patches leave targets unchanged. Search skips `.git` and `target`, limits traversal to 10,000 files and results to 1,000, and grep ignores binary and files larger than 4 MiB. Bash is a normal registered tool and remains explicitly authorized through the central permission broker.

## Implementation
- Updated `crates/tools/src/lib.rs` with `edit`, `apply_patch`, `glob`, `grep`, and `bash` tools, bounded traversal, patch parsing, and tests.
- Updated `crates/devfoundry/src/runtime.rs` to use `ToolRegistry::standard()`.

## Verification
- `cargo fmt --all -- --check`: blocked; `cargo` is unavailable in the environment.
- `cargo check -p devfoundry-tools -p devfoundry`: blocked; `cargo` is unavailable in the environment.
- Focused unit tests added for stale edits, failed patch atomicity, and standard registration; not executable without Cargo.
- `git diff --check`: run after implementation.

## Risks And Follow-Up
- Multi-file patch replacement is prevalidated but filesystem rename failures across multiple files are not a kernel-level transaction; a future patch transaction abstraction could improve rollback guarantees.
- Glob matching intentionally implements a small `*`, `?`, and `**` subset rather than a third-party glob grammar.
- Search currently uses literal substring matching; regex search and configurable sensitive-file patterns remain deferred.
