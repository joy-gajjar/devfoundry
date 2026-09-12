# Distribution And Operations

## Release Artifacts

Build checksummed archives for the supported targets listed in [`../support-matrix.md`](../support-matrix.md), with one binary and license metadata. The machine-readable source is `release/platforms.json`; each archive has a SHA-256 sidecar and is validated before upload. Provide a single binary with no runtime dependency for the MVP. Signing, macOS x64/Linux ARM64 builds, and package publication remain deferred.

## Installation Channels

Start with GitHub release archives and the documented `scripts/manage-install.sh` install/upgrade/rollback scaffold. Package-manager intent is recorded in `release/package-managers.json`, while actual publication remains deferred until the signed handoff and maintainer credentials are available.

## Updates

Never silently replace a running binary. Show version, channel, commit, and update source. Support rollback to the prior known-good release.

## Diagnostics

`doctor` should report OS, binary version, project root, database path, provider configuration presence (not secrets), connectivity, shell, terminal, and permission capabilities. Add exportable redacted diagnostics.

DevFoundry reports the provider kind, base URL, and token-presence status without printing credentials. Production startup requires a configured GitHub Copilot token; the fake provider is test-only and is not a production configuration option.

`devfoundry --json doctor` provides redacted machine-readable diagnostics. Release CI builds, packages, checksum-verifies, and uploads macOS Apple Silicon, Linux x64, and Windows x64 artifacts, runs workspace tests, and checks that a sentinel provider secret is absent from doctor output.

The API remains local-only by default. For a non-loopback bind, provide the API token through `DEVFOUNDRY_API_TOKEN` or a variable name selected by `DEVFOUNDRY_API_TOKEN_ENV`, and provide exact browser origins through `DEVFOUNDRY_ALLOWED_ORIGINS`. Do not place token values in command arguments, configuration files, or logs.

## Logging

Structured logs with levels and request/session correlation. Default logs are useful but quiet. Debug logs must still redact credentials and avoid dumping full source files or prompts.

## Telemetry

No telemetry by default. If introduced, it must be opt-in, documented, minimal, and never include source content, prompts, provider payloads, or secrets.

## Backups And Data Management

Document database location, session export, deletion, and backup. A consistent database backup is available with `devfoundry database backup --database PATH --output PATH`; it refuses to overwrite an existing destination. Use `devfoundry --json database diagnose --database PATH` for bounded, content-free integrity and row-count diagnostics. Portable session export/import and managed tool-output cleanup remain separate follow-up work.

## Support Model

Publish troubleshooting for credentials, permissions, terminal restoration, database corruption, provider errors, and network proxies. Every internal error should carry a diagnostic ID users can report without exposing content.

## Release Gate

Verify clean install, upgrade, rollback, migration, offline startup, cancellation, redaction, and artifact checksum on every supported platform. CI proves packaging, archive contents, checksum verification, doctor redaction, and an unsigned publication plan; `scripts/test-release.sh` proves the local lifecycle, while signing and live publication remain trusted-owner actions.
