# Task: Copilot Smoke Harness Hardening

## Goal

Make opt-in Copilot smoke validation easy to exercise locally while keeping live credentials out of
tests, output, and repository files.

## Implementation

- Added `copilot-smoke --mock` for deterministic in-process validation with no token lookup.
- Made `--mock` and `--allow-network` mutually exclusive and required one explicitly.
- Preserved bounded event/text output and safe authentication, rate-limit, network, and provider
  error classification for real requests.
- Documented the offline-first check and interactive real-token invocation.

## Verification

- Unit coverage verifies mock mode succeeds without reading its configured token variable.
- Real mode remains manual-only and requires an operator-supplied token through an interactive
  environment read.
- CI and the normal workspace test suite remain offline.
