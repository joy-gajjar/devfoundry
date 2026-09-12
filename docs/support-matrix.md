# Supported Platforms

The release workflow builds the targets listed in `release/platforms.json`. A platform is supported when its release archive builds, its checksum verifies, and the workspace tests pass on the associated GitHub Actions runner.

| Target | Runner | Artifact | Status |
| --- | --- | --- | --- |
| `aarch64-apple-darwin` | `macos-14` | `devfoundry-<version>-aarch64-apple-darwin.tar.gz` | Supported |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `devfoundry-<version>-x86_64-unknown-linux-gnu.tar.gz` | Supported |
| `x86_64-pc-windows-msvc` | `windows-latest` | `devfoundry-<version>-x86_64-pc-windows-msvc.tar.gz` | Supported |

Linux ARM64 and macOS x64 are release targets in the distribution plan but are not yet CI-supported targets. Do not describe them as supported until they are added to the metadata and release matrix.

## Runtime Notes

The macOS build uses `/bin/sh` for shell tools, creates a separate Unix process group for each command, and terminates that group on timeout or cancellation. Windows CI runs the workspace build, clippy, and tests plus a focused descendant-cleanup test; Windows shell commands use a Job Object so timeout or cancellation terminates descendants as well. Release CI also extracts and launches every packaged binary with an empty provider-token environment, including the Windows `.exe`, and checks credential-free JSON doctor output. The server handles both Ctrl-C and SIGTERM. TUI cleanup restores terminal state on normal return and panic unwinding.

The focused TUI contract runs on all three release runners and validates resize,
multiline paste, input dispatch, exit handling, and cleanup with an in-memory
backend. Native terminal behavior is release-owner evidence; see
[`tui-validation.md`](tui-validation.md) for the macOS, Linux, and Windows
smoke checklist and its PTY/ConPTY limits.

Project configuration is `devfoundry.json` and interactive state is `.devfoundry.db` under the discovered project root. SQLite paths are opened through filesystem-aware options, so project paths containing spaces are supported.

## Artifact Verification

Each archive contains an executable target binary and `LICENSE`. Verify the adjacent SHA-256 file and archive contents before installation:

```sh
sha256sum -c devfoundry-<version>-<target>.tar.gz.sha256
```

On macOS, use `shasum -a 256 -c <file>.sha256`.

The repository helper checks the expected target directory, binary, and license:

```sh
bash scripts/verify-release.sh --archive devfoundry-<version>-<target>.tar.gz --checksum devfoundry-<version>-<target>.tar.gz.sha256 --target <target> --version <version>
```

On CI, add `--launch-doctor` to extract and execute the packaged binary while checking that diagnostic output reports no configured provider token. This smoke is secretless and does not contact a provider.

The archives are unsigned until the signing handoff in [`release.md`](release.md) is completed. Checksum verification detects transfer or storage corruption; it does not establish publisher identity.
