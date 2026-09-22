---
pipeline_id: 14933c7e-335e-40cb-abff-f44bfaa41c03
ticket: forge#361 (df3f5b6d-ada1-4c8f-ba04-4e9f66953370) · local docs/planning/tickets/open/TICKET-361-font-family-monospace-warn.md
aar_id: 8feaaf3b-a4dd-4053-87bc-65cc0f5c40a7
status: Phase 5 — Complete PASS
title: appearance.font_family — warn when a resolvable font is NOT monospace (fold monospace into #344's resolve_font_family + a gpui advance probe)
type: feature
milestone: M22
references: [settings.rs:260 resolve_font_family (the pure policy — fold monospace: bool) / :248 FontResolution{applied,flash} / :1105-1123 t344 units, app.rs:2144 font_resolves (the app probe to mirror) / :1902-1905 the boot wiring / :555 TERMINAL_FONT="Menlo" / :4973,:15633 window.text_system().resolve_font+em_advance (the idiom), gpui-0.2.2 text_system.rs:197 advance(font_id,size,ch)->Result<Size<Pixels>> / :150 resolve_font(&Font)->FontId (PANICS on missing) / :807 font(family), #344 #337]
---

## Title
Extend #344's font policy to catch a resolvable-but-PROPORTIONAL family. Fold `monospace: bool` into
`resolve_font_family` (resolves && !monospace → fall back to the built-in mono + a "not monospace" flash), decided
by a pure `is_monospace_advance(narrow, wide, epsilon)` fed by an app-side `font_is_monospace` probe that compares
gpui's `advance('i')` to `advance('m')`.

## Scope
### In
- **settings.rs (pure, cov/MSI 100):** add a `monospace: bool` param to `resolve_font_family` — a new arm
  `resolves && !monospace → { applied: "", flash: Some("Font \"<requested>\" is not monospace — using <builtin>") }`
  (mirroring the not-found arm); + a pure `is_monospace_advance(narrow_w: f32, wide_w: f32, epsilon: f32) -> bool`
  (`wide_w <= 0.0 → true` conservative; else `(wide_w - narrow_w).abs() / wide_w <= epsilon`). Update the 3 t344
  units for the new signature + add the not-monospace + the is_monospace_advance units.
- **app.rs (app-side probe, cov-excluded):** `font_is_monospace(cx, family) -> bool` — `resolve_font(&font(family))`
  → `advance(id, SIZE, 'i')` + `advance(id, SIZE, 'm')` → `is_monospace_advance(...)`; a `Result::Err` (no glyph)
  → conservative `true` (don't false-warn). Wire into the boot: `let monospace = !resolves || Self::font_is_monospace(cx, &family);`
  (short-circuits when !resolves → `resolve_font` never called on a missing font → no panic) then
  `resolve_font_family(&family, resolves, monospace, TERMINAL_FONT)`.
- (Phase 3.5 correction) NO headless drive — the resolvable-font path is unreachable under `#[gpui::test]`'s
  `NoopTextSystem` (`all_font_names()` resolves nothing → `!resolves` short-circuits before `font_is_monospace`;
  the #344 garbage drive documents this). The DECISION is proven by pure `settings.rs` units; the app-side metric
  READ is cov-excluded + headed-only, mirroring #344's `font_resolves`.

### Out (explicitly deferred)
- Any change to the #344 doesn't-resolve path (the not-found arm + `font_resolves` are UNCHANGED — monospace is
  moot when a font doesn't resolve).
- A configurable epsilon / a live re-check when the setting changes at runtime (boot-time only, as #344).
- CJK/double-width nuance — `i`/`m` are single-width ASCII in every font; the ratio is a documented heuristic.

## Reference (§20)
N/A — Marley's own settings/font policy. No Warp/Zed source read.

### Prior art
- **Our permissive deps (the highest-yield leg — CONFIRMED the plan's central risk):** gpui-0.2.2 owns the metric
  seam — `TextSystem::advance(font_id, font_size, ch) -> Result<Size<Pixels>>` (text_system.rs:197) is PUBLIC and
  PER-CHARACTER (internally `glyph_for_char` + `platform.advance` / `units_per_em`), so the `i`-vs-`m` comparison
  is feasible; `resolve_font(&Font) -> FontId` (:150) is public but PANICS on a missing font → gate on
  `font_resolves` first. The app already uses `window.text_system().resolve_font(&mono_font).em_advance(...)`
  (app.rs:4973/:15633) — the exact idiom. No hand-rolled font metrics needed.
- **Our own code:** #344's `resolve_font_family` (settings.rs:260) is the pure-policy shape to extend; `font_resolves`
  (app.rs:2144) is the app-probe shape to mirror. The app-vs-pure cov split + the assertion-on-a-non-default
  lessons (#337/#344) apply.
- **Behavior maps / published:** N/A — no reference-app behavior; "compare i vs m advance" is the standard
  is-this-monospace heuristic.

## Locked-In Decisions
- **D1 — fold `monospace: bool` into `resolve_font_family`** (a 4th arm), NOT a separate policy — one pure fn, one
  `FontResolution` per case: `empty → built-in`; `!resolves → fallback+not-found`; `resolves && !monospace →
  fallback+not-monospace`; `resolves && monospace → apply`. Order matters (not-found before not-monospace).
- **D2 — the app computes `monospace = !resolves || font_is_monospace(...)`** — the `!resolves ||` short-circuit
  means `font_is_monospace` (and thus `resolve_font`, which panics on a missing font) is NEVER called when the
  font doesn't resolve. Passing `monospace = true` when `!resolves` is safe (the not-found arm fires first).
- **D3 — the epsilon is a size-independent RATIO, a documented tolerant heuristic.** `is_monospace_advance` uses
  `(wide - narrow).abs() / wide <= epsilon`; both advances scale with `font_size` so the ratio cancels it.
  Recommend `epsilon = 0.05` (5%): a true monospace has `advance('i') == advance('m')` (0% diff), a proportional
  differs 40–70%, so 5% cleanly separates them with headroom for a near-mono font. `wide <= 0.0` → conservative
  `true`.
- **D4 — `i` (narrowest ASCII) vs `m` (widest ASCII)** — the maximal spread, the clearest signal. An `advance` Err
  on either → the probe returns `true` (assume mono; never false-warn on a measurement failure).
- **D5 — cov/MSI: the DECISION is pure** (`resolve_font_family` extended + `is_monospace_advance`, cov/MSI 100 via
  units); the METRIC READ (`font_is_monospace`'s `resolve_font`/`advance` calls) is app-side (cov-excluded,
  integration-only, like `font_resolves`). The headless drive carries the wiring.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-PROPORTIONAL-WARNS | WHEN a resolvable but PROPORTIONAL `appearance.font_family` is set, boot shall apply the built-in mono AND `status_flash` a "<family> … not monospace" warning. | pure unit: `resolve_font_family("Arial", true, false, "Menlo")` → `applied ""` (→ built-in) + `flash Some("… is not monospace …")`. (End-to-end is headed-only — `#[gpui::test]`'s NoopTextSystem cannot resolve a real font; see notes P3.5.) |
| REQ-MONO-SILENT | WHEN a resolvable MONOSPACE font_family is set, the system shall apply it with no warning. | pure unit: `resolve_font_family("Fira Code", true, true, "Menlo")` → `applied == "Fira Code"`, `flash None`; + `is_monospace_advance(eq, eq, 0.05) == true`. |
| REQ-344-UNCHANGED | WHEN the font_family does NOT resolve, the system shall behave exactly as #344 (fall back + not-found flash), regardless of monospace. | pure unit: `resolve_font_family("Nope", false, true, "Menlo")` → `applied ""`, `flash Some(not-found)` — the not-found arm fires first. |
| REQ-EPSILON-DECISION | The monospace decision shall be a size-independent ratio within a tolerant epsilon. | pure units: equal advances → true; a 40% spread → false; a 3% spread (near-mono) → true; `wide==0` → true. |

## Phase Plan
- **P2 Design** — ratify D1-D5: the exact `resolve_font_family` 4-arm body + the `is_monospace_advance` body + the
  `font_is_monospace` app probe (resolve_font→advance i,m→decision; the Err/short-circuit safety) + the fixed
  probe size + the test lane (pure units + the Helvetica-boot headless drive; confirm the flash accessor +
  `applied.font_family` accessor from #344's drive).
- **P3 Implement** — settings.rs (the policy + is_monospace_advance) + app.rs (font_is_monospace + the boot wire).
- **P3.5 Inspect** — ★ resolve_font gated on font_resolves (no panic on a missing font); the ratio size-independent;
  the epsilon tolerant (no false-warn); the Err conservative; the not-found arm precedes not-monospace; the pure
  decision cov/MSI 100; #344 path byte-unchanged.
- **P4 Validate** — the pure units + the Helvetica-boot headless drive (a font present on the mac) + the `--diff`
  gate. No live drive.
- **P5 Complete** — CHANGELOG + the FontFamily settings doc + arch.
