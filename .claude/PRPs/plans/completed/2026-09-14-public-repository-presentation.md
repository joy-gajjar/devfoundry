# Public Repository Presentation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the existing DevFoundry implementation into a coherent public GitHub repository with a root README, architecture presentation, contributor/security guidance, CI visibility, demo material, and a `main` branch suitable for review.

**Architecture:** Preserve the existing Rust workspace, optional browser workspace, and current documentation as the sources of truth. Add a concise public navigation layer rather than moving or rewriting the existing task/planning records. Keep GitHub branch/default-branch operations outside repository code so hosted permission failures remain explicit.

**Tech Stack:** Rust 1.88 workspace, Cargo, SQLite/SQLx, Ratatui TUI, Axum API, Vite/TypeScript browser workspace, Playwright, GitHub Actions, Mermaid diagrams.

**Spec:** Approved public-repository design in the preceding task discussion; existing architecture source: `docs/planning/02-system-architecture.md`.

## Global Constraints

- Do not claim the entire product is production-ready; link `docs/PENDING_WORK.md` and state the remaining gates.
- Do not include secrets, tokens, credentials, private local paths, or generated dependency directories.
- Keep the existing Copilot-only provider boundary and fail-closed credential behavior.
- Use ASCII-safe Markdown and repository-relative links.
- Validate every changed code/config/documentation surface immediately after editing.
- Do not change existing application behavior unless required for the public presentation or CI.
- GitHub operations may create/push `main` and a PR, but must not merge or change default-branch settings without successful authenticated permission checks.

---

### Task 1: Create Public README

**Files:**
- Create: `README.md`
- Read: `docs/planning/02-system-architecture.md`, `docs/PENDING_WORK.md`, `docs/support-matrix.md`, `docs/release.md`, `docs/credentials.md`, `apps/workspace/package.json`

**Interfaces:**
- Consumes existing project commands and documentation paths.
- Produces the repository landing page with verified commands, architecture summary, status, demo link, and navigation links.

- [ ] **Step 1: Write the README sections**
  Include: project purpose, status disclaimer, feature highlights, architecture Mermaid diagram, repository map, quick start, API/TUI/browser commands, security model, validation commands, demo section, support matrix, roadmap, contributing/security/release links.
- [ ] **Step 2: Validate all README links and commands**
  Run a script or shell checks that every referenced local path exists and that command snippets match `Cargo.toml`, `scripts/`, and `apps/workspace/package.json`.
- [ ] **Step 3: Commit the README task**
  ```bash
  git add README.md
  git commit -m "docs: add public project README"
  ```

### Task 2: Add Architecture and Contributor Documentation

**Files:**
- Create: `docs/architecture.md`
- Create: `CONTRIBUTING.md`
- Create: `SECURITY.md`
- Create: `docs/demo.md`
- Read: `docs/security.md`, `docs/planning/13-testing-and-quality.md`, `docs/release.md`, `docs/tui-validation.md`

**Interfaces:**
- `docs/architecture.md` expands the README diagram into component boundaries, request flow, storage/recovery, security boundaries, and known deferred areas.
- `CONTRIBUTING.md` defines setup, validation, branch/commit expectations, and test commands.
- `SECURITY.md` points to the existing security model and gives a safe vulnerability-reporting process without inventing a private contact address.
- `docs/demo.md` provides a reproducible local demo script and links to any committed demo asset.

- [ ] **Step 1: Write architecture documentation**
  Describe `crates/schema`, `protocol`, `core`, `storage`, `tools`, `llm`, `server`, `tui`, `devfoundry`, and `apps/workspace`; include Mermaid data-flow and trust-boundary diagrams.
- [ ] **Step 2: Write contributor/security/demo documentation**
  Use only verified commands and state external prerequisites such as macOS Keychain fixtures, live Copilot credentials, Linux/Windows runners, and release archives.
- [ ] **Step 3: Validate documentation paths and command references**
  Run local-link checks and `git diff --check`.
- [ ] **Step 4: Commit the documentation task**
  ```bash
  git add docs/architecture.md docs/demo.md CONTRIBUTING.md SECURITY.md
  git commit -m "docs: add architecture and contributor guide"
  ```

### Task 3: Improve CI Public Visibility

**Files:**
- Modify: `.github/workflows/ci.yml`
- Read: `.github/workflows/release.yml`, `Cargo.toml`, `apps/workspace/package.json`

**Interfaces:**
- CI must retain current Rust/macOS/Windows checks and add browser typecheck, unit tests, build, Playwright, and secretless validation.
- CI must use repository-supported toolchain floors and avoid requiring live credentials.

