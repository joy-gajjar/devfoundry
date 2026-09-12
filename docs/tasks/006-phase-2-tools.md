# Task: Phase 2 Tool Runtime

## Goal

Provide a typed, permission-aware tool boundary with safe project path resolution, bounded file output, and cancellable shell execution.

## Scope

- Included: tool/permission traits, registry, read/write tools, project-root containment, output bounds, command timeout, and cancellation.
- Excluded: patch/edit/glob/grep implementations, PTY, network tools, plugins, and persistent permission policies.

## Design

Tools receive a project root, central permission broker, and cancellation token. Path resolution canonicalizes the nearest existing parent so new files can be created without permitting traversal or symlink escape. Shell execution is explicitly authorized, bounded by 30 seconds and 64 KiB output, and killed on cancellation.

## Implementation

- Added `crates/tools`.
- Added permission broker and allow-all test/runtime seam.
- Added tool registry, read/write tools, and cancellation-aware command execution.
- Corrected new-file path resolution during the Phase 5 integration review.
- Hardened final symlink validation, atomic writes, bounded command output, and child cleanup; see `028-tool-security-hardening.md`.

## Verification

Initial implementation verification was previously blocked because the Rust toolchain was unavailable. The security hardening verification is recorded in `028-tool-security-hardening.md`.

## Risks And Follow-Up

- The default allow-all broker is only a composition seam and must not be the production policy.
- Add edit, patch, glob, grep, persistent approvals, descendant process cleanup, and hostile concurrent-filesystem tests before Phase 2 closes. The current cancellation branch cleans up the spawned child but does not guarantee descendant process cleanup.
