---
ticket: TICKET-017
forge: forge#17 (cf0cc61f-6a5b-4073-be7a-828dfc178b80)
status: closed
type: feature
milestone: M1
sprint: M1.B — The Cockpit (seq 1/5)
branch: ticket-017-ui-components-widgets
pipeline: docs/planning/pipeline/active/ui-components-widgets.spec.md
spec: docs/specs/SPEC-ui-components.spec.md
aar: 2946f859-4356-4651-9bbc-25322c038e0c
---

# TICKET-017 — marley_ui_components widgets

Sprint M1.B "The Cockpit" seq 1/5. The 5 reusable widgets of SPEC-ui-components (R1-R16) added to the
existing `marley_ui_components` crate: **button, switch, dialog, tooltip, keyboard-shortcut chip** — each
a long-lived struct + persistent interaction state, rendering from a borrowed `&ThemeColors`, exposing
PURE decision accessors. The primitives the command palette (#19) + settings (#21) render with.

## Acceptance
- The PURE decision surface (visual_state/button_colors/label_text/content_order, knob_position/
  track_color/knob_travel_x, content_summary/outcome_for, is_visible/poll, parse/keys + the latches +
  IconName) — **cov 100 / MSI 100**.
- The gpui render (`render/` subdir) + the widget-gallery bin — ACCEPTED-UNTESTABLE (mutants::skip +
  rust_cov exclude), proven by a `#[ignore]` headed widget-gallery test (AX + per-widget masked baselines).
- **Decision B**: hand-rolled on raw gpui, NO gpui-component (32-dep kit avoided); Marley-owned IconName.
- FULL `scripts/gates.sh` → `GATE GREEN` incl gate-15. §21: CHANGELOG + arch doc + the SPEC reuses/
  IconName delta.

## Notes
See the pipeline spec/notes for the full per-widget design + mutation map + headed plan. Persistent state:
each widget is a field on the host Render Entity (gpui re-renders the same instance). is_visible/animation
stay pure (elapsed/progress as inputs; the clock lives in the excluded render).
