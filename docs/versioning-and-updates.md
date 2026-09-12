# Versioning And Updates

DevFoundry uses the Cargo package version in `Cargo.toml` as the release version. Release tags use the `v<version>` form, and archive names use `devfoundry-<version>-<target>`. Keep the package version, tag, archive directory, release notes, and package-manager manifests aligned.

Before preparing a release, run `bash scripts/validate-secretless.sh`. This
checks the semantic version, release JSON, shell syntax, workflow environment
wiring, and required documentation without contacting a provider, registry, or
signing service.

## Update Notifications

The design is defined in [`../release/update-policy.json`](../release/update-policy.json). Update checks are opt-in, disabled by default, anonymous, bounded by a short timeout, and never block startup. A check may read the public GitHub latest-release endpoint, but it sends no project path, provider token, or credentials. It never downloads or installs an update.

The CLI surface is `devfoundry update --check` on the supported macOS build. It makes one anonymous GET request for public release metadata with a two-second total timeout and a descriptive user agent. A notification is a single line on stderr containing the newer version and release URL. Network, parsing, rate-limit, and endpoint failures are silent. `DEVFOUNDRY_NO_UPDATE_CHECK` disables checks even when a caller opts in. `DEVFOUNDRY_UPDATE_ENDPOINT` is a public-metadata test/deployment override; it must never contain credentials.

The implementation must persist only a last-check timestamp and the observed latest version, outside the project directory. This explicit command currently performs no persistence. It must not run during ordinary startup. It never sends project paths, provider tokens, or other credentials, and never downloads or installs an update. Package-manager users update through their package manager; archive users follow the install and rollback procedure in [`release.md`](release.md).

## Release Notes

Start each release from [`release-notes-template.md`](release-notes-template.md). Include migration impact, distribution status, signing status, supported targets, and validation results. Do not include tokens, private keys, provider request bodies, local paths, or unredacted diagnostics.
