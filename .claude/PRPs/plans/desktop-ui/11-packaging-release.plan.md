# Tauri Packaging and Release Plan

**Goal:** Package the Tauri desktop application with macOS Apple Silicon first, then Windows and Linux, without weakening release guarantees.
**Architecture:** Use Tauri bundler output plus the existing Rust archive/checksum/evidence workflow. Signing and notarization remain trusted-host handoffs; secrets never enter CI.
**Spec:** `00-overview.plan.md`, `docs/release.md`, `docs/release-verification.md`, `scripts/verify-release.sh`.

## Global Constraints

- Packaging priority is macOS Apple Silicon, then Windows, then Linux.
- Signing secrets remain outside repository automation; evidence precedes claims.

## Tasks

1. Add Tauri bundler configuration and version synchronization with workspace version.
2. Add macOS Apple Silicon build/package and launch/doctor smoke.
3. Add Windows and Linux matrix jobs after macOS passes.
4. Add checksum/archive/evidence validation and install/rollback integration.
5. Document signing/notarization handoff and unsigned prerelease behavior.
6. Add desktop smoke tests for launch, backend cleanup, update/error states, and no secret wiring.

## Acceptance

- macOS Apple Silicon artifact launches and reaches backend health.
- Child backend is cleaned up after app close.
- Package/checksum/evidence validation passes.
- Signing keys/tokens do not enter repository automation.
- Windows/Linux status is reported only after runner evidence.
