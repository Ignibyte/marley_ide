---
pipeline_id: 4b23e9ab-70f4-48ac-9db4-2130ebf6573b
ticket: forge#189 (790fa62a-c887-4395-a99f-7cf8309dac39) · local docs/planning/tickets/open/TICKET-189-pane-right-gutter.md
aar_id: bdf574a2-37f2-477c-b86b-b7d5cab7b804
status: Phase 1 — Plan PASS · Phase 2 — Design PASS · Phase 3 — Implement PASS · Phase 3.5 — Inspect PASS · Phase 4 — Validate PASS · Phase 5 — Complete PASS
title: M12.1 — split panes fit inside the window (right gutter); no overflow
type: bug
milestone: M12.1 — Cockpit polish & fixes
references:
  - crates/marley_app/src/layout.rs (region_widths — pure region math)
  - crates/marley_app/src/app.rs (center_bounds construction; the shim)
---

## Title
[chad live-app feedback #4] In a multi-pane split (the restored 5-pane "Release Run" tab), the rightmost panes
sit flush against — or bleed past — the window's right edge; it "feels like there should be padding there".
Give the pane-tiling center band a small RIGHT GUTTER so every pane ends inside the window with breathing room.

## Root cause (confirmed by reading the geometry)
`center_bounds` = { x: regions.left + files_w, w: (regions.center - files_w).max(0), … } spans exactly
[regions.left + files_w, bounds.w] — its right edge is EXACTLY bounds.w (the window edge), with NO gutter.
`pane_rects` tiles by ratios (child.w = bounds.w * ratio) filling center_bounds, so the rightmost pane ends
flush at bounds.w. Repeated ⌘⇧A splits (halve-the-focused) give a big first pane + shrinking ones, the last
flush against the edge → reads as "running off". Fix: inset center_bounds's width by a PANE_GUTTER on the right
(and confirm in design that the split ratios sum to 1.0 — if a persisted grid's ratios drift >1.0 that is a
SECOND overflow source to normalize).

## Scope
### In
- A `PANE_GUTTER` inset on the right of `center_bounds.w` so the pane grid ends PANE_GUTTER px before bounds.w
  (a right margin). Small (~8px). The cockpit/code tabs use the same center_bounds, so they inset too
  (consistent). Keep center_bounds.x unchanged (the left inset by files_w is correct).
- DESIGN measures the live rects (a temporary eprintln of pane_rects output, or read the capture) to CONFIRM the
  ratios sum to ~1.0 (no drift). If they drift, normalize in the split path — else gutter-only.

### Out
- Per-pane inner padding / gaps BETWEEN panes (a later polish); resizable gutter; changing the split ratios' feel.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | The pane-tiling center band shall end at least PANE_GUTTER px before the window's right edge. | unit (center_bounds math) or driven |
| REQ-002 (visual) | WHEN a multi-pane split is shown, the rightmost pane shall render fully inside the window with a right gutter (no bleed). | driven capture |
| REQ-003 | gate GREEN; any pure math cov/MSI 100. | gate |

## Phase Plan
P2: measure the live rects (confirm ratio sum) + decide gutter-only vs also-normalize. P3 the gutter inset (+
normalize if needed). P3.5 1 critic (the max(0) clamp on a narrow window; the cockpit/code consistency; ratio
drift). P4 unit (the width math) + driven (the multi-pane tab with a visible right gutter) + gate. P5 docs.
