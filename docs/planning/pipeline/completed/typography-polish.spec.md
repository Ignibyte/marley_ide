---
pipeline_id: 7e989151-8413-484a-a636-c01df10ecec6
ticket: forge#39 (85e6b003-36f3-4dbf-8f4f-d8885d84b5f6) · local docs/planning/tickets/open/TICKET-039-typography-polish.md
aar_id: af2f8cc1-6a56-4ecb-aee0-8bedd4533c40
status: Phase 5 — Complete PASS
title: typography + focus/hover affordances pass (M1.E finale)
type: feature
milestone: M1.E
references:
  - crates/ui_components/src/lib.rs (ThemeColors — gains `muted`)
  - crates/marley_app/src/typography.rs (NEW — the pure type scale)
  - crates/marley_app/src/app.rs (apply the scale + affordances — shim)
  - docs/specs/SPEC-ui-components.spec.md + SPEC-app-shell.spec.md
---

## Title
M1.E FINALE — the cohesion pass. With the font (#34), palette (#35), Blocks (#36), prompt (#37), and
docks (#38) in place, formalize the ad-hoc `FontWeight`/`text_sm` scattered in the render into ONE
tested type scale, add the `muted` caption color (#38 flagged), and refine the focus/hover
affordances — so `cargo run -p marley` reads as a finished Warp-style terminal.

## Scope
### In
- `crates/ui_components/src/lib.rs` (PURE — cov/MSI 100): `ThemeColors` gains a `muted: Hsla` field
  (the secondary/caption text color — dimmer than `foreground`, brighter than `border`). Both
  `default_for` arms get a dark + light value; the field-equality tests updated.
- `crates/marley_app/src/typography.rs` (NEW, gpui-FREE PURE — cov/MSI 100):
  - `Role { Command, Output, Caption }`, `TextWeight { Normal, Medium, Bold }`,
    `TextStyle { size: f32, weight: TextWeight }`.
  - `type_scale(role: Role) -> TextStyle` — `Command → (14, Medium)`, `Output → (14, Normal)`,
    `Caption → (12, Normal)` (the one scale replacing the scattered literals).
  - `weight_value(w: TextWeight) -> f32` — the CSS numeric weight `Normal→400 / Medium→500 /
    Bold→700` (the shim wraps it in `gpui::FontWeight(..)`, keeping this module gpui-free).
- `crates/marley_app/src/lib.rs` — `mod typography;`.
- `crates/marley_app/src/app.rs` (SHIM) — apply `type_scale` + `weight_value` to the command header
  (#36, `Command`), the output rows (`Output`), the dock caption (#38, `Caption` in `muted`);
  consolidate row padding toward the tokens; refine the focused-pane accent border; add a hover
  affordance to Block/palette rows (the palette #25 already highlights the active row — extend).
- SPEC-ui-components (`muted`) + SPEC-app-shell (type scale + affordances). CHANGELOG + arch docs.

### Out (explicitly deferred)
- Custom user themes / a theme editor / more built-in themes (M2). User-configurable font size/family
  (M2). A full spacing-token redesign (this consolidates toward the EXISTING
  control_height/control_padding/corner_radius, not new tokens). Animation/transitions. Dock icons
  (#38 deferred). The ANSI bold flag stays driven by the terminal's own `Flags` (#31), not the scale.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `typography.rs` is gpui-FREE: `type_scale`/`weight_value` return plain `f32`/marley enums; the
  app.rs shim maps `weight_value → gpui::FontWeight` + `size → px`. Fully testable, no gpui in the
  pure surface.
- D2 — `muted` is a new `ThemeColors` role (the #38 caption gap) — a contained ripple (default_for +
  the field-equality tests; the widget `dark()` fixtures call `default_for`, no ripple — same as #35).
- D3 — The scale has 3 roles (Command/Output/Caption); ANSI bold is separate (#31 Flags). Values are
  a starting scale chad tunes visually.
- D4 — PURE: `muted` (data) + `type_scale`/`weight_value` (logic), cov/MSI 100; the render
  application + hover/focus tuning are SHIM (app.rs, masked visual).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `default_for(Dark)`/`(Light)` is called, it shall include a `muted` color distinct from `foreground`, `background`, and `border`. | unit tests (the exact per-appearance values + distinctness) |
| REQ-002 | WHEN `type_scale(role)` is called, it shall return the role's `TextStyle`: `Command→(14, Medium)`, `Output→(14, Normal)`, `Caption→(12, Normal)`. | unit tests (each role) |
| REQ-003 | WHEN `weight_value(w)` is called, it shall return `Normal→400.0`, `Medium→500.0`, `Bold→700.0`. | unit tests (each weight) |
| REQ-004 | WHEN the cockpit is rendered, the command header/output/dock-caption shall use `type_scale` (the caption in `muted`), with a refined focus border + a Block/palette-row hover affordance. | shim + the masked full-cockpit visual baseline — chad's final sign-off |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `muted`/`type_scale`/`weight_value`; the full-cockpit visual rides the masked deferral. | gate exit 0 + receipt + gate:15 |

## Phase Plan
- **P2 Design** — the exact `muted` dark/light values + the `typography.rs` shapes (roles, the scale
  values, weight_value) + the app.rs application points + hover/focus tuning, the SPEC clauses +
  mutation targets.
- **P3 Implement** — `muted` + typography.rs + mod + the app.rs application + specs + CHANGELOG.
- **P3.5 Inspect** — critics: type_scale/weight_value arms killable, muted distinct + clean-room, the
  shim uses the scale (no re-scatter), no widget/#35 regression.
- **P4 Validate** — the muted + type_scale + weight_value unit tests + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #39 — **and close the M1.E sprint**.
