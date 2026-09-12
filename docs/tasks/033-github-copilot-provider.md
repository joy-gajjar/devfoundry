# Task: GitHub Copilot Provider Configuration

## Goal
Expose one supported external provider, GitHub Copilot, with a safe environment-backed token and configurable base URL.

## Scope
- Included: Copilot-compatible chat-completions transport, `devfoundry.json` configuration, token-presence diagnostics, local HTTP/SSE fixtures, and documentation.
- Explicitly excluded: Anthropic, Gemini, Bedrock, other external adapters, production fake-provider mode, secret persistence, and interactive OAuth/device-token acquisition.

## Design
The provider kind is `github-copilot`. `provider.base_url` defaults to `https://api.githubcopilot.com` and `provider.token_env` defaults to `GITHUB_COPILOT_TOKEN`; the adapter appends `/chat/completions` and sends the token as a Bearer credential. Users may obtain a Copilot token through their approved GitHub OAuth/device-token tooling and hand it to DevFoundry through the environment. The application only reports token presence, never its value. The fake provider remains available only inside provider tests.

## Implementation
- Renamed the production adapter and runtime selection to `GithubCopilotProvider` and `github-copilot`.
- Replaced API-key configuration with base URL and token environment configuration.
- Updated session defaults, release doctor smoke checks, and provider planning records.
- Added a base URL/token fixture assertion and retained deterministic HTTP/SSE fixtures.

## Verification
- `cargo fmt --all -- --check`: not run; `cargo` is unavailable in the execution environment.
- `cargo check --workspace --all-targets`: not run; `cargo` is unavailable in the execution environment.
- `cargo clippy --workspace --all-targets -- -D warnings`: not run; `cargo` is unavailable in the execution environment.
- `cargo test --workspace`: not run; `cargo` is unavailable in the execution environment.
- `cargo run -p devfoundry -- --json doctor`: not run; `cargo` is unavailable in the execution environment.
- `git diff --check`: passed.

## Risks And Follow-Up
- Live Copilot completion and token-expiry behavior still need controlled external validation via
  the bounded, opt-in procedure in `docs/copilot-network-smoke.md`.
- OAuth/device-token acquisition and OS keychain storage remain intentionally deferred.
