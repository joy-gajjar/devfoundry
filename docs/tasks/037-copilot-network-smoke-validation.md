# Task: GitHub Copilot Network Smoke Validation

## Goal

Make controlled GitHub Copilot real-network validation useful without making credentials part of
tests, CI, logs, or repository files.

## Scope

- Included: opt-in `copilot-smoke --allow-network`, token/base-URL/model validation, bounded
  streaming output, secret-free missing/empty/authentication/network diagnostics, interactive
  credential handoff documentation, and manual validation evidence.
- Explicitly excluded: automatic token acquisition, token persistence, CI network calls, and live
  credentials in fixtures or task records.

## Design

The smoke command exits before reading the token unless `--allow-network` is supplied. It validates
the provider kind, HTTP(S) base URL, model, and token environment variable name, rejects missing or
blank environment values locally, and bounds the stream to 256 events and 4,096 text characters.
It classifies HTTP 401/403 as authentication failures, distinguishes safe network/rate-limit/
provider failures, and never includes token values, authorization headers, response text, or
provider response bodies in output. The normal workspace tests remain offline and deterministic;
CI does not invoke the command or require credentials.

## Verification

- Deterministic provider tests cover authentication classification.
- DevFoundry unit tests cover smoke input validation and error redaction.
- Manual validation is documented in `docs/copilot-network-smoke.md` and must use an operator-
  supplied token outside shell history and CI logs.
- Run `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, and
  `cargo test --workspace` for offline validation.

## Follow-Up

Record live validation only as aggregate status, model, endpoint classification, and timestamp;
never record the token, prompt, response text, headers, or raw provider error body.
