# Plan: Configurable Boss and Worker Agent UI

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe first Boss/Worker configuration and TUI slice for solo developers: layered global/project configuration, built-in agent profiles, active-agent selection, and a read-only worker dashboard backed by existing session and worker contracts.

**Architecture:** Extend the existing JSON configuration loader with global and project layers, preserving the current `devfoundry.json` compatibility path. Keep agent policy enforcement in `crates/core`, expose only built-in profiles (`boss`, `build`, `plan`, `review`, `test`) with explicit tool-policy metadata, and add TUI state/reducer/rendering for agent settings and worker status. Do not invent automatic decomposition or merge behavior; the UI can configure/select profiles and inspect explicit worker assignments already supported by server/core.

**Tech Stack:** Rust 1.88, Cargo workspace, serde/serde_json, SQLx/SQLite, Ratatui, existing Axum JSON/SSE API, existing worker registry/projections.

**Spec:** Product capability request in the conversation; Granular reference: `https://granular.build/docs/boss-and-worker-agents/`; existing architecture: `docs/planning/02-system-architecture.md`.

## Global Constraints

- Primary user: solo developers on local development laptops.
- Provider remains GitHub Copilot only for production execution.
- Project instructions are untrusted context and cannot grant permissions or change policy.
- Built-in profile selection must not imply unsupported arbitrary custom-agent policies.
- Workers remain isolated/reviewable; no automatic merge or self-acceptance is added in this slice.
- Global configuration must not override project security policy or leak secrets.
- Existing `devfoundry.json` project configuration remains readable during migration.
- Every new state transition needs deterministic unit tests and full Rust validation.

---

## User Story

As a solo developer, I want to configure and select a Boss or Worker profile from
the TUI and see explicitly assigned workers, so that I can coordinate parallel
development without editing configuration files or mistaking a worker outcome
for an accepted merge.

## Complexity

Large. This crosses configuration, schema/runtime policy, TUI state/rendering,
server/session selection, and worker projections. It intentionally excludes the
larger automatic Boss decomposition and merge engine.

## UX Design

### Before

```text
TUI
  +-- transcript
  +-- prompt
  +-- command palette

Agent selection: implicit build/plan session value
Worker visibility: limited projection; no configuration surface
Configuration: project devfoundry.json only
```

### After

```text
TUI
  +-- Agent Settings
  |     +-- active profile: boss/build/plan/review/test
  |     +-- model and step budget
  |     +-- worker concurrency
  |     +-- project/global source indicator
  |
  +-- Worker Dashboard
  |     +-- assignment and lease status
  |     +-- attempt/evidence/review state
  |     +-- isolated worktree path shown only as bounded metadata
  |
  +-- transcript + prompt
```

### Interaction Changes

| Touchpoint | Before | After | Notes |
| --- | --- | --- |
| Command palette | safe command dispatch only | opens Agent Settings or Worker Dashboard | Preserve existing Esc/Ctrl-C semantics |
| Session agent | stored/displayed string | selectable built-in profile | Reject unknown profiles in UI; preserve API compatibility |
| Configuration | project `devfoundry.json` | global + project layered JSON | Global path is platform-aware and never contains secret values |
| Worker view | projection/status surfaces | explicit dashboard state | Read-only in this phase; assignment/merge actions remain separate |

## Mandatory Reading

| Priority | File | Lines | Why |
| --- | --- | --- | --- |
| P0 | `crates/devfoundry/src/config.rs:1-123` | all | Existing project config format, root discovery, errors, tests |
| P0 | `crates/core/src/lib.rs:66-140` | agent policy/context assembly | Current `plan`/`build` semantics and untrusted AGENTS.md handling |
| P0 | `crates/tui/src/lib.rs:1-220` | state/input patterns | Existing TUI reducer, key handling, command palette, tests |
| P0 | `crates/tui/src/lib.rs:900-1200` | rendering | Existing layout, transcript, modal, and palette rendering |
| P1 | `crates/server/src/lib.rs:569-723` | session API | Session creation/update and agent field behavior |
| P1 | `crates/server/src/routes_workers.rs:1-240` | worker API | Existing worker assignment/status boundaries and validation |
| P1 | `crates/core/src/workers.rs:1-220` | worker contracts | Worker briefs, evidence, outcomes, and failure types |
| P1 | `crates/schema/src/lib.rs:240-330` | session/schema types | `AgentName`, `Session`, model, status serialization |
| P2 | `crates/tui/src/client.rs:1-220` | client transport | Existing request/event client style and error handling |
| P2 | `docs/PENDING_WORK.md:17-39` | backlog boundaries | Deferred subagents, budgets, loop detection, and orchestration scope |

