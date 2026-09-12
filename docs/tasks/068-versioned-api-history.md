# Task: W06 Versioned API, History And Client

## Goal
Expose a bounded v2 message-history query and versioned session snapshot/replay descriptor while preserving the existing v1 API behavior.

## Scope
- Included: server-owned v2 wire DTOs, keyset message pagination, snapshot payloads, v2 replay route compatibility, contract tests, and this task record.
- Explicitly excluded: storage/core/protocol/schema changes, a new client crate, TUI changes, instance-wide live events, and process-local idempotency.

## Design
`GET /api/v2/sessions/{session_id}/messages` accepts an opaque message ID `after` cursor and a bounded `limit` from 1 through 100. The response is `{ "version": 2, "messages": [...], "next": "..." | null }`; message DTOs are server-owned wire types converted from domain messages. Reads use the existing storage keyset query and 1 MiB byte bound.

`GET /api/v2/sessions/{session_id}/snapshot` returns a version-2 session DTO, a bounded initial message projection, and `{ "after": <durable event sequence>, "events_url": "/api/v2/sessions/{id}/events" }`. The v2 events route uses the existing durable SSE handler, retaining SSE IDs, replay cursors, framing, duplicate filtering, and best-effort live semantics. Unknown event kinds remain client-safe because event payloads are forwarded as JSON.

`POST /api/v2/sessions/{session_id}/prompt` deliberately returns `501 idempotency_unavailable`. Storage already has an idempotent `admit_run` API, but the current host/runner path uses the separate legacy prompt inbox and cannot safely bridge a v2 receipt without a core/storage contract change. The implementation does not claim idempotency through an HTTP-local map or duplicate side effects.

## Implementation
- Added `crates/server/src/routes_v2.rs` with separate history, snapshot, replay descriptor, session, and message wire DTOs.
- Added v2 history, snapshot, prompt capability, and event routes in `crates/server/src/lib.rs`.
- Added named API contract tests in `crates/server/tests/api_lifecycle.rs` covering bounded pagination, invalid cursor/limit errors, versioned snapshot shape, SSE content type, v1 compatibility, and explicit idempotency refusal.

## Compatibility Impact
- Existing `/api/v1` routes and response shapes remain unchanged; the full v1 server lifecycle suite remains green.
- V2 is additive except that v2 prompt admission is intentionally unavailable until an owner-approved host admission bridge exists.
- No public schema/protocol types, migrations, manifests, core, storage, or TUI files were changed.

## Verification
- `cargo test -p devfoundry-server --test api_lifecycle v2_`: RED first (4 named tests failed with 404 because v2 routes were absent); GREEN after implementation (4 passed).
- `cargo test -p devfoundry-server --test api_lifecycle v2_replay_keeps_sse_framing_and_v1_remains_available`: passed.
- `cargo fmt --all`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo test -p devfoundry-server --test api_lifecycle`: passed, 19 tests.
- `cargo check -p devfoundry-server --all-targets`: passed.
- `cargo clippy -p devfoundry-server --all-targets -- -D warnings`: passed.

## Risks And Follow-Up
- Snapshot acquisition is not one storage transaction: messages and the event watermark can change between reads. A future storage-owned snapshot watermark API is required for strict race-free reconstruction.
- V2 message pagination follows the existing message-ID ordering contract, while the legacy full-history API orders by timestamp then ID; clients must not mix cursors across endpoints.
- V2 prompt idempotency remains blocked by the missing host admission bridge. W06 stops at the boundary rather than editing core/storage/protocol owners.
- The v2 event handler preserves current best-effort live-event behavior; durable replay is authoritative and reconnect must use the returned cursor.
- Instance-wide live events remain deferred because no current host-owned bounded instance event source exists.
