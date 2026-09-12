# Task: Version, Release, And Security Validation

## Goal

Make version/update behavior, release handoff, and security expectations
auditable without adding signing credentials or external provider access.

## Scope

- Included: version and update documentation, security reporting guidance,
  release metadata validation, shell syntax checks, and secretless CI checks.
- Excluded: signing keys, registry credentials, live package publication, and
  live provider smoke tests.

## Implementation

- Added `docs/security.md` with reporting, secret handling, and release
  security policy.
- Added `scripts/validate-secretless.sh` as the local validation entry point.
- Validated every release JSON file and every shell script before release
  handoff.
- Kept update checks opt-in, non-blocking, anonymous, and download-free.
- Kept provider credentials out of release CI and validation commands.

## Verification

- `bash -n scripts/*.sh`
- `jq empty release/*.json`
- `bash scripts/validate-secretless.sh`
- `bash scripts/validate-distribution.sh`
- `bash scripts/test-release.sh`

## Risks And Follow-Up

- Artifact signing and package publication remain external trusted-host work.
- Live provider validation remains manual and requires user-provided
  credentials outside repository automation.
- A repository-specific private security contact should be configured before
  public distribution.
