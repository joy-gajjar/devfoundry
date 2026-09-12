# DevFoundry Pending Work

This is the prioritized backlog remaining before DevFoundry can be called production-ready.

> A defect-level audit with file:line references and a sequenced remediation plan is maintained in [audit/](audit/README.md). Start there for known bugs before implementing backlog items.

## P0: Production Blockers

- Run controlled live external-provider completion with a user-provided GitHub Copilot token; the
  opt-in smoke command now has a credential-free `--mock` preflight, validates configuration,
  bounds output, and provides secret-free diagnostics, but live evidence remains manual.
- Replace remaining permission/file integration timing waits with durable completion signals.
- Add live external-provider rate-limit/content-filter/retry validation; deterministic provider-layer classification fixtures now exists.
- Complete hostile concurrent filesystem rollback validation.
- Add OS-level signal and external pseudo-terminal terminal-restoration tests for the TUI; deterministic resize, interrupt-key, panic/drop, and interactive-loop validation now exists.

## P1: Core Feature Completion

- Multi-file patch rollback on hostile concurrent filesystem failures.
- Regex grep and explicit secret-file policy.
- Add persistent permission policies with once/session/project scopes.
- Add permission expiry and cleanup.
- Add prompt inbox recovery and explicit resume after restart.
- Persist provider attempts and tool-call attempt IDs for idempotency.
- Update session status durably during running, waiting, cancellation, completion, and error transitions.
- Add message history retrieval and pagination.
- Add session rename, delete, fork, and resume operations.
- Add model and agent catalogs plus selection/update API contracts; the TUI currently displays stored session metadata.
- Add message-history retrieval and pagination for complete resumed-session transcript backfill.

## P1: Context And Agent Behavior

- Connect configured instruction files after precedence and path contracts are decided.
- Add context budget accounting.
- Add compaction and context epochs.
- Add model/provider capability checks.
- Add queued prompts and steering semantics.
- Add loop detection and stricter tool-call validation.
- Add subagents and background tasks only after the base runner is stable.

## P2: Provider And Integration Expansion

- Add GitHub Copilot model catalog/metadata when the model contract is stable.
- Complete GitHub Copilot OAuth/device-token acquisition validation; the external
  operator handoff is documented in [`credentials.md`](credentials.md) and
  [`copilot-network-smoke.md`](copilot-network-smoke.md), but acquisition remains
  outside DevFoundry.
- Extend the bounded macOS stdio MCP seam with tool metadata, timeout/output policy, and explicit remote trust configuration.
- Extend the minimal LSP lifecycle/diagnostics seam with reviewed JSON-RPC framing and protocol handling.
- Replace the pipe-backed `pty` terminal capability with a real PTY adapter once
  resize, terminal-mode restoration, and interactive input contracts are stable.
- Add filesystem sandboxing or an explicit trusted-command policy for PTY
  commands; canonical `current_dir` is not containment.
- Add reviewed LSP JSON-RPC framing, request cancellation, and protocol message
  limits; the current lifecycle seam only drains bounded process output.

- Extend the local native MCP/LSP/PTY harnesses with real-server interoperability
  evidence only after external fixtures and native PTY behavior are supported.

The provider-neutral MCP/LSP/PTY contract seam and bounded MCP stdio/LSP/PTY seams are implemented; full real-server interoperability and a native PTY adapter remain deferred.

## P2: API And Server Hardening

- Complete API contract tests for every route, status, header, payload, error, and SSE frame. Current routes, prompt/interrupt/permission/session discovery, replay, bounded durable fanout, and authentication/origin policy are covered; message history and instance-wide events remain separate work.
- Add live instance-wide event stream distinct from durable session events; durable session streams now have bounded live fanout.
- Authentication and exact-origin CORS policy are required for non-loopback binding; loopback defaults remain unauthenticated.
- Add graceful server shutdown and cancellation propagation.
- Add embedded in-memory client transport using the same router.
- Add API versioning and compatibility policy.
- Add rate limits and request/output size limits.

## P2: Storage And Operations

- Add encrypted export/import for secrets if a future requirement needs credentials; portable session export/import is implemented without secrets, runtime records, or database paths.
- Add database corruption recovery automation beyond diagnostics and safe backups.
- Add managed tool-output retention and cleanup.
- Add configurable data directory resolution.
- Add secret redaction tests for logs and diagnostics.
- Add crash/restart tests during provider, tool, permission, and transaction boundaries.

## P2: Release And Distribution

- Complete artifact signing and package publication using the documented trusted-host handoff; metadata and unsigned publication plans now exist.
- Add live Homebrew/cargo-binstall/package-manager publication after credentials and maintainer approvals are available.
- Add production launcher integration around the install/upgrade/rollback scaffold.
- Add cross-platform rollback testing and migration compatibility tests.
- Implement the opt-in `devfoundry update --check` command described in [`versioning-and-updates.md`](versioning-and-updates.md). The macOS-only, anonymous, bounded, download-free checker is implemented; persistence and a repository-specific public endpoint remain release-configuration work.
- Populate release notes and migration notes for each public release using [`release-notes-template.md`](release-notes-template.md).
- Add manual pseudo-terminal release smoke tests.
- Configure a repository-specific private security contact for vulnerability reports.

## P3: Ecosystem And Product Expansion

- Add trusted in-process plugin traits.
- Add external process plugin protocol.
- Add plugin capability declarations and isolation.
- Add web client using the public API.
- Add optional desktop shell around the server/TUI capabilities.
- Add themes and richer Markdown/code rendering.
- Add offline/local-only mode documentation.
- Add optional opt-in telemetry policy, if needed.

## TUI-First Product Roadmap

The detailed TUI feature backlog, descriptions, priorities, dependencies, differentiators, and production milestone are maintained in [`TUI_FEATURE_ROADMAP.md`](TUI_FEATURE_ROADMAP.md).

## Explicitly Deferred

- Mobile application.
- Cloud multi-tenant hosting.
- Arbitrary unrestricted computer control.
- Native dynamic plugin loading before a stable extension need exists.
- Automatic retries of non-idempotent tools.

## Production-Ready Exit Criteria

DevFoundry should not be labeled production-ready until all of the following are true:

- A real provider can complete a multi-step task with tools.
- Permission approval and denial work through the TUI and API.
- Restart and reconnect preserve or safely recover durable state.
- Filesystem and shell security tests pass.
- API integration and provider fixture suites pass.
- TUI terminal cleanup is verified.
- Release artifacts build reproducibly on supported platforms.
- Secrets are absent from logs and diagnostics.
- Upgrade, migration, backup, and rollback behavior is documented and tested.
