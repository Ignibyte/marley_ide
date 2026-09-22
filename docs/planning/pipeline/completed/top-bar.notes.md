# the Warp top bar (M7) — Notes

- **Forge ticket:** #132 `4bed8453-ad63-46fd-840f-1dbadb6e8c59` · **AAR:** `3f8c5b3c-a044-44c1-b62d-23434ffbc10d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-132-top-bar.md

## Phase 1 — Plan
- **Request:** forge #132 (M7 run 1/5, FOUNDATION) — real top bar; search in-bar; fixes the close button (#2+#3).
- **Pre-flight:** STATUS_BAR_H (193); content_h = bounds.h - STATUS_BAR_H (1784); dock_panel top(0.0) (227, called
  2136/2266); center_bounds y:0 (2277); the pane title × at 2862 (workspace.close); the #117 search + #126 tabs
  float (occlude) over the top.
- **Decisions:** D1 content_band = vertical region_widths; D2 docks+grid start at content_top; tabs float for now.
- **AAR id:** `3f8c5b3c-a044-44c1-b62d-23434ffbc10d`.

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
- **layout.rs (PURE):** content_band(window_h,top_bar_h,status_bar_h)->(content_top,content_h): content_top=top_bar_h; content_h=(window_h-top_bar_h-status_bar_h).max(0).
- **app.rs (SHIM):** TOP_BAR_H=28; let (content_top,content_h)=content_band(bounds.h,TOP_BAR_H,STATUS_BAR_H); dock_panel top(0.0)→top(content_top) (both calls); center_bounds.y=content_top; render a top-bar row (y=0,h=TOP_BAR_H,surface bg,border_b); move the #117 search into the bar (in-flow centered; its results dropdown → y=TOP_BAR_H). The #126 tabs stay floating (→#134).
- **Test plan:** content_band_cases ((1000,28,22)=(28,950); short window clamps content_h=0).
- **Risks:** the dock_panel signature (does it take a y-origin, or hardcode top(0.0)?); center_bounds y offset flows to pane_rects (title bars shift down — good); the search dropdown reposition; STATUS_BAR_H stays bottom (bounds.h - STATUS_BAR_H for the status bar y).

## Phase 3 — Implement
- **Built (layout.rs PURE):** content_band(window_h,top_bar_h,status_bar_h)->(content_top,content_h). **(app.rs SHIM):** TOP_BAR_H=30; import content_band; content_h→content_band; dock_panel gained a `y` param (both calls pass content_top); center_bounds.y=content_top; a full-width top-bar bg row (y=0,h=TOP_BAR_H,surface,border_b) drawn before the search/tabs. The search + #126 tabs keep their positions (now within the bar zone, backed by the bar); the panes moved down (content_top) so the title bars are un-occluded.
- **DEVIATION:** did not physically re-parent the search INTO the bar div — it stays absolute at top:3 (within the 0..30 bar zone, drawn on the bar bg); functionally in-bar + not over the terminal. Its dropdown still overlays below (intended). Simpler + lower-risk than re-parenting.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a vertical-band fn + a reserved-height structural shift).
- **Lenses — no findings:** content_band = (top_bar_h, (window_h-top_bar_h-status_bar_h).max(0)) — both subtractions + the clamp. The docks + center_bounds now start at content_top; content_h shrank by TOP_BAR_H so the band sits between the bars; the status bar is bottom-anchored (unchanged). The top-bar bg draws before the search/tabs (they render on it); the panes start at content_top so their title bars (+ the close-×, workspace.close) are no longer under the search/tabs overlays (#3 fixed). Search no longer floats over the terminal (#2). No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** content_band_cases (REQ-001 (1000,28,22)=(28,950); REQ-002 short window→(28,0)). Pass.
- **Self-test:** LIVE capture (topbar132.png) — a real full-width top bar (border-b) holds the search (centered) + the Details/Agents/Forge tabs; the Files sidebar + terminal pane now start BELOW it. The terminal pane title bar (⋮ ×) is below the bar, clear of overlays → the close-× is un-occluded (#3 fixed structurally; workspace.close already tested; synthetic click env-blocked). The search is in the bar, not over the terminal (#2 fixed). REQ-003/004 PASS.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG; forge #132 → done. **M7 1/5.** content_band (pure) + a real top bar; search in-bar (#2); panes below → close-× un-occluded (#3). cov/MSI 100.
