---
pipeline_id: 335d94c4-c6c5-4a5f-99f9-cc32444b49ac
ticket: docs/planning/tickets/open/TICKET-895-rail-shrink-motion-and-focus-ring.md
status: Phase 4 — Complete PASS
title: The rail's shrink in motion, and one focus ring everywhere
type: feature
slice: The Marley look, W6 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
The rail's labels fade as it folds: for the fold's first 60 ms the rail still draws its rows,
fading out, before the strip takes their place; opening, the rows fade in as the width eases.
One focus ring everywhere: 2 px of the focus color just outside a control's shape, on keyboard
focus, added once in Zed's shared `ui` components (Chad, 2026-10-23). Both honour reduced motion
(the ring does not move).

## Scope
### In
- `rail_strip.rs`, `rail.rs`: the labels' fade, and `MARLEY_MOTION_SCALE` in debug builds, which
  stretches the fold so a visual check can see it.
- `ui`: `focus_ring` (`styles/elevation.rs`), a box shadow spread 2 px in `border_focused`, used
  by `ButtonLike`, `Switch`'s and `Checkbox`'s toggle, `TreeViewItem` and `ChoiceCard` in their
  `focus_visible` styles.
- `ui_input`: the text field's focused border gains the ring.
- `settings_ui`: the Settings window's search field, the ticket's example, gains the ring while
  focused (one line; added in Code once the probe showed it draws its own field).

### Out (explicitly deferred)
- Controls that draw their focus their own way outside `ui` and `ui_input`, left as they are
  (Chad's answer): `settings_ui`'s number field and its page controls, `agent_ui`'s elicitation form, the
  editor and the terminal (a caret, not a ring), pickers and the rail (a selected row).

## Reference (§20)
N/A — Marley-specific look over gpui's `focus_visible` (keyboard focus only, as CSS's
`:focus-visible`) and the rail strip (#830).

### Prior art
- gpui's `InteractiveElement::focus_visible`; `ButtonLike`, `Switch`, `TreeViewItem` and
  `ChoiceCard` already restyle on it (a border or a background).
- `Strip::width` and `EASE` (#830); `cx.reduce_motion()`.
- The brain (consultation 82603e73): nothing on this seam.

## UI proof
Scenario `script/e2e/895-rail-fold-and-focus.sh` (sway), `MARLEY_MOTION_SCALE=40` so the fold
takes 7.2 s and the labels' fade 2.4 s.
- `895-01-fold-early`, `895-02-fold-late`: the fold under way: the rows fading, then the strip
  narrowing (REQ-001).
- `895-03-ring`: the Settings window, Tab onto a button: the ring outside it (REQ-002).
- `895-04-click`: the same control clicked with the mouse: no ring (REQ-002).
- `895-05-field`: the search field focused: the ring around it (REQ-002).
- `895-06-day`: the ring in Marley Day (REQ-002).

## Locked-In Decisions
- D1 — One ring, `ui::focus_ring`, the theme's `border_focused` (the coat's orange in Marley's
  themes), drawn as a shadow so it sits outside the shape.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the rail folds, its labels shall fade over the first 60 ms of the 180 ms fold before the strip shows, and WHILE reduced motion is on the rail shall fold at once. | `895-01-fold-early`, `895-02-fold-late` |
| REQ-002 | WHEN a `ui` control or text field takes the keyboard focus, it shall show a 2 px ring in the focus color outside its shape, and none after a mouse click on a control, in Marley Night and Marley Day. | `895-03-ring`, `895-04-click`, `895-05-field`, `895-06-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
