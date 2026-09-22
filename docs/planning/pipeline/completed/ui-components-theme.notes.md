---
pipeline_id: 56adbd8a-2071-439c-b265-7dde69ec9608
aar_id: 585c0d67-ebf5-4e72-863c-33b3028c811c
---

# marley_ui_components (M1.A theme cut) — pipeline notes

## Phase 1 — Plan (2026-06-29)

**Intent:** the M1.A cut of SPEC-ui-components — the theming value vocabulary (`Appearance` +
`ThemeColors` + `default_for`) that #14/#15/#16 render from. Sprint M1.A seq 2/5.

**The scope call (key planning decision):** SPEC-ui-components defines marley_ui_components as the theme
vocabulary + 5 widgets (button/switch/dialog/tooltip/keyboard-shortcut). Read of the spec + the M1.A
needs: the widgets serve the command-palette/settings UI (M1.B); the terminal/editor/shell render
text/containers/lists from `&ThemeColors` directly and use NO widget. So **#13 ships the theme
vocabulary; the 5 widgets defer to a new M1.B ticket** — leanest path to the usable terminal. The
widgets' spec (R1–R16, the pure decision accessors + render shims + per-widget baselines) is fully
written + ready to /work when M1.B's palette/settings land.

**Consequence:** #13 is a PURE value-type library (no Render/headed surface) → gate-15 N/A, no
ACCEPTED-UNTESTABLE shim, cov 100 / MSI 100 on the WHOLE crate. Small + foundational + fast.

## Carry to Design (Phase 2)
1. Module layout — likely a single `lib.rs` (small): `Appearance` enum + `ThemeColors` struct (7 Hsla +
   3 metrics, all `pub`) + `impl ThemeColors { pub fn default_for(Appearance) -> ThemeColors }`.
2. The Light/Dark fallback palettes — pick reasonable terminal-friendly Hsla values (a dark default that
   reads well for a terminal; a light default). They must DIFFER (R17). Document the chosen roles.
3. Decide `ThemeColors`/`Appearance` derives — `Debug`, `Clone`, `Copy`(Appearance), `PartialEq` (for the
   AC-1 `!=` test). NOT `Default` on ThemeColors (so the cargo-mutants `-> Default::default()` mutant on
   default_for is UNVIABLE — the marley_spike/harness lesson; confirm) — unless a Default is genuinely
   wanted, in which case it becomes a viable mutant the test must kill.
4. Mutation map — `default_for`'s `match appearance { Light => …, Dark => … }` is the key: the AC-1 test
   (`default_for(Light) != default_for(Dark)` + each field check) kills the branch-swap. Watch the
   `Default::default()` return mutant (unviable iff no Default). gpui Hsla literals aren't mutated.
5. Confirm `gpui` provides `Hsla`/`Pixels`/`Edges` as value types usable in a pure context (no GPU). Deny:
   gpui tree already allowlisted; no gpui-component (deferred).

