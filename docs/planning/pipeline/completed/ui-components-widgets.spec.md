---
pipeline_id: 2b88f960-2379-4da2-9ba6-e48e532c2158
ticket: forge#17 (cf0cc61f-6a5b-4073-be7a-828dfc178b80) · local docs/planning/tickets/open/TICKET-017-ui-components-widgets.md
aar_id: 2946f859-4356-4651-9bbc-25322c038e0c
sprint: M1.B — The Cockpit (cbc92bf0) seq 1/5
status: Phase 5 — Complete PASS
title: marley_ui_components widgets — button / switch / dialog / tooltip / keyboard-shortcut
type: feature
milestone: M1
references:
  - ../../../specs/SPEC-ui-components.spec.md
  - ../../../marley_architecture/ui_components.md
---

## Title

TICKET-017 — the 5 reusable widgets of [`SPEC-ui-components`](../../../specs/SPEC-ui-components.spec.md)
(R1–R16) ADDED to the EXISTING `marley_ui_components` crate (which already ships `Appearance` +
`ThemeColors` + `default_for` from #13). Sprint M1.B "The Cockpit" seq 1/5 — the UI primitives the
command palette (#19) + settings (#21) render with. Each widget is a long-lived struct holding
persistent interaction state, renders from a borrowed `&ThemeColors`, and exposes **pure decision
accessors** alongside its gpui render.

## Scope

### In (this ticket — SPEC-ui-components R1–R16)
- **Button** (R2–R8, R15, R16): `visual_state` (Rest/Hover/Press/Disabled precedence), `button_colors`
  (per-(kind,state) from `&ThemeColors`), `label_text` (R3), `content_order` (icon/label, R4) + the
  `set_hover`/`set_press` latch writes.
- **Switch** (R9, R2): `knob_position` (Leading/Trailing), `track_color` (on→accent/off→surface) +
  `knob_travel_x(on, progress, track, knob)` (continuous knob position; `progress` is an animation INPUT).
- **Dialog** (R11, R12): `content_summary` (scrim + card sections), `outcome_for` (control→outcome).
- **Tooltip** (R13, R15, R16): `is_visible` (a stored latch) + `poll(hovering, elapsed, delay)` (the
  threshold transition — `elapsed` is an INPUT, not a clock).
- **KeyboardShortcut** (R14): `parse(&str)` (split on `+`, trim, drop empties → ordered keys), `keys()`.
- **Marley-owned `IconName`** (spec-delta — replaces gpui-component's; see Decisions).
- §21 — CHANGELOG + UPDATE `docs/marley_architecture/ui_components.md`.

### Out / deferred
- gpui-component (NOT used — decision B; see below). The Component/Params/Options trait triad (already
  struck by the spec in favor of per-widget structs + `render(&ThemeColors, props)`).
- Any change to `Appearance`/`ThemeColors` (frozen from #13). The R2-vs-hover-visual tension is resolved
  by a render-side overlay, NOT a new ThemeColors field (see Decisions).

## Acceptance Criteria (EARS — adopt SPEC-ui-components R1–R16 verbatim)
- **AC-button (R2–R8)** — `visual_state` yields Disabled (dominant) / Press / Hover / Rest by precedence;
  `button_colors(kind, &ThemeColors, state)` returns a bundle whose every color EQUALS a `ThemeColors`
  field (no literal), MUTUALLY DISTINCT per state; `label_text`/`content_order` per R3/R4; R5 pinned:
  `(Primary, Rest)` → `{background: accent, foreground: on_accent}`.
- **AC-switch (R9)** — `knob_position(on)` = Trailing/Leading; `track_color(&,on)` = accent/surface.
- **AC-dialog (R11/R12)** — `content_summary` flags scrim + title/body/confirm + cancel-iff-Some;
  `outcome_for(Confirm/Cancel)` = Confirm/Cancel.
- **AC-tooltip (R13/R15/R16)** — `poll(hovering,elapsed,delay)` sets visible iff `hovering && elapsed >=
  delay` (leave→hidden); `is_visible` reads the latch; default hidden.
- **AC-shortcut (R14)** — `parse("cmd+shift+p").keys()` = `[cmd, shift, p]` (no empty entries, ordered).
- **AC-state (R15/R16)** — the latch writes (`set_hover`/`set_press`/`poll`) mutate the SAME persistent
  instance across renders; a fresh widget defaults to Rest/hidden.
- **AC-gate** — the pure decision surface (the accessors + latches, all 5 widgets + IconName) cov 100 /
  MSI 100; the gpui render (`render/` subdir) + the gallery bin ACCEPTED-UNTESTABLE (mutants::skip +
  rust_cov exclude); a `#[ignore]` headed widget-gallery visual test (AX + per-widget masked baselines);
  FULL `scripts/gates.sh` → `GATE GREEN [diff]`.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **DECISION B — hand-roll on RAW gpui, NO gpui-component.** gpui-component 0.5.1 DOES resolve with gpui
   0.2.2 (declares `gpui ^0.2.2` from crates.io, no conflict) BUT pulls 32 non-optional deps
   (markdown/html5ever/lsp-types/tree-sitter/…) for 5 trivial widgets. The pure accessors need none of
   it; `marley_app/src/app.rs` proves raw-gpui renders headed; the spec already struck gpui-component's
   component model. So build the widget renders on raw gpui (`div()/flex()/bg(Hsla)/.child()`), and make
   **`IconName` Marley-owned** (an `icon.rs` enum). SPEC-DELTA to apply in §21: amend the spec's
   `reuses: [gpui, gpui-component]` → `[gpui]` + the "IconName reused from gpui-component" line →
   Marley-owned (also cleaner clean-room). Cargo.toml gets NO new normal dep.
2. **The pure/shim seam** — PURE (cov 100/MSI 100): button.rs/switch.rs/dialog.rs/tooltip.rs/
   keyboard_shortcut.rs/icon.rs (the accessors + latches; gpui-`Hsla`/`Pixels`/`SharedString` only). SHIM
   (ACCEPTED-UNTESTABLE, mutants::skip + rust_cov exclude `ui_components/src/render/|ui_components/src/
   bin/`): a `render/` subdir (per-widget `render(&mut self, &ThemeColors, props)->AnyElement` + the
   on_hover/on_mouse handlers calling the pure latches) + `bin/marley_widgets_gallery.rs`.
3. **Persistent state** — each widget is a FIELD on a host `Render` Entity (like `RootView`), so gpui
   re-renders the SAME instance → the `hovered`/`pressed`/`visible` latches persist (R15). NEVER
   reconstruct a widget per frame. The clock (`Instant`/animation) lives in the excluded render, which
   feeds `elapsed`/`progress` INTO the pure `poll`/`knob_travel_x` (keeping them clock-free/testable).
4. **The R2-vs-hover tension** — `ThemeColors` has no "accent-hover" field; `button_colors` stays
   field-equality-clean (each state a distinct existing field, for killable arm-swap mutants), and the
   hover/press BACKGROUND pixel delta is a translucent overlay in the EXCLUDED `render/button.rs`
   (asserted by the `button_hover` baseline). Do NOT redesign `ThemeColors`.
5. Deps: gpui="0.2.2" (present). dev: marley_visual_harness(path) + mutants. `[[bin]]
   marley_widgets_gallery`. NO gpui-component.

## Phase Plan
- **P2 Design** — the detailed per-widget structs + accessor signatures + the mutation map + the headed
  fixture (the analysis is captured in the notes' Carry-to-Design). Confirm the raw-gpui render shapes vs
  app.rs (write-first). The file split + the rust_cov exclude.
- **P3 Implement** — the 5 pure widget modules + icon.rs FIRST (zero gpui), then the render/ shim +
  the gallery bin; the rust_cov exclude + the spec-delta.
- **P3.5 Inspect** — critics: the button_colors distinct-per-state (mutant killability), the tooltip
  poll threshold, the parse edge cases, the seam (only render/+bin excluded), clean-room (IconName).
- **P4 Validate** — the pure unit suite (all accessors, non-trivial fixtures) + the #[ignore] headed
  widget-gallery (AX + masked baselines); FULL gate incl gate-15.
- **P5 Complete** — §21 (CHANGELOG + arch doc + the SPEC-ui-components reuses/IconName delta); close #17;
  → #18 keymap.
