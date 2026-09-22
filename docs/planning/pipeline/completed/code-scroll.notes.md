# M10 — code tab wheel-scroll — Notes

- **Forge ticket:** #165 `fd1caabd-bb28-4b40-9eeb-be5ba4468e12` · **AAR:** `f2f0f603-6b77-4a5f-a354-28b885e3a7fb`

## Phase 1 — Plan
- Regression CONFIRMED statically: `cv.scroll` has zero mutation sites since #154 (only the read at
  app.rs:1595); the pane-era scroll path died with the tiled CodeView pane.
- **AAR id:** `f2f0f603-6b77-4a5f-a354-28b885e3a7fb`.

## Phase 2 — Design (folded into the spec)
- Pure clamp: `scroll_code`; shim: `code_scroll_remainder` + `.on_scroll_wheel` on the code-branch wrapper
  (steps via the existing `scroll_steps`; sign per the terminal handler); a `scrollat` drive verb.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- code_view.rs: scroll_code(scroll, steps:i64, height, total) — clamp [0, total.saturating_sub(height)].
- app.rs: code_scroll_remainder:f32 field (+init); the code-branch wrapper div gains on_scroll_wheel → rows via pixel_delta/fallback_cell(TERMINAL_FONT_SIZE).h → scroll_steps(remainder) → cv.scroll = scroll_code(cv.scroll, -steps, VIEWER_ROWS, len) via code_view_mut + notify. scroll_code imported.
- drive.swift: scrollWheel + a scrollat:fx,fy,clicks verb (mouseMoved first — gpui routes wheel to the hovered element).
- fmt; check 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: adversarial SELF-REVIEW (a small pure fn + one handler; gate:5 is the mechanical adversary), sign
convention traced end-to-end.

- **SIGN TRACE (the likely bug class here):** terminal: positive pixel-delta rows → steps>0 → scroll_up
  (toward OLDER). Code tab: steps>0 → `-steps` → scroll DECREASES (earlier lines) ✓; steps<0 → scroll
  increases (later) ✓ — same convention. The scrollat verb posts CGEvent line-units where positive wheel1 =
  toward earlier ✓ (its default -3 = "3 clicks toward later").
- **CLAMP EDGES:** total<height → max_start=0 → always 0 ✓; the casts are safe (lines ≪ i64::MAX — the viewer
  caps input at VIEWER_MAX_BYTES); clamp(0, max_start) with max_start≥0 can't invert.
- **[accepted] the remainder isn't reset on tab/file switch** — a stale sub-row (<1 row) can bleed into the
  next code tab's FIRST wheel tick; bounded by <1 row, cosmetic. Noted; a per-tab remainder needs an f32 in
  the Eq-deriving CodeViewState — not worth breaking Eq.
- **notify only when steps≠0** — no per-frame churn on micro-deltas.
- **The wheel handler sits on the WRAPPER div (center_bounds-sized)** — the whole code area scrolls, no
  dead margins.

Lenses: sign parity, clamp bounds, cast safety, remainder lifecycle, notify discipline, hit-area.

## Phase 4 — Validate
- **Tests:** scroll_code_cases (interior both ways, both pins, short-file 0, zero-identity). 1/1.
- **Self-test (a NEW scrollat verb):** cs_down.png — 15 wheel clicks down over enforce-commit-gate.sh → the gutter starts at 16 (line-perfect, was 1); cs_up.png — 60 clicks up → pinned back to 1. REQ-002 both directions.
- **Gate:** GREEN [diff] 15/15, MSI 100 (scroll_code mutation-killed).

## Phase 5 — Complete
- CHANGELOG (Fixed) + app_shell #165 note; forge #165 → done. **M10 5/10 (161,162,164,165 + the #159 guard).** LESSON: a render-MOVE can orphan STATE-MUTATION paths silently — after retiring a surface, grep every mutable field its handlers touched for a surviving writer.
