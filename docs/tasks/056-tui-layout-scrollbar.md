# Task: TUI Layout And Scrollbar

## Goal

Make the DevFoundry TUI readable and visibly navigable for long project explanations.

## Implementation

- Added a right-side Details panel with session, agent, model, scroll, and keybinding information.
- Added a visible transcript scrollbar.
- Added `j/k` alongside arrow keys for line scrolling.
- Preserved PageUp/PageDown and Home/End navigation.
- Kept the prompt footer focused on input while showing navigation hints in the sidebar.

## Verification

- Run the full macOS workspace gates.
- Manually submit a long project explanation and use arrows, `j/k`, PageUp/PageDown, Home, End, and mouse wheel.
