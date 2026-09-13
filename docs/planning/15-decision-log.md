# Decision Log

## 2026-09-05: Post-Commit Session Event Fanout

Durable session SSE subscribers register before replay and receive post-commit events through a bounded process-local broadcast channel. Duplicate or older sequences are ignored, while lag closes the stream and requires cursor-based reconnect. This preserves durable storage as authority and avoids pretending that best-effort live delivery can recover missed events.

## 2026-09-05: API Contract Coverage Boundary

Contract tests exercise the existing versioned HTTP routes through the Axum router, including request IDs, structured errors, prompt and permission lifecycle responses, discovery, SSE framing, replay cursors, and live post-commit delivery. The suite deliberately does not introduce authentication, instance-wide events, message-history routes, or restart resume behavior; those remain separate contracts.

## 2026-09-05: GitHub Copilot As The Sole External Provider

The production provider boundary accepts only `github-copilot`. Configuration uses `base_url` and an environment-selected `token_env`, defaulting to `https://api.githubcopilot.com` and `GITHUB_COPILOT_TOKEN`. Authentication is deliberately an environment-backed handoff: OAuth or device-token acquisition remains outside DevFoundry so credentials are never persisted or logged. The fake provider is compiled only for tests; Anthropic, Gemini, Bedrock, and other external adapters are not supported.

## 2026-09-05: Sensitive-Path Boundary And Patch Rollback

Built-in tools deny common credential and private-key filenames before the permission broker, and search tools omit those paths by default. Multi-file patches stage replacements and retain sibling backups until all installs succeed, restoring backups on failure. This closes the ordinary partial-mutation path without claiming a kernel-level transaction against hostile concurrent filesystem changes.

## 2026-09-05: Minimal Context And Agent Tool Policies

The runner loads `AGENTS.md` files from the project root toward the requested directory and places them in one minimal system context message. Build retains the standard tool registry, while plan is read-only through `read`, `glob`, and `grep`; execution repeats the check because provider tool calls are untrusted input. Configured instruction-file precedence, budgets, compaction, and subagents remain deferred until their contracts are explicit.

## 2026-09-05: Unix Command Process-Group Cleanup

Timed-out or cancelled shell commands use a dedicated Unix process group or a Windows Job Object before the shell is reaped. Unix receives group-scoped `SIGKILL`; Windows uses `TerminateJobObject` and kill-on-job-close. Other platforms retain direct-child cleanup.

## 2026-09-05: Bound API Work And Correlate Errors

The API server uses a generated or caller-provided request ID, structured error fields, and explicit request/prompt/event replay limits. Active executions share a registry-owned shutdown cancellation path. This keeps diagnostics actionable and prevents unbounded transport memory use without changing the versioned route set. Full workspace verification remains blocked by the existing tools crate compile error documented in task 029.

## 2026-09-05: Classify Provider Failures Without Retrying

The provider adapter exposes transient, rate-limited, content-filtered, and permanent classifications on provider errors, but does not retry requests. This keeps retry budgets and settlement safety outside the wire adapter while giving the owning layer enough information to make an explicit decision.

## 2026-09-05: Enforce SQLite Relationships And Bound Diagnostics

SQLite foreign keys are enabled in connection options rather than relying on migration-local pragmas, because pooled connections each have independent enforcement state. Recovery updates stale prompts and pending permissions in one transaction. Storage diagnostics report only foreign-key state, integrity status, and table counts; content export is intentionally deferred to avoid leaking prompts, tool targets, or secrets.

## 2026-09-05: Target-Named Tar Archives And Checksums

Release artifacts use target-named `tar.gz` archives with SHA-256 sidecars because the format is available on all current GitHub Actions runners and is easy to verify without changing the binary or runtime behavior. Publisher signatures, native installers, and package-manager formats remain deferred.

## 2026-09-05: Secretless Release Lifecycle Scaffold

Keep packaging and lifecycle preparation usable without release secrets. Validate the archive and checksum before extraction, install into versioned user-owned directories, retain one previous symlink target, and make rollback an explicit operator action. Publisher signing remains a handoff performed on a trusted signing host so private keys never enter repository automation.

## D001: Rust Workspace From Empty Repository

Status: accepted. The local repository has no implementation or commits, so there is no migration constraint. Start with a clean Cargo workspace.

## D002: Local-First MVP

Status: accepted. A single-user local workflow provides the shortest path to a useful and safe product. Remote multi-user concerns are deferred.

## D003: SQLite Durable Store

Status: accepted. It minimizes installation and supports transactions, migrations, and replay. Repository traits preserve future flexibility.

## D004: Axum, SSE, And JSON

Status: accepted for MVP. HTTP plus SSE is inspectable and sufficient for commands and replayable events. WebSockets remain deferred.

## D005: Provider-Neutral Normalized Events

Status: accepted. Provider wire formats must not leak into the runner or domain model.

## D006: Central Permission Broker

Status: accepted. Individual tools cannot be trusted to implement complete policy consistently.

## D007: Sequential Tool Execution Initially

Status: accepted. It simplifies ordering, approval, cancellation, and recovery. Parallel execution requires a later dependency/conflict model.

## D008: In-Process Plugins Before Native Loading

Status: accepted. A native ABI is difficult to stabilize and can bypass safety boundaries. External process plugins may follow.

## D009: Behavior Before Exact Upstream Compatibility

Status: accepted. Preserve user-visible concepts first; add contract compatibility only when fixtures and ownership are clear.

## D010: Atomic Tool Writes And Bounded Process Reads

Status: accepted. Tool writes stage content in a unique sibling file and rename it into place after syncing. Command streams are capped before aggregation and returned output is bounded by bytes. This preserves existing targets on ordinary write failures and prevents large command output from consuming unbounded memory without adding a broader filesystem abstraction.

## D010: Sequence-Aware SSE Reconnect And Authoritative Refresh

Status: accepted. The TUI treats durable event sequence IDs, `GET /sessions/{id}`, and pending-permission listing as authoritative. It reconnects from the last applied sequence with bounded backoff and periodically refreshes snapshots. Live server-side fanout remains deferred.

## D011: Constrained Core Search And Patch Tools

Status: accepted. Core patching uses a repository-local, line-oriented patch envelope and validates every change before mutating files. Search tools use bounded directory traversal and literal matching to avoid unbounded work and avoid adding a regex engine or broader secret policy to the tool boundary prematurely.

## Open Decisions

- 2026-09-13: W09 starts with a pure, sequential scheduler policy and immutable worker/evidence contracts. It does not add a migration or route until durable lease/attempt/evidence persistence can be composed through existing public APIs. Reviewed evidence cannot imply acceptance or Git integration; unknown child effects are never automatically replayed.

- ULID versus UUID for public IDs.
- Exact config format and migration policy.
- Whether provider streaming deltas are durable individually or only at settlement.
- OS keychain implementation and secret precedence.
- Memory/semantic search scope.
- Whether external plugins use JSON-RPC or another protocol.
- Exact stable client naming (`session` versus `sessions`).

Each open decision needs an owner, options, compatibility impact, and deadline before the affected gate.
