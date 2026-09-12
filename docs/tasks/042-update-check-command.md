# Task: Opt-In Update Check

## Goal

Provide a safe, explicit way for supported macOS users to check public release metadata.

## Implementation

- Added `devfoundry update --check`.
- Uses an anonymous public metadata request with a two-second timeout.
- Emits only a validated newer tag and GitHub release URL on stderr.
- Fails silently for network, parsing, rate-limit, endpoint, and unsupported-platform failures.
- Never downloads, installs, persists state, or sends project paths or Copilot credentials.
- Honors `DEVFOUNDRY_NO_UPDATE_CHECK`; `DEVFOUNDRY_UPDATE_ENDPOINT` is a non-secret endpoint seam for tests and release configuration.

## Verification

- Unit tests cover release tag and URL output validation.
- The command remains separate from ordinary startup and the Copilot-only provider path.
