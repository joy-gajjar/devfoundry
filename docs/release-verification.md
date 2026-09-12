# Release Verification Checklist

Use this checklist for every release candidate and again on the trusted host
before publication. A release is not verified merely because its archive
downloaded successfully.

## Before Handoff

- Confirm the Cargo version, `v<version>` tag, archive names, release notes,
  supported targets, and package metadata all agree.
- Run the secretless and distribution gates:

  ```sh
  bash scripts/validate-secretless.sh
  bash scripts/validate-distribution.sh
  ```

- Run the workspace format, check, clippy, and test gates documented in
  [`release.md`](release.md).
- Build each supported target and create an archive plus `.sha256` sidecar.
- Run `scripts/verify-release.sh` for every target, including
  `--launch-doctor`; confirm the credential-free JSON doctor output.
- Generate the unsigned publication plan with
  `scripts/prepare-publication.sh`. Do not publish from this step.
- Confirm release notes contain migration impact, supported targets, validation
  results, package publication status, and signing status without secrets.

## Artifact Checks

For each archive, verify the sidecar before extracting it:

```sh
bash scripts/verify-release.sh \
  --archive dist/devfoundry-<version>-<target>.tar.gz \
  --checksum dist/devfoundry-<version>-<target>.tar.gz.sha256 \
  --target <target> \
  --version <version> \
  --launch-doctor
```

The verifier checks the SHA-256 digest, target/version directory, exact binary
and `LICENSE` entries, executable mode, and optional credential-free startup.
SHA-256 detects corruption or transfer errors; it does **not** authenticate the
publisher.

## Trusted Signing And Publication

1. Download artifacts only from the successful, immutable workflow run.
2. Re-run archive and checksum verification on the trusted signing host.
3. Sign the exact archive and checksum manifest with the approved publisher
   identity.
4. Publish the detached signature, public-key fingerprint, algorithm, key
   identifier, and verification command alongside the artifacts.
5. Validate the handoff manifest with
   `scripts/validate-release-handoff.sh`.
6. Publish package-manager entries only after the signed handoff validates and
   maintainer approval is recorded.
7. Record artifact digests, workflow run, signing status, and package channels
   in the release notes.

Until step 5 succeeds, describe the release as checksum-verified and unsigned.
Never place private keys, signing tokens, registry credentials, or Copilot
tokens in the workflow artifacts or handoff manifest.
