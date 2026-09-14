# Desktop Design System and Shell Plan

**Goal:** Establish the shared desktop layout and accessible visual system before feature screens are added.
**Architecture:** Extract reusable React layout primitives from the existing workspace rather than maintaining separate browser and desktop implementations.
**Tech Stack:** React/Vite/TypeScript/CSS/Playwright.
**Spec:** `00-overview.plan.md`; UI/UX Pro Max research selected Minimalism/Swiss, Atkinson Hyperlegible, Indigo/Orange semantic tokens.

## Global Constraints

- Preserve existing browser behavior and tests while extracting shared UI.
- Keyboard access, visible focus, reduced motion, and contrast are required.

## Mandatory Reading

- `apps/workspace/src/App.tsx:1-90`.
- `apps/workspace/src/styles.css:1-25`.
- `apps/workspace/playwright.config.ts`.

## Tasks

1. Add design tokens for surface, text, border, primary, accent, danger, focus, spacing, and typography.
2. Extract shell components: top bar, project rail, main surface, inspector, bottom drawer, mobile tabs.
3. Preserve current Chat/Tasks/Docs/Preview/Terminal/Settings surfaces.
4. Add light/dark theme state and reduced-motion CSS.
5. Add keyboard focus order, skip-to-main, semantic headings, and accessible labels.
6. Add responsive desktop containment tests at 1280x800 and 1440x900.

## Acceptance

- Existing browser tests remain green.
- Tauri and browser use the same renderer components.
- No horizontal overflow at supported desktop sizes.
- Keyboard-only navigation and visible focus work.
- Contrast and reduced motion requirements are documented and tested.
