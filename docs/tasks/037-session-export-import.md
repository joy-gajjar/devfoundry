# Task 037: Portable Session Export And Import

## Status

Implemented.

## Contract

`devfoundry session export` writes versioned JSON with the `devfoundry.session`
format identifier. The payload contains normalized session metadata and message
history only. It does not contain project roots, session status, events, prompt
inbox rows, permissions, database paths, provider configuration, or credentials.

`devfoundry session import` requires an existing project ID. Imports are
transactional, reject unknown versions and duplicate session IDs, and always
start in `idle` state rather than restoring active runtime work.

Examples:

```text
devfoundry session export --database .devfoundry.db --session <SESSION_ID> --output session.json
devfoundry session import --database .devfoundry.db --project <PROJECT_ID> --input session.json
```

The JSON contract is defined in `devfoundry_schema::SessionExport` and storage
coverage verifies round trips, runtime-record exclusion, version rejection, and
duplicate protection. The file format uses ordinary UTF-8 JSON and filesystem
paths are passed through Rust path APIs, including on macOS.
