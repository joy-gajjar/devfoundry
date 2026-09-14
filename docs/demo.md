# DevFoundry Demo

This is the reproducible, credential-free local demo. It demonstrates the
public surfaces without making a live provider request or exposing secrets.

## Browser Workspace

```bash
cd apps/workspace
npm ci
npm run dev
```

Open the local Vite URL and walk through Chat, Tasks, Docs, Workers, Terminal,
and Preview. The automated equivalent is:

```bash
npx playwright test
```

## CLI and TUI

From the repository root:

```bash
cargo run -p devfoundry -- doctor
cargo run -p devfoundry -- copilot-smoke --mock
cargo run -p devfoundry
```

The mock smoke is deterministic and does not read a provider credential. The
interactive TUI may use a project database and local terminal capabilities.

## API

```bash
cargo run -p devfoundry -- serve --bind 127.0.0.1:4096 --database .devfoundry.db
curl http://127.0.0.1:4096/health
```

## Recording Guidance

To create a public recording, use a clean checkout, the mock smoke path, and a
browser session with no secrets visible. Do not record live provider tokens,
Keychain dialogs, local usernames, private filesystem paths, or database
contents. This repository currently provides the reproducible demo script; it
does not claim a hosted video URL. A future recording can be attached to a
GitHub release or linked from this section without changing the application.
