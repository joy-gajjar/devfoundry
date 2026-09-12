# OpenCode And DevFoundry Comparison

Comparison basis:

- Upstream OpenCode `dev` branch README, package layout, and `CONTEXT.md`.
- Current DevFoundry workspace, tests, planning documents, and release scripts.

DevFoundry is an independent Rust implementation inspired by OpenCode. It is not an exact source-level port.

> A verified, defect-aware feature difference is maintained in [audit/08-upstream-feature-diff.md](audit/08-upstream-feature-diff.md). Where this document and the audit disagree, the audit is authoritative.

## Summary

| Area | Upstream OpenCode | DevFoundry status |
|---|---|---|
| Primary runtime | Bun/TypeScript monorepo | Rust/Tokio workspace |
| Terminal agent | Mature OpenTUI/Solid terminal product | Ratatui TUI with live session events, model picker, permissions, scrolling, Markdown-like blocks, and progress states |
| Session persistence | SQLite/session runtime | SQLite sessions, messages, events, prompt inbox, recovery, export/import |
| Agent loop | Mature multi-step runner | Implemented sequential runner with bounded tool loop, cancellation, recovery tests, and plan/build policy |
| Tools | Broad built-in tool ecosystem | Read, write, edit, apply_patch, glob, grep, bash, git read-only tools, PTY seam |
| Permissions | Mature permission/question UX | Durable permission broker, API resolution, TUI `y/n`, sensitive-file denial |
| Providers | Multiple providers and model ecosystem | GitHub Copilot only for production; fake provider for tests |
| MCP | MCP support/ecosystem | Bounded local stdio MCP seam and native fixture; broader interoperability remains |
| LSP | LSP integration | Bounded local LSP diagnostics/process seam; broader server support remains |
| API | Large protocol/server/client/SDK surface | Rust HTTP/SSE API with lifecycle, prompt, permission, replay/live session events, auth/origin policy |
| SDK | Generated JS/Effect SDKs and SDK-next | No generated public SDK yet; typed Rust client exists for the TUI |
| Web UI | Solid web application | Not implemented |
| Desktop | Beta Electron desktop application | Not implemented |
| Plugins | Plugin package and extension ecosystem | No public plugin ABI; capability contracts are documented |
| Distribution | npm, Homebrew, Scoop, Chocolatey, Pacman, Nix, desktop installers | Cross-platform archive/release CI, checksums, install/upgrade/rollback scaffolding, package metadata placeholders |
| Community maturity | Large established project/community | New implementation with focused test coverage |

## Feature Matrix

### Core Agent Workflow

| Feature | DevFoundry |
|---|---|
| Create/resume project | Implemented |
| Create/list/resume sessions | Implemented |
| Prompt admission | Implemented with durable inbox |
| Prompt status query | Implemented |
| Cancellation/interrupt | Implemented |
| Session restart recovery | Implemented conservatively; interrupted work is not auto-rerun |
| Streaming assistant text | Implemented through durable delta events and SSE |
| Multi-turn tool loop | Implemented with bounded sequential continuation |
| Tool event visibility | Implemented |
| Provider failure handling | Implemented and tested |
| Context compaction | Not implemented |
| Queued steering prompts | Not implemented |
| Subagents/background tasks | Not implemented |

### Agents And Context

| Feature | DevFoundry |
|---|---|
| Build agent | Implemented |
| Plan agent | Implemented read-only tool policy |
| Agent switching UI/API | Partial; agent metadata exists, full selector remains |
| Project `AGENTS.md` discovery | Implemented ordered baseline discovery |
| Configured instruction files | Partial/follow-up |
| Context source registry | Not equivalent to upstream full context epoch system |
| Token budgeting | Not implemented |
| Compaction/context epochs | Not implemented |

### Tools And Safety

| Feature | DevFoundry |
|---|---|
| Read | Implemented |
| Write | Implemented with atomic writes |
| Edit | Implemented with stale-content detection |
| Apply patch | Implemented with validation and rollback attempts |
| Glob | Implemented bounded matching |
| Grep | Implemented bounded literal search |
| Bash | Implemented with permission, output, timeout, and process cleanup |
| Git status/diff/log | Implemented read-only fixed-operation seam |
| Git mutation/worktrees | Not implemented |
| PTY | Safe pipe-backed seam, native PTY remains future work |
| Sensitive-file protection | Implemented conservative defaults |
| Symlink/traversal protection | Implemented and tested |
| MCP tools | Local bounded stdio seam |
| Browser/Playwright MCP | Opt-in documentation only; not production-enabled |

