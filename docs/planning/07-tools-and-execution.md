# Tools And Execution

## Tool Contract

Every tool declares name, description, JSON input schema, risk class, permission requirement, timeout, output limit, and whether it is read-only or mutating. Execution receives a project capability, cancellation token, and permission decision; it does not receive raw global application state.

## MVP Tools

- `read`: bounded file/range reads with encoding handling.
- `write`: atomic create/replace with parent and mode policy.
- `edit`: exact or structured replacement with stale-content detection.
- `apply_patch`: parse and apply a constrained patch format atomically.
- `glob`: project-scoped matching with ignore rules and result limits.
- `grep`: bounded text search with binary and secret-file safeguards.
- `bash`: explicit command execution with environment, timeout, cwd, and output limits.
- `git_status`: fixed-argument, read-only Git status, diff, log, and optional worktree metadata queries.
- `pty`: bounded terminal command execution using the `pty_session` permission
  class and platform process cleanup; currently pipe-backed.

## Atomicity

Write and edit operations use temporary files and atomic rename where supported. Multi-file patches stage replacements and retain sibling backups until every install succeeds; rollback restores all prior targets on failure. Common credential and private-key filenames are denied by the built-in path policy and excluded from search. Preserve file permissions deliberately and test symlink behavior.

## Command Execution

Commands run with a sanitized environment, project working directory, timeout, output cap, and cancellation. On macOS and Linux, each shell command is its own process group; on Windows, each command is assigned to a Job Object. Timeout or cancellation terminates the platform cleanup boundary before reaping the shell so ordinary descendants do not survive. Shell invocation must be explicit; avoid string interpolation into a shell when an argument-vector execution is possible.

## Output Management

Return a bounded preview containing beginning and end where useful. Store complete textual output in a temporary managed directory with a unique name and retention policy. The tool's structured result must not be silently altered by preview truncation.

## Tool Discovery

The model receives only tools allowed for the selected agent and project policy. Build exposes the standard registered tools. Plan exposes only read-only tools: `read`, `glob`, `grep`, and `git_status`; the runner checks this policy again before execution. Descriptions must state side effects and important limits.

The runner currently projects each registered tool as a provider-neutral object schema. Runtime composition must register only tools permitted by the active policy; an allow-all broker is reserved for tests and must not be the production default.

## Integration Boundary

`git_status` is the first Git integration seam. It does not expose a general Git command runner: the repository root is fixed by the project capability, operations are a closed allowlist, output is bounded by the command boundary, and each query has its own permission class. Mutating Git operations and worktree creation/removal remain separate integrations with their own threat models.

## Future Tools

PTY, web access, mutating Git operations, MCP tools, LSP queries, subagents, and worktrees require separate threat models. The current `pty` tool is a safe pipe-backed abstraction, not an interactive pseudo-terminal; do not add terminal input, resize, or mode changes without a separate contract.

## Tool Tests

Test empty files, binary data, large files, missing paths, symlinks, traversal, concurrent edits, invalid patches, command exit codes, timeouts, cancellation, environment leakage, and output truncation.
