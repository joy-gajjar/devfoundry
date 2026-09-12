# Task: TUI J/K Input Collision

## Problem

Binding `j` and `k` globally to scrolling made it impossible to type ordinary words containing those letters into the prompt editor.

## Fix

- `j/k` are normal text input by default.
- `Ctrl-S` toggles explicit transcript scroll mode.
- In scroll mode, `j/k` scroll one line and prompt text input is paused.
- Arrow keys, PageUp/PageDown, Home, End, and mouse wheel remain available.
- The sidebar/footer show when scroll mode is active.

## Verification

- Test typing `j` and `k` into the prompt.
- Toggle `Ctrl-S`, scroll with `j/k`, then toggle again to resume typing.
- Run the full macOS workspace gates.
