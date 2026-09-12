# Task: Playwright MCP Testing Profile

## Goal

Document a safe, opt-in Playwright MCP testing workflow without enabling it in production or release automation.

## Scope

- Included: local setup guidance, absolute executable/fixed argv rules, permission/security boundaries, test goals, and limitations.
- Excluded: committed personal paths, browser credentials, remote MCP, and default runtime registration.

## Verification

- Documentation is secret-free.
- `git diff --check` passes.
- Existing native MCP fixture suite remains the deterministic CI test path.
- Windows CI remains a separate host validation path; Playwright MCP is not enabled in release CI.

## Follow-Up

Add a local-only harness after MCP executable discovery and process lifecycle contracts are stabilized. Do not add Playwright credentials or browser state to CI.
