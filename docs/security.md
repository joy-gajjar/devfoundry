# Security Policy

## Scope

DevFoundry is a local-first coding agent. Security-sensitive boundaries include
project file access, shell execution, provider configuration, the local API,
session storage, release artifacts, and update notifications.

The default posture is fail-closed: operations that cross a project, process,
network, or credential boundary require explicit policy and, where applicable,
user approval. The detailed runtime threat model and security gates are in
[`planning/08-permissions-and-security.md`](planning/08-permissions-and-security.md).

## Reporting A Vulnerability

Do not open a public issue for an unpatched vulnerability. Use the private
GitHub vulnerability reporting flow described in the repository-level
[`SECURITY.md`](../SECURITY.md). If that flow is unavailable, report privately
to the repository owner and mark the message `security-sensitive`.

Include the affected version or commit, operating system, impact, reproduction
steps, and the smallest safe proof of concept. Do not include real credentials
or private project data.

Maintainers should acknowledge reports within five business days, coordinate a
fix and disclosure timeline with the reporter, and credit the reporter unless
they request anonymity. Configure the repository-specific private contact
before public distribution.

## Secret Handling

The complete operator credential lifecycle, including the Copilot token
handoff, is documented in [`credentials.md`](credentials.md).

- Never commit provider tokens, signing keys, registry credentials, or private
  key material.
- Release automation is intentionally secretless. It builds, packages,
  checksums, and validates artifacts without signing credentials or external
  provider access.
- Signing and package publication happen only as a separate trusted-host
  handoff described in [`release.md`](release.md).
- Diagnostics, logs, fixtures, and release notes must contain redacted values,
  not tokens, authorization headers, prompts, or local secret paths.

## Release Security

Every release candidate must pass the local secretless gate:

```sh
bash scripts/validate-secretless.sh
```

Archives are verified with SHA-256 before installation. A checksum detects
corruption but does not prove publisher identity. Until an external signing
handoff is complete, releases must be described as unsigned.

## Supported Versions

Security fixes are applied to the current release line. Users should upgrade
to the latest release and follow the documented rollback procedure if an
upgrade causes an operational regression.
