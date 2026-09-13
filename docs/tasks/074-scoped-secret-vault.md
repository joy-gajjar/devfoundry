# Task: W11 Scoped Secret Vault

## Goal

Provide value-free secret reference metadata and fail-closed scoped bindings
for approved project/process/account use. Secret values must remain outside
core metadata, serialization, logs, exports, and browser-shaped data.

## Scope

- Included: `crates/core/src/secret_bindings.rs` metadata and scope validation.
- Included: `crates/devfoundry/src/credentials.rs` typed store boundary, fake
  store, and unavailable macOS Keychain adapter behavior.
- Included: focused regression tests in both owned modules.
- Included: this task record and exact verification evidence.
- Excluded: provider/runtime wiring, persistent vault schema, token rotation,
  arbitrary secret-bearing commands, remote secret managers, and plaintext
  config/database fallback.

## Design

`SecretReference`, `SecretBinding`, `BindingScope`, and
`SecretBindingRequest` contain only identifiers, labels, scope metadata, and
state. A request validates exact reference identity, enabled/revoked state,
project identity, and the explicitly supplied process/account scope. A
project binding cannot satisfy a process/account request, and a process or
account binding cannot satisfy an unscoped project request. Process and
account scopes also carry the project identity, preventing same-named
processes/accounts in another project from matching.

`SecretStore` is an async typed abstraction. `SecretHandle` is deliberately
not serializable and has a redacted `Debug` implementation. The fake store is
deterministic and resolves only an exact approved binding. The macOS adapter
currently returns `Unavailable`: no vetted Keychain dependency was present in
the workspace, so the implementation does not shell out to `security`, read
environment variables, or fall back to plaintext files/configuration.

The store does not approve access. Central permission-broker integration and
provider/process handoff remain separate follow-up work; this package only
establishes the safe metadata and storage boundary.

## Implementation

- Added `crates/core/src/secret_bindings.rs` and exported its value-free types
  from `devfoundry-core`.
- Added `crates/devfoundry/src/credentials.rs` with the `SecretStore` trait,
  redacted `SecretHandle`, typed errors, fake store, and fail-closed Keychain
  adapter placeholder.
- Added `serde` as an existing workspace dependency of `devfoundry-core`.
- Added regression coverage for locked/missing stores, disabled/revoked
  bindings, project/process/account scope mismatch including cross-project
  process/account IDs, ambient provider-token
  exclusion, secret debug redaction, metadata serialization, exact lookup,
  and unavailable native storage.

## Verification

- RED: `cargo test -p devfoundry-core secret_bindings --lib` could not compile
  because the new W11 types were not implemented. The first bare `cargo`
  invocation was also blocked because Cargo is not on `PATH`; authoritative
  commands use `$HOME/.cargo/bin/cargo`.
- RED: `$HOME/.cargo/bin/cargo test -p devfoundry-core secret_bindings --lib`
  failed on unresolved `SecretReference`, `SecretBinding`, `BindingScope`,
  `SecretBindingRequest`, and `SecretBindingError`, as intended.
- RED: `$HOME/.cargo/bin/cargo test -p devfoundry credentials --bin devfoundry`
  failed on the same missing implementation surface, as intended.
- GREEN: `$HOME/.cargo/bin/cargo test -p devfoundry-core secret_bindings --lib`
  — 5 passed, 0 failed.
- GREEN: `$HOME/.cargo/bin/cargo test -p devfoundry credentials --bin devfoundry`
  — 5 passed, 0 failed.
- Format: `$HOME/.cargo/bin/cargo fmt --all` completed successfully.

- Workspace check: `$HOME/.cargo/bin/cargo check --workspace --all-targets`
  passed with warnings before the intentional dead-code annotations were
  added; the warnings concerned the not-yet-wired store boundary.
- Workspace Clippy: `$HOME/.cargo/bin/cargo clippy --workspace --all-targets
  -- -D warnings` was rerun after the annotations and passed.
- Workspace tests: `$HOME/.cargo/bin/cargo test --workspace` — all workspace
  tests passed (including 12 `devfoundry`, 7 `devfoundry-core`, and the
  existing workspace integration suites).
- Format check: `$HOME/.cargo/bin/cargo fmt --all -- --check` — passed.
- Diff check: `git diff --check` — passed.
- No coverage command was available/configured for this Rust workspace; the
  targeted and workspace test results above are the recorded evidence.

## Risks And Follow-Up

- No native Keychain smoke test can pass until a vetted compatible Keychain
  dependency is approved and available. Access therefore fails closed.
- Provider/runtime integration is intentionally not included; no claim is made
  that a live provider currently consumes this store.
- An explicitly trusted secret-bearing process could still transform or
  exfiltrate a value it receives. This package does not provide OS sandboxing.
- No persistent vault migration or automatic rotation is implemented.
- No commit was made, per execution request.

## Status

Implementation is locally GREEN for the focused W11 tests and the workspace
check, Clippy, formatting, workspace tests, and diff checks all pass on this
macOS host. Native Keychain smoke remains intentionally blocked because no
vetted compatible dependency is available. No commit was made.
