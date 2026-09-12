# Task: Model Catalog And Session UX

## Goal

Make new sessions understandable and let users choose from the supported GitHub Copilot model catalog.

## Implementation

- Added `GET /api/v1/models`.
- Added model option wire contract.
- Added a visible model picker before creating a new session.
- New sessions receive timestamped titles instead of `New session`.
- The TUI header continues to display the selected provider/model.

## Current Catalog

The catalog is a curated DevFoundry list of GitHub Copilot model IDs and context windows. It is not a claim that every account has access to every model; the provider may reject unavailable models.

## Verification

- Full workspace Cargo gates pass.
- Model/session picker should be manually tested from `cargo run -p devfoundry`.
