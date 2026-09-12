# Task: Provider Configuration

## Goal

Replace the implicit fake provider runtime with explicit GitHub Copilot configuration and an environment-backed access token.

## Scope

- Included: provider config schema, GitHub Copilot provider selection, base URL configuration, token environment lookup, and safe doctor diagnostics.
- Explicitly excluded: production fake-provider configuration, keychain storage, OAuth/device flow implementation, multiple simultaneous providers, provider catalog, and other provider adapters.

## Design

`devfoundry.json` selects `provider.kind`, `provider.base_url`, `provider.token_env`, and `model`. The only production kind is `github-copilot`; startup fails clearly when its token environment variable is missing. The default is `https://api.githubcopilot.com` with `GITHUB_COPILOT_TOKEN`. The token may be a GitHub-issued Copilot access token obtained through a separately managed OAuth/device-token handoff; DevFoundry does not implement that interactive handoff. Doctor reports provider kind, base URL, and whether the configured environment variable exists, never the secret value.

## Implementation

- Added provider configuration types and defaults.
- Added runtime provider factory.
- Added GitHub Copilot provider selection.
- Restricted the fake provider implementation to test builds.
- Added safe provider diagnostics.
- Added SQLite lifecycle integration coverage as part of production verification.

## Verification

Run:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p devfoundry -- doctor`

## Risks And Follow-Up

- The provider currently sends only the initial request/tool shape and needs live Copilot completion coverage.
- Tokens remain environment-backed; OS keychain and interactive OAuth/device-token support are future work.
