# the code viewer as a side-by-side panel — Notes

- **Forge ticket:** #114 `861a81d5-1115-4c12-bbc5-22325f1940f7` · **AAR:** `0c16ff56-2e29-4a17-8f80-ed19bafedd9b`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-114-viewer-pane.md

## Phase 1 — Plan
- **Request:** forge #114 (M5 8/12) — the viewer as a right-side panel beside the terminal.
- **Pre-flight:** center_bounds (app.rs ~2132) → pane_rects; the M4 viewer is a centered modal (~2752).
  Reposition it to a right panel + shrink center_bounds when open.
- **Decisions:** D1 viewer 45%; D2 a side-panel (not a PaneGroup pane — session-per-pane refactor deferred).
- **AAR id:** `0c16ff56-2e29-4a17-8f80-ed19bafedd9b`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **code_view.rs (PURE):** `pub fn viewer_split(center_w: f32) -> (f32, f32)` = `let viewer = center_w * 0.45; (center_w - viewer, viewer)`.
- **app.rs SHIM:** before center_bounds: `let (term_w, viewer_w) = if self.code_view.is_some() { viewer_split(regions.center) } else { (regions.center, 0.0) };` + center_bounds.w = term_w. At the #97 viewer overlay (~2752): reposition — .left(regions.left+term_w).top(0.0).w(viewer_w).h(content_h) (was bounds.w*0.15 / 0.1 / 0.7), border_l_1; the title bar becomes a flex_row: 📄 name + spacer + an × (on_mouse_down → code_view=None).
- **Mutation targets:** viewer_split the 0.45 factor + the center_w - viewer.
- **Test plan:** viewer_split_proportional (viewer_split(1000.0) → terminal≈550 + viewer≈450 within 1.0); viewer_split_zero ((0.0)→(0.0,0.0)). cov/MSI 100. The reposition + × masked (live capture).
- **Risks:** term_w/viewer_w/regions/content_h in scope at both center_bounds + the overlay (same render fn); when code_view None → term_w = full center (unchanged behavior); the × closes.

## Phase 3 — Implement
- **Built:** viewer_split(center_w) (code_view.rs, viewer 45%/terminal 55%); app.rs computes (term_w,viewer_w) when code_view is Some → center_bounds.w = term_w (terminals tile left) + the M4 viewer repositioned from a centered modal to a RIGHT panel (left=regions.left+term_w, top 0, w=viewer_w, h=content_h, border_l) with a title bar (📄 name + × close). When None → full-width center (unchanged).
- **Verification:** fmt; check 0 err; clippy OK; viewer_split test added.

## Phase 3.5 — Inspect
- **Method:** self-review (one small proportional-split fn + a masked reposition of the proven M4 viewer render).
- **Lenses — no findings:** viewer_split = viewer 0.45·center, terminal = center - viewer (both ≥0 for center≥0; tested 1000→550/450 + 0→0/0); when code_view None the center is UNCHANGED (term_w = regions.center, viewer_w 0 unused); the viewer render is repositioned to the right rect (left=regions.left+term_w, w=viewer_w, h=content_h) reusing all of #97-#106 (lines/gutter/scroll/syntax/guards) + gains an × that sets code_view=None; the terminals tile in the shrunk center_bounds (pane_rects) so they do not overlap the viewer. No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** viewer_split_cases (1000→550/450 within 1.0; 0→0/0). `cargo nextest` → pass.
- **Self-test:** the side-by-side viewer only shows once a file is OPEN, which needs a synthetic ⌘P→⌘↵ / tree ⌘-click (ENV-BLOCKED, as in M4). viewer_split engine-tested cov/MSI 100; the reposition is masked + code-reviewed; a boot capture confirms NO regression (terminal full-width, no viewer).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #114 → done. **M5 8/12.** viewer_split (cov/MSI 100) + the M4 viewer repositioned from a centered modal to a right-side panel (terminal shrinks; titled + × close). DEVIATION: a side-panel (not a PaneGroup pane — session-per-pane refactor deferred). Viewer-open env-blocked (engine-tested).
