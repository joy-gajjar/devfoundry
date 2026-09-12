# LLM And Providers

## Provider-Neutral Contract

The LLM crate exposes a request with model reference, messages, tools, generation controls, and cancellation. It returns a stream of normalized events:

- `TextDelta`
- `ReasoningDelta`
- `ToolCallDelta`
- `ToolCallComplete`
- `Usage`
- `Finish`
- `ProviderError`

## Adapter Boundary

The GitHub Copilot adapter owns authentication headers, wire encoding, streaming parsing, provider options, error mapping, and native continuation metadata. The runner sees no provider wire types.

## First Adapter

Implement the GitHub Copilot-compatible adapter as the sole external provider. Support base URL override, environment-backed token, model, streaming, tool calls, usage, cancellation, and retry classification.

## Provider Registry

Registry entries declare provider ID, model catalog, capabilities, context limit, supported tools, reasoning support, and credential source. Secrets are resolved at request time, not stored in model records.

## Retry Policy

Retry only transport failures, explicit rate limits, and selected transient server errors. Never blindly retry malformed requests, authentication failures, tool calls with uncertain settlement, or user cancellation.

The provider layer classifies transport and selected HTTP failures as `Transient`, HTTP 429 as `RateLimited`, content filtering as `ContentFiltered`, and other HTTP failures as `Permanent`. Classification is metadata for the owning retry policy; the provider does not retry requests itself.

## Context And Token Budget

The provider adapter reports usage when available. Core owns context budgeting, truncation, compaction, and system context. Provider-specific limits are metadata, not scattered conditionals in the runner.

## Streaming Rules

- Parser tolerates chunk boundaries and empty keep-alives.
- Malformed chunks become typed provider errors with request ID.
- Partial text is persisted according to a deliberate policy; final durable message settlement is atomic.
- Cancellation closes the response and records the attempt outcome.

## Future Adapters

GitHub Copilot is the only supported external provider. Other external adapters and local inference are intentionally out of scope. Provider-specific features must be capability-gated, not silently emulated.

## Evaluation

Maintain a fixture suite for text, reasoning, parallel and serial tools, malformed streams, rate limits, content filtering, usage, and cancellation. Compare adapters at the normalized event layer.
