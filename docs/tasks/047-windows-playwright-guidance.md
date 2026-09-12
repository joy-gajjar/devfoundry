# Task: Windows Host And Playwright MCP Guidance

## Goal

Document where the Windows validation runs and how Playwright MCP is tested safely without mixing credentials or enabling it in production.

## Windows Host

Windows validation runs on GitHub Actions `windows-latest` through `.github/workflows/ci.yml` and `.github/workflows/release.yml`. The workflow builds, runs Clippy/tests, checks the Job Object cleanup test, and validates packaged `devfoundry.exe` doctor output without provider tokens.

## Copilot Token

For an optional GitHub Actions smoke workflow, add `GITHUB_COPILOT_TOKEN` at repository Settings -> Secrets and variables -> Actions. It must be referenced only through `${{ secrets.GITHUB_COPILOT_TOKEN }}` in an explicitly opt-in workflow. It is not required by normal CI and must never be printed.

## Playwright MCP

Playwright MCP is local-only and opt-in. Follow `docs/playwright-mcp-testing.md`. Do not add it to standard CI, release workflows, or production startup. It receives no Copilot token and no browser credentials.

## Verification

- Full macOS workspace gates pass.
- Windows workflow YAML includes build/test/Job Object/package doctor checks.
- Secretless and distribution scripts pass.