## External Documentation

| Topic | Source | Key Takeaway |
| --- | --- | --- |
| Boss/worker UX reference | `https://granular.build/docs/boss-and-worker-agents/` | Boss delegates to parallel isolated workers; workers are visible; merges are reviewable |
| Rust configuration | Existing serde/JSON config code | No new config dependency; preserve current JSON style |
| Ratatui UI | Existing `crates/tui/src/lib.rs` | Extend current reducer/layout instead of adding a second UI framework |

## Patterns to Mirror

### NAMING_CONVENTION

// SOURCE: `crates/tui/src/lib.rs:25-76`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptEntry { ... }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionApprovalAction { ... }
```

Use explicit state structs, small enums, and derived equality for deterministic
tests. New UI state should use names such as `AgentSettingsState`,
`AgentProfile`, `WorkerDashboardState`, and `WorkerRow`.

### ERROR_HANDLING

// SOURCE: `crates/devfoundry/src/config.rs:37-61`

```rust
#[derive(Debug, Error)]
pub enum ConfigError { ... }

serde_json::from_str(&contents).map_err(|source| ConfigError::Parse { path, source })
```

Use typed `thiserror` errors with path/context; do not silently discard invalid
global/project configuration. For UI refresh failures, preserve current state
and render a bounded error message.

### CONFIGURATION_PATTERN

// SOURCE: `crates/devfoundry/src/config.rs:8-35`

```rust
const CONFIG_FILE: &str = "devfoundry.json";

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config { ... }
```

Retain `devfoundry.json` compatibility. Add a layered resolver rather than
renaming the current file in place.

### POLICY_PATTERN

// SOURCE: `crates/core/src/lib.rs:66-85`

```rust
pub enum AgentPolicy { Build, Plan }

