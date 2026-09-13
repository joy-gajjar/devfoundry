# Task: W22 Resource Library

## Goal
Provide a project-scoped resource/skill manifest boundary that can inspect and preview bounded content, and can install/update/remove only through explicit central permission decisions.

## Scope
- Included: forward migration `0005_resources.sql`, schema manifests, storage metadata, core orchestration, archive validation/staging, Rust resource routes, security regression tests, and this record.
- Explicitly excluded: browser UI, credentials, scheduler, worktrees, PTY, LLM/provider changes, hooks, executable resources, capability auto-grants, and commits.

## Design
- Resource manifests are declarative. Required capabilities are stored as metadata and never granted automatically.
- Archive inputs fail closed on traversal, absolute paths, links/special files, duplicate targets, malformed paths, and bounded file/count/expanded-size limits.
- SHA-256 content hashes are used for manifest/content comparison.
- The core service receives a `PermissionBroker`; it never self-approves. HTTP mutation routes currently fail closed because this branch has no resource broker wired into `ServerState`.
- Installation stages bounded files and rejects target conflicts. Existing legacy migrations are unchanged.
- Removal intentionally remains blocked until the persisted installed-file projection supports edited-file-preserving deletion; this is safer than deleting or overwriting user edits.

## Implementation
- `migrations/0005_resources.sql`
- `crates/schema/src/resources.rs`, `crates/schema/src/lib.rs`
- `crates/storage/src/resources.rs`, `crates/storage/src/lib.rs`
- `crates/core/src/resources.rs`, `crates/core/src/lib.rs`
- `crates/tools/src/archive.rs`, `crates/tools/src/lib.rs`, `crates/tools/Cargo.toml`
- `crates/server/src/routes_resources.rs`, `crates/server/src/lib.rs`
- `crates/tools/tests/resource_security.rs`

## Verification
- `cargo test -p devfoundry-tools --test resource_security`: 4 passed.
- `cargo check -p devfoundry-server --lib`: passed.
- `cargo test -p devfoundry-storage --lib`: passed (0 tests).
- `cargo test -p devfoundry-core --lib`: 7 passed.
- Final integrated Rust gates after W20 browser-host changes landed passed:
  workspace check, workspace Clippy with `-D warnings`, workspace tests and
  `git diff --check`. W22 remains intentionally partial at the mutation
  boundary; install publication and edited-file-preserving removal are not
  claimed complete.
- `cargo fmt --all`: run; formatting changes are limited to touched files plus pre-existing formatter drift reported by the workspace.

## Risks And Follow-Up
- HTTP mutation routes fail closed until the existing central permission broker is explicitly added to `ServerState`; no route may substitute an allow-all/default broker.
- Install publication is not yet a complete multi-file atomic rename transaction and should not be treated as crash-safe.
- Persisted `resource_files` needs an installed-hash update and removal transaction before W22 can claim edited-file-preserving uninstall/update behavior.
- Archive parsing adapters for zip/tar are intentionally not added without a separate format/dependency security review.
- The unrelated W20 browser compile blocker must be resolved by its owner before full workspace gates can pass.
