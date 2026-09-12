# Task: API Authentication And Origin Policy

## Goal

Protect remote API listeners without changing the local loopback experience.

## Design

Loopback binds remain unauthenticated. Non-loopback binds fail before listening unless a non-empty bearer token is supplied through the environment variable named by `DEVFOUNDRY_API_TOKEN_ENV` (default `DEVFOUNDRY_API_TOKEN`) and `DEVFOUNDRY_ALLOWED_ORIGINS` contains one or more exact origins. Tokens are compared without early-exit value comparison, never logged, and never included in responses. CORS preflight and requests with an `Origin` header are accepted for configured origins only; non-preflight requests require authentication.

## Verification

- Server tests cover loopback compatibility, missing/invalid authentication, successful bearer authentication, CORS preflight, and token redaction.
- Runtime validation rejects malformed binds and remote listeners without the required environment-backed settings.
- Run `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
