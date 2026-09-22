# The editor's horizontal scrollbar thumb — Notes

- **Forge ticket:** #341 `ccdf9d4a-69cb-4168-b697-9e682eee8f18`
- **AAR:** `e6764acf-6091-425a-b395-988670972188`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-341-editor-hscroll-thumb.md
- **Pipeline spec:** 341-editor-hscroll-thumb.spec.md
- **pipeline_id:** `2e405f77-e57a-4d86-bd1e-4fbc5ddca9f5`

## Phase 1 — Plan

- **Request:** ship #336's deferred REQ-008 (the h-scroll thumb). Classification: work pipeline, `feature`,
  M22, one small slice (thumb only). FIRST of the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334`.
- **Forge recall:** bulletins empty. `knowledge-context` (Plan) to log the #336 h-scroll AD (411bc001) + the
  column/shared-primitive PRs (71b36786 "width-in-chars-is-not-cells", 8e692a3a "hook-the-shared-primitive").

### Recon (on `1f894d5`) — the confident sentences, attacked
- **`viewport::scrollbar_thumb` (viewport.rs:98) CONFIRMED axis-agnostic** — `Some((start/content,
  capacity/content))`, `None` when `content ≤ capacity`. Pure, already cov/MSI-tested (`scrollbar_thumb_geometry`
  at :268). Its doc even notes the height-fraction needs no clamp because `capacity/content < 1` — which relies
  on `start + capacity ≤ content` (the terminal's no-overscroll invariant).
- **★ THE REUSE HAS A SLACK TRAP (D2):** #336's `max_scroll_x` (h_scroll.rs:40) = `overflow + SLACK_CELLS·cell_w`
  (2 cells of overscroll). So the editor's `scroll_x` ranges `[0, content_px − viewport_px + slack]` — it EXCEEDS
  `content_px − viewport_px`. Feeding `content = content_px` to `scrollbar_thumb` would give `left_f + width_f =
  (scroll_x + viewport)/content_px > 1.0` at max scroll → the thumb overflows the track. Fix: pass `content =
  viewport_px + max_scroll_x` (the virtual extent). Then at max: `left_f + width_f = (max + viewport)/(viewport +
  max) = 1.0`. Exactly right. The px→cols caveat in the ticket is MOOT — the editor works in px, so it's a
  px→usize cast (fine for a visual affordance), not a cross-unit conversion.
- **The clamp funnel is `h_scroll_clamp` (app.rs:12804)** = `clamp_scroll_x(x, editor_content_w, geom.code_w,
  geom.cell_w)`. Single funnel; the wheel handler (:4941) routes through it. A drag (if ever) must too.
- **`EditorFrameGeom` (app.rs:684)** publishes `x0` (WINDOW-domain, rides the shift — :5281 `bounds.origin.x`),
  `code_w` (screen-domain, stable, clipper-probed — the h-scroll divisor), `cell_w`. `editor_content_w` (a Cell)
  holds `content_px`.
- **#198 render pattern (app.rs:16259-16271)** — `if let Some((top_f, h_f)) = scrollbar_thumb(...) { thumb_h =
  (h_f·track_h).max(MIN_THUMB_PX); thumb_top = (top_f·track_h).min(track_h − thumb_h); .top(px).h(px) }`. Mirror
  horizontally: top→left, h→w, track_h→code_w. `MIN_THUMB_PX` at app.rs:870.
- **★ DRAG HAS ZERO PRECEDENT → OUT OF SCOPE (D1):** app.rs:871 says #198's thumb is "display-only position
  indicator — drag-to-scroll is a deferred non-goal". So NO drag plumbing exists anywhere. The #336 deferred
  marker (app.rs:5556) gated drag-v1 on "cheap once seen"; it is not cheap (new handler + hit-test + funnel) →
  a follow-up ticket, not this slice.
- **★ THE "THIRD PROBE" MAY NOT BE NEEDED (D4, recon delta):** the deferred marker (app.rs:5556-5564) says a
  "third, body-relative probe of the code column's origin — new geometry" is required because `x0` is a window
  coord that rides the scroll. BUT `gutter_width(total)` is ALREADY computed in the SAME render scope
  (app.rs:4959, `total = ed.buffer.len_lines()`). If the code column's body-relative origin within the
  `relative()` body is `git_lane_w + gutter_w` (both known at render), the thumb's `.left(...)` origin is
  derivable inline — no new Cell/probe. Phase 2 reads the exact row-div left-composition (git lane + gutter +
  code) to confirm derive-vs-probe. Either way the pure fraction math (D2) is settled.

### Decisions (spec)
D1-DISPLAY-ONLY, D2-VIRTUAL-TRACK (`content = viewport_px + max_scroll_x`), D3-PURE-SEAM-PX (h_scroll.rs, px
domain, no px→cols), D4-ORIGIN-DERIVE-OR-PROBE (Phase-2 read) — all recorded. EARS REQ-001..005 authored.

### §20 + Prior art
§20 = N/A (Marley chrome; no Warp/Zed analog). Prior art: our own `scrollbar_thumb` + #198 render + the
`h_scroll_clamp` funnel; gpui (no scrollbar element, adoption); ropey/regex/tree-sitter not their seam.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture.** Two homes: the pure geometry in `h_scroll.rs` (the #336 px-domain module, cov/MSI 100), and
the render + the origin constant in the app.rs editor-body shim (coverage-excluded, `mutants::skip`). No new
Cell/probe. §20 = N/A (Marley chrome) confirmed.

### ★ D4 SETTLED — DERIVE (no probe), single-source gap constant
The editor row (app.rs:5350-5396) is `div().w_full().flex().flex_row().gap_2()` with children in order:
`git-lane` (`.w(px(3.0))`, app.rs:5364) → `gutter` (`gutter_label(row+1, gw)`, width `gw` CHARS) → `clipper`
(`.flex_1().min_w_0()`, the code viewport = `code_w`). `gutter_width(total) = total.to_string().len().max(2)`
is a **char count** (code_view.rs:511), so gutter px = `gw · cell_w`. With `gap_2` between the three children
(two gaps), the code column's **body-relative left origin** = `3.0 + gap + gw·cell_w + gap = 3.0 + 2·gap +
gw·cell_w`. Every term is known at the render site (`gw` at app.rs:4959, `cell.w`, the constants) → **DERIVABLE
inline, NO third probe** (contrary to the #336 deferred marker's "new geometry"). To make the derive
**correct by construction without a live pixel** (chad at the machine → no visual check), replace the row's
`.gap_2()` with `.gap(px(ROW_GAP_PX))` so the row AND the origin formula read the SAME constant — alignment is
then guaranteed, not asserted circularly. `ROW_GAP_PX = 8.0` (gpui `gap_2` = 0.5rem at the default 16px rem —
visually identical; the swap changes nothing on screen). Constants: `GIT_LANE_PX = 3.0`, `ROW_GAP_PX = 8.0`.

### File manifest
- `crates/marley_app/src/h_scroll.rs` — ADD `pub fn h_thumb(...)` (the thumb fractions, D2) + `pub fn
  code_area_left_px(gutter_chars, cell_w)` (the D4 origin) + `pub const GIT_LANE_PX`/`ROW_GAP_PX`; + their
  `#[cfg(test)]` tests. (Optionally expose `max_scroll_x` as `pub(crate)` so `h_thumb` reuses it — it's private
  today; `h_thumb` can call it directly since same module.)
