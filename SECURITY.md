# Security Policy

DevFoundry is under active development. Do not use this repository's current
status as a claim of production security or platform completeness.

## Reporting a Vulnerability

Please do not publish credentials, exploit details, or sensitive reproduction
data in a public issue. Use GitHub's private security advisory/reporting flow
for this repository when available. If that flow is not enabled, open a minimal
public issue containing only the affected area and the phrase `security report`
so a maintainer can provide a private channel. Never include tokens or secret
values in the issue.

## Security Model

Read [`docs/security.md`](docs/security.md) for the detailed model. Important
properties include:

- Tool capabilities and permissions are explicit and separated by trust domain.
- Filesystem operations enforce project-root, traversal, symlink, size, and
  stale-content checks.
- Process operations bound arguments, output, timeouts, cancellation, and
  descendant cleanup where supported.
- Provider credentials are not inherited by child tools or serialized into
  exports and diagnostics.
- Native Keychain resolution requires a validated secret binding and is
  fail-closed on unsupported platforms.

## Dependency Advisories

Run `cargo audit` and `cargo deny check` before dependency changes. The current
RUSTSEC-2023-0071 exception is narrowly documented in [`.cargo/audit.toml`](.cargo/audit.toml)
