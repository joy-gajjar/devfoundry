# Task: W23 Native Credential Backend

## Goal

Provide a reviewed native credential backend and approved scoped process
injection without weakening the existing fail-closed secret boundary.

## Scope

- Included: dependency/MSRV/platform review, the existing credential adapter,
  native-backend decision, focused RED/GREEN evidence, and this task record.
- Explicitly excluded: browser, scheduler, preview, resources, worktree, LLM
  provider, database migrations, secret deletion, and process injection unless
  an existing safe owned process boundary is available.

## Design

The existing `SecretStore` remains the authority boundary. `SecretHandle` is
not serializable and has redacted `Debug`; `SecretBindingRequest::validate`
continues to enforce exact reference, project, process/account, enabled, and
revocation checks before a backend may resolve a value.

The candidate reviewed was `keyring` 3.6.3. Its declared MSRV is Rust 1.75,
which is compatible with this workspace MSRV of Rust 1.85. It exposes explicit
features for macOS Keychain (`apple-native`), Windows Credential Manager
(`windows-native`), and Linux stores. The review also found that the Linux
feature choice has materially different runtime dependencies and evidence
requirements, so Linux support cannot be inferred from compilation.

`keyring` 3.6.3 is enabled with only its `apple-native` feature. The local
`.cargo/audit.toml` narrowly ignores RUSTSEC-2023-0071 because `rsa 0.9.10` is
retained by SQLx's optional PostgreSQL package metadata, while the supported
SQLite-only macOS target has no active RSA dependency path. The exception must
be revisited when SQLx publishes a graph that removes `rsa` from the lockfile.
`cargo deny check` passes advisories, bans, licenses and sources.

Process injection remains deferred. No new process abstraction or ambient
environment lookup was introduced. Existing process policy continues to clear
ambient provider credentials; worktrees remain explicitly non-sandboxing.

## Implementation

- Added `keyring` 3.6.3 with `apple-native` only; non-macOS builds remain
  unavailable rather than falling back to environment/config/CLI secrets.
- Added the narrowly scoped audit exception in `.cargo/audit.toml`.
- Added a macOS Keychain fixture smoke test that asserts `Debug` redacts the
  secret value.

## Verification

### RED evidence

- Candidate native smoke test now passes with the approved disposable fixture.
- The attempted targeted command was:

  ```text
  /Users/joy/.cargo/bin/cargo test -p devfoundry native_store_missing_reference_does_not_use_a_plaintext_fallback
  ```

- It did not reach the new credential test. Compilation stopped on an
  unrelated pre-existing untracked W22 file,
  `crates/tools/src/archive.rs:51`, with an unterminated character literal.
  This is not treated as valid W23 RED evidence.

### Dependency review

- `/Users/joy/.cargo/bin/cargo info keyring`: inspected `keyring` 3.6.3,
  declared MSRV 1.75, MIT OR Apache-2.0 license, and explicit native feature
  model.
- `/Users/joy/.cargo/bin/cargo search keyring --limit 5`: candidate discovery
  completed.
- `/Users/joy/.cargo/bin/cargo audit --version`: unavailable; no
  `cargo-audit` subcommand installed.
- `/Users/joy/.cargo/bin/cargo deny --version`: unavailable; no `cargo-deny`
  subcommand installed.
- `/Users/joy/.cargo/bin/cargo install cargo-audit --locked`: completed; cargo-audit 0.22.2 installed.
- `/Users/joy/.cargo/bin/cargo install cargo-deny --locked`: completed; cargo-deny 0.20.2 installed.
- `/Users/joy/.cargo/bin/cargo deny check`: advisories, bans, licenses and sources **PASS** with checked-in `deny.toml`; duplicate-version warnings remain.
- `/Users/joy/.cargo/bin/cargo audit`: **PASS** under the exact
  `.cargo/audit.toml` exception; this is not a claim that RSA is fixed.
- `security find-generic-password -a devfoundry-test -s devfoundry-test-credential -w`: disposable macOS Keychain fixture **PRESENT**. The value was only used for existence verification and was not printed, persisted, or exposed.
- Lockfile regeneration and `cargo tree --target x86_64-apple-darwin -i rsa` confirmed that `rsa` remains recorded through SQLx's optional `sqlx-postgres` package metadata but is not in the active macOS workspace dependency tree. This removes the immediate runtime exposure from the SQLite-only build, but does not make an unqualified `cargo audit` pass.
- `/Users/joy/.cargo/bin/cargo tree --target x86_64-apple-darwin -i rsa`:
  **PASS/no active path**.
- Result: the dependency gate is complete by documented risk acceptance.

### Rust/release gates

The combined workspace was later validated after W20/W22 integration:

- Workspace check, workspace Clippy with `-D warnings`, workspace tests,
  formatting, secretless validation and `git diff --check` all passed.

The dependency gate and macOS fixture prerequisite are complete. The release
owner must rerun the following after binding-aware caller integration is
approved:

```bash
/Users/joy/.cargo/bin/cargo fmt --all -- --check
/Users/joy/.cargo/bin/cargo check --workspace --all-targets
/Users/joy/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
/Users/joy/.cargo/bin/cargo test --workspace
bash scripts/validate-secretless.sh
bash scripts/verify-release.sh
git diff --check
```

Migration, backup, upgrade, rollback, clean-install, artifact checksum,
signing, doctor, structured logging, correlation-ID, and no-telemetry gates
are unchanged and not applicable to this no-schema/no-runtime-change W23
result. Their existing release status must remain documented separately; this
task does not claim platform runtime support.

## Risks And Follow-Up

- **Accepted and tracked:** RUSTSEC-2023-0071 is ignored only in
  `.cargo/audit.toml`, with the inactive SQLite-only target rationale above.
  Revisit it when SQLx removes `rsa` from the lockfile.
- **Pending:** connect the binding-aware Keychain store to a runtime command or
  server flow that has an authoritative persisted `SecretBinding`; no current
  production caller supplies that binding, so arbitrary Keychain lookup remains
  intentionally unavailable.
- **Completed prerequisite:** a release-owner-approved disposable macOS
  credential fixture was confirmed without exposing its value. This task did
  not create, overwrite, or delete keychain entries.
- **Deferred:** Windows Credential Manager and Linux Secret Service require
  native runner evidence and explicit runtime availability checks.
- **Deferred:** process injection requires an existing permission-owned child
  process boundary with allowlisting, cleanup, redaction, and cancellation
  tests. No ambient `GITHUB_COPILOT_TOKEN` inheritance is permitted.
- **Residual threat:** any trusted child process that receives a secret can
  exfiltrate it; no OS sandbox is claimed.

## Status

The dependency exception, macOS Keychain dependency, binding validation,
fixture smoke, and workspace verification are complete. W23 remains
**partially complete** until a runtime caller with authoritative persisted
binding state is wired and reviewed; the adapter cannot be used as an
arbitrary Keychain lookup.
