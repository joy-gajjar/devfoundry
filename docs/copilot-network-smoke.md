# GitHub Copilot Network Smoke Test

This is an operator-only credential handoff. The reusable credential rules and
interactive handoff are in [`credentials.md`](credentials.md).

The real-provider smoke test is GitHub Copilot-only, opt-in, and never part of the normal test
suite or CI. It requires a valid access token in `GITHUB_COPILOT_TOKEN` (or the configured
`provider.token_env`). DevFoundry does not acquire, persist, or print the token.

## Offline Mock Check

Run this first to verify the harness, bounded stream handling, and output format without network
access or credentials. It never reads `provider.token_env`:

```sh
cargo run -p devfoundry -- copilot-smoke --mock
cargo run -p devfoundry -- --json copilot-smoke --mock
```

The mock emits one `ok` text event and one finished event. It is deterministic, does not contact a
provider, and reports only aggregate status and counts.

## Exact Real-Provider Procedure

Run these commands from the repository root. The first command is offline and displays the
configured provider, base URL, model, and token environment variable presence without reading the
token. The smoke command performs the strict input validation before it reads the token:

```sh
cargo run -p devfoundry -- --json doctor
```

The smoke command uses the configured model, or `gpt-4o-mini` when no model is configured. To
override it, add `--model MODEL`. The base URL must be an `http` or `https` URL without embedded
userinfo. The request is made only when `--allow-network` is present.

Obtain the token through the approved GitHub OAuth/device-token flow for your environment. Hand
it to the command through an interactive shell read so it is not present in command history:

```sh
read -r -s GITHUB_COPILOT_TOKEN
export GITHUB_COPILOT_TOKEN
printf '\n'
```

If `provider.token_env` is customized, replace both occurrences above with that environment
variable name. Do not paste the token into a command, `devfoundry.json`, a CI variable dump, or a
task log.

```sh
cargo run -p devfoundry -- copilot-smoke --allow-network
```

`--allow-network` and `--mock` are mutually exclusive. Omitting both fails closed before any
credential lookup.

The command must report `status=ok` and exit successfully. A failed command exits non-zero. Common
safe failures are missing/blank token, authentication failure (401/403), network failure, rate
limit, invalid configuration, or a provider rejection. Correct the indicated configuration or
credential issue and rerun; never add the token to diagnostics.

Use `--json` for automation:

```sh
cargo run -p devfoundry -- --json copilot-smoke --allow-network
```

The command sends one short streaming completion to the configured Copilot-compatible endpoint,
with a 30-second timeout, a 256-event limit, and a 4,096-character text limit. `--allow-network`
is mandatory; without it the command exits before reading the token. Output contains only status
and aggregate event/character counts. It does not print the token, authorization headers, prompt or
response text, and provider error bodies are reduced to a generic classification. Missing and
empty variables are reported by name only. HTTP 401/403 responses are reported as authentication
failures without exposing the response body.

The provider accepts successful `text/event-stream` responses, successful JSON completion responses,
and valid usage/metadata frames from GitHub Copilot. A `provider returned invalid data` error
therefore indicates an actually malformed response rather than a normal non-streaming or metadata
frame.

Run `unset GITHUB_COPILOT_TOKEN` (or the configured variable) when the smoke check is complete.
Run this only in a controlled environment. The request may consume provider quota and the token
must be supplied by the operator; do not put it in shell history or CI logs. CI intentionally does
not run this command and does not require credentials.