- `crates/marley_app/src/app.rs` — (a) the editor row `.gap_2()` → `.gap(px(crate::h_scroll::ROW_GAP_PX))`
  (single source of truth); (b) replace the #336 REQ-008 deferred-marker comment (app.rs:5556) + add the thumb
  render (an `.absolute().bottom_0()` bar) just before `return body;`; both in the coverage-excluded shim.

### The pure fns (h_scroll.rs, cov/MSI 100)
```
pub const GIT_LANE_PX: f32 = 3.0;   // the #328 git-diff lane width
pub const ROW_GAP_PX: f32 = 8.0;    // the editor row's inter-child gap (was .gap_2())

/// The code column's body-relative left origin: git-lane + two row gaps + the gutter's char width.
pub fn code_area_left_px(gutter_chars: usize, cell_w: f32) -> f32 {
    GIT_LANE_PX + 2.0 * ROW_GAP_PX + gutter_chars as f32 * sane(cell_w, 0.0).max(0.0)
}

/// The horizontal thumb as (left_fraction, width_fraction) over the VIRTUAL scroll extent
/// `viewport_px + max_scroll_x` — `None` when the code fits (max == 0). D2: the virtual extent (not
/// content_px) is what makes `left_f + width_f == 1.0` at scroll_x == max_scroll_x, so the thumb reaches
/// the track's right edge exactly when fully scrolled (the #336 slack is already inside max_scroll_x).
pub fn h_thumb(scroll_x: f32, viewport_px: f32, content_px: f32, cell_w: f32) -> Option<(f32, f32)> {
    let view = sane(viewport_px, 0.0).max(0.0);
    let max = max_scroll_x(content_px, viewport_px, cell_w); // the existing #336 fn (finite, >= 0)
    if max <= 0.0 || view <= 0.0 {
        return None;                                         // content fits, or no viewport yet
    }
    let track = view + max;
    let sx = sane(scroll_x, 0.0).clamp(0.0, max);
    Some((sx / track, view / track))
}
```
`max_scroll_x` is already total + `sane()`-guarded; `h_thumb` inherits its totality. At `scroll_x == max`:
`sx/track + view/track = (max + view)/(view + max) = 1.0` ✓ (REQ-002). RECOMMEND this px-native fn over
`viewport::scrollbar_thumb` (D3): the module is all px f32; a usize cast splits the domain and drops sub-px.

