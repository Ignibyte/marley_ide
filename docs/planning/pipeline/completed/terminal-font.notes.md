# monospace terminal font — Notes

- **Forge ticket:** #34 `2d48faf5-6191-4a84-8066-0e5af1b70b67`
- **AAR:** `55039a8f-140d-4ba1-b0f3-ffa6bdfe86cc`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-034-terminal-font.md
- **Pipeline spec:** terminal-font.spec.md
- **Intake (promoted):** docs/planning/intake/terminal-monospace-font-and-cell-metric.md

## Phase 1 — Plan
- **Request:** forge #34 (M1.E "The Warp Look" seq-1, auto-approved) — the monospace font +
  accurate cell metric. The foundation for the sprint.
- **Classification / tier:** work pipeline, `feature`, mostly-SHIM (the render font-set + metric
  read are app.rs, cov-excluded) with one thin PURE surface (`fallback_cell`). marley_app only.
- **Discovery (§18):**
  - The cell-metric read (app.rs:635-646, from #30) uses `window.text_style().font()` — the AMBIENT
    proportional font — for `em_advance`. #34 redirects it to an explicit mono font.
  - gpui ships `gpui::font(family) -> Font` (text_system.rs:807) → resolve by name; `.font_family(..)`
    on a div sets the span font. So `TERMINAL_FONT="Menlo"` + `.font_family(TERMINAL_FONT)` on the
    terminal content + `resolve_font(&gpui::font(TERMINAL_FONT))` for the metric.
  - CellSize (workspace.rs) is the metric type; `plan_resize`/`grid_axis` (#30, tested) consume it —
    no change to the arithmetic, just a correct input.
  - Source: the intake `terminal-monospace-font-and-cell-metric.md` (raised by the #30 inspect #3).
- **Decisions:** D1–D4 in the spec (Menlo system mono; metric-from-font; pure `fallback_cell`; mono
  on terminal-only not chrome).
- **Open questions for Design:** where `.font_family` applies (the pane content container so
  command/output/prompt inherit, vs per-span) — leaning container; whether `fallback_cell` lives in
  workspace.rs (beside CellSize) or a new module; the exact fallback ratios (0.6 advance / 1.2
  height are typical mono — confirm against Menlo at ~14pt if cheap); whether to expose
  `TERMINAL_FONT` for #35+ reuse.
- **AAR id:** `55039a8f-140d-4ba1-b0f3-ffa6bdfe86cc`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/workspace.rs` (beside `CellSize`)
```rust
/// A monospace cell size derived from the font size when the real font metric is unavailable:
/// advance ≈ 0.6·size, line height ≈ 1.2·size (typical mono ratios). A non-positive size falls back
/// to 14.0 so the cell is always sensible/positive.
pub fn fallback_cell(font_size: f32) -> CellSize {
    let size = if font_size > 0.0 { font_size } else { 14.0 };
    CellSize { w: size * 0.6, h: size * 1.2 }
}
```

### SHIM — `crates/marley_app/src/app.rs` (mutants::skip, cov-excluded)
```rust
const TERMINAL_FONT: &str = "Menlo";       // macOS system monospace (clean-room; gpui resolves by name)
const TERMINAL_FONT_SIZE: f32 = 14.0;
...
// metric read (replaces the #30 window.text_style() read):
let fallback = fallback_cell(TERMINAL_FONT_SIZE);
let font_id = window.text_system().resolve_font(&gpui::font(TERMINAL_FONT));
let advance = window.text_system().em_advance(font_id, px(TERMINAL_FONT_SIZE));
let cell = CellSize { w: advance.map(f32::from).unwrap_or(fallback.w), h: fallback.h };
```
- The cell HEIGHT is always `fallback.h` (= `size·1.2`), and the terminal rows render with
  `.line_height(px(fallback.h))`, so the metric (cols/rows for #30/#32/#33) and the drawn rows AGREE
  exactly. The WIDTH is the real mono advance, falling back to `fallback.w` only if `em_advance` fails.
- The pane content container gets `.font_family(TERMINAL_FONT).text_size(px(TERMINAL_FONT_SIZE))
  .line_height(px(fallback.h))` — so the command/output/prompt/alt-grid children inherit the mono
  font + size + row height; the docks/palette/titlebar (separate divs) keep the UI font.

### Decisions
- D-2.1 `fallback_cell` in workspace.rs (with `CellSize`); the height it returns is ALWAYS used
  (render + metric), so it's not dead code — the width is the fallback path.
- D-2.2 `Menlo` @ 14pt as consts (macOS-only target). D-2.3 One line-height ratio (1.2) shared by
  `fallback_cell` + the render `.line_height` → metric/render consistency.
- D-2.4 mono applied to the pane content container (inherit), not the cockpit chrome.

### File manifest
- M `crates/marley_app/src/workspace.rs` — `fallback_cell` + tests.
- M `crates/marley_app/src/lib.rs` — export `fallback_cell`.
- M `crates/marley_app/src/app.rs` — `TERMINAL_FONT`/`TERMINAL_FONT_SIZE` consts, the metric read,
  the pane `.font_family/.text_size/.line_height`.
- M `docs/specs/SPEC-app-shell.spec.md` — R41 (terminal font + metric) + AC + Mutation-Targets.
  CHANGELOG; arch doc.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `fallback_cell_scales_and_guards` — `fallback_cell(14.0)` → (≈8.4, ≈16.8); `(10.0)` → (≈6.0, ≈12.0) [ratios scale — kills `*0.6`/`*1.2` → other]; `(0.0)` → (≈8.4, ≈16.8) [non-positive → 14.0 default — kills the `>0.0` guard]; `(-5.0)` → (≈8.4, ≈16.8) | unit |
| REQ-002 | mono font applied to terminal content + metric read from it | shim + masked visual (aligned columns — chad-verified) |
| REQ-003 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs font application + `em_advance` metric read (existing shim exclude; needs a
live text system + window).

### Risks / decisions
- `em_advance` line height vs the fixed `fallback.h`: using `size·1.2` for BOTH the metric height and
  the render `.line_height` guarantees they match (avoids the metric-vs-render drift the #30 inspect
  #3 flagged). If Menlo's natural line height differs, the explicit `.line_height` overrides it.
- Menlo is macOS-only; a Linux/Windows port needs a bundled/fallback family (deferred; the CI is
  macOS-only per the cross-platform-mutation note).
- float asserts use a tolerance (0.6/1.2 aren't exact in f32); a wrong-ratio mutant is still caught
  (0.7·14 = 9.8 ≫ tolerance from 8.4).

## Phase 3 — Implement
- **Built (per manifest):** workspace.rs — `fallback_cell(font_size)` (`>0` guard→14.0 default;
  `size*0.6` w / `size*1.2` h); lib.rs export; app.rs — `TERMINAL_FONT="Menlo"` +
  `TERMINAL_FONT_SIZE=14.0` consts, the metric read now resolves the mono font (`em_advance` from
  `gpui::font(TERMINAL_FONT)`, w = advance-or-`fallback.w`, h = `fallback.h`), and the pane content
  div gets `.font_family(TERMINAL_FONT).text_size(px(14)).line_height(px(fallback.h))` so the
  command/output/prompt/alt-grid inherit mono + the row height matches the metric. SPEC-app-shell
  R41 + AC row 41 + Mutation-Targets; CHANGELOG; intake promoted.
- **Deviations from design:** none. (The old hardcoded `px(8.0)` width fallback is replaced by
  `fallback.w`; the old ambient `window.line_height()` height by `fallback.h` — both now font-size-
  relative + shared with the render.)
- **Verification at this phase:** `cargo check -p marley` 0 errors; fmt; clippy `-D warnings` 0;
  docs gate 0; 90 lib tests pass. The `fallback_cell` unit suite is Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (24 probe checks + a real cargo-mutants on fallback_cell + a full build + gpui-source
  reads). Verdict: fallback_cell CORRECT, **NO equivalent/unkillable mutant** (7/7 viable killable;
  the #31/#33 trap is clear) — the primary risk cleared.
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | MED | Third row-height consumer diverged: the scroll handler still divided pixel deltas by `window.line_height()` (≈26px = phi·16) while the metric + rows moved to `fallback.h` (16.8px) — so trackpad precise scroll was ~35% too slow (16.8/26 gain). Diff-introduced (before, all three shared `window.line_height()`). Non-fatal (scroll_steps accumulates), shim-hidden. | REAL (bug I introduced) | Scroll closure now uses `px(fallback.h)` as the pixel_delta arg + divisor — the SAME row height the rows draw with. `_window` (now unused). |
  | F2 | MED (Phase-4) | fallback_cell has 7 viable mutants, all MISSED (no tests yet — expected at inspect). | REAL (Phase-4) | The critic's kill-map baked into Phase 4: the `>→<` guard mutant needs a NEGATIVE fixture (14.0 is the guard's own default → invisible there); tolerance asserts (0.6/1.2 aren't exact f32); avoid size=6 as a lone h-fixture (6·1.2==6+1.2). Set: `{14.0→(8.4,16.8), 0.0→(8.4,16.8), -5.0→(8.4,16.8)}`. |
  | F3 | LOW (non-mac) | On a non-mac host, if Menlo is absent, `resolve_font` returns a DIFFERENT font's id → `em_advance` succeeds with the wrong (proportional) advance silently (the `unwrap_or` only guards a shaping error, not font-missing); or panics if NO font resolves. Unreachable on the macOS-only CI target. | ACCEPTED (noted) | The `Menlo`-is-macOS-only risk is already in the design; a non-mac port needs a bundled fallback family (a future cut). |
- **Verified (critic):** metric/render height consistency HELD (both `fallback.h` from one binding —
  `rows = floor(rect.h/16.8)` matches the drawn 16.8px rows); font scoped to terminal-only (docks/
  palette/titlebar are separate root children — no #24/#25 regression); clean-room (`"Menlo"` string
  only, no bundled/lifted asset); no panic on the mac path.
- **Post-fix:** the F1 scroll fix compiles + clippy clean. Lesson:
  `PR-claude-derived-value-change-must-sweep-all-consumers` (moving a shared derived quantity —
  here the row height — to a new source must update EVERY consumer, not just the obvious ones; a
  third consumer in shim code silently diverged).

## Phase 4 — Validate
- **Test added (workspace.rs):** `fallback_cell_scales_and_guards` — the critic's kill-map:
  `fallback_cell(14.0)`→(8.4,16.8), `(10.0)`→(6.0,12.0) [ratios scale — kills `*0.6`/`*1.2`],
  `(0.0)`→(8.4,16.8) [14.0 default — kills `>=`/`==`], `(-5.0)`→(8.4,16.8) [NEGATIVE — kills
  `>`→`<`, which 14.0/0.0 can't]; tolerance asserts (0.6/1.2 aren't exact f32).
- **Gate saga (environmental, NOT code):** run 1 HUNG on the real-PTY test
  `workspace_two_real_sessions_are_independent` (>420s) — the #27 contention pattern from this
  session's ~10 gate runs; killed the runners + stray zsh, the test passed ISOLATED in 0.039s. Run
  2 then FAILED gate:5 with NO caught/missed summary — the `pkill -9` had orphaned 11
  cargo-mutants temp dirs → workers `File exists (os error 17)` → baseline-invalid (debug.log:
  baseline `outcome=Success`, workers EEXIST). Cleaned `$TMPDIR/cargo-mutants-Marley-*` + a
  targeted `cargo mutants --re fallback_cell` = **7 caught / 1 unviable / 0 missed** (code clean).
  Run 3 GREEN. Captured `PR-…-pkill-cargo-mutants-orphans-temp-dirs-clean-before-rerun`.
- **Runs (actual):** `cargo nextest run -p marley` → the new test PASS (94 marley); full workspace
  green (496 tests).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **7 caught /
  0 missed → MSI 100.0%**; receipt written.
- **Pre-existing:** none. (The real-PTY hang + the temp-dir collision were both environmental
  artifacts of this long session, cleaned — the code + tests are clean.)

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `app_shell.md` gains a terminal-font bullet.
  SPEC-app-shell R41 at implement. The `terminal-monospace-font-and-cell-metric` intake is PROMOTED
  (→ this ticket) — its gap is now closed.
- **Knowledge captured:** `PR-…-derived-value-change-must-sweep-all-consumers` (the F1 scroll
  divisor that diverged when the row height re-sourced) + `PR-…-pkill-cargo-mutants-orphans-temp-
  dirs-clean-before-rerun` (the gate:5 EEXIST from the killed runs). aar-submit `completed`. Win:
  binding `fallback.h` ONCE and reusing it for metric + render line-height + scroll divisor makes
  the three consistent by construction (the fix for F1's class). The gate flakiness was 100%
  environmental (this session's long run) — the code landed MSI 100 first-try on the targeted run.
- **Ticket:** forge #34 → done; local doc → closed/; pipeline pair archived. 1 of 6 in M1.E.
