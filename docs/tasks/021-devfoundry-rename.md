# Task: DevFoundry Product Rename

## Goal

Use DevFoundry as the product, binary, crate, database, and user-facing identity instead of reusing the upstream product name.

## Scope

- Included: binary/package names, Rust crate names, imports, default database/config filenames, CLI labels, TUI branding, and planning references.
- Excluded: explicit historical references to the upstream OpenCode repository used as architectural inspiration, and the `.opencode` directory required by the configured agent loader.

## Design

DevFoundry is an independent Rust implementation. The upstream name remains only when identifying the reference project or its public behavior. No executable command, Rust package, Rust import, default database filename, or user-facing product label uses the upstream name.

## Implementation

- Renamed the binary package and executable to `devfoundry`.
- Renamed workspace crate package identities to `devfoundry-*`.
- Renamed internal Rust imports to `devfoundry_*`.
- Renamed default config/database conventions to `devfoundry.json` and `devfoundry.db`.
- Renamed TUI/runtime branding to DevFoundry.

## Verification

Required:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p devfoundry -- --version`

## Risks And Follow-Up

- Existing users with old local database/config filenames will not be migrated automatically yet.
- Existing generated `target/` artifacts may contain old names until rebuilt.
