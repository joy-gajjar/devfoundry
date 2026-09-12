# Task: TUI Explicit Scroll Controls

## Problem

Transcript scrolling was implemented but not obvious in the terminal, especially while the prompt editor had focus.

## Fix

Added:

- Up/Down: one line
- PageUp/PageDown: five lines
- Home: jump to oldest transcript content
- End: return to newest content
- Ctrl-U/Ctrl-D: larger scroll movement
- Mouse wheel: scroll transcript
- Footer state: `at bottom` versus `scrolling history`

New prompts and assistant deltas continue to follow the bottom automatically.

## Verification

Run `cargo run -p devfoundry`, submit a long prompt, then use the controls above to inspect the transcript.
