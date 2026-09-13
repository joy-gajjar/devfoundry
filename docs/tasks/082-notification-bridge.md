# Task: W25 Optional Notification Bridge

## Goal

Provide an opt-in, sanitized notification boundary for task status updates with
durable deduplication and revocable local pairing. Notifications must never
become a remote shell, credential channel, or privileged approval path.

## Scope

- Included: notification binding metadata, bounded SQLite outbox, deduplication,
  pairing nonce/revision validation, setup/status/revoke API routes, browser
  settings projection, and fake-fixture tests.
- Explicitly excluded: scheduler schema changes, scheduler behavior changes,
  Telegram credentials, live Telegram smoke tests, remote shell, command
  execution, secret approval, permission approval, Git integration, and secret
  payload delivery.

## Design

The `notification_bindings` table stores only project/user/chat identifiers,
pairing nonce, task revision, enabled/revoked state, and nonce consumption.
The `notification_outbox` table stores bounded sanitized JSON containing only a
task ID, task revision, task status, and local deep link. A unique dedup key
prevents duplicate enqueueing. Existing scheduler and task tables remain
authoritative and unchanged.

Notifications are disabled until an explicit setup request stores a pairing
binding. Revoke persists disabled/revoked state and consumes the nonce. Reply
validation requires exact project, user, chat, nonce, enabled state, and task
revision; consumed, stale, wrong-chat, and malformed replies fail closed.

No external Telegram client dependency was added. The transport boundary is
not enabled by this package; future delivery work must use disposable fake HTTP
fixtures first and require separate dependency and credential approval for live
smoke testing.

## Implementation

- Added forward migration `migrations/0006_notifications.sql`.
- Added `crates/storage/src/notifications.rs` with the notification repository,
  bounded payloads, outbox deduplication, pairing validation, and revocation.
- Added `crates/core/src/notifications.rs` as the application service boundary;
  server handlers do not access notification storage directly for domain use.
- Added `crates/server/src/routes_notifications.rs`:
  - `GET /api/v2/projects/{project_id}/notifications/status`
  - `POST /api/v2/projects/{project_id}/notifications/setup`
  - `POST /api/v2/projects/{project_id}/notifications/revoke`
- Added browser typed status access and a settings projection. Browser storage
  is not used for notification credentials or pairing secrets.
- Added storage, server, and browser tests for disabled defaults, redaction,
  deduplication, wrong chat, stale revision, replay nonce, and UI behavior.

## Verification

### RED evidence

Before implementation, the new tests failed as expected:

- Storage failed to resolve the notification types and methods.
- Server status route returned `404` instead of `200`.
- Browser client lacked `getNotificationStatus`.
- Browser settings component did not exist.

### Targeted GREEN evidence

Passed:

```text
/Users/joy/.cargo/bin/cargo test -p devfoundry-storage --test notifications
2 passed

/Users/joy/.cargo/bin/cargo test -p devfoundry-server --test notifications
1 passed

cd apps/workspace && npm test -- --run src/api/notifications.test.ts src/notifications/NotificationSettings.test.tsx
2 test files, 2 tests passed

cd apps/workspace && npm run typecheck
passed
```

The tests use temporary SQLite stores and local in-process HTTP request
fixtures only. No credentials or external network calls are used.

### Full gate evidence

Passed:

```text
/Users/joy/.cargo/bin/cargo fmt --all -- --check
/Users/joy/.cargo/bin/cargo check --workspace --all-targets
/Users/joy/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
/Users/joy/.cargo/bin/cargo test --workspace
cd apps/workspace && npm test
cd apps/workspace && npm run typecheck && npm run build
cd apps/workspace && npx playwright test
8 Playwright tests passed
bash scripts/validate-secretless.sh
passed
```

`bash scripts/verify-release.sh` was not applicable: the script requires an
explicit release archive and checksum, and W25 does not produce release
artifacts. It was invoked without those arguments and printed its usage.

## Security boundaries and residual risks

- Default state is disabled.
- Payloads exclude secret values, tokens, prompts, command text, provider
  output, evidence contents, paths, and approval data.
- Pairing is bound to user/chat/project/nonce/revision and is single-use when
  consumed.
- Revoke invalidates the binding and nonce.
- Setup accepts only bounded opaque pairing metadata; it does not accept or
  persist a bot token.
- No delivery worker or live third-party transport is claimed by this task.
- A future trusted notification transport remains an external availability and
  metadata privacy boundary and needs independent rate-limit/retry review.

## Status

Implemented and integrated locally. Rust, browser, and secretless gates passed.
No release artifact verification was claimed because W25 produces no archive or
checksum; `scripts/verify-release.sh` correctly returned its required-argument
usage. Live Telegram delivery remains disabled and unconfigured.
