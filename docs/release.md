# Release Operations

Supported targets and binary names are defined in [`../release/platforms.json`](../release/platforms.json). Archives are target-named `tar.gz` files containing the binary and `LICENSE`, with an adjacent SHA-256 sidecar.

Use [`release-verification.md`](release-verification.md) as the end-to-end
candidate and trusted-host checklist. Credential lifecycle and Copilot token
handling are documented in [`credentials.md`](credentials.md).

## Build And Validate

The release workflow builds each target, runs workspace tests, packages the archive, validates its contents and executable mode, verifies the checksum, launches the packaged binary with `doctor --json`, and writes `release-evidence.json` beside the archive. The same evidence path runs on macOS Apple Silicon, Windows, and Linux; it removes `GITHUB_COPILOT_TOKEN` for the smoke and checks that no provider token is configured. These steps require no signing secret.

The workflow also runs the focused TUI terminal contract on every release
runner. This covers resize, paste/input dispatch, exit handling, and cleanup
without requiring a host TTY. A release owner must additionally perform the
native-terminal smoke in [`tui-validation.md`](tui-validation.md) for each
artifact and record the terminal environment with the release evidence.

Run the secretless release gate before building or handing artifacts to a
trusted signing host:

```sh
bash scripts/validate-secretless.sh
```

The gate validates release JSON, shell syntax, version format, workflow
credential wiring, and the version/update/security documentation locally.

```sh
bash scripts/package-release.sh --binary target/<target>/release/devfoundry --target <target> --version <version> --output dist
bash scripts/verify-release.sh --archive dist/devfoundry-<version>-<target>.tar.gz --checksum dist/devfoundry-<version>-<target>.tar.gz.sha256 --target <target> --version <version>
bash scripts/release-evidence.sh --archive dist/devfoundry-<version>-<target>.tar.gz --checksum dist/devfoundry-<version>-<target>.tar.gz.sha256 --target <target> --version <version> --output dist/release-evidence.json --launch-doctor
```

Validation must happen before installation. The verifier checks the target/version directory, exact binary and `LICENSE` entries, executable mode, and checksum. SHA-256 detects corruption or transfer errors; it does not authenticate the publisher.

## macOS Smoke Gate

The supported Apple target is `aarch64-apple-darwin` on `macos-14`. Before publishing, run the workspace format, check, clippy, and test gates plus:

```sh
cargo build --release --target aarch64-apple-darwin -p devfoundry
```

The CI macOS job also exercises SQLite paths containing spaces and validates the target build.

## Install, Upgrade, And Rollback

`scripts/manage-install.sh` verifies the sidecar and archive contents before extraction, installs into a versioned user-owned directory, points `current` at the new release, and retains one `previous` release.

```sh
bash scripts/manage-install.sh install <archive> <archive>.sha256 [<prefix>]
bash scripts/manage-install.sh upgrade <archive> <archive>.sha256 [<prefix>]
bash scripts/manage-install.sh rollback [<prefix>]
```

The default prefix is `$HOME/.local/share/devfoundry`; set `DEVFOUNDRY_PREFIX` or pass a prefix explicitly for tests. This scaffold does not replace a running binary, modify shell startup files, migrate databases, or install updates. The explicit macOS-only `devfoundry update --check` command only reads public release metadata and reports a newer release; a launcher/service integration must stop new processes before switching versions.

Rollback is filesystem-only and requires both `current` and `previous` releases. Database migrations must remain forward-compatible until a separate migration rollback contract exists.

## Package Metadata And Publication Handoff

`release/platforms.json` defines build targets. `release/package-managers.json` records the intended Homebrew, cargo-binstall, and WinGet channels without claiming that any package has been published. `release/publication-plan.json` describes the artifact handoff contract. The workflow runs `scripts/prepare-publication.sh` to create an unsigned plan containing archive/checksum counts; it does not publish to a package manager.

```sh
bash scripts/prepare-publication.sh \
  --version <version> \
  --artifact-dir dist \
  --output dist/publication-plan.json
```

The generated plan, evidence record, archives, and checksums are uploaded together as one workflow artifact for the release owner. No registry token is required by this step. The plan is a handoff request, not proof of publication: the release owner must independently update Homebrew, WinGet, or cargo-binstall using the exact verified digest.

Package-manager scaffolding is maintained in `release/package-managers.json`. The Homebrew and WinGet files are templates with release-owner placeholders; cargo-binstall consumes the target-named GitHub release archives. Validate all metadata locally with:

```sh
bash scripts/validate-distribution.sh
```

This check parses the metadata, confirms all package templates exist, and rejects secret markers. It does not contact a registry or publish anything.

Release notes should start from [`release-notes-template.md`](release-notes-template.md). Versioning and the opt-in, non-blocking update notification design are documented in [`versioning-and-updates.md`](versioning-and-updates.md).

## Signing Handoff

Signing is intentionally not performed in this repository. The release owner should:

1. Download the immutable archive, `.sha256`, and `release-evidence.json` artifacts from the successful workflow run.
2. Re-run `scripts/release-evidence.sh` or `scripts/verify-release.sh` on a trusted signing host and compare the resulting SHA-256 with the evidence record.
3. Sign the exact archive and checksum manifest with the approved publisher identity, preferably using a hardware-backed or isolated key.
4. Publish the detached signature, public-key fingerprint, algorithm, key identifier, and verification command beside the artifacts.
5. Publish packages only from the verified signed artifacts, then record signed artifact digests, package URLs, and the workflow run in release notes.

The handoff can be checked without exposing a private key. The manifest is a simple `key=value` file containing `archive`, `checksum`, `signature`, `public_key_fingerprint`, `algorithm`, `verification_command`, and numeric `workflow_run` fields:

```sh
bash scripts/validate-release-handoff.sh \
  --manifest handoff.txt \
  --artifact-dir signed-artifacts
```

The validator requires the archive, checksum, and detached signature, re-runs archive/checksum validation, rejects private-key or secret markers, and records no credentials.

Private keys and signing tokens must not enter repository automation. Until this handoff is complete, releases are checksum-verified and unsigned.
