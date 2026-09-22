---
pipeline_id: 613b7bf6-b4d1-42cf-8280-b6ee70dd3151
ticket: forge#35 (5181c3d7-0a86-49de-b906-f0866e360338) · local docs/planning/tickets/open/TICKET-035-warp-palette.md
aar_id: 219f0c18-e640-4709-a6b8-716d797e2c07
status: Phase 5 — Complete PASS
title: Warp-matched theme palette (dark + light)
type: feature
milestone: M1.E
references:
  - crates/ui_components/src/lib.rs (ThemeColors + default_for — the palette values + the field-equality tests)
  - crates/ui_components/src/switch.rs:72 + button.rs:186 (dark() test fixtures — the success ripple)
  - docs/specs/SPEC-ui-components.spec.md (gains the palette values + provenance clause)
---

## Title
`ThemeColors::default_for` holds PLACEHOLDER values — pure-grayscale bg/fg/surface/border (`hsla(0,
0, x)`) + a stock blue accent — so the cockpit looks unfinished. Everything renders from this token
bundle (bg/fg/accent/on_accent/surface/border/danger + the metric tokens), so replacing the values
re-skins the whole app. Swap in an ORIGINAL Warp-matched palette (dark + light) and add a semantic
`success` color (green) the #36 exit-status indicator needs.

## Scope
### In
- `crates/ui_components/src/lib.rs` — `ThemeColors` gains a `success: Hsla` field (the positive/OK
  semantic, alongside `danger`). Both `default_for` arms (Light + Dark) get an ORIGINAL Warp-matched
  palette: a deep near-black dark bg with a faint tint (not pure gray), a soft warm off-white light
  bg, a distinctive Marley accent (a modern teal-cyan), tuned surface/border elevation, a warm
  `danger` red, and a `success` green — all DERIVED original `hsla` values (clean-room §20: NOT
  Warp's source tokens or a copyrighted theme's palette). The metric tokens (corner_radius,
  control_height, control_padding) are UNCHANGED.
- `crates/ui_components/src/switch.rs:72` + `button.rs:186` — the `dark()` test fixtures gain the
  `success` field (compile).
- `crates/ui_components/src/lib.rs` tests — the field-equality tests updated to the new values +
  `success` (distinct from `danger`/`accent`/`background`).
- SPEC-ui-components: the palette values table + a clean-room provenance note. CHANGELOG + arch doc.

### Out (explicitly deferred)
- The Block card chrome that USES `success` (#36). The prompt/dock/typography (#37/#38/#39). Tuning
  the #31 `ANSI_RGB` 16-color table (it's already an original 16 that reads on both — leave it;
  revisit only if it clashes, a #36 render call). User-editable themes / a theme editor (M2). More
  themes beyond the two built-ins.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — ADD a `success` field to `ThemeColors` now (this is the palette ticket; #36's status indicator
  needs green + the existing red `danger`).
- D2 — ORIGINAL clean-room values (§20): match Warp's OBSERVABLE aesthetic (deep dark, warm light, a
  bright accent) by choosing our own `hsla`, never lifting source tokens. Design picks the exact
  numbers; chad verifies the LOOK (headed env-blocked).
- D3 — Metric tokens (corner_radius/control_height/control_padding) UNCHANGED — this is a COLOR swap.
- D4 — Pure DATA + the tested `default_for` accessor — fully gate-testable (cov/MSI 100 via the
  field-equality tests; the values aren't mutated — cargo-mutants doesn't mutate `hsla` literals —
  so the tests are the regression guard, the match-arm is the mutable surface). No render/shim change.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `default_for(Dark)` / `default_for(Light)` is called, it shall return the new Warp-matched palette — a non-grayscale (tinted) `background`, a distinct `accent`, and a populated `success` — with the two appearances differing. | unit tests (the exact new field values per appearance; `light != dark`) |
| REQ-002 | The `ThemeColors` `success` color shall be present and distinct from `danger`, `accent`, and `background` in both appearances. | unit tests (`success != danger`, `!= accent`, `!= background`) |
| REQ-003 | WHEN `default_for` is called twice for the same appearance, it shall return equal values (pure); the metric tokens (corner_radius/control_height/control_padding) shall be unchanged from the prior release. | unit tests (determinism + the metric tokens still `px(6)`/`px(28)`/the edges) |
| REQ-004 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `default_for`; the shell_dark/shell_light visual baselines re-shot with the new palette ride the masked deferral. | gate exit 0 + receipt + gate:15 |

## Phase Plan
- **P2 Design** — the exact dark + light `hsla` values (the aesthetic call) + the `success` value,
  the struct + 4 construction-site changes, the field-equality test updates, the SPEC clause.
- **P3 Implement** — the palette + success + the test-fixture updates + spec + CHANGELOG.
- **P3.5 Inspect** — critics: the values are non-grayscale/distinct/contrast-sane, clean-room
  (original not lifted), the success ripple hit all 4 sites, no metric-token drift.
- **P4 Validate** — the updated field-equality tests + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #35.
