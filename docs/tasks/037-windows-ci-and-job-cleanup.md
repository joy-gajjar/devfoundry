# Task: Windows CI And Job Object Validation

## Goal

Make Windows build, test, and descendant-process cleanup a first-class CI gate while preserving the existing macOS process-group behavior.

## Implementation

- Added a dedicated `windows-latest` CI job covering formatting, workspace check/build, clippy, and tests.
- Added an explicit, uncaptured CI invocation of the Windows Job Object descendant-cleanup test with a delayed descendant marker assertion.
- Added a packaged-binary doctor smoke to the release matrix; it runs without credentials and checks the archive's own executable on Windows, Linux, and macOS.
- Kept Unix process-group setup and `/bin/sh` selection behind Unix-only code paths.
- Normalized the temporary marker path in the Windows shell test so the Git Bash shell can write it reliably.

## Verification

- Windows CI is the runtime validation environment for `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` and `TerminateJobObject`; the focused test waits beyond the timeout to prove the descendant cannot write its marker afterward.
- macOS CI remains unchanged and continues to run its Apple Silicon target build and workspace tests.
- Release CI extracts each packaged artifact and checks its credential-free JSON doctor output, including the Windows `.exe`.
- Local non-Windows validation should run `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, and `cargo test --workspace`.

## Follow-Up

The Windows runner remains required for runtime validation because Job Object APIs cannot be exercised on macOS or Linux. Package smoke is intentionally secretless and does not make a provider request.
