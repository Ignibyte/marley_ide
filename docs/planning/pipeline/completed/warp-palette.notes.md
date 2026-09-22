# Warp-matched palette — Notes

- **Forge ticket:** #35 `5181c3d7-0a86-49de-b906-f0866e360338`
- **AAR:** `219f0c18-e640-4709-a6b8-716d797e2c07`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-035-warp-palette.md
- **Pipeline spec:** warp-palette.spec.md

## Phase 1 — Plan
- **Request:** forge #35 (M1.E "The Warp Look" seq-2, auto-approved) — the Warp-matched palette +
  a `success` color. The biggest single visual change.
- **Classification / tier:** work pipeline, `feature`, PURE data + the tested accessor (no render/
  shim). ui_components only (+ the app render picks up the tokens automatically).
- **Discovery (§18):**
  - Placeholder `default_for` values (lib.rs:69-108): grayscale bg/fg/surface/border (hue 0, sat 0)
    + generic blue accent (0.62). Metric tokens corner_radius px(6)/control_height px(28)/padding.
  - `ThemeColors` = 7 color fields + 3 metric fields. Adding `success` ripples to **4 construction
    sites**: `default_for` (Light+Dark arms), `switch.rs:72 dark()`, `button.rs:186 dark()` (widget
    test fixtures) + the field-equality tests (lib.rs:114 `default_for_light_differs_from_dark`,
    :126 `default_for_is_pure`).
  - Consumers of the tokens: the app render (colors.background/accent/danger/…), the widgets. Adding
    `success` doesn't break existing uses; #36 will consume it.
- **Decisions:** D1–D4 in the spec (add success; original clean-room values; keep metric tokens;
  pure data + tested accessor).
