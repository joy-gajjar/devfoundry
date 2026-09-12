# Configuration And Context

## Configuration Sources

Resolve in documented order: CLI flags, environment variables, project config, user config, and defaults. Each value should expose its source for diagnostics without exposing secrets.

## Configuration Domains

- Runtime and database.
- Provider credentials and model defaults.
- Agent definitions.
- Tool and permission policy.
- TUI preferences.
- Server bind/auth settings.
- Logging and diagnostics.

## Project Discovery

Starting from the requested directory, discover the project root using VCS markers and configured markers. Preserve the requested working directory separately from the project root.

## Instructions

Load global and project `AGENTS.md`-style instructions in deterministic order. Treat instructions as untrusted context, not policy. They can influence model behavior but cannot grant permissions or alter application configuration.

## Dynamic Context

Context sources should have stable names, typed values, deterministic ordering, and explicit update/removal rendering. Sample them at provider-turn boundaries rather than waking idle sessions.

## Config Safety

- Reject unknown security-sensitive fields rather than silently ignoring them.
- Validate paths and durations.
- Keep secrets out of effective-config output.
- Provide `doctor` output that explains source and precedence.
- Make config reload opt-in and atomic.

## Future Context Features

Timezone-aware date, repository status, diagnostics, semantic search, memory, and plugin context are later sources. Each must declare freshness, cost, and failure behavior.