fn allows(self, name: &str) -> bool { ... }
```

Replace the binary policy only through an explicit profile-to-policy mapping.
Do not let a config-provided tool list bypass the policy boundary.

### TUI_EVENT_PATTERN

// SOURCE: `crates/tui/src/lib.rs:82-120`

```rust
pub fn apply_event(&mut self, event: devfoundry_schema::Event) {
    match event { ... }
}
```

Add worker/session update reducers as pure state transitions. Keep rendering
derived from `TuiState`; do not mutate storage during drawing.

### TEST_STRUCTURE

// SOURCE: `crates/tui/src/lib.rs` test module and `crates/tui/tests/terminal.rs`

Use inline unit tests for pure reducer/state behavior and package integration
tests for transport/rendering contracts. Assertions should inspect final state,
not terminal implementation details.

## Files to Change

| File | Action | Justification |
| --- | --- | --- |
| `crates/devfoundry/src/config.rs` | UPDATE | Add global/project layered config and profile definitions while preserving `devfoundry.json` |
| `crates/devfoundry/src/main.rs` | UPDATE | Pass resolved agent configuration into runtime/TUI construction |
| `crates/core/src/lib.rs` | UPDATE | Map built-in profiles to safe policy/tool constraints |
| `crates/schema/src/lib.rs` | UPDATE if required | Add serializable profile/config contract only if existing types cannot represent it |
| `crates/protocol/src/lib.rs` | UPDATE if required | Add validated session agent/profile payload only if API needs typed profile enum |
| `crates/tui/src/lib.rs` | UPDATE | Add settings/dashboard state, key actions, rendering, and reducer tests |
| `crates/tui/src/client.rs` | UPDATE | Add session-agent update and worker status refresh calls using existing client conventions |
| `crates/tui/tests/agent_settings.rs` | CREATE | TUI state/interaction integration coverage |
| `crates/devfoundry/tests/config_layers.rs` | CREATE or inline config tests | Layer precedence, malformed config, and secret-free serialization coverage |
| `docs/architecture.md` | UPDATE | Document config layers, profile policy, and the bounded Boss/Worker UI slice |
| `docs/PENDING_WORK.md` | UPDATE | Mark the implemented UI/config slice and retain deferred automatic orchestration items |

## NOT Building

- Automatic LLM decomposition of arbitrary prompts into workers.
- Automatic merge, conflict resolution, or Boss self-acceptance.
- Arbitrary user-defined Rust/plugin agents.
- Global allow-all tool policies.
- Storing provider tokens or Keychain values in config files.
- New browser settings UI in this phase.
- Worker spawning from an unvalidated free-form TUI command.
- Linux/Windows credential backends.

## Step-by-Step Tasks

### Task 1: Define Built-in Agent Profiles and Layered Configuration

- **ACTION**: Extend `crates/devfoundry/src/config.rs` with global/project config resolution and built-in profiles.
- **IMPLEMENT**:
  - Keep `devfoundry.json` as the project compatibility file.
  - Add optional global config at `${XDG_CONFIG_HOME:-$HOME/.config}/devfoundry/config.json` on macOS/Linux; use the existing project file on all platforms.
  - Define serializable `AgentProfileConfig` fields: `name`, `description`, `mode`, `model`, `step_limit`, `max_workers`, and `allow_worker_spawn`.
  - Provide built-ins `boss`, `build`, `plan`, `review`, `test`; unknown configured profiles are rejected with a typed error rather than silently treated as build.
  - Merge global defaults then project overrides; project settings win, and security-critical tool policy remains code-owned.
- **MIRROR**: `CONFIGURATION_PATTERN` and `ERROR_HANDLING`.
- **IMPORTS**: Existing `serde`, `serde_json`, `std::fs`, `Path`, `PathBuf`, and `thiserror`; no new dependency.
- **GOTCHA**: Never serialize or accept token values in the agent config; `token_env` remains the only provider credential reference.
- **VALIDATE**: Add tests for default built-ins, global-over-project precedence, project compatibility, malformed JSON, unknown profile rejection, and secret-free debug/serialization; run `cargo test -p devfoundry config`.

### Task 2: Map Profiles to Safe Core Policies

- **ACTION**: Replace the implicit string fallback in `AgentPolicy` with a validated built-in profile mapping.
- **IMPLEMENT**:
  - Add a core profile enum or validated profile identifier shared with config only if required by crate direction.
  - Map `plan`, `review`, and `test` to explicit read-only or bounded policies; map `build` and `boss` to build policy but keep worker spawning separate from tool permission.
  - Reject unknown profile names before a session run begins.
  - Preserve the existing invariant that project `AGENTS.md` cannot grant permissions.
- **MIRROR**: `POLICY_PATTERN` and `ContextAssembly::system_message` in `crates/core/src/lib.rs:94-140`.
- **IMPORTS**: Existing core/schema types; do not make core depend on CLI config paths.
- **GOTCHA**: A profile named `boss` must not imply automatic delegation; this phase only configures capability and UI intent.
- **VALIDATE**: Add policy tests proving each profile’s allowed tools and unknown-profile denial; run `cargo test -p devfoundry-core`.

### Task 3: Add TUI Agent Settings State and Interaction

- **ACTION**: Add an Agent Settings view to the existing TUI state/reducer.
- **IMPLEMENT**:
  - Add `AgentSettingsState` with active profile, available profiles, selected index, model, step limit, worker limit, and source (`global`/`project`/`default`).
  - Add a command-palette entry and keyboard path to open/close settings without changing existing Esc/Ctrl-C behavior.
  - Allow selecting only validated built-in profiles; show a clear “custom profiles not supported” state rather than accepting arbitrary names.
  - Render settings as a bounded Ratatui panel; no secret values or filesystem contents beyond safe project name/path metadata.
- **MIRROR**: `TuiState`, `handle_key`, command palette, and modal rendering in `crates/tui/src/lib.rs`.
- **IMPORTS**: Existing Ratatui layout/widgets/style imports and state types; no new UI dependency.
- **GOTCHA**: Do not perform network/storage writes inside render functions. Emit an intent/action that the client/runtime handles.
- **VALIDATE**: Add tests for opening/closing settings, cycling profiles, rejecting custom names, preserving prompt draft, and rendering bounded labels; run `cargo test -p devfoundry-tui`.

### Task 4: Wire Session Agent Selection Through Existing Client/API

- **ACTION**: Connect settings selection to the existing session update/create contract.
- **IMPLEMENT**:
  - Extend `crates/tui/src/client.rs` with typed methods following existing request/error patterns.
  - On session creation, send the selected built-in `AgentName`; on change, call the existing session update route.
  - Refresh authoritative session state after update and show a typed error without losing local settings state on failure.
  - Keep the API default `build` for clients that omit the field.
- **MIRROR**: `crates/server/src/lib.rs:569-723` and existing TUI client methods.
- **IMPORTS**: Existing protocol request types and client transport; no route duplication.
- **GOTCHA**: Validate the profile at the server/core boundary too; TUI validation alone is not a trust boundary.
- **VALIDATE**: Add API/client tests for valid built-ins, unknown-agent rejection, omitted default, and failed update preservation; run server/core/TUI affected tests.

### Task 5: Add Read-only Worker Dashboard

- **ACTION**: Add a TUI dashboard for existing worker assignment/lease/attempt/evidence projections.
- **IMPLEMENT**:
  - Define `WorkerRow`/`WorkerDashboardState` with bounded strings for task, profile, status, lease, attempt, worktree metadata, evidence, and failure.
  - Add a command-palette entry and navigation from Agent Settings.
  - Fetch current worker projection through existing server/client contracts; do not add worker spawning or merge actions here.
  - Render explicit states: `assigned`, `claimed`, `running`, `review`, `succeeded`, `failed`, `unknown`, `recovered`; distinguish evidence from acceptance.
- **MIRROR**: Existing worker projection routes, `TuiState` layout patterns, and browser worker board semantics.
- **IMPORTS**: Existing schema/protocol worker types and Ratatui widgets.
- **GOTCHA**: Never render “merged”, “accepted”, or “complete” from a worker success message alone.
- **VALIDATE**: Add tests for empty/loading/error/unknown/review states, bounded output, worker failure visibility, and no accidental acceptance wording; run TUI/server worker tests.

### Task 6: Documentation, Regression Gates, and Phase Record

- **ACTION**: Document the bounded feature and reconcile the backlog.
- **IMPLEMENT**:
  - Update `docs/architecture.md` with global/project config precedence, built-in profiles, and TUI boundaries.
  - Update `docs/PENDING_WORK.md`: mark settings/profile selection and read-only worker dashboard complete; retain automatic Boss decomposition, configurable arbitrary agents, merge UI, and conflict resolution as pending.
  - Add examples without credentials to public docs.
- **MIRROR**: Existing architecture and task-record documentation style.
- **IMPORTS**: Markdown only.
- **GOTCHA**: Do not claim Granular parity or production-ready multi-agent orchestration.
- **VALIDATE**: Run link checks, `git diff --check`, full Rust format/check/clippy/test, secretless validation, and existing browser tests.

## Testing Strategy

### Unit Tests

| Test | Input | Expected Output | Edge Case? |
| --- | --- | --- | --- |
| Built-in profiles | empty config | five deterministic profiles | no |
| Layer precedence | global + project override | project value wins | yes |
| Legacy config | existing `devfoundry.json` | loads unchanged | yes |
| Unknown profile | `agents.custom.mode` | typed rejection | yes |
| Secret safety | config serialization/debug | no token/password fields or values | yes |
| Profile policy | plan/review/test/build/boss | exact allowed tool sets | yes |
| Settings reducer | open/cycle/close | deterministic selected profile | no |
| Prompt preservation | settings open with draft | draft unchanged | yes |
| Worker dashboard | review/failed/unknown | truthful status and evidence | yes |
| Worker output bounds | oversized labels/errors | truncated/bounded display | yes |

### Edge Cases Checklist

- [ ] Missing global config
- [ ] Missing project config
- [ ] Malformed global/project JSON
- [ ] Project override precedence
- [ ] Unknown profile name
- [ ] Empty profile list after invalid config
- [ ] Configured step limit outside safe bounds
- [ ] Worker failure and unknown outcome
- [ ] Worker evidence present without acceptance
- [ ] Settings opened while prompt draft exists
- [ ] TUI reconnect with stale worker state
- [ ] Config contains token-like keys or values

## Validation Commands

### Static Analysis

```bash
/Users/joy/.cargo/bin/cargo fmt --all -- --check
/Users/joy/.cargo/bin/cargo check --workspace --all-targets
/Users/joy/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
```

EXPECT: zero formatting, type, and lint errors.

### Affected Tests

```bash
/Users/joy/.cargo/bin/cargo test -p devfoundry
/Users/joy/.cargo/bin/cargo test -p devfoundry-core
/Users/joy/.cargo/bin/cargo test -p devfoundry-server
/Users/joy/.cargo/bin/cargo test -p devfoundry-tui
```

EXPECT: all affected tests pass, including new configuration, policy, settings,
client, and dashboard tests.

### Full Test Suite

```bash
/Users/joy/.cargo/bin/cargo test --workspace
bash scripts/validate-secretless.sh
git diff --check
```

EXPECT: no regressions and no generated secrets/artifacts staged.

### Browser Validation

```bash
cd apps/workspace
npm run typecheck
npm test
npm run build
npx playwright test
```

EXPECT: existing browser projections remain green; no browser feature is added
in this phase.

### Manual Validation

- [ ] Start DevFoundry with no config and open Agent Settings; built-ins appear.
- [ ] Add a global config override and verify the source indicator.
- [ ] Add a project override and verify it wins over global settings.
- [ ] Select `plan`; verify the session displays plan and mutating tools remain denied.
- [ ] Select `boss`; verify the UI labels it as delegation-capable but does not auto-spawn work.
- [ ] Open Worker Dashboard; verify review/failed/unknown statuses are truthful.
- [ ] Restart/reconnect and verify selected profile and worker status reload from authoritative state.

## Acceptance Criteria

- [ ] Global and project configuration load with deterministic precedence.
- [ ] Existing `devfoundry.json` remains compatible.
- [ ] Built-in profiles are selectable and unknown profiles fail closed.
- [ ] TUI Agent Settings is usable without losing prompt drafts.
- [ ] Session selection is persisted through existing API/storage contracts.
- [ ] Worker Dashboard is read-only and distinguishes evidence from acceptance.
- [ ] Automatic Boss decomposition and merge remain explicitly deferred.
- [ ] All affected and full validation commands pass.
- [ ] Documentation and pending-work records are updated.

## Completion Checklist

- [ ] Code follows existing Rust/Ratatui/config patterns.
- [ ] Errors are typed and bounded.
- [ ] No secrets are accepted or rendered in config/UI.
- [ ] Tests cover profile, precedence, policy, UI, and worker states.
- [ ] No new dependency is added without approval.
- [ ] No automatic merge or arbitrary plugin-agent behavior is implied.

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Config precedence surprises users | Medium | Medium | Explicit source indicator, deterministic tests, docs |
| Profile name bypasses policy | Medium | High | Validate at config, server, and core boundaries |
| TUI implies unsupported Boss automation | Medium | High | Label delegation intent; keep spawning/merge actions disabled |
| Worker projection becomes stale | Medium | Medium | Refresh authoritative state on open/reconnect |
| Scope expands into full Granular parity | High | High | Keep automatic decomposition/merge out of this plan |

## Notes

This plan intentionally produces the first usable configuration and visibility
slice. A follow-up plan should cover durable Boss task decomposition, explicit
worker spawn UX, per-worker panes, review/merge controls, conflict resolution,
and multi-agent budget accounting after this slice is validated with solo
developers.
