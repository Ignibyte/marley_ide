---
pipeline_id: 2e405f77-e57a-4d86-bd1e-4fbc5ddca9f5
ticket: forge#341 (ccdf9d4a-69cb-4168-b697-9e682eee8f18) · local docs/planning/tickets/open/TICKET-341-editor-hscroll-thumb.md
aar_id: e6764acf-6091-425a-b395-988670972188
status: Phase 5 — Complete PASS
title: The editor's horizontal scrollbar thumb (deferred from #336 REQ-008)
type: feature
milestone: M22
references: [h_scroll.rs (the #336 px-domain scroll math), viewport.rs:98 scrollbar_thumb (axis-agnostic ratio), app.rs:16262 (#198 terminal scrollbar render — the pattern), app.rs:5556 (the #336 REQ-008 deferred marker), app.rs:12804 h_scroll_clamp (the single clamp funnel), AD-claude-hscroll-shift-clip-split-and-two-probe-domains-001]
---

## Title
Draw the editor's horizontal scrollbar **thumb** — the discoverability affordance #336 deferred (REQ-008).
A bottom-edge bar over the code column whose left + width come from the pure scroll fractions, shown only
when the code overflows its viewport. Display-only: it makes the h-scroll *visible*; the wheel + caret-follow
already make it *reachable* (#336 REQ-001).

## Scope
### In
- A horizontal thumb rendered at the bottom of the editor code area, positioned + sized from a pure fraction
  fn, hidden when the content fits (`max_scroll_x == 0`). `MIN_THUMB_PX` floor as #198.
- The pure geometry seam in `h_scroll.rs` (cov/MSI 100): the thumb's `(left_fraction, width_fraction)`
  accounting for #336's overscroll **slack** (the reason a naive `scrollbar_thumb(content_px, …)` would
  overflow the track at max scroll).

### Out (explicitly deferred)
- **Drag-to-scroll** — a NEW follow-up ticket. There is ZERO drag precedent in the codebase: #198's terminal
  scrollbar explicitly deferred drag ("drag-to-scroll is a deferred non-goal", app.rs:871), so a drag handler +
  track hit-test + track-fraction→px→`h_scroll_clamp` is new plumbing, not the "cheap" add the #336 spec's
  "draggable v1 only if that cost is small once seen" condition allowed. Seen; not cheap → deferred.
- Any change to the wheel / caret-follow / click paths (#336 REQ-001..007 ship green, untouched).

## Reference (§20)
N/A — a code-editor horizontal scrollbar is Marley-specific chrome; there is no Warp/Zed BEHAVIOR to match
(Warp is a terminal; the Zed editor source is GPL and off-limits). The *feel* (a thumb that changes length as
you scroll vertically, VS Code-style) is already encoded by #336's `content_px` (visible-rows-only). Marley
draws its own bar (gpui 0.2.2 ships no scrollbar element — verified in #336).

### Prior art
1. **OUR OWN CODE (highest yield):** `viewport::scrollbar_thumb(content, capacity, start) -> Option<(f32,f32)>`
   (viewport.rs:98) is the axis-agnostic ratio — a horizontal thumb reads its pair as (left_fraction,
   width_fraction). BUT it assumes **no overscroll** (`start + capacity ≤ content`, true for the terminal's
   ROW scrolling); the editor's #336 slack breaks that, so the reuse passes `content = viewport_px +
   max_scroll_x` (the virtual track), not `content_px`. The #198 render (app.rs:16262) is the placement
   pattern (top/height → left/width). The single clamp funnel is `h_scroll_clamp` (app.rs:12804).
2. **gpui (Apache-2.0, adoption):** confirmed in #336 that gpui 0.2.2 has no scrollbar element; an
   `.absolute()` child of a `relative()` parent is parent-relative — the crux of the placement problem.
3. Checked ropey/regex/tree-sitter — not their seam.

## Locked-In Decisions
- **D1-DISPLAY-ONLY** — ship the thumb as a position/size indicator; drag is a separate follow-up (rationale
  above). This matches #198's terminal scrollbar exactly.
- **D2-VIRTUAL-TRACK** — the thumb's fraction denominator is the virtual scroll extent `viewport_px +
  max_scroll_x`, NOT `content_px`. This makes `left_fraction + width_fraction == 1.0` at `scroll_x ==
  max_scroll_x` (the slack is already inside `max_scroll_x`), so the thumb reaches the track's right edge
  exactly when fully scrolled. A naive `content_px` denominator would let the thumb overflow past the track.
- **D3-PURE-SEAM-PX** — the geometry is a pure fn in `h_scroll.rs` in the module's own **px f32** domain (no
  px→cols conversion — everything is already px; the width-in-chars-is-not-cells trap does not apply). It may
  compose `viewport::scrollbar_thumb` internally (px cast to usize is lossless-enough for a visual affordance)
  or inline the ratio; Design picks, both cov/MSI 100.
- **D4-ORIGIN-DERIVE-OR-PROBE** — *(recon delta vs the ticket)* the thumb's body-relative left origin (the code
  column's start within the `relative()` body) is a Phase-2 read: `gutter_width(total)` is ALREADY computed in
  the same render scope (app.rs:4959), so if the code origin is `git_lane_w + gutter_w` (constants/known at
  render), it is derivable inline and NO new probe/Cell is needed — contrary to the deferred marker's "a third
  body-relative probe … new geometry". Design reads the row-div left-composition to confirm derive-vs-probe.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the code overflows its viewport, the system shall render a horizontal thumb whose left + width fractions come from the pure geometry fn over the virtual extent `viewport_px + max_scroll_x`. | headless: the pure fn's `(left_f, width_f)` for a known scroll state; the shim maps `left = origin + left_f·code_w`, `w = (width_f·code_w).max(MIN_THUMB_PX)`. |
| REQ-002 | WHEN `scroll_x == max_scroll_x`, the system shall place the thumb's right edge at the track's right edge (`left_f + width_f == 1.0`). | headless pure-fn assert at max scroll. |
| REQ-003 | WHEN the code fits the viewport (`max_scroll_x == 0`), the system shall render NO thumb. | headless: the pure fn returns `None`; the shim renders nothing. |
| REQ-004 | The system shall keep the thumb's length proportional to `viewport_px / (viewport_px + max_scroll_x)`, floored at `MIN_THUMB_PX`. | headless pure-fn table (width fraction) + the shim's `.max(MIN_THUMB_PX)`. |
| REQ-005 | The system shall leave the wheel, caret-follow, and click h-scroll paths (#336) byte-identical. | diff review — h_scroll.rs's scroll producers unchanged; only an additive thumb fn + the render. |

## Phase Plan
- **P2 Design** — the exact pure fn signature + the D4 origin read (derive vs probe); the render block mirroring
  app.rs:16262 horizontally; the headless test plan; confirm no drag input is added.
- **P3 Implement** — the h_scroll.rs thumb fn + the app.rs render shim (mutants::skip). `cargo check`.
- **P3.5 Inspect** — critic on the geometry (the virtual-track denominator; the origin derivation) + the render.
- **P4 Validate** — the pure-fn tests (cov/MSI 100 in h_scroll.rs); a headless geometry assert; the `--diff`
  gate. **Live pixel deferred (chad at the machine — no synthetic drives); stated, not masked.**
- **P5 Complete** — CHANGELOG + editor.md; AAR; file the drag follow-up; close + archive.
