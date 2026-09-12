# Task: TUI Paste Latency

## Problem

Pasting a prompt into the connected TUI was slow because the loop awaited permission/session HTTP refreshes before processing more input and only polled terminal events every 100 ms. Paste input was also handled as individual key events.

## Fix

- Handle Crossterm `Event::Paste` as one string append.
- Reduce terminal polling to 10 ms.
- Schedule permission/session refreshes every 500 ms without making them the input-loop cadence.
- Add a direct paste regression test.

## Verification

Run:

- `cargo fmt --all`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Manual Test

```bash
cargo run -p devfoundry
```

Paste a multi-line prompt. It should appear as one immediate input update, then press Enter to submit.
