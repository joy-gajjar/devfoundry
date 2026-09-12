# TUI Feature Gaps and Fragilities

Assessment of `crates/tui/src/lib.rs` and `crates/tui/src/client.rs` against `docs/TUI_FEATURE_ROADMAP.md`. The P0 core skeleton works but fails several of its own acceptance criteria; P1 to P3 are largely unbuilt.

> Note: `docs/TUI_FEATURE_ROADMAP.md` "Current Capabilities" overstates reality. Claims of "durable event replay", tool "exit codes", and "recovery" are not fully present in code. That section should be corrected.

---

## P0 Gaps

### Item 1 — Transcript view (Partial)
- Implemented: headings, bullets, fenced code with language label, tables, indent-preserving code wrap.
- Missing: link/file-path highlighting (no detection logic).
- Half: tables have no horizontal scrolling; cell wrapping mangles alignment (`lib.rs:706`).
- Fragility: wrapping is by char count, not Unicode display width (`lib.rs:641`), so CJK/wide glyphs misalign the 80-col target.

### Item 2 — Structured tool timeline (Minimal, acceptance not met)
- Tool events are flat transcript strings (`lib.rs:81-98`); a yellow gutter is the only structure.
- Missing: expand/collapse, start time/elapsed, exit code/failure reason, per-tool output correlation (`call_id` unused).

### Item 3 — Permission modal (Partial)
- Implemented: modal, risk heuristic, Y/N/Esc reducer, two-step open-then-decide.
- Missing: "Allow for session" scope (hardcoded "once", `lib.rs:467`); real "Cancel execution" (Cancel only closes the modal, `lib.rs:240`); reason/why text (risk is a string heuristic, `lib.rs:514`).

### Item 4 — Session browser (Partial)
- Implemented: filter, resume, rename, agent/model change, detail pane.
- Missing: delete action, export action, last-activity time, interrupted/error badges (status shown as raw debug, `lib.rs:1142`).

### Item 5 — Model/agent picker (Partial)
- Implemented: model/agent select, persisted via `update_session`.
- Missing: context window/capability display (`ModelOption.context_window` ignored, `lib.rs:965`); keyboard search; agent descriptions (hardcoded `["build","plan","review"]`, `lib.rs:825`); current-selection indicator.

### Item 6 — Input editor (Not met; single-line only)
- Missing: multi-line editing, cursor movement, word navigation, delete-word/delete-to-end, history, draft restoration, emacs/vim modes.
- Bug: draft is lost on prompt failure — input is consumed by `push_user_prompt` before send (`lib.rs:116,1355`); the failure path does not restore it.

---

## P1 to P3 (Essentially Unbuilt)

| Item | Priority | Status | Reference |
|---|---|---|---|
| Context inspector | P1 | Missing | — |
| Diff review panel | P1 | Missing | — |
| Test/validation panel | P1 | Missing | — |
| Recovery center | P1 | Missing (reconnect loop only, `lib.rs:1240`) | — |
| Transcript search | P1 | Missing | — |
| Session tabs/workspaces | P1 | Missing (single session, `lib.rs:1231`) | — |
| Command palette (fuzzy/desc/keybinds) | P1 | Stub (3 hardcoded keys, `lib.rs:256`) | — |
| Cost/token monitor | P1 | Missing | — |
| Git / Diagnostics / MCP / LSP / PTY panels | P2 | Missing | — |
| Agent plan/timeline | P2 | Missing | — |
| Bookmarks/annotations | P2 | Missing | — |
| Themes/layouts | P2 | Missing (hardcoded colors) | — |
| Evidence-first timeline | P3 | Missing | — |
| Explainable permissions | P3 | Partial | Item 3 |
| Recovery-first sessions | P3 | Missing | Item 10 |
| Context transparency | P3 | Missing | Item 7 |
| Provider capability visibility | P3 | Missing | Item 5 |

---

## Cross-Cutting Fragilities

| ID | Issue | Location |
|---|---|---|
| T1 | Full transcript rendered twice per frame (O(2N)) | `lib.rs:740,350` |
| T2 | Draft lost on prompt failure | `lib.rs:116,1355` |
| T3 | Arrows/PageUp/Home/End captured as scroll even while typing | `lib.rs:174-205` |
| T4 | `Resize` not handled in the connected loop | `lib.rs:1302` |
| T5 | Permission arrival relies on 500 ms poll; `PermissionRequested` is a no-op | `lib.rs:99,1276` |
| T6 | `ToolStarted`/`ToolFinished` drop `call_id`; concurrent tools ambiguous | `lib.rs:81-98` |
| T7 | SSE `id:` requires trailing space; `id:` without space hard-errors the frame | `client.rs:180` |
| T8 | Home sentinel `usize::MAX` desyncs the history/bottom label | `lib.rs:199,380` |

---

## Missing TUI Tests

- Table horizontal handling and Unicode width wrapping.
- Draft loss on prompt failure (would catch T2).
- Picker keybindings (`r`/`a`/`m`/`n`/`/`).
- `run_connected_loop` event routing and permission-resolve flow.
- Reconnect/backoff behavior.
- Scroll clamping / Home sentinel interaction.
- `context_window` surfacing in the model picker.
- `client.rs` error mapping and `events` id-parse failure.