**Phase 1 status:** PASS (autonomous-through-commit per chad's /goal). → Phase 2 Design.

## Phase 2 — Design (2026-06-29)

### gpui types confirmed (pure value types, no GPU)
`Hsla { h, s, l, a: f32 }` + `hsla(h,s,l,a) -> Hsla`; `Pixels` (+ `px()`); `Edges<T: Clone + Debug +
Default + PartialEq> { top, right, bottom, left }`. So `Edges<Pixels>` works (Pixels satisfies the
bounds). Hsla/Pixels derive PartialEq → `ThemeColors` can derive `PartialEq` (needed for AC-1 `!=`).

### Architecture — one small pure module
`crates/ui_components/src/lib.rs` (package `marley_ui_components`):
- `pub enum Appearance { Light, Dark }` — derive `Debug, Clone, Copy, PartialEq, Eq`.
- `pub struct ThemeColors { background, foreground, accent, on_accent, surface, border, danger:
  gpui::Hsla; corner_radius, control_height: gpui::Pixels; control_padding: gpui::Edges<gpui::Pixels> }`
  — derive `Debug, Clone, PartialEq`. **NO `Default`** (so cargo-mutants' `-> Default::default()` return
  mutant on `default_for` is UNVIABLE — the spike/harness lesson).
- `impl ThemeColors { pub fn default_for(appearance: Appearance) -> ThemeColors { match appearance {
  Light => <light bundle>, Dark => <dark bundle> } } }`.
- Light/Dark palettes (implement picks exact Hsla; structure): DARK = dark bg `hsla(0.,0.,0.12,1.)` /
  light fg `~0.92` / a blue accent / surface `~0.18` / border `~0.30` / red danger; LIGHT = light bg
  `~0.98` / dark fg `~0.10` / accent / surface `~0.94` / border `~0.80` / danger. Metrics: corner_radius
  `px(6.)`, control_height `px(28.)`, control_padding `Edges{ top px(6.), right px(12.), bottom px(6.),
  left px(12.) }`. **Light ≠ Dark field-wise** (background 0.98 vs 0.12 — load-bearing for R17).
- §14: pure, no IO/panic/global. `#![deny(missing_docs)]`.

### File manifest
- `crates/ui_components/Cargo.toml` — `[package] name = marley_ui_components`, edition/license workspace;
  `[dependencies] gpui`; `[dev-dependencies] mutants`. NO gpui-component, NO marley_visual_harness (no
  headed surface). NO `[[bin]]`.
- `crates/ui_components/src/lib.rs` — the enum + struct + `default_for`.
- NO scripts/gates.sh change (no shim/exclude). §21 docs at complete.

### Regression Test Plan (one row per AC)
| AC | Test (in-crate `#[cfg(test)]`) | Proves |
|---|---|---|
| AC-1 (R17) | `default_for_light_differs_from_dark` — `assert_ne!(default_for(Light), default_for(Dark))` + `assert_eq!(default_for(Light).background, hsla(0.,0.,0.98,1.))` + `assert_eq!(default_for(Dark).background, hsla(0.,0.,0.12,1.))` (each arm yields its palette; covers both match arms) | R17, both arms |
| AC-3 | `default_for_is_pure` — `assert_eq!(default_for(Dark), default_for(Dark))` (same input → same output) | R1 foundation |
| AC-2 | `theme_colors_carries_all_roles` — construct a ThemeColors literal + read each of the 10 fields (the value-type contract downstream binds to) | the bundle |
| AC-4 | FULL gate green; **cov 100 whole-crate** (gate-15 N/A — no UI surface) | gate |

### Mutation map / risk
`default_for` is the only logic. cargo-mutants surface: `-> Default::default()` return (UNVIABLE — no
Default on ThemeColors); match-arm DELETION (deletes an arm → non-exhaustive match → won't compile →
UNVIABLE). So `default_for` likely has **0 viable mutants** (vacuous MSI 100, like marley_spike's
block.rs/config.rs) — the bar here is **cov 100** (the AC-1 test exercises both arms + the fields).
Hsla/Pixels float literals are not mutated by cargo-mutants. Confirm at validate that the Default mutant
reports UNVIABLE, not MISSED.

**Risks/decisions:** (1) the palette Hsla values are aesthetic, not load-bearing for the gate — only
their Light≠Dark difference (R17) is asserted; implement picks readable values, app_shell (#16) supplies
the real M1 palette anyway. (2) `Edges<Pixels>` needs `Pixels: Default` (it does) for the Edges bound.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-06-29)

Built `crates/ui_components` (package `marley_ui_components`): `Cargo.toml` (gpui dep, mutants dev-dep)
+ `src/lib.rs` — `Appearance{Light,Dark}` (Debug/Clone/Copy/PartialEq/Eq), `ThemeColors` (7 Hsla + 3
metrics, Debug/Clone/PartialEq, NO Default), `ThemeColors::default_for(Appearance)` (Light/Dark
palettes; Light.background lightness 0.98 vs Dark 0.12). `#![deny(missing_docs)]`, every item doc'd.
check + clippy(-D warnings) + fmt all clean. **No deviation** — `gpui::hsla(h,s,l,a)`, `gpui::px(f)`,
and the `gpui::Edges{ top, right, bottom, left }` struct literal all compiled as designed. Pure
value-type library: no Render/headed surface, no mutants::skip, no rust_cov exclude.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — 1 critic — NO FINDINGS

Lenses covered (all PASS): **R17 correctness** — Light vs Dark differ field-wise (background 0.98/0.12,
foreground 0.10/0.92, surface 0.94/0.18, border 0.80/0.30, accent+danger differ; on_accent + the 3
metrics identical in both = intentional); Light is the light bundle, Dark the dark, not swapped, no
copy-paste error. **Mutation** — `cargo mutants --list` emits exactly ONE mutant (`default_for ->
Default::default()`), UNVIABLE (no Default on ThemeColors → won't compile); no match-arm-deletion mutant
generated → **0 viable mutants → MSI 100 vacuous**; the operative bar is **cov 100**. **§14** — pure, no
panic/unwrap/index/IO, exhaustive match, `Appearance: Copy`. **Coverage reachability** — the only
executable lines are in `default_for`; AC-1 (`assert_ne!(default_for(Light), default_for(Dark))` + per-arm
background asserts) hits BOTH arms; AC-2 (read all 10 fields) covers the field contract → cov 100
reachable. **Clean-room §20** — all identifiers Marley-original; `grep warp` empty; only gpui (allowlisted)
+ mutants dev-dep. No source edits; no forge failure (nothing real). Validate: confirm the lone mutant
reports UNVIABLE (not MISSED) in the run log.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate (2026-06-29)

**Tests** (in-crate `#[cfg(test)]`): `default_for_light_differs_from_dark` (AC-1/R17 — both arms +
background/foreground per-arm asserts), `default_for_is_pure` (AC-3), `theme_colors_carries_all_roles_
and_metrics` (AC-2 — all 7 roles + 3 metrics read), `derives_are_exercised` (Clone/Debug/Copy/PartialEq
exercised for coverage). `cargo nextest -p marley_ui_components` = **4 passed**; `cargo llvm-cov` =
**100%** (90/90 regions, 5/5 functions, 74/74 lines). fmt + clippy clean.

**FULL gate (009):** `GATE GREEN [diff]` (21:30:33) — **15 passed, 0 failed**. Coverage **100%**;
mutation `--in-diff` = **0 viable mutants — pass** (the lone `default_for -> Default::default()` reports
UNVIABLE, as inspect predicted; MSI 100 vacuous). gate-15 N/A (no visual_acceptance). Commit receipt
written (41 b).

**Phase 4 status:** PASS.
