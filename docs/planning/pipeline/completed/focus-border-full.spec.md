---
pipeline_id: d65e4185-2527-49f4-9843-c70515644b0e
ticket: forge#191 (578fad03-7e09-467a-8f56-2c13d43907a7) · local docs/planning/tickets/open/TICKET-191-focus-border-full.md
aar_id: 5aa2c0f2-07eb-4d87-9db9-892bbd667157
status: Phase 1 — Plan PASS · Phase 2 — Design PASS · Phase 3 — Implement PASS · Phase 3.5 — Inspect PASS · Phase 4 — Validate PASS · Phase 5 — Complete PASS
title: Full 4-side accent border on the focused pane (was a left+top edge)
type: feature
milestone: M12.1
references: []
---

## Title
The focused pane's accent affordance currently draws only a left + top 2px edge (the deliberate #131 Warp
choice). chad wants the "blue box" to frame the ENTIRE pane — all four sides. (chad live-app feedback #2.)

## Scope
### In
- Add a PURE `focus_border_rects(content: Rect, thickness: f32) -> [Rect; 4]` (workspace.rs, next to
  `inset_right`): the left/top/right/bottom edge rects of a `thickness`-wide border sitting INSIDE `content` —
  right/bottom insets by `thickness` so the border never overflows the pane's right/bottom edge (works with
  #189's right gutter so the right edge is visible in-window). Unit-tested + mutation-covered.
- The focused-pane render (app.rs ~4607) draws 4 thin accent divs from those rects, replacing the two inline
  left+top bars. Unfocused panes draw none. Shim render.

### Out (explicitly deferred)
- A full-frame bordered div — it would hit-test over the whole pane and swallow terminal clicks; 4 thin 2px bars
  only overlay the edges, keeping the interior clickable (the established #131 pattern).
- Changing the border thickness/color (keep the #131 2px `colors.accent`) or animating it.
- The title-bar region above `ay` (the border frames the content rect `[r.x, r.y+PANE_TITLE_H, r.w, ah]`, as #131 did).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — 4 thin bars, not a frame div.** Preserves interior click/hit-testing (a frame div blocks the terminal).
- **D2 — border inset within the content rect.** right = `x + w - thickness`, bottom = `y + h - thickness`, so
  the border is fully inside `[content]` and the right edge is visible (never clipped by the window / #189 gutter).
- **D3 — thickness 2.0, `colors.accent`** — unchanged from #131.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN computing the focus border for a content rect, the system shall return four edge rects (left, top, right, bottom) each `thickness` wide/tall and fully inside the content rect (right/bottom inset by `thickness`). | unit `focus_border_rects` (exact 4 rects; right/bottom insets) + mutation |
| REQ-002 | WHEN a pane is focused, the shell shall draw a full 4-side accent border around its content rect. | driven capture — focused pane shows all 4 accent edges |
| REQ-003 | WHEN a pane is NOT focused, the shell shall draw no focus border on it. | driven capture — a 2-pane split shows the border only on the focused pane |

## Phase Plan
- **P2 Design** — the pure `focus_border_rects` signature + the render wiring (4 `.child` divs from the rects);
  test plan (unit for the geometry, driven captures for REQ-002/003).
- **P3 Implement** — add the helper; rewire the render.
- **P3.5 Inspect** — critics: geometry correctness (insets, zero/thin panes), does the border block clicks, is
  the unfocused path clean.
- **P4 Validate** — unit + mutation; driven capture (focused vs unfocused pane); gate.
- **P5 Complete** — CHANGELOG + app_shell.md (#131→#191 border), archive, close.
