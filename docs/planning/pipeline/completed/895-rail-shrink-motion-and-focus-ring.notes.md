# The rail's shrink in motion, and one focus ring everywhere — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-895-rail-shrink-motion-and-focus-ring.md
- **Pipeline spec:** 895-rail-shrink-motion-and-focus-ring.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "lets finish up the remaining tickets besides the mac issues";
  his answer: one ring app-wide in the shared components, keyboard only, the rest listed; the
  labels fade.
- **Classification / tier:** feature, S–M. Marley crate plus `ui` and `ui_input`.
- **Recall (§18.3):** the ticket's notes: a burst of shots 60 ms apart saw only the fold's end, so
  the check stretches the fold (`MARLEY_MOTION_SCALE`, debug builds only). gpui's
  `focus_visible` is keyboard-only already.
- **Design:** `Strip::label_fade` gives the rows' opacity while a fold runs; the rail draws its
  rows in a wrapper with that opacity and the strip only after the fade. `ui::focus_ring(style,
  cx)` adds a `BoxShadow` (no offset, no blur, spread 2 px, `border_focused`); each
  `focus_visible` in `ui` adds it beside its own change.
- **Manifest:** Marley: `rail_strip.rs`, `rail.rs`. Zed: `crates/ui/src/styles/elevation.rs`,
  `button/button_like.rs`, `toggle.rs`, `tree_view_item.rs`, `choice_card.rs`;
  `crates/ui_input/src/input_field.rs` (a ledger row each).

## Phase 2 — Code
- `ui::focus_ring` (generic over `Styled`, so a `Div` and a `StyleRefinement` both take it), in
  `ButtonLike`, the switch, `TreeViewItem`, `ChoiceCard` and `InputField`.
- `Strip::rows_opacity`, `LABEL_FADE`, `ease()` and `motion_scale()`; `Rail::render` draws the
  rows in a faded wrapper while a fold runs.
- Review: clippy wanted `motion_scale` const once `#[cfg(debug_assertions)]` removed its body in
  its profile; `cfg!` keeps one body.
- Deviation: the Settings window's search field is `settings_ui`'s own, not `InputField`; the
  probe showed it without a ring, so it got the one line and its ledger row grew.
- Gate: `just gate-diff` 17 passed, 0 failed.

## Phase 3 — Test
- Scenario `script/e2e/895-rail-fold-and-focus.sh` under sway, `MARLEY_MOTION_SCALE=40`.
- `895-01-fold-early` (REQ-001): the rail narrowing with its rows still drawn; the labels' peak
  brightness 195 against 243 on a still rail, so caught mid-fade.
- `895-02-fold-late` (REQ-001): the strip, the width still above 60 px.
- `895-05-field` (REQ-002): the search field clicked: the ring around it (a field shows it on
  any focus, as CSS does).
- `895-03-ring` (REQ-002): three Tabs: the ring outside the Appearance row.
- `895-04-click` (REQ-002): a control clicked with the mouse: no ring anywhere.
- `895-06-day` (REQ-002): Marley Day, one Tab: the ring around the theme dropdown.
- Reduced motion: not driven; `motion` stays `None` under it (#830's path), so neither fades.

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md` (the labels' fade), look-plan States row, seven ledger rows.
- Knowledge: nothing new.
- Brain: decided on consultation 82603e7391064bca857d4da6821ef773.
