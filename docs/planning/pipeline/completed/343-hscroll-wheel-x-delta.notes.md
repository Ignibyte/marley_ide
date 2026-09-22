# 343 — h-scroll wheel x-delta — notes

- pipeline_id d9a648e6-1fce-4a65-8520-616c09b395f0 · forge #343 79ef81d4-fff4-4d43-a0bb-2cd7360a83e8
- aar_id ce6bbd26-bdac-433f-8c78-ee953f9bac21 · on `98f3425`

## Phase 1 — Plan

**Intent:** #336's editor wheel handler scales the horizontal x-delta by LINE HEIGHT (`cell_h`) instead of cell
WIDTH (`cell_w`) — a real mouse's tilt-wheel scrolls ~1.5-2.4x too fast. Narrow (trackpads bypass it).

**Recon (live on `98f3425`) — confirmed:**
1. **The handler** (app.rs:4935-4946, inside `code_view_body` = `mutants::skip` + coverage-excluded):
   `let dx = f32::from(event.delta.pixel_delta(px(geom.cell_h.max(1.0))).x);` → `if dx == 0.0 { return; }` →
   `let next = view.h_scroll_clamp(view.scroll_x.get() - dx);`. The `cell_h` bug is exactly here.
2. **gpui `ScrollDelta`** (interactive.rs:395): `Pixels(Point<Pixels>)` [trackpad, exact px] |
   `Lines(Point<f32>)` [mouse, lines]. `pixel_delta(line_height)` (:418) scales BOTH axes by the one scalar
   (`point(lh*x, lh*y)`) — can't do per-axis. `.precise()` (:410) → true for Pixels. Prior-art sweep: gpui's
   ONLY ScrollDelta converters are `pixel_delta` + `coalesce` — no per-axis helper → the match is required.
