# Test Coverage Matrix

Per-crate coverage assessment. Test counts are from inline `#[cfg(test)]` modules and `tests/` integration files at the audit snapshot.

## Coverage By Crate

| Crate | Package | Unit tests | Integration | Assessment |
|---|---|---|---|---|
| `schema` | devfoundry-schema | 3 | none | Thin for a ~352-line domain surface |
| `protocol` | devfoundry-protocol | 1 | none | Thin (DTO serialization only) |
| `storage` | devfoundry-storage | 0 | `tests/lifecycle.rs` (8) | No inline unit tests in ~717 lines |
| `tools` | devfoundry-tools | 45 | `native_integrations.rs` (9) | Best-covered crate |
| `llm` | devfoundry-llm | 16 | none | Good unit fixtures; no HTTP/SSE integration tests |
| `core` | devfoundry-core | 2 | `session_runner.rs` (10) | Reasonable |
| `server` | devfoundry-server | 0 | `api_lifecycle.rs` + `permission_api.rs` (33) | No inline unit tests in ~873 lines |
| `tui` | devfoundry-tui | 22 | none | Unit-only; no terminal-restoration harness in-tree |
| `devfoundry` | devfoundry (bin) | 7 | none | Config/runtime covered inline; no CLI integration tests |

## Weakest Areas

- `protocol` (1 test) and `schema` (3 tests) are thinnest relative to surface area.
- `server` and `storage` have zero inline unit tests; all coverage is integration-level, leaving internal error branches under-tested.
- `llm` has no integration test exercising real HTTP/SSE framing.
- `tui` has no automated terminal-restoration/PTY harness; validation is manual per `docs/tui-validation.md`.

## Concrete Missing Tests

### Provider (`llm`)
- Non-streaming JSON path with tool calls (would catch C2).
- Retry behavior for retryable classes (would catch C3).
- Content-filter classification precision and 429 ordering (would catch C4).
- Restore and assert the tool-call fragmentation unit test (C6).

### Server / Storage
- Message-history endpoint and pagination (once added, C5).
- Prompt-admission transaction/crash boundaries (M4).
- Interrupt vs error status (M3).
- Event schema versioning/replay of old payloads (M5).

### Tools
- `bash`/`pty` sensitive-path screening (C1).
- Multi-segment `..` path traversal for non-existent targets (M9).
- `file_name()` None handling (M7).

### TUI
- Draft loss on prompt failure (T2).
- Picker keybindings and connected-loop event routing.
- Unicode-width wrapping and table horizontal handling.
- Scroll clamping / Home sentinel interaction.
- Model picker context-window surfacing.

## Acceptance Gates (unchanged)

All fixes must keep these green on macOS Apple Silicon:

```bash
cargo fmt --all
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```
