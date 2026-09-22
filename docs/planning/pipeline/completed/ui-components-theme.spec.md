---
pipeline_id: 56adbd8a-2071-439c-b265-7dde69ec9608
ticket: forge#13 (5aebc012-a61f-41f8-8a83-bce7a1891a90) · local docs/planning/tickets/open/TICKET-009-ui-components.md
aar_id: 585c0d67-ebf5-4e72-863c-33b3028c811c
sprint: M1.A — The Usable Terminal (aa46e22f) seq 2/5
status: Phase 5 — Complete PASS
title: marley_ui_components (M1.A cut) — the theming value vocabulary (Appearance + ThemeColors)
type: feature
milestone: M1
references:
  - ../../../specs/SPEC-ui-components.spec.md
  - ../../../marley_architecture/crate-map.md
---

## Title

TICKET-009 — `marley_ui_components`, **M1.A cut**: the workspace's single **theming value vocabulary**
(`Appearance` + `ThemeColors` + `ThemeColors::default_for`) — the lowest gpui-dependent crate, owning
the `Hsla`-bearing palette that the gpui-free `marley_core` cannot hold (SPEC-ui-components §Purpose,
seam-contracts §6). It is what `terminal_blocks` (#14), `editor` (#15), and `app_shell` (#16) render
from for a consistent look — the genuinely foundational part of SPEC-ui-components that M1.A needs.

## Scope

### In (this ticket, M1.A)
- **`Appearance`** — the `Light`/`Dark` discriminant enum.
- **`ThemeColors`** — the shared bundle every consumer renders from: the 7 `gpui::Hsla` roles
  (`background`/`foreground`/`accent`/`on_accent`/`surface`/`border`/`danger`) + the 3 metrics
  (`corner_radius`/`control_height`/`control_padding` as `gpui::Pixels`/`Edges<Pixels>`).
- **`ThemeColors::default_for(Appearance) -> ThemeColors`** — the built-in Light/Dark fallback palettes
  (R17); the two differ.
- This is a **pure value-type library** — NO `gpui::Render`/Element-tree path, NO headed surface →
  **gate-15 N/A** (no `visual_acceptance`), **no ACCEPTED-UNTESTABLE shim**: 100% of the crate is pure +
  testable (cov 100 / MSI 100 whole-crate).
- §21 — CHANGELOG + `docs/marley_architecture/ui_components.md`.

### Out / DEFERRED to M1.B (a new ticket — note in complete)
- **The 5 widgets** — `button`, `switch`, `dialog`, `tooltip`, `keyboard_shortcut` (SPEC-ui-components
  R1–R16, the pure decision accessors + the gpui render shims + the per-widget visual baselines). They
  serve M1.B's command-palette/settings UI; the M1.A terminal/editor/shell render TEXT/containers/lists
  from `&ThemeColors` directly and use NO widget. Building them now is premature (M1.B work) and would
  delay the usable terminal. `gpui-component` (the widget primitives dep) defers with them.
- The brand palettes / `Theme`/`ThemeRegistry` — `marley_app` (#16) owns those + supplies the M1 palette
  (SPEC-ui-components §Dependencies); this crate ships only the value types + `default_for` fallback.

## Acceptance Criteria (EARS)
- **AC-1 (R17)** — `ThemeColors::default_for(Appearance::Light) != ThemeColors::default_for(Appearance::Dark)`,
  and each *shall* be the corresponding fallback palette (the Light branch returns the light bundle, the
  Dark branch the dark). (Unit test.)
- **AC-2 (the bundle)** — `ThemeColors` *shall* carry all 7 color roles + 3 metric fields as public
  fields the consumer reads; `Appearance` *shall* be the Light/Dark discriminant. (Compile + a
  field-presence/round-trip test; the value-type contract downstream binds to.)
- **AC-3 (purity)** — `default_for` *shall* read no process-global/thread-local state and depend only on
  its `Appearance` argument (R1's foundation). (Unit test: same input → same output.)
- **AC-4 (gate)** — library crate, gate-15 N/A (no UI surface); **cov 100 / MSI 100 whole-crate** (no
  exclude — all pure); FULL `scripts/gates.sh` → `GATE GREEN [diff]`.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **M1.A cut = the theme vocabulary ONLY.** The 5 widgets defer to M1.B (they're palette/settings UI,
   unused by the terminal). Lean = the usable terminal comes fast. Documented deferral (a new M1.B ticket
   created at complete).
2. **Pure value-type library** — no Render/headed path in #13, so NO `mutants::skip`, NO `rust_cov`
   exclude, NO `visual_acceptance`. 100% of the crate is the testable surface.
3. **Marley-original naming** — `Appearance` (the Light/Dark enum) + `ThemeColors` (the bundle), per
   seam-contracts §6 / clean-room §20. No fork names.
4. **Deps: `gpui`** only (for `Hsla`/`Pixels`/`Edges`). NO `gpui-component` (defers with the widgets).

## Phase Plan
- **P2 Design** — the module layout (`appearance` + `theme_colors`, or one `lib.rs`); the exact
  `default_for` Light/Dark palettes (pick reasonable Hsla values — terminal-friendly dark default); the
  regression-test plan + mutation map (the Light/Dark branch is the key viable mutant; `default_for`
  must be both-sides-tested). Confirm `ThemeColors` needs no `Default` (so the Default mutant is unviable)
  OR derive it deliberately.
- **P3 Implement** — the value types + `default_for`. Pure; documented; check/clippy/fmt.
- **P3.5 Inspect** — critic: the Light/Dark branch + the palette-difference correctness; clean-room
  naming; any non-obvious mutant.
- **P4 Validate** — the unit tests (AC-1..3); FULL gate (gate-15 N/A).
- **P5 Complete** — §21; close #13; **create the M1.B widgets ticket** (the deferred 5); archive; → #14.
