# Task: W10 Project Brain, Task Board And Context Manifest

## Goal

Add a SQLite-authoritative, revisioned project task board and a safe Markdown/context projection for the Granular-inspired workspace.

## Scope

- Included: schema types, forward migration `0003_workspace_brain.sql`, storage task/document/context repositories, core task/context services, v2 task/roadmap/document routes, and contract/integration tests.
- Excluded: browser/TUI, tools/worktrees, LLM, scheduler/workers, secrets, preview, and resource marketplace behavior.

## Design

- SQLite remains authoritative for task status, revisions, dependencies, and roadmap imports.
- Task dependency updates reject direct and indirect cycles and require the expected revision.
- Markdown indexing is a rebuildable projection. Wikilinks produce backlink queries; symlinked and escaping document paths are rejected.
- Context manifests record source identity, content hash, provenance, and unresolved tool groups. Documents and context cannot grant permissions.
- Roadmap import validates task identity/revision/dependencies before mutation; stale/conflicting records are rejected.

## Implementation

- Added `Task`, `TaskStatus`, `ContextManifest`, `ContextSource`, and `DocumentMetadata` schema types.
- Added `0003_workspace_brain.sql` with tasks, dependencies, documents, and context manifests.
- Added storage task, document, and context repositories.
- Added core `TaskService` and `ContextManifestBuilder`.
- Added protocol DTOs and v2 API routes for tasks, roadmap export/import, and project documents.
- Added tests for schema serialization, task dependency cycles, stale revisions, Markdown backlinks/symlink refusal, context provenance, protocol DTOs, and API revision mapping.

## Verification

- `$HOME/.cargo/bin/cargo fmt --all -- --check`: passed.
- `$HOME/.cargo/bin/cargo check --workspace --all-targets`: passed.
- `$HOME/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `$HOME/.cargo/bin/cargo test --workspace`: passed, including all W10 tests and existing suites.
- `git diff --check`: passed.

## Risks And Follow-Up

- Markdown content updates and hash-based stale-write protection are not yet exposed as a write endpoint.
- Roadmap export/import currently uses a minimal recognized record format and requires richer acceptance/evidence fields before scheduler integration.
- Task state is durable but not yet connected to W09 boss-worker assignment or W08 worktree metadata.
- Context manifests are captured but not yet integrated into provider request admission or token budgeting.
