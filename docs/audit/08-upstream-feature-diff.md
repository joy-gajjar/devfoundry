# Upstream Feature Difference

Verified comparison between DevFoundry (this repo) and upstream OpenCode (`anomalyco/opencode`, `dev` branch) at the audit snapshot.

## Basis

- Upstream is a Bun/TypeScript monorepo with roughly 30 packages under `packages/`.
- DevFoundry is a Rust/Tokio workspace with 9 crates: `schema, protocol, storage, tools, llm, core, server, tui, devfoundry`.
- DevFoundry is an independent reimplementation inspired by OpenCode, not a source-level port.

## Upstream Package Inventory

Observed upstream `packages/`:

`app, cli, client, codemode, console, containers, core, desktop, docs, effect-drizzle-sqlite, effect-sqlite-node, enterprise, function, http-recorder, httpapi-codegen, identity, llm, opencode, plugin, protocol, schema, script, sdk, sdk-next, server, session-ui, slack, stats, storybook, tui, ui, web`.

## Package-Level Parity

| Upstream package | Purpose | DevFoundry equivalent | Status |
|---|---|---|---|
| `core` | Agent/session runtime | `crates/core` | Partial |
| `tui` | Terminal UI | `crates/tui` | Partial |
| `server` | HTTP API server | `crates/server` | Partial |
| `protocol` | Wire/protocol types | `crates/protocol` | Partial |
| `schema` | Domain schema | `crates/schema` | Partial |
| `llm` | Provider/LLM gateway | `crates/llm` | Partial (Copilot only) |
| `cli` / `opencode` | CLI entrypoint | `crates/devfoundry` | Partial |
| `client` | API client | `crates/tui/src/client.rs` | Partial (internal only) |
| `sdk` / `sdk-next` | Generated public SDKs | — | Missing (non-goal) |
| `web` / `console` / `session-ui` / `ui` | Web application/UI | — | Missing (non-goal) |
| `desktop` | Electron desktop app | — | Missing (non-goal) |
| `sdks/vscode` | VS Code integration | — | Missing (non-goal) |
| `plugin` | Plugin ABI/ecosystem | — | Missing |
| `slack` | Slack integration | — | Missing (non-goal) |
| `enterprise` / `identity` | Auth/enterprise | — | Missing (non-goal) |
| `containers` | Container execution | — | Missing |
| `codemode` | Code-execution mode | — | Missing |
| `function` | Serverless functions | — | Missing (non-goal) |
| `stats` / `storybook` / `http-recorder` / `httpapi-codegen` | Tooling/infra | — | Missing (tooling) |
| `effect-*-sqlite` | Storage adapters | `crates/storage` (SQLx/SQLite) | Different implementation |
| `docs` | Documentation site | `docs/` (Markdown) | Different implementation |

## Feature Difference By Area

### Core agent workflow
- Both: sessions, prompts, streaming, multi-step tool loop, cancellation.
- DevFoundry gaps: context compaction/epochs, queued steering prompts, subagents/background tasks, retry on retryable errors (see 01-critical-bugs C3), configurable loop cap (02-medium-risks M2).

### Agents and context
- Both: build and plan agents; `AGENTS.md` discovery.
- Upstream extra: a `general` subagent invocable via `@general`; richer context/epoch system.
- DevFoundry gaps: full agent selector UI, token budgeting, context source registry.

### Tools and safety
- Both: read/write/edit/patch/glob/grep/bash, permissions.
- DevFoundry: git read-only seam, pipe-backed PTY seam, bounded MCP stdio seam, bounded LSP process seam.
- DevFoundry gaps: git mutation/worktrees, native PTY, regex grep, full MCP/LSP interoperability. Safety defect: bash/pty bypass sensitive-file policy (01-critical-bugs C1).

### Providers
- Upstream: multiple providers and a model ecosystem (LLM gateway).
- DevFoundry: GitHub Copilot only for production; fake provider for tests. Anthropic/Gemini/Bedrock/local are intentional non-goals.

### API, SDK, clients
- Both: local HTTP server, JSON API, SSE, request ids, structured errors.
- Upstream: generated JS/Effect SDKs, `sdk-next`, embedded client, broad protocol surface.
- DevFoundry gaps: message-history endpoint (01-critical-bugs C5), session delete/fork/resume routes, instance-wide event stream, public SDK, OpenAPI parity.

### User interfaces
- Upstream: mature terminal UI plus web console, desktop app, VS Code, Slack.
- DevFoundry: Ratatui TUI only (see 04-tui-gaps for its internal gaps). Web/desktop/VS Code/Slack are non-goals unless a product decision changes that.

### Distribution
- Upstream: npm, Homebrew, Scoop, Chocolatey, Pacman, Nix, desktop installers.
- DevFoundry: cross-platform archive/release CI, checksums, install/upgrade/rollback scaffolding, package-manager metadata placeholders; signing/publication require external credentials.

## Where DevFoundry Differs Intentionally

- Single Rust binary rather than a Bun/Node runtime dependency.
- Deliberately narrow provider scope (Copilot only).
- Compile-time crate boundaries and typed Rust contracts.
- Portable session export that excludes secrets, paths, permissions, events, and runtime status.
- Explicit macOS filesystem/process/TUI validation.

## Largest True Gaps (not intentional non-goals)

1. Provider retry, context compaction, subagents, and steering prompts in the runner.
2. Message-history API and session lifecycle routes (delete/fork/resume).
3. Full MCP/LSP interoperability and native PTY.
4. TUI P1+ features (see 04-tui-gaps).
5. Plugin ABI.

## Intentional Non-Goals (require product decision to change)

- Additional providers beyond GitHub Copilot.
- Web UI, desktop app, VS Code extension, Slack, enterprise/identity, serverless functions.
- Generated public SDKs.

## Cross-References

- Defects behind "Partial" status: [01-critical-bugs.md](01-critical-bugs.md), [02-medium-risks.md](02-medium-risks.md).
- TUI internal gaps: [04-tui-gaps.md](04-tui-gaps.md).
- Doc drift and inflated claims to correct in `OPENCODE_COMPARISON.md`: [05-docs-and-naming.md](05-docs-and-naming.md).