### The render shim (app.rs, mutants::skip, coverage-excluded)
Just before `return body;` (replacing the deferred-marker comment), mirroring the #198 render (app.rs:16262):
```
let geom = self.editor_geom.get();
if let Some((left_f, width_f)) =
    crate::h_scroll::h_thumb(scroll_x, geom.code_w, self.editor_content_w.get(), geom.cell_w)
{
    let origin = crate::h_scroll::code_area_left_px(gw, cell.w);
    let thumb_w = (width_f * geom.code_w).max(MIN_THUMB_PX).min(geom.code_w.max(MIN_THUMB_PX));
    let thumb_left = origin + (left_f * geom.code_w).min((geom.code_w - thumb_w).max(0.0));
    body = body.child(
        div().absolute().bottom_0().left(px(thumb_left)).w(px(thumb_w))
             .h(px(THUMB_H)).rounded_full().bg(<scrollbar/overlay token>),
    );
}
```
`THUMB_H` ≈ 5px (subtle; drawn LAST like the sticky band, so a small height doesn't cover the last code row —
risk (c)). NO input handler (D1 display-only). The bar color reuses a muted/overlay theme token (match #198's
thumb color). `gw`, `cell`, `scroll_x` are all in scope at the render site.

### Regression Test Plan (headless — live drives OFF-LIMITS, chad at the machine)
| # | Proof | How |
|---|---|---|
| REQ-001 | thumb geometry from the virtual extent | `h_thumb` table: a mid-scroll (e.g. content 600, view 400, cell 8, scroll 100) → known `(left_f, width_f)`; assert the pair. |
| REQ-002 | right edge at track edge at max scroll | `h_thumb(max, view, content, cell)` → `left_f + width_f == 1.0` (compute max = overflow + 2·cell). |
| REQ-003 | no thumb when content fits | `h_thumb` with content ≤ view → `None`; and an exact-fit → `None` (mirrors clamp pinning 0). |
| REQ-004 | length ∝ visible/total | the width fraction row(s) in the REQ-001 table; the `MIN_THUMB_PX`/`code_w` clamps are the shim's (diff-review, cov-excluded). |
| REQ-004b | the origin derive | `code_area_left_px` table: `(gutter_chars=3, cell=8) → 3 + 16 + 24 = 43`; `(2, 10) → 3+16+20 = 39`. |
| REQ-005 | #336 paths untouched | diff review — h_scroll.rs scroll producers unchanged; the existing #336 tests stay green; the only render delta is the gap-constant swap (identical px) + the additive thumb. |
| totality | hostile input | `h_thumb` over the #336 hostile array → always `None` or a finite pair in `[0,1]`, never a panic. |