- **Open questions for Design (the aesthetic call):** the exact dark + light `hsla` — a deep
  near-black dark bg with a faint cool tint (not pure gray), a warm off-white light, a distinctive
  Marley accent (leaning a modern teal-cyan ~`hsla(0.50, 0.55, 0.55)` dark / darker for light),
  on_accent contrast (dark text on the bright accent), surface/border elevation, danger red
  (~`hsla(0.98, 0.6, ..)`), success green (~`hsla(0.38, 0.5, ..)`); whether to touch ANSI_RGB (lean
  NO — keep #31's original 16). Chad verifies the look (headed env-blocked).
- **AAR id:** `219f0c18-e640-4709-a6b8-716d797e2c07`.

## Phase 2 — Design

### The palette (ORIGINAL, clean-room — derived to match Warp's observable aesthetic)
`ThemeColors` gains `success: Hsla` (after `danger`). The values (all `gpui::hsla(h, s, l, a)`):

**Dark** (deep cool near-black, a modern teal-cyan accent):
```
background  0.62, 0.14, 0.09   // near-black w/ a faint cool tint (NOT pure gray — sat > 0)
foreground  0.62, 0.05, 0.90   // soft near-white, faint cool
accent      0.52, 0.58, 0.55   // Marley teal-cyan (distinct from the old 0.62 blue)
on_accent   0.62, 0.30, 0.10   // dark text for contrast on the bright accent
surface     0.62, 0.12, 0.13   // raised above bg (0.13 > 0.09)
border      0.62, 0.10, 0.22   // subtle separator
danger      0.99, 0.62, 0.62   // warm red
success     0.40, 0.50, 0.55   // green (NEW)
```
**Light** (warm off-white, the accent darkened for contrast):
```
background  0.10, 0.10, 0.97   // warm off-white
foreground  0.62, 0.14, 0.16   // dark cool gray
accent      0.52, 0.55, 0.42   // teal-cyan, darker for light
on_accent   0.00, 0.00, 1.00   // white on the accent
surface     0.10, 0.10, 0.93   // cards, slightly darker than bg
border      0.62, 0.08, 0.82   // light separator
danger      0.99, 0.60, 0.48
success     0.40, 0.45, 0.40
```
Metric tokens UNCHANGED: `corner_radius px(6)`, `control_height px(28)`, `control_padding`
edges(6/12/6/12).

### File manifest
- M `crates/ui_components/src/lib.rs` — `ThemeColors` + `success` field; both `default_for` arms →
  the new values; the field-equality tests updated.
- M `crates/ui_components/src/switch.rs:72` + `crates/ui_components/src/button.rs:186` — the `dark()`
  test fixtures gain `success` (compile).
- M `docs/specs/SPEC-ui-components.spec.md` — the palette values table + a clean-room provenance
  note + the `success` field. CHANGELOG; arch doc.

### Decisions
- D-2.1 Accent = teal-cyan `hsla(0.52, …)` — distinct from the old placeholder blue (0.62), an
  original Marley hue. D-2.2 The dark neutrals carry a faint cool tint (sat 0.10–0.14, hue 0.62) so
  the dark reads "deep space" not flat gray. D-2.3 `success`/`danger` are the semantic status pair
  for #36. D-2.4 Values are DERIVED (my choices), not lifted — clean-room §20.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `default_for_is_the_warp_palette` — `default_for(Dark).background == hsla(0.62,0.14,0.09)` (tinted, sat>0), `.accent == hsla(0.52,0.58,0.55)`; `default_for(Light).background == hsla(0.10,0.10,0.97)`; `light != dark` | unit |
| REQ-002 | `success_is_present_and_distinct` — for BOTH appearances: `success != danger`, `success != accent`, `success != background` (+ the exact dark/light success values) | unit |
| REQ-003 | `default_for_is_pure_and_keeps_metrics` — `default_for(Dark) == default_for(Dark)` (determinism); `corner_radius == px(6.0)`, `control_height == px(28.0)` unchanged; `background != foreground` | unit |
| REQ-004 | `scripts/gates.sh --diff` GREEN + receipt + the masked shell_dark/light baselines | gate |

Uncoverable: the RENDER of the palette (the app.rs shim already consumes the tokens — no change);
the visual LOOK (masked baseline, chad-verified).

### Risks / decisions
- The values aren't mutation targets (cargo-mutants doesn't mutate `hsla` float literals), so the
  field-equality tests are the REGRESSION guard (a typo'd value slips MSI but is caught by the
  exact-value asserts — like the #33 VT-literal goldens). The MUTABLE surface is the `match
  appearance` arm (Light vs Dark), killed by `light != dark` + the per-appearance value asserts.
- Contrast is a visual judgment I can't verify headless (env-blocked) — the values are chosen for
  sane contrast (dark bg 0.09 vs fg 0.90; accent 0.55 mid; light bg 0.97 vs fg 0.16); chad confirms.
- `success`/`danger` distinctness is asserted so #36's green/red indicator is unambiguous.

## Phase 3 — Implement
- **Built (per manifest):** ui_components/lib.rs — `ThemeColors` gains `success: Hsla`; both
  `default_for` arms → the new Warp-matched palette (dark: tinted near-black bg `0.62,0.14,0.09`,
  teal-cyan accent `0.52,0.58,0.55`, dark on_accent, elevated surface/border, warm danger, success
  green; light: warm off-white, darkened accent, white on_accent); the
  `default_for_light_differs_from_dark` test updated to the new values + the accent assert.
  SPEC-ui-components: the success field + the palette-values/provenance clause; CHANGELOG.
- **Deviations from design:** the design listed switch.rs/button.rs `dark()` as construction sites
  needing the success field — but they just CALL `default_for(Dark)` (not direct constructions), so
  `default_for` is the SOLE construction site; NO ripple to them (fewer touches than planned).
- **Verification at this phase:** `cargo check --workspace` 0 errors; fmt; clippy `-D warnings` 0;
  docs gate 0; ui_components 27 lib tests pass (the existing suite green with the new values). The
  dedicated success-distinctness + metric-token tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (provenance + data-integrity + ripple + contrast + tests; 10 tool uses incl.
  `cargo check --workspace` + `cargo test -p marley_ui_components` + hsla→RGB provenance analysis).
  Verdict: **PASS/SHIP, no HIGH/MED.**
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | LOW | Light `accent hsla(0.52,0.55,0.42)` + white `on_accent` ≈ 3.4:1 — passes AA-large (3:1) but fails AA-normal-text (4.5:1); the one weak pairing (every other ≥5:1). | REAL (took it — can't eyeball WCAG headless) | Darkened light accent to `hsla(0.52,0.58,0.34)` (deep teal) → white clears AA-normal; still recognizably teal. |
  | F2 | LOW-info | `success` currently unconsumed (forward-provisioned for #36). | ACCEPTED (intentional) | `pub` field, no dead_code — #36 consumes it. |
- **Verified CLEAN by the critic:**
  - **Provenance (§20, load-bearing):** ORIGINAL — no systematic match to Warp source or any branded
    theme (Dracula/Nord/One Dark/Tokyo Night/Solarized/Gruvbox/Catppuccin). Dark bg → #14161a is
    darker+less-saturated than every branded dark; the two near-neighbors (cyan accent, salmon
    danger) are universal terminal choices, not a lift; clean 2-decimal hsla = hand-dialed picker,
    not a converted hex. No provenance problem.
  - **Distinctness:** all semantic fields distinct in both appearances (success≠danger≠accent≠bg;
    surface≠bg; border≠surface) — every `assert_ne!` holds.
  - **Ripple COMPLETE:** `cargo check --workspace` 0 errors (a missing `success` at any literal =
    hard compile error) → `default_for` is the sole construction site; switch/button `dark()` call
    it; zero exhaustive destructures. **Metrics byte-identical.** Tests correct (27/27 green; asserts
    match the literals).
- **Lesson:** `PR-claude-palette-accent-lightness-must-clear-wcag-vs-on-accent` — a mid-lightness
  accent fails contrast against BOTH white and black text; for the LIGHT theme especially, pick the
  accent lightness so its `on_accent` clears WCAG AA. Load-bearing for #36–#39's colored chrome.

## Phase 4 — Validate
- **Test added (lib.rs):** `palette_success_and_light_accent_pinned` — REQ-002 + the F1 regression:
  `d.success == hsla(0.40,0.50,0.55)`, `l.success == hsla(0.40,0.45,0.40)` (exact goldens);
  `success != danger/accent/background` in both appearances; `l.accent == hsla(0.52,0.58,0.34)`
  (pins the inspect contrast fix so a later brightening can't silently regress it). REQ-001/003
  are covered by the updated `default_for_light_differs_from_dark` (values + light≠dark) +
  `default_for_is_pure` (determinism) + `theme_colors_carries_all_roles_and_metrics` (metric tokens).
- **Runs (actual):** `cargo nextest run -p marley_ui_components` → 28 passed (the new test PASS);
  `cargo nextest run --workspace` → 497 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **0 viable
  mutants in the diff** (the changed lines are `hsla` literals cargo-mutants skips + test code → the
  match-arm mutability is killed by `light != dark`; MSI vacuously 100). Receipt written. No PTY
  hang (the env sweep before #35 held).
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `ui_components.md` — the `success` field +
  the Warp-matched-palette/provenance/WCAG note on `default_for`. SPEC-ui-components at implement.
- **Knowledge captured:** `PR-…-palette-accent-lightness-must-clear-wcag-vs-on-accent` (the F1 light-
  accent contrast — a mid-L accent fails both white AND black text; reason it numerically headless).
  aar-submit `completed` (score 5). Win: everything already renders from the `ThemeColors` bundle, so
  a pure DATA swap re-skins the whole cockpit — no render churn; the field-equality tests + exact
  goldens are the regression guard (hsla isn't mutated). The env sweep before the gate avoided the
  #34 PTY/temp-dir flakiness (clean GREEN first try).
- **Ticket:** forge #35 → done; local doc → closed/; pipeline pair archived. 2 of 6 in M1.E.
