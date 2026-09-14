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

No dependency was added. `cargo-audit` and `cargo-deny` are now installed in
the local Cargo toolchain and `deny.toml` records the reviewed license/source
policy. `cargo deny check` passes advisories, bans, licenses and sources, but
`cargo audit` reports the existing transitive `rsa 0.9.10` Marvin timing
advisory (RUSTSEC-2023-0071) with no fixed upstream version. Per the approval
constraint, the native adapter remains explicitly fail-closed rather than
adding a credential dependency while the locked graph has an unresolved
security advisory.

Process injection remains deferred. No new process abstraction or ambient
environment lookup was introduced. Existing process policy continues to clear
ambient provider credentials; worktrees remain explicitly non-sandboxing.

## Implementation

- Preserved `crates/devfoundry/src/credentials.rs` unchanged after the
  dependency review failed its required audit gate.
- Preserved `Cargo.toml`, `Cargo.lock`, and
  `crates/devfoundry/Cargo.toml` without a new dependency.
- Added this W23 task record documenting the decision and evidence.

## Verification

### RED evidence

- Candidate native smoke test was staged locally but not retained because the
  dependency audit gate did not pass.
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
- `/Users/joy/.cargo/bin/cargo audit`: **BLOCKED** by RUSTSEC-2023-0071 affecting transitive `rsa 0.9.10`; no fixed upgrade is available.
- Result: native credential dependency gate remains **BLOCKED by the unresolved RSA advisory**, not by missing audit tools.

### Rust/release gates

The combined workspace was later validated after W20/W22 integration:

- Workspace check, workspace Clippy with `-D warnings`, workspace tests,
  formatting, secretless validation and `git diff --check` all passed.

W23 itself remains fail-closed and blocked from native enablement because the
dependency audit tooling and disposable native credential fixture are absent.
The release owner must rerun the following after those prerequisites are
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

- **Blocked:** resolve or formally risk-accept RUSTSEC-2023-0071 affecting
  transitive `rsa 0.9.10`, review the exact locked graph, and repeat native CI
  review before reconsidering `keyring`.
- **Blocked:** native macOS smoke requires a release-owner-approved disposable
  credential fixture. This task does not create, overwrite, or delete keychain
  entries.
- **Deferred:** Windows Credential Manager and Linux Secret Service require
  native runner evidence and explicit runtime availability checks.
- **Deferred:** process injection requires an existing permission-owned child
  process boundary with allowlisting, cleanup, redaction, and cancellation
  tests. No ambient `GITHUB_COPILOT_TOKEN` inheritance is permitted.
- **Residual threat:** any trusted child process that receives a secret can
  exfiltrate it; no OS sandbox is claimed.

## Status

Inspected and documented locally. The explicit fail-closed adapter remains in
place. No dependency, runtime code, process injection, secret mutation, or
commit was made. W23 is **blocked pending dependency audit and native fixture /
platform evidence**.
