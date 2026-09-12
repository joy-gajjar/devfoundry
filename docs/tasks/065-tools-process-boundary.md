# Task: W05 Tools Process Boundary

## Goal

Make the owned file and process tools fail closed on missing nested paths,
external search symlinks, sensitive aliases, nonzero command outcomes, and
process-output cleanup deadlines while preserving the existing tool API.

## Scope

- Included: `crates/tools/src/lib.rs` path resolution, recursive search
  containment, sensitive alias regression coverage, child environment policy,
  command exit outcomes, bounded output draining, and deadline cleanup.
- Included: regression tests for the W05 acceptance cases and this task record.
- Excluded: schema, protocol, core, server, TUI, MCP framing, LSP framing,
  native PTY behavior, and OS sandboxing.

## Design

`ToolContext::resolve` canonicalizes the project root, rejects absolute paths
and parent-directory components, preserves every missing suffix component, and
checks existing final symlinks against the canonical root. Recursive search
does not follow symlinks; canonicalized files and directories must remain under
the root before they are traversed or read. This reduces race and link escape
risk but is not an OS sandbox and does not eliminate TOCTOU races against a
hostile concurrent filesystem actor.

Commands continue to use the existing permission broker and shell API. The
child environment is cleared and only the host `PATH` is reintroduced, so
provider credentials and unrelated ambient variables are not inherited. The
child's `ExitStatus` is retained: nonzero exit codes and signal termination
return `ToolError::Failed` instead of a successful `ToolOutput`. Output readers
retain at most one byte over the preview limit while continuing to drain pipes,
preventing a producer from blocking on a full pipe. After timeout/cancellation,
the process boundary is terminated and reaped; after normal exit, output joins
are bounded by a cleanup deadline and the Unix process group is terminated if
that deadline is exceeded.

The implementation makes no claim of OS-level sandboxing. Permission approval
remains centralized in `PermissionBroker`; tools do not self-approve.

## Implementation

- Updated `crates/tools/src/lib.rs` only within the owned tools crate.
- Added regression tests:
  - `missing_nested_suffix_is_preserved`
  - `grep_rejects_external_symlink`
  - `sensitive_alias_is_denied`
  - `exit_seven_is_failed_with_exit_code`
  - `grandchild_pipe_does_not_outlive_deadline`
  - `provider_token_is_not_in_child_environment`
- Preserved the existing `ToolOutput` and `ToolError` public API.
- No MCP framing changes were made.

## Verification

### RED evidence

- Initial targeted test invocation was malformed because Cargo accepts one test
  filter; it returned exit 1 with `unexpected argument`.
- Corrected RED invocation:
  `"$HOME/.cargo/bin/cargo" test -p devfoundry-tools --lib -- --nocapture`
  reached compilation but was blocked by a pre-existing unrelated change in
  `crates/llm/src/lib.rs`: `cannot find function validate_arguments in this scope`.
  The tools tests therefore did not execute in the first RED attempt.

### Final targeted evidence

- `"$HOME/.cargo/bin/cargo" fmt -p devfoundry-tools -- --check`: passed.
- `"$HOME/.cargo/bin/cargo" check -p devfoundry-tools --all-targets`: passed.
- `"$HOME/.cargo/bin/cargo" clippy -p devfoundry-tools --all-targets -- -D warnings`: passed.
- `"$HOME/.cargo/bin/cargo" test -p devfoundry-tools --lib -- --nocapture`:
  passed, 50 tests, 0 failed.
- `"$HOME/.cargo/bin/rustfmt" --edition 2024 --check crates/tools/src/lib.rs`:
  passed during implementation.
- `git diff --check -- crates/tools/src/lib.rs`: passed during implementation.

The initial post-implementation run found three test issues: canonical macOS
temporary paths, a Git permission test fixture lacking a repository, and
bounded output treating a producer-side signal as failure. These were corrected
in the owned tests/implementation; the final targeted run above is green.

## Risks And Follow-Up

- Filesystem validation and subsequent operations are not one kernel-level
  containment transaction; concurrent directory replacement remains a TOCTOU
  residual risk.
- Shell execution remains intentionally explicit trusted-process authority;
  clearing the environment is not a command sandbox and shell text is not
  substring-filtered into a false security guarantee.
- The retained preview is bounded, but complete output is not persisted in a
  managed retention-safe artifact in this W05 surface; that broader output
  management contract remains follow-up work.
- Windows Job Object behavior is compile-only in this macOS run; native Windows
  runtime evidence remains required by the broader plan.
- The workspace still contains unrelated pre-existing modifications in
  `crates/llm`, `crates/tui`, and `crates/storage/tests`; they were not edited.

## Status

Changed locally and targeted-verified. No commit, push, destructive cleanup, or
edits outside the W05-owned tools source/tests and this task record were made.
