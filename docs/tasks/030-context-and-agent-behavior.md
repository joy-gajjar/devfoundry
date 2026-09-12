# Task: Context And Agent Behavior

## Goal
Implement the first deterministic context and agent-policy slice: ordered project instructions, minimal session context, and distinct build/plan tool exposure.

## Scope
- Included: `AGENTS.md` discovery from project root to requested directory, system context assembly, build/plan tool policy, and focused tests.
- Excluded: configured instruction-file precedence, context budgets, compaction, context epochs, subagents, provider capability checks, and API/TUI changes.

## Design
The runner loads existing `AGENTS.md` files in outermost-to-innermost order. Instruction text is untrusted model context and cannot grant permissions or alter application policy. The system message contains only the baseline safety statement, session agent/model metadata, and discovered instructions. The `build` policy exposes the registered tool set; `plan` exposes only `read`, `glob`, and `grep`. Policy is applied to provider definitions and checked again before execution.

## Implementation
- Updated `crates/core/src/lib.rs` with context discovery/assembly and build/plan policy enforcement.
- Updated `crates/tools/src/lib.rs` with filtered tool-definition projection.
- Added unit coverage for instruction ordering and policy boundaries.

## Verification
- `cargo fmt --all -- --check`: blocked because `cargo` is unavailable in the environment.
- `cargo test --workspace`: blocked because `cargo` is unavailable in the environment.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: blocked because `cargo` is unavailable in the environment.
- `git diff --check`: passed; the checkout has no tracked files, so the working-tree status was inspected instead.

## Risks And Follow-Up
- Configured instruction files in `devfoundry.json` are not yet connected to the runner because their precedence and path contract need an explicit decision.
- Context budgets, compaction, subagents, and background tasks remain deferred until the base runner contract is expanded and tested.