### Provider Support

| Feature | DevFoundry |
|---|---|
| GitHub Copilot | Production provider path |
| Copilot token environment handoff | Implemented |
| Copilot JSON/SSE response handling | Implemented and fixture-tested |
| Rate-limit/auth/content-filter classification | Implemented |
| Provider cancellation | Implemented |
| Anthropic | Not supported by product decision |
| Gemini | Not supported by product decision |
| Bedrock | Not supported by product decision |
| Local OpenAI-compatible models | Not currently supported as a production provider path |
| Model catalog | Curated Copilot catalog endpoint/UI (currently hardcoded; see audit L4) |

### API, SDK, And Clients

| Feature | DevFoundry |
|---|---|
| Local HTTP server | Implemented |
| JSON API | Implemented |
| Durable SSE replay | Implemented |
| Live durable-event fanout | Implemented with bounded channel and cursor reconnect |
| Request IDs | Implemented |
| Structured errors | Implemented |
| Request/output limits | Implemented |
| Loopback default | Implemented |
| Non-loopback bearer auth | Implemented opt-in |
| Exact-origin CORS | Implemented opt-in |
| Rust API client | Implemented for TUI |
| Generated JS/Effect SDK | Not implemented |
| Embedded in-memory HTTP client | Not implemented |
| OpenAPI compatibility with upstream | Not implemented |

### User Interfaces

| Feature | DevFoundry |
|---|---|
| Ratatui TUI | Implemented |
| Project/session picker | Implemented |
| Model picker | Implemented |
| Permission approval | Implemented with `y/n` |
| Command palette | Implemented |
| Live loader/thinking state | Implemented |
| Markdown-like response formatting | Implemented lightweight headings/bullets/code fences |
| Transcript scroll | Implemented with arrows, `j/k` scroll mode, pages, Home/End, mouse wheel, scrollbar |
| Paste handling | Implemented |
| Web UI | Not implemented |
| Desktop application | Not implemented |
| VS Code extension | Not implemented |

### Distribution And Ecosystem

| Feature | DevFoundry |
|---|---|
| macOS Apple Silicon build | Supported/verified locally |
| Linux x64 build | CI target |
| Windows x64 build | CI target |
| Archive/checksum validation | Implemented |
| Install/upgrade/rollback scripts | Implemented scaffolding and smoke tests |
| Homebrew publication | Metadata/handoff only |
| cargo-binstall publication | Metadata/handoff only |
| WinGet publication | Metadata/handoff only |
| Release signing | Handoff documentation; credentials external |
| Plugin ABI | Not implemented |
| MCP ecosystem | Bounded contract/client, not broad ecosystem parity |
| LSP ecosystem | Bounded contract/client, not broad ecosystem parity |

## Where DevFoundry Is Ahead Or Different

- Rust-native single-binary architecture instead of Bun/Node runtime dependency.
- Production provider scope is deliberately narrow: GitHub Copilot only.
- Explicit macOS filesystem/process/TUI validation.
- Stronger compile-time crate boundaries and typed Rust contracts.
- Sensitive-file policy is enforced below the permission broker for read/write/edit/apply_patch. Note: the `bash` and `pty` tools currently do not apply this screen, so an allowed shell command can still read sensitive files. See [audit/01-critical-bugs.md](audit/01-critical-bugs.md) (C1).
- Portable session export excludes secrets, paths, permissions, events, and runtime status.
- Local API, TUI, storage, and runner are owned by one Rust implementation rather than adapted from upstream packages.

## Largest Current Gaps

1. Upstream has much broader provider and model support.
2. Upstream has mature web, desktop, SDK, plugin, and ecosystem packages.
3. DevFoundry does not yet have context compaction/epochs, subagents, MCP/LSP full interoperability, or native PTY parity.
4. DevFoundry release signing and package publication still require external credentials.
5. DevFoundry real Copilot smoke requires a user token and has not been run in this workspace by automation.

## Practical Conclusion

DevFoundry is currently a serious Rust-native terminal coding-agent core with a working TUI, API, session store, permission system, Copilot provider, tools, recovery, and macOS validation. It is not yet feature-equivalent to the full upstream OpenCode product, especially in ecosystem breadth and client surfaces.

The correct next priority is not copying every upstream package. It is finishing the DevFoundry core product loop, validating real Copilot execution, and then deciding whether web/desktop/SDK/plugin parity is actually required.
