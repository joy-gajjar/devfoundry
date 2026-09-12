# Task: Contract Freeze And Identities

## Goal

Add the minimal typed identities and transport contracts needed for the staged execution roadmap without changing existing runtime routes.

## Scope

- Included: run/attempt/task/artifact IDs, revision/outcome types, versioned event envelope, prompt idempotency/attachment request types, and serialization tests.
- Excluded: persistence migrations, API route adoption, scheduler behavior, provider changes, and UI changes.

## Design

- Existing IDs remain unchanged and new IDs use the same ULID transparent wrapper pattern.
- `RunOutcome::OutcomeUnknown` represents a possible side effect without a durable result.
- `SubmitPrompt` carries an idempotency key, expected session revision, and immutable attachment references for later admission validation.
- `EventEnvelope` is transport-neutral and carries a version, sequence, run identity, aggregate revision, kind, and JSON payload.

## Implementation

- `crates/schema/src/lib.rs`: added `RunId`, `AttemptId`, `TaskId`, `ArtifactId`, `Revision`, `RunOutcome`, and `EventEnvelope`.
- `crates/schema/Cargo.toml`: added the runtime `serde_json` dependency for the envelope payload.
- `crates/protocol/src/lib.rs`: added `AttachmentRef` and `SubmitPrompt`.
- Added round-trip tests for prompt submission and terminal run outcomes.

## Verification

- `cargo test -p devfoundry-protocol`: passed, 3 tests.
- Full workspace gates are required before W01 acceptance.

## Risks And Follow-Up

- These types are not yet persisted or consumed by runtime code.
- W02 must assign migration numbers and persist revisions/attempts transactionally.
- W06 must add API fixtures and validate attachment ownership/capability before admission.
