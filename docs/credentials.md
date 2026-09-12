# Credential Handling

DevFoundry treats credentials as operator-owned, short-lived input. It does
not acquire, persist, export, or display provider tokens. Release automation is
secretless and must not receive provider, registry, or signing credentials.

Playwright MCP testing is separate from provider credentials. Never pass
`GITHUB_COPILOT_TOKEN` to an MCP process, and never expose browser cookies,
storage state, or login profiles to DevFoundry diagnostics.

## Local Use

- Prefer an environment variable supplied by an approved credential tool or
  interactive shell read.
- Never put a token in `devfoundry.json`, command arguments, shell history,
  task notes, fixtures, diagnostics, logs, crash reports, or issue comments.
- Do not print the environment, authorization headers, provider request bodies,
  or response bodies while troubleshooting.
- Use the configured `provider.token_env` name. The default is
`GITHUB_COPILOT_TOKEN`.

For GitHub Actions, add this under repository **Settings -> Secrets and variables -> Actions -> New repository secret** with the name `GITHUB_COPILOT_TOKEN`. Do not add it to workflow YAML, `.env` files, or committed configuration. Use it only in an explicitly opt-in smoke workflow, never in normal CI.
- Unset the variable as soon as the operation finishes and revoke or rotate a
  token that may have been exposed.

The application may report whether a configured variable is missing or present,
but never reports its value. Provider errors must be reduced to safe
classifications before they reach diagnostics or release artifacts.

## Copilot Token Handoff

OAuth or device-token acquisition happens outside DevFoundry through the
approved GitHub tooling for the operator's environment. The handoff boundary
is the process environment; DevFoundry reads the token only when a network
request explicitly requires it.

First run the credential-free preflight:

```sh
cargo run -p devfoundry -- --json doctor
cargo run -p devfoundry -- copilot-smoke --mock
```

For a controlled live smoke test, read the token without placing it in command
history, then remove it after the test:

```sh
read -r -s GITHUB_COPILOT_TOKEN
export GITHUB_COPILOT_TOKEN
printf '\n'
cargo run -p devfoundry -- copilot-smoke --allow-network
unset GITHUB_COPILOT_TOKEN
```

Replace the variable name in all three commands if `provider.token_env` is
customized. `--allow-network` is mandatory and the command validates its
configuration before looking up the token. Do not run this flow in CI or on a
shared shell, and do not paste its output into a task log. See
[`copilot-network-smoke.md`](copilot-network-smoke.md) for limits and safe
failure handling.

## Release And Package Credentials

The release workflow builds, packages, checksums, verifies, and uploads
artifacts without credentials. Signing and package publication happen only on a
trusted host after an explicit handoff. Private keys, signing tokens, registry
tokens, and package-manager login state must stay off repository automation.

Before handing over artifacts, run:

```sh
bash scripts/validate-secretless.sh
bash scripts/validate-distribution.sh
```

After signing, validate the detached signature and artifact set with
`scripts/validate-release-handoff.sh`; never send a private key with the
manifest or artifact bundle.
