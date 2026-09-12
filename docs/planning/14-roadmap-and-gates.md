# Roadmap And Gates

## Gate 0: Foundation

Status: foundation compiles and passes format, Clippy, and tests with Rust stable 1.98.1. Product-level Phase 0 configuration and CLI expansion remain incomplete.

Cargo workspace, CI, schema IDs, config parser, CLI help/version. Gate: clean macOS/Linux build and schema round trips. Do not mark this gate complete until the commands in `docs/tasks/000-phase-0-foundation.md` pass.

## Gate 1: Durable Core

Status: implementation started; SQLite foreign-key enforcement, migration, restart, recovery, and diagnostics tests exist. Process-kill and interrupted-commit validation remain pending.

SQLite migrations, repositories, project/session/message/event persistence, restart recovery. Gate: kill/restart tests preserve facts and ordering.

## Gate 2: Safe Tools

Status: implementation started; the complete core tool set, sensitive-file policy, hostile path/symlink/timeout/cancellation/denial tests, and rollback tests now exist, including Unix process-group descendant cleanup. Full workspace verification remains pending.

Tool trait, registry, read/write/edit/patch/glob/grep/bash, bounded output, permission broker. Gate: hostile path, symlink, timeout, cancellation, and denial tests.

## Gate 3: Provider

Status: implementation started; normalized contract, test-only fake provider, GitHub Copilot adapter, and focused wire fixtures exist. Full workspace quality checks remain pending.

Normalized streaming contract, test-only fake provider, GitHub Copilot adapter, retry and cancellation. Gate: wire fixtures produce deterministic normalized events.

## Gate 4: Runner

Status: implementation started; durable admission, process-local session serialization, bounded tool continuation, deterministic minimal context assembly, build/plan tool policies, permission resolution, deterministic core integration tests, and API-level completion signaling exist, but configured instruction-file precedence and automatic restart recovery remain pending.

Prompt admission, build/plan agents, tool loop, durable events, abort, step limits. Gate: end-to-end repair task with approval and restart.

## Gate 5: API

Status: implementation started; health, durable prompt admission/status queries, runner dispatch, interrupt, sequence-aware SSE replay with bounded live fanout, structured errors, request IDs, transport limits, graceful shutdown, and broad route contract tests exist, but restart recovery, authentication, instance-wide events, and full workspace verification remain pending.

Versioned HTTP commands/queries, durable SSE replay, live stream, embedded transport. Gate: external client drives a complete session.

## Gate 6: TUI

Status: implementation started; Ratatui state/rendering, connected API client, durable sequence-aware SSE consumption/reconnect, authoritative session and permission refresh, automatic local server bootstrap, interactive project/session selection/resume, and deterministic terminal lifecycle validation exist. Live fanout, message-history backfill, model/agent catalogs, and OS-level pseudo-terminal/signal review remain.

Transcript, input, tool activity, dialogs, reconnect, session management. Gate: pseudo-terminal smoke suite and manual usability review.

## Gate 7: Advanced Runtime

Compaction, queued prompts, subagents, MCP/LSP, worktrees, PTY. Gate: stress and recovery suite.

## Gate 8: Ecosystem And Release

Plugins, packaging, update strategy, documentation, support diagnostics, optional desktop. Gate: reproducible artifacts, security review, upgrade/rollback test.

Current readiness: build/test gates pass and release CI now packages and checksum-verifies the three supported targets with a doctor redaction smoke check. DevFoundry is not yet production-ready until real-provider end-to-end tests, reconnect/recovery validation, permission/tool security testing, release signing, and package publication are complete.

## Parallel Work Rules

Schema/protocol can proceed first. Storage and fake provider can proceed in parallel after schema. Tools can proceed in parallel with storage. Runner waits for all three contracts. TUI waits for protocol and runner behavior. Integrations wait for stable tool and permission contracts.

## Stop Conditions

Pause feature work if durable ordering, permissions, cancellation, or secret handling is ambiguous. Resolve the invariant before adding UI or provider breadth.
