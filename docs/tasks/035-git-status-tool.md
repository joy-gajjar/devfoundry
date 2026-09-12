# Task 035: Read-Only Git Queries

## Status

Implemented as a fixed-argument Git integration seam.

## Contract

- `git_status` is registered through the existing `Tool` and `ToolRegistry` contracts.
- The default operation runs `git --no-optional-locks status --short --branch --porcelain=v1`.
- `operation` is limited to `status`, `diff`, and `log`, using fixed commands for each query.
- `diff` uses `--no-ext-diff`; `log` is capped at 50 oneline entries.
- `include_worktrees: true` optionally appends `git --no-optional-locks worktree list --porcelain`.
- The project root is the only working directory; unknown arguments and arbitrary commands are rejected.
- Permissions use distinct `git_status`, `git_diff`, `git_log`, and `git_worktree` operations.
- Existing command timeout, cancellation, process-group cleanup, and output bounds apply.
- The tool is available to plan agents because it is read-only.

## Follow-up

Do not turn this into a general Git command tool. Mutating Git operations and worktree creation/removal remain separate integrations.