**Live pixel: DEFERRED + STATED** (chad at the machine → synthetic drives hit his window). The geometry is
fully provable headlessly, and the gap-constant single-source makes on-screen alignment correct by
construction (not a circular assert). A `render_to_image` offscreen snapshot is a nice-to-have iff the harness
supports it on gpui 0.2.2 (it did not in #259 — the test platform has no draw); if unavailable, the pure fns +
the shared-constant carry it. Re-verify the pixel when the machine is free (30s, no ticket).

### Risks / decisions
- (a) D4 resolved to DERIVE (no probe) — the only subtlety was the gap; the shared `ROW_GAP_PX` constant
  removes it.
- (b) D2 virtual-track denominator — REQ-002 is its guard.
- (c) `THUMB_H` small + drawn last (z-over the list) so it doesn't cover the last code row.
- (d) the `.gap_2()`→`.gap(px)` swap must be px-identical (8.0) or the row's spacing visibly changes — pinned
  by ROW_GAP_PX = the gap_2 value.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest — 2 files:
1. **h_scroll.rs** — `pub const GIT_LANE_PX = 3.0` / `ROW_GAP_PX = 8.0`; `pub fn code_area_left_px(gutter_chars,
   cell_w)` (the D4 origin, `sane()`-guarded); `pub fn h_thumb(scroll_x, viewport_px, content_px, cell_w) ->
   Option<(f32,f32)>` (D2 virtual-track, calls the existing private `max_scroll_x`, inherits its totality).
   Exactly as designed.
2. **app.rs** (coverage-excluded shim) — (a) the editor row's `.gap_2()` → `.gap(px(crate::h_scroll::ROW_GAP_PX))`
   (single-source-of-truth); (b) replaced the #336 REQ-008 deferred-marker comment with the shipped-thumb doc +
   the render (an `.absolute().bottom(px(2.0)).left(px(thumb_left)).w(px(thumb_w)).h(px(4.0)).rounded(px(2.0))`
   bar, mirroring #198 app.rs:16262).

**Deviation (one, forced by the borrow checker):** `colors` is MOVED into the 'static row closure (used by
`token_color`), so it is not available at the thumb-render site after the list. Fixed by capturing the color
BEFORE the closure — `let thumb_muted = colors.muted;` (`Hsla` is `Copy`) — and the thumb renders `.bg(thumb_muted)`.
Matches #198's `.bg(colors.muted)` token exactly. THUMB_H = 4.0 + `.rounded(px(2.0))` + `.bottom(px(2.0))` all
mirror #198's thumb thickness/inset (design said ~5px; 4px matches #198 precisely — a better match).

**Verification:** `cargo clippy -p marley --all-targets -- -D warnings` → clean (the `block v0.1.6` line is a
pre-existing dep future-incompat, not this change). `cargo fmt --all` applied. Diff = exactly h_scroll.rs +
app.rs. No test expansion (Phase 4). No Zed/Warp.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

Scaled to **1 critic** (general-purpose, geometry+render lens) + own review — a small change (2 pure fns + a
render shim + a px-identical gap swap; the risk is entirely the math + the origin derivation, no new subsystem).

**★ Own-verified crux #1 — the gap swap is px-identical.** `grep -rE "with_rem_size|rem_size"` across
`crates/marley_app/src` → NOTHING. The app uses gpui's DEFAULT rem (16 px), so `.gap_2()` == 0.5 rem == 8 px ==
`ROW_GAP_PX`. The swap changes nothing visually; it just makes the constant single-source so the thumb origin
and the row read the SAME 8 px. (If the app ever sets a custom rem, `ROW_GAP_PX` would need to track it — noted.)

**★ Own-verified crux #2 (the alignment invariant) — the git-lane renders UNCONDITIONALLY.** Read app.rs:5368-5380
myself. The git-mark `match` has a `None => div().w(px(3.0)).h(px(cell.h))` arm (:5379) — a non-git file STILL
renders the 3 px lane. So `GIT_LANE_PX` is present on every row and the origin is never off by `3 + 8 = 11` px for
the common (non-git) case. The row is exactly THREE flex children (git-lane :5368 · gutter `gutter_label(row+1,
gw)` :5389, no padding/min-width → width exactly `gw·cell_w` · clipper :5396) → exactly TWO `.gap(px(ROW_GAP_PX))`
before the code column. `code_area_left_px = 3 + 2·8 + gw·cell_w` is correct BY CONSTRUCTION, and `gw` is the SAME
binding (:4959) used both to render the gutter (:5389) and to compute the origin (:5576).

**★ Own-verified crux #3 — the #198 render is mirrored exactly.** #198 (app.rs:16290): `thumb_h =
(h_f*track_h).max(MIN_THUMB_PX)`, `thumb_top = (top_f*track_h).min((track_h-thumb_h).max(0))`, drawn
`.right(px(2.0)).top(thumb_top).w(px(4.0)).h(thumb_h).rounded(px(2.0)).bg(colors.muted)`. Mine is the horizontal
transpose (bottom/left/w↔h) with the SAME token, inset (2 px), thickness (4 px), radius (2 px) — and adds an
UPPER `.min(code_w.max(MIN_THUMB_PX))` cap #198 lacks, so it is strictly tighter.

### Critic findings (verdicts)
| # | Sev | Finding | Verdict |
|---|-----|---------|---------|
| C1 | LOW | (d) The 4 px thumb at `.bottom(px(2.0))` overlaps the last row's bottom 2 px (body is `py_1`=4 px). | **REJECT — no change.** Standard overlay-scrollbar behavior (VS Code does the same); it does not hide a row, and #198's thumb sits in the same padding band. Cosmetic + matches the shipped bar. |
| C2 | LOW | (c) When `code_w < MIN_THUMB_PX`(16), `thumb_w` floors to 16 and overflows the tiny code column. | **REJECT — no change.** Unreachable (a sub-16px code column shows no code) AND strictly tighter than #198, which has the SAME floor with NO upper cap. Matching/exceeding the shipped precedent is the bar. |

Critic confirmed **OK** on (a) virtual-track denominator + totality (no div-by-zero — past the guard `view>0 ⟹
track>0`; `left_f+width_f==1.0` at max scroll; every hostile input via `sane()`→ finite pair or `None`, never a
panic/NaN/out-of-range), (b) origin equals the real code-column left, (c) render clamps + arg sources (the
`h_thumb` args are IDENTICAL to the clamp's inputs at app.rs:12832 → the Some/None boundary matches the actual
scroll extent; no drag/mouse handler → display-only), (d) `colors.muted` token + Hsla-Copy + z-order (thumb added
after the list + sticky band → topmost), (e) the false "REQ-008 deferred" doc is GONE + replaced accurately, (f)
the seven #336 producers UNTOUCHED (purely additive), no `unsafe`, no Zed/Warp.

**Non-finding logged (consistency, not a bug):** the thumb reads previous-frame `geom.code_w/cell_w` while
`editor_content_w` was set this pass — the EXACT same mix `scroll_editor`'s clamp uses (app.rs:12832), so thumb
and clamp stay consistent and the one-frame transient self-corrects. Consistency-with-the-clamp is the right
invariant (a divergent geom would be the bug; there is none).

**Result: 0 findings requiring a fix.** Both critic notes rejected-as-no-change (cosmetic; match/exceed the #198
precedent). Lenses covered: geometry/totality, origin derivation, render clamps, z-order/token, regression,
provenance. No forge failure-record (no real bug). `git status --porcelain` = the 2 src files + the #341 docs only.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests — 6 added to `h_scroll.rs` `#[cfg(test)] mod tests` (the sole cov/MSI surface; app.rs is
`code_view_body` `mutants::skip` + coverage-excluded):**
| test | REQ | what it pins |
|------|-----|--------------|
| `t341_h_thumb_fractions_track_scroll_width_is_constant` | 001/004 | the `(left,width)` table over the virtual extent (÷320 = exact f32, no epsilon); width CONSTANT across scroll; a NON-ZERO scroll row (kills `/`→`*`/`%` on left — the at-rest row can't, `0*x==0/x`); NaN scroll_x → 0 |
| `t341_h_thumb_reaches_the_track_edge_at_max_scroll` | 002 | `left_f + width_f == 1.0` exactly at `scroll_x == max` — the D2 virtual-extent guarantee |
| `t341_h_thumb_is_none_when_the_code_fits` | 003 | narrower + EXACT-fit → None (two-sided, kills `over<=0`→`>` + `-`→`+`/`/`); +content-inf→None; 1px over → Some |
| `t341_h_thumb_is_none_before_the_first_frame` | 003 | viewport 0 (real frame-1 state) + negative + NaN viewport → None (kills `view<=0`→`>` + `\|\|`→`&&`) |
| `t341_code_area_left_px_composes_the_row` | origin | 3 points (0,10)=19 · (3,10)=49 · (5,8)=59 pin offset+slope+multiply; non-finite/negative cell_w floors gutter term to 0 |
| `t341_h_thumb_is_total_over_hostile_input` | totality | the #336 4-nested hostile sweep — no panic; every `Some` pair finite AND ∈ [0,1] |

**★ Implement-deviation logged at validate (a mutation-robustness refinement, behavior-identical):** the guard was
`max <= 0.0 || view <= 0.0` over `max_scroll_x`'s FLOORED (non-negative) output. `cargo mutants --list` (traced,
not guessed — #199/#204) showed the only `<=` mutant is `<=`→`>` (this version does not emit an `== 0` twin), so
the floored form would in fact have passed; but a `<= 0` over a value that never reaches the far side is a latent
equivalent-mutant hazard. Restructured to guard on the UNFLOORED `view = sane(viewport_px, 0.0)` and `over =
sane(content_px, 0.0) − view`, both of which DO reach below 0 (a negative viewport; content narrower than the
viewport) → every boundary mutant is provably killable. Same `None`-condition and same fractions on every real
input; `track = view + max > 0` past the guard (no div-by-zero). Also fixed the pre-first-frame semantics
precisely: `code_w` is 0 until the canvas measures while `editor_content_w` is set synchronously the same pass, so
`{view==0, content>0}` is a REAL frame-1 state — `view <= 0` hides the thumb then (else a 16px glitch at origin).

**Runs (all `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`):**
- `cargo nextest run -p marley t341` → **6/6 PASS** (0.015s). Full crate green under gate:3 (158 visual + the marley
  suite, 0 failed).
- `cargo mutants --in-diff <h_scroll.diff> -f h_scroll.rs --test-tool=nextest` → **32 mutants tested in 5m: 32
  caught** (0 missed). The 32 = 11 on `code_area_left_px` (3 body-consts + 8 arith) + 10 `h_thumb` body-replacements
  + 11 `h_thumb` operators (`-`→`+`/`/`, `\|\|`→`&&`, two `<=`→`>`, `+`→`-`/`*` on track, `/`→`%`/`*` ×2). All killed.
- Gate:14 docs: fixed two intra-doc-link traps BEFORE the gate (the #348 class) — a `pub fn` doc cannot `[link]` a
  private item (`max_scroll_x`) or a foreign-module private (`EditorFrameGeom::code_w`); both to plain backticks.

**LIVE PIXEL — DEFERRED + STATED (not masked).** chad is at the machine; a synthetic drive hits his frontmost
window, so no live capture was taken. The AC is a headless geometry assert and is fully met: the pure-fn table +
the D2 sum==1 + the origin's 3 points prove the geometry, and alignment is correct-BY-CONSTRUCTION (the render and
`code_area_left_px` read the SAME `ROW_GAP_PX`; the git-lane renders unconditionally so the origin never shifts).
Re-eyeball the live thumb when the machine is free (~30s, no ticket) — nothing rides on it that the headless proof
does not already establish.

**FULL `--diff` GATE → `GATE GREEN [diff]` 15/15** (gate:4 coverage ≥100, gate:5 mutation MSI 100, gate:6 miri
skip-clean [no unsafe], gate:13 SAST clean, gate:14 docs PASS, gate:15 visual PASS). Receipt
`de1e820425de27efdb210a8a549306fc2b325e01`, verified worktree-bound (== `gate_state_hash`). #334 coverage flake
did not recur.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete
