# Documentation and Naming Issues

The Rust source is clean: no `opencode.json`/`opencode.db`/`"opencode"` references remain in any `crates/*/src`, config uses `devfoundry.json`, and the binary is `devfoundry`. The remaining stale references are in docs and leftover artifacts.

---

## Stale `opencode` References in Docs

### User-facing CLI naming (highest priority)
- `docs/planning/10-tui-and-cli.md:5-10` documents commands as `opencode run`, `opencode serve`, `opencode config`, `opencode doctor`. The actual binary is `devfoundry`. This doc mis-names the shipped surface.

### Task docs referencing the old name/CLI
- `docs/tasks/000-phase-0-foundation.md:5,14,20,31,32` — "OpenCode reimplementation", `crates/opencode`, `cargo run -p opencode`
- `docs/tasks/001-phase-0-configuration-and-boundaries.md:14,16,34` — `opencode.json`, `opencode-protocol`, `opencode-storage`, `cargo run -p opencode`
- `docs/tasks/007-phase-5-integration.md:14,20` — `opencode serve`, `opencode.db`
- `docs/tasks/013-recovery-and-cancellation.md:21` — `opencode serve`
- `docs/tasks/020-cli-server-smoke-fix.md:26-28,33,34` — `cargo run -p opencode`, `target/debug/opencode serve`, `/tmp/opencode-rust-smoke.db`

### Planning docs
- `docs/planning/README.md:1` — "Rust OpenCode Preplanning"
- `docs/planning/17-development-agents.md:5` — references `.opencode/agent/` (legitimate: the agent loader still uses `.opencode/` per `docs/tasks/021-devfoundry-rename.md`)

### Legitimate references (keep)
These describe upstream as inspiration and should remain:
- `docs/OPENCODE_COMPARISON.md`
- `docs/TUI_FEATURE_ROADMAP.md` (upstream comparison section)
- `docs/planning/00-goals-and-boundaries.md`
- `docs/planning/README.md` (upstream attribution line)

---

## Task Record Numbering

`docs/tasks/` contains duplicated numeric prefixes from parallel work batches: `028` x3, `029` x4, `030` x5, `031` x4, `033` x3, `037` x5, `044` x3, `060` x3. This makes "next task number" ambiguous. The next single record created by this audit is `061`.

- Direction: Decide whether to renumber historically or accept duplicates and only enforce uniqueness going forward.

---

## Doc-vs-Code Drift

### Model/agent catalog
- `docs/OPENCODE_COMPARISON.md:96,123` claims a curated catalog endpoint and model picker are implemented.
- `docs/PENDING_WORK.md:26,41` still lists model/agent catalog + selection API as pending.
- Code has catalog/model support in `crates/server/src/lib.rs` and `crates/tui/src/lib.rs`, but the catalog is hardcoded (see 03-low-and-cleanup L4).
- Direction: Reconcile the two docs; describe the catalog as implemented-but-hardcoded.

### Inflated safety claim
- `docs/OPENCODE_COMPARISON.md:157` states sensitive-file policy cannot be overridden by an allow-all broker. The audit (01-critical-bugs C1) shows `bash`/`pty` bypass it. This claim must be corrected.

### TUI roadmap "Current Capabilities"
- `docs/TUI_FEATURE_ROADMAP.md` lists "durable event replay", tool "exit codes", and "recovery" as current. These are not fully present (see 04-tui-gaps). Correct the section.

---

## Summary

| Area | Issue |
|---|---|
| CLI naming | `planning/10-tui-and-cli.md` uses `opencode` commands |
| Task docs | Several use old crate/CLI/db names |
| Numbering | Duplicate task prefixes; ambiguous next number |
| Catalog drift | COMPARISON says done, PENDING says pending, code is hardcoded |
| Safety claim | COMPARISON:157 contradicted by bash/pty bypass |
| Roadmap | "Current Capabilities" overstates implemented features |
