---
pipeline_id: d9a648e6-1fce-4a65-8520-616c09b395f0
ticket: forge#343 (79ef81d4-fff4-4d43-a0bb-2cd7360a83e8) · local docs/planning/tickets/open/TICKET-343-hscroll-wheel-x-delta.md
aar_id: ce6bbd26-bdac-433f-8c78-ee953f9bac21
status: Phase 5 — Complete PASS
title: The editor h-scroll wheel scales its x delta by cell WIDTH, not line height
type: bug
milestone: M22
references: [app.rs:4935-4946 (the editor wheel handler, inside code_view_body — mutants::skip), gpui interactive.rs:395 ScrollDelta + :418 pixel_delta + :410 precise, h_scroll.rs (the #336 pure seam — caret_px/follow_caret_x/sane), AD-claude-hscroll-shift-clip-split-and-two-probe-domains-001 (411bc001 — the #336 sign convention)]
---

## Title
Fix the editor's horizontal scroll wheel: a `ScrollDelta::Lines` (mouse) x-delta must scale by the cell
**WIDTH**, not the line **HEIGHT**. Today it uses `cell_h` (~19px) where it should use `cell_w` (~8px), so a
real mouse's horizontal tilt-wheel scrolls ~1.5-2.4x too fast (font-dependent).

## Scope
### In
- A pure `h_scroll::wheel_x_px(raw_x, precise, cell_w) -> f32` (cov/MSI 100) that computes the px x-delta:
  `Pixels` (precise) → the exact px unchanged; `Lines` (mouse) → `raw_x * cell_w` (floored at 1.0, `sane`d).
- The editor wheel handler (app.rs:4937) matches `event.delta` → `(raw_x, precise)` and calls `wheel_x_px`,
  replacing the buggy `pixel_delta(cell_h).x`.

### Out
- The vertical / rows wheel handlers (app.rs:14854, 15176, 15776) — they scale `.y` by `row_h` then `/row_h`,
  which is CORRECT (rows), untouched.
- The sign convention (dx negated; positive-right) — verified by #336, re-confirmed, NOT changed.
- Any change to the clamp / follow / thumb (#336/#341 ship green).

## Reference (§20)
N/A — Marley-specific editor h-scroll feel; no Warp (a terminal) or Zed (GPL, off-limits) BEHAVIOR to match.
Reading gpui's `ScrollDelta` to learn Pixels-vs-Lines is ADOPTION (outside the wall).

### Prior art
1. **gpui (Apache-2.0, adoption) — the whole reason for the fix.** `ScrollDelta` (interactive.rs:395) is
   `Pixels(Point<Pixels>)` [trackpad, exact px] | `Lines(Point<f32>)` [mouse, inexact lines]. Its ONLY
   converter, `pixel_delta(line_height)` (interactive.rs:418), applies ONE scalar to BOTH axes
   (`point(line_height*x, line_height*y)`) — it CANNOT express per-axis scaling, which is exactly why #336's
   `pixel_delta(cell_h).x` scaled x by the wrong dimension. `.precise()` (interactive.rs:410) → true for
   Pixels. No other gpui helper does per-axis scroll scaling (checked interactive.rs) → the fix must match on
   `ScrollDelta` and scale x by `cell_w` itself.
2. **OUR OWN CODE:** `h_scroll.rs` is the pure px-domain home (the #336 `caret_px`/`follow_caret_x`/`sane`,
   cov/MSI 100) — `wheel_x_px` lands beside them with the same `sane()` totality idiom. The #336 sign
   convention (dx negated) is `AD-...-001` (411bc001).
3. Checked ropey/regex/tree-sitter — N/A (this is an input-event scalar, not text/parse).

## Locked-In Decisions
- **D1-MATCH-NOT-PIXEL-DELTA** — the handler matches `event.delta` directly (`Pixels` → exact px, `Lines` →
  scale x by `cell_w`) rather than `pixel_delta(_)`, because gpui's single-scalar converter cannot scale the
  two axes differently. (Prior art leg 1 — the substrate confirms it.)
- **D2-PURE-SEAM** — the scalar arithmetic is `h_scroll::wheel_x_px` (cov/MSI 100); the handler is a
  `code_view_body` shim (`mutants::skip` + coverage-excluded), so the math MUST live in the pure fn to be
  tested. Signature `wheel_x_px(raw_x: f32, precise: bool, cell_w: f32) -> f32`.
- **D3-FLOOR-AND-SANE** — `wheel_x_px` floors `cell_w` at 1.0 (mirrors the original `cell_h.max(1.0)` — guards
  a 0/degenerate cell pre-first-frame) and `sane()`s the inputs (mirrors the sibling h_scroll totality: no
  non-finite input panics or yields a non-finite delta).
- **D4-SIGN-UNCHANGED** — the shim still feeds `h_scroll_clamp(scroll_x - dx)` (negated, positive-right). The
  fix changes only the MAGNITUDE of `dx` for `Lines`, not its sign.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-LINES-CELL-WIDTH | WHEN a `Lines` (mouse) wheel event is handled, the system shall scale the x delta by the cell WIDTH: `wheel_x_px(lines_x, false, cell_w) == lines_x * cell_w` (cell_w ≥ 1). | unit: `wheel_x_px(3.0, false, 8.0) == 24.0` (a concrete non-identity value; a `cell_h`≈19 would give 57). |
| REQ-PIXELS-PASSTHROUGH | WHEN a `Pixels` (trackpad) wheel event is handled, the system shall use the exact px, ignoring cell_w. | unit: `wheel_x_px(24.0, true, 8.0) == 24.0` AND `wheel_x_px(24.0, true, 99.0) == 24.0` (cell_w ignored when precise). |
| REQ-TOTALITY | The system shall not panic and shall return a finite delta for any input (a non-finite raw_x/cell_w, a 0/negative cell). | unit: the hostile-input sweep (mirror the #336 `*_are_total_over_hostile_input`) — every result `.is_finite()`. |
| REQ-SIGN-UNCHANGED | The system shall keep feeding the clamp the NEGATED delta (`scroll_x - dx`), so the scroll direction is unchanged. | diff review — the shim's `h_scroll_clamp(scroll_x - dx)` line is byte-identical; the #336 sign is untouched. |

## Phase Plan
- **P2 Design** — confirm the `wheel_x_px` signature + body + the shim match; the test plan (3 unit tests in
  h_scroll.rs); confirm the shim stays skip'd + the sign line unchanged; §20 N/A confirmed.
- **P3 Implement** — `h_scroll::wheel_x_px` + the app.rs shim (the `pixel_delta(cell_h)` → the match). `cargo check`.
- **P3.5 Inspect** — the Pixels-vs-Lines unit semantics; cell_w is the right divisor; the sign is unchanged; no
  other wheel handler touched.
- **P4 Validate** — the 3 unit tests (h_scroll.rs cov/MSI 100); the `--diff` gate. **NO live drive — a mouse
  tilt-wheel event can't be synthesized headlessly; the pure fn + the diff-reviewed shim are the proof (stated,
  not masked).**
- **P5 Complete** — CHANGELOG + editor.md (the wheel x now scales by cell_w); AAR; close + archive.
