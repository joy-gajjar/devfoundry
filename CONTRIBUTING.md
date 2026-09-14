# Contributing to DevFoundry

DevFoundry is a Rust workspace with an optional browser workspace. Small,
focused changes are preferred over broad refactors. Preserve the existing
fail-closed security boundaries and document any intentionally deferred
platform or provider behavior.

## Setup

```bash
git clone https://github.com/joy-gajjar/devfoundry.git
cd devfoundry
rustup show
```

The workspace requires Rust 1.88 or newer. Browser work additionally requires
Node.js and npm.

## Development Workflow

1. Create a focused feature branch from `main`.
2. Read the relevant architecture and task records before changing contracts.
3. Add or update deterministic tests before changing behavior.
4. Run the affected package tests during development.
5. Run the complete validation matrix before opening a pull request.
6. Keep commits focused and use Conventional Commit-style subjects.

## Validation Matrix

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash scripts/validate-secretless.sh
git diff --check
```

For browser changes:

```bash
cd apps/workspace
npm ci
npm run typecheck
npm test
npm run build
npx playwright test
```

For dependency or release changes:

```bash
cargo audit
cargo deny check
```

## Security Rules

- Never commit tokens, passwords, Keychain values, `.env` files, or private
  provider output.
- Never use an environment/config/CLI fallback to bypass a secret-store
  boundary.
- Keep tool permissions capability-specific and validate inputs before spawn or
  file mutation.
- Do not claim Linux, Windows, live Copilot, or signed-release support without
  evidence from the appropriate runner or artifact.

## Pull Requests

Describe the behavior change, security impact, tests run, platform limitations,
and any documentation updates. Do not merge a change that weakens a fail-closed
path merely to make a smoke test pass.