3. **The fix — D2 pure seam:** `h_scroll::wheel_x_px(raw_x, precise, cell_w)` = `Pixels`→x; `Lines`→`x*cell_w`
   (floored 1.0, sane'd). The shim matches `event.delta` → `(raw_x, precise)` → calls it. app.rs is skip'd, so
   the arithmetic MUST live in the pure fn to be MSI'd.
4. **Only ONE handler.** The other on_scroll_wheel sites (:14854/:15176/:15776) are y-axis/rows
   (`pixel_delta(row_h).y / row_h`) — correct, untouched.
5. **Sign (D4):** dx is NEGATED (`scroll_x - dx`, positive-right — `AD-...-001`/411bc001). The fix changes only
   the Lines MAGNITUDE, not the sign. Re-confirmed, unchanged.

**knowledge-context (Plan):** 13 nodes logged into the AAR — incl. the #336 h-scroll AD (411bc001, the sign
convention), PR 8e692a3a (hook the shared primitive), PR 71b36786 (a width in chars ≠ cells — the col-domain
sibling of "line-height ≠ cell-width"), PR 6247b050 (#342's unique-to-one-site rule). The 71b36786 lesson is the
same shape: a magnitude in the wrong domain (height vs width) rides through unseen.

**Prior-art sweep:** (1) gpui — pixel_delta is single-scalar (the cause + the reason to match); no per-axis
helper. (2) our own #336 h_scroll (caret_px/follow_caret_x/sane) — wheel_x_px lands beside them. (3)
ropey/regex/tree-sitter — N/A (an input scalar). §20 = N/A/gpui-adoption.

**EARS:** REQ-LINES-CELL-WIDTH (`wheel_x_px(3,false,8)==24`), REQ-PIXELS-PASSTHROUGH (`(24,true,8)==24` +
cell_w-ignored), REQ-TOTALITY (hostile-input sweep, all finite), REQ-SIGN-UNCHANGED (diff review).

**Risks:** (a) tiny — the one real question is whether `Pixels` should ALSO be scaled (no: Pixels is already
exact px; scaling it would double-apply — the passthrough is correct, and REQ-PIXELS-PASSTHROUGH's cell_w-ignored
assert guards it). (b) the `.max(1.0)` floor on cell_w (mirrors the original) — keep it; a 0 cell (pre-first-
frame) would zero every Lines delta otherwise.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture.** A pure `h_scroll::wheel_x_px` (the px-domain module's home, cov/MSI 100) + a one-line match in
the editor wheel shim (`code_view_body`, `mutants::skip`). No crate boundary; §20 = N/A/gpui-adoption confirmed
(reading gpui `ScrollDelta` is adoption). D1-D4 confirmed against live code.

**The pure fn (h_scroll.rs, near the #336 fns):**
```rust
pub fn wheel_x_px(raw_x: f32, precise: bool, cell_w: f32) -> f32 {
    let x = sane(raw_x, 0.0);
    if precise { x } else { x * sane(cell_w, 0.0).max(1.0) }
}
```
Doc (plain backticks): `Pixels` (trackpad, `precise`) is already exact px → passthrough; `Lines` (mouse) is a
line count → scale by the cell WIDTH (gpui's `pixel_delta` is single-scalar, so the editor scales the x axis
itself — #336 wrongly reused the line height). `.max(1.0)` mirrors the original `cell_h.max(1.0)` (a 0 cell
pre-first-frame would zero every Lines delta); `sane` is the sibling totality idiom.

**The shim (app.rs:4937, replaces the `pixel_delta(cell_h).x` line):**
```rust
let (raw_x, precise) = match event.delta {
    gpui::ScrollDelta::Pixels(p) => (f32::from(p.x), true),
    gpui::ScrollDelta::Lines(v) => (v.x, false),
};
let dx = crate::h_scroll::wheel_x_px(raw_x, precise, geom.cell_w);
```
- `ScrollDelta` is NOT imported (only `ScrollWheelEvent` is, app.rs:17) → PATH-QUALIFY `gpui::ScrollDelta::…` in
  the match (no import change). `event.delta` is `ScrollDelta` (Copy, interactive.rs:394) → match by value is fine.
- The `if dx == 0.0 { return; }` early-return is PRESERVED and still correct: a pure-vertical wheel has x=0 →
  `wheel_x_px(0.0, _, _) == 0.0` → `dx == 0.0` → return (the list's business).
- The `let next = view.h_scroll_clamp(view.scroll_x.get() - dx);` line is BYTE-IDENTICAL (D4 sign unchanged).
- Whole handler stays inside `code_view_body` (`mutants::skip` + coverage-excluded) → NO new app.rs mutation surface.

**File manifest:**
| file | change |
|------|--------|
| `crates/marley_app/src/h_scroll.rs` | +`pub fn wheel_x_px` (the pure seam) + its 3 `#[cfg(test)]` tests. cov/MSI 100 surface. |
| `crates/marley_app/src/app.rs` | the editor wheel handler's `dx` line → the `match event.delta` + `wheel_x_px` call. Inside `code_view_body` (skip'd) → no mutation surface. |

**Mutation/coverage:** gate:5 `--diff` mutates ONLY `wheel_x_px`'s body in h_scroll.rs (the app.rs match is
skip'd). Expected mutants + killers: `x * …`→`/`/`+`/`%` (killed by `(3,false,8)==24` + `(2,false,10)==20`);
`if precise` branch delete/swap (killed by the Pixels-passthrough `(24,true,8)==24` + `(24,true,99)==24`
[cell_w-ignored] AGAINST the Lines-scale — if precise ever scaled, Pixels fails; if Lines never scaled, Lines
fails); `.max(1.0)` `<`/`>` boundary (killed by a `cell_w < 1` totality/edge case where the floor bites);
body→default (killed by any concrete value). Confirm with `cargo mutants --list` at validate.

### Regression Test Plan (headless — pure unit tests, NO live drive)
| # | REQ | test (h_scroll.rs `#[cfg(test)]`) |
|---|-----|-----------------------------------|
| T1 | REQ-LINES-CELL-WIDTH | `wheel_x_px(3.0, false, 8.0) == 24.0` and `wheel_x_px(2.0, false, 10.0) == 20.0` (pins the `*` slope; a `cell_h`≈19 would give 57). |
| T2 | REQ-PIXELS-PASSTHROUGH | `wheel_x_px(24.0, true, 8.0) == 24.0` and `wheel_x_px(24.0, true, 99.0) == 24.0` (cell_w IGNORED when precise — kills the `if precise` delete). |
| T3 | REQ-TOTALITY | a hostile sweep `{NaN, ±inf, -1e30, -1.0, 0.0, 0.5, 3.0, 1e30}` × `{true,false}` × the same set for cell_w → every result `.is_finite()`; includes cell_w < 1 (exercises `.max(1.0)`) and a Lines value to confirm no non-finite escapes. |
| T4 | REQ-SIGN-UNCHANGED | diff review — the shim's `h_scroll_clamp(scroll_x - dx)` line is byte-identical; no test. |

Uncoverable: the LIVE mouse tilt-wheel — DEFERRED + stated. A `ScrollDelta::Lines` event can't be synthesized
headlessly (no mouse tilt in the test harness), and chad is at the machine; the pure fn is the exact proof and
the shim is a byte-diff-reviewed match.

**Risks:** (a) the `if precise` mutant needs BOTH the Pixels-cell_w-ignored assert AND the Lines-scale assert to
die — planned (T1+T2). (b) `f32::from(p.x)` compiles (`p.x: Pixels`, `f32: From<Pixels>` — the original used it).
(c) match-by-value on `event.delta` — Copy, fine.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest — 2 files, exactly as designed:
1. **h_scroll.rs** — `pub fn wheel_x_px(raw_x, precise, cell_w)` added after `h_thumb` (before `mod tests`):
   `let x = sane(raw_x, 0.0); if precise { x } else { x * sane(cell_w, 0.0).max(1.0) }`. Doc uses plain
   backticks (no intra-doc `[link]`). No tests (Phase 4).
2. **app.rs** — the editor wheel handler (`code_view_body`, `mutants::skip`): the `dx` line became the
   `match event.delta { gpui::ScrollDelta::Pixels(p) => (f32::from(p.x), true), gpui::ScrollDelta::Lines(v) =>
   (v.x, false) }` + `let dx = crate::h_scroll::wheel_x_px(raw_x, precise, geom.cell_w);`. Path-qualified
   `gpui::ScrollDelta::…` (not imported). The `if dx == 0.0 { return; }` + `h_scroll_clamp(scroll_x - dx)` lines
   UNCHANGED (D4). Added a 3-line comment explaining the match. `geom.cell_h` no longer read in this handler
   (still used elsewhere — no unused warning).

**Deviation:** none. All exactly as designed.

**Verification:** `cargo clippy -p marley --all-targets -- -D warnings` rc=0 (the `block v0.1.6` line is a
pre-existing dep future-incompat). `cargo fmt --all` applied. Diff = exactly h_scroll.rs + app.rs. No Zed/Warp.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

Scaled to **1 critic** (general-purpose, correctness+semantics lens) + own review — the smallest change yet (a
3-line pure fn + a one-line→match shim swap). Critic verdict: **SHIP** (all points OK).

### Findings (verdicts)
| # | Sev | Finding | Verdict |
|---|-----|---------|---------|
| C1 | LOW | The `wheel_x_px` doc says "total via `sane`", but an absurd finite overflow (`huge × huge`) can still return `±inf`. | **REJECT — no change.** Consistent with the module's OWN documented totality (h_scroll.rs:12-16: "`sane` rejects non-finite INPUTS, it does not prevent overflow from finite ones, so `caret_px`/`content_px` can still return ±inf"). `wheel_x_px` inherits the same accepted caveat, and downstream `h_scroll_clamp`→`clamp_scroll_x`→`sane(x,0)` maps an `inf` back to 0 (never poisons `scroll_x`). Matching the sibling idiom is correct. |
| C2 | — | ★ Test-design refinement for Phase 4 (not a code finding): because `wheel_x_px` is UNCLAMPED (like `caret_px`), the T3 totality sweep must assert **`!r.is_nan()`** (no NaN-poison — the module's real "total" contract), NOT `.is_finite()` — a `1e30 × 1e30` input legitimately overflows to `inf`. | **ADOPT in Phase 4.** T3 asserts no-NaN + no-panic (allowing inf on absurd overflow); it still includes the mutation-killing inputs (raw_x=NaN→0, cell_w=NaN/inf→sane→.max(1.0)=1, cell_w<1→floored) with EXACT-value asserts on the finite cases. |

Critic confirmed **OK** (own-spot-checked the diff):
- **(a) `wheel_x_px` semantics + totality:** precise→exact px (cell_w ignored); !precise→`x * cell_w` (floored ≥1); scaling a line-count by cell_w is the right mono analog (mirrors `caret_px(col, cell_w) = col*cell_w`). `sane` kills a non-finite raw_x/cell_w; a negative cell_w (finite) is floored to 1.0 by `.max` → no sign-flip. No panic on any input.
- **(b) the match:** `Pixels(p)=>(f32::from(p.x), true)` / `Lines(v)=>(v.x, false)` — byte-for-byte the same polarity as gpui's `precise()` (interactive.rs:410); `f32::from(p.x)` extracts the px; `geom.cell_w` (width) now used, not `cell_h` — the fix's point.
- **(c) sign + early-return UNCHANGED:** `if dx == 0.0 { return; }` intact (a pure-vertical wheel → x=0 → wheel_x_px→0 → dx=0 → return; the zero-set is identical to the old code); `h_scroll_clamp(scroll_x - dx)` byte-identical (dx still NEGATED, positive-right — the fix changes only the Lines magnitude).
- **(d) one handler:** the diff is a single app.rs hunk (@:4937); the other 3 `on_scroll_wheel` sites (:14861/:15183/:15783, y-axis/rows) are untouched + correct.
- **(e) provenance:** the doc uses plain backticks (no `[link]` to private `sane`); `geom.cell_h` still read elsewhere (no dead field); no Zed/Warp; no unsafe.

**Result: 0 findings requiring a code fix.** C1 rejected (module-consistent); C2 is a Phase-4 test-assertion refinement (adopt `!is_nan()`). No forge failure-record (no bug). Lenses: semantics, totality, match-polarity, sign/early-return, caller-scope, provenance. `git status --porcelain` = the 2 src files + the #343 docs.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests — 3 added to `h_scroll.rs` `#[cfg(test)] mod tests` (the sole cov/MSI surface; the app.rs shim is
`code_view_body` skip'd + coverage-excluded):**
| test | REQ | pins |
|------|-----|------|
| `t343_wheel_x_px_lines_scale_by_cell_width` | LINES-CELL-WIDTH | `(3,false,8)==24` + `(2,false,10)==20` — the `*` slope; a `cell_h`≈19 would give 57. |
| `t343_wheel_x_px_pixels_pass_through_ignoring_cell_w` | PIXELS-PASSTHROUGH | `(24,true,8)==24` + `(24,true,99)==24` (cell_w ignored) + `(-16,true,8)==-16` (sign preserved). |
| `t343_wheel_x_px_is_total_over_hostile_input` | TOTALITY | the 9×2×8 hostile sweep asserts `!is_nan()` (C2 — inf from overflow is allowed, unclamped like `caret_px`); 4 exact rows kill the `sane`/`.max(1.0)` behavior (NaN raw→0, NaN cell→1x, 0.5 cell→floored, −5 cell→floored). |

**Runs (all `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`):**
- `cargo nextest run -p marley t343_wheel` → **3/3 PASS**.
- `cargo mutants --list … --in-diff` → **5 mutants** (3 body-replacements `0.0`/`1.0`/`-1.0` + `*`→`+` + `*`→`/`;
  no `if precise` or `.max` mutant — a bool-`if` and method calls aren't mutated here). `cargo mutants
  --in-diff … --test-tool=nextest` → **5 mutants tested in 2m: 5 caught** (MSI 100). All killed by T1's
  `(3,false,8)==24` (a `+`→11, `/`→0.375, and 0/1/−1 all ≠ 24).

**NO LIVE DRIVE — STATED (not masked).** A mouse tilt-wheel `ScrollDelta::Lines` event can't be synthesized in
the headless harness (no mouse-tilt), and chad is at the machine. The pure `wheel_x_px` is the EXACT proof (the
scalar math is the whole fix) and the shim is a byte-diff-reviewed `match` over gpui's own `ScrollDelta` variants.

**FULL `--diff` GATE → `GATE GREEN [diff]` 15/15** (gate:4 coverage ≥100 [wheel_x_px fully tested, app.rs
excluded]; gate:5 mutation MSI 100 [5/5]; gate:14 docs PASS [plain backticks]; gate:6 miri skip-clean [no
unsafe]). Receipt `5ed92d04337f03a930694df480c013905f785564`, verified worktree-bound. #334 flake did not recur.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete
