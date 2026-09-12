# Task: Permission Broker

## Goal

Replace development-only allow-all execution with a durable, interruptible permission workflow that can be answered by a client.

## Scope

- Included: permission request schema, storage, pending broker waiters, permission events, resolution endpoint, and runner integration.
- Excluded: TUI approval dialog, remembered policy scopes, timeout expiry, and multi-process permission coordination.

## Design

Each session runner owns a permission service registered while its execution is active. A tool authorization creates a durable pending request, publishes `PermissionRequested`, and waits on a oneshot response. The API resolves a pending request by ID, persists allowed/denied status, wakes the waiter, and publishes `PermissionResolved`. Unknown or already-settled requests fail safely.

## Implementation

- Added permission request/status domain records.
- Added `permission_requests` migration table.
- Added storage create/get/resolve methods.
- Added session permission service and runner resolution lookup.
- Added `POST /api/v1/permissions/{permission_id}/resolve`.
- Removed runner use of `AllowAllPermissions`.
- Corrected the pending-status SQL predicate to match serialized enum storage.

## Verification

Blocked locally because the Rust toolchain is unavailable. `git diff --check` is required; Cargo format, compilation, Clippy, tests, and permission integration tests remain pending.

## Risks And Follow-Up

- Permission services are process-local and pending requests are not recovered after restart yet.
- Cancellation must remove/wake pending permission waiters cleanly.
- Add TUI approval UI, timeouts, scope policies, and permission race tests before public use.