- [ ] **Step 1: Add browser and secretless CI jobs**
  Install Node dependencies with `npm ci`, run browser typecheck/test/build/Playwright, and run `bash scripts/validate-secretless.sh` in a separate job.
- [ ] **Step 2: Validate workflow syntax and command availability**
  Parse YAML with an available YAML tool or Ruby/Python standard-library fallback, then verify each referenced command exists locally.
- [ ] **Step 3: Commit CI changes**
  ```bash
  git add .github/workflows/ci.yml
  git commit -m "ci: publish browser and secretless validation"
  ```

### Task 4: Produce Demo Material

**Files:**
- Create: `docs/assets/` only for intentionally committed small demo assets
- Modify: `docs/demo.md`, `README.md`

**Interfaces:**
- Prefer a reproducible demo script and screenshots over a binary video when recording tooling or repository size makes video unsuitable.
- If a video is generated, it must be a local browser demo of the existing workspace, contain no secrets, and be linked from the README.

- [ ] **Step 1: Run the browser build and launch the static demo**
  Use the existing Vite preview/dev command and confirm desktop/mobile routes load.
- [ ] **Step 2: Capture a demo artifact**
  Use Playwright screenshot/video support if available; otherwise add a documented demo walkthrough and checked-in screenshots only when their size and content are appropriate.
- [ ] **Step 3: Validate the artifact**
  Confirm it contains no credentials, build caches, or machine-specific paths, and update README/demo links.
- [ ] **Step 4: Commit demo material**
  ```bash
  git add README.md docs/demo.md docs/assets
  git commit -m "docs: add local workspace demo"
  ```

### Task 5: Validate the Public Repository Surface

**Files:**
- Modify: documentation only if validation finds broken claims or links.

- [ ] **Step 1: Run Rust gates**
  ```bash
  cargo fmt --all -- --check
  cargo check --workspace --all-targets
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  ```
- [ ] **Step 2: Run browser gates**
  ```bash
  cd apps/workspace
  npm ci
  npm run typecheck
  npm test
  npm run build
  npx playwright test
  ```
- [ ] **Step 3: Run security/release checks**
  ```bash
  cargo audit
  cargo deny check
  bash scripts/validate-secretless.sh
  git diff --check
  ```
  Run `scripts/verify-release.sh` with real archive/checksum arguments only if artifacts exist; otherwise record the usage/blocker honestly.
- [ ] **Step 4: Review status and generated-file exclusions**
  Ensure `node_modules`, browser output, Cargo `target`, databases, test artifacts, and OS files are not staged.

### Task 6: Publish Branch and Prepare Main

**Files:**
- No source files; GitHub state only.

- [ ] **Step 1: Review commits and status**
  ```bash
  git status --short
  git log --oneline --decorate -10
  git diff main...HEAD --stat 2>/dev/null || true
  ```
- [ ] **Step 2: Push the feature branch**
  ```bash
  git push -u origin feat/granular-workspace
  ```
- [ ] **Step 3: Create and push `main` if absent**
  ```bash
  git branch main HEAD
  git push -u origin main
  ```
- [ ] **Step 4: Create a PR if GitHub permits**
  ```bash
  gh pr create --base main --head feat/granular-workspace --title "docs: publish DevFoundry repository surface" --body-file /tmp/devfoundry-pr.md
  ```
  Do not merge automatically. If repository administration permission is available, set `main` as default; otherwise report the exact permission blocker.

## Acceptance Criteria

- Root README exists and links to verified architecture, security, contribution, release, roadmap, and demo material.
- Architecture diagrams accurately reflect the current Rust/browser boundaries and explicitly identify deferred behavior.
- Contributor and security guidance contain executable local validation commands and no invented secrets/contact data.
- CI covers Rust, browser, secretless, macOS, and Windows checks without live credentials.
- Demo instructions are reproducible; a video is linked only if a real secret-free video artifact was produced.
- All local validation gates pass, with release artifact verification reported as pass or externally blocked.
- `feat/granular-workspace` is pushed, `main` exists remotely, and a PR is created when GitHub permissions allow.
- No claim is made that the complete pending product backlog is finished.

## Plan Self-Review

- Spec coverage: README, architecture, contributors/security, CI, demo, validation, and GitHub publication each have an explicit task.
- Placeholder scan: no `TBD`, `TODO`, or unspecified implementation steps are used.
- Scope: repository presentation and publication only; application behavior remains unchanged.
- Known external gates: GitHub administration permissions, live credentials, release artifacts, and cross-platform runtime evidence remain reported rather than fabricated.
