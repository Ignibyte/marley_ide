---
pipeline_id: 1905e187-bdfb-4059-a870-d3f4db5747af
ticket: forge#198 (a71a6924-b6d2-47f6-a45b-3734defc3ffb) · local docs/planning/tickets/open/TICKET-198-warp-scrollbar.md
aar_id: f61dd6f9-6679-4a61-b2e0-9f3ca05fd361
status: Phase 5 — Complete PASS (all phases PASS; 303 tests, MSI 100; live captures confirm REQ-001..005)
title: Pane scrollbar thumb + jump-to-bottom affordance
type: feature
milestone: M12.2
references: []
---

## Title
A scrolled-up pane gives no visual scroll indicator + no quick way back to the latest output. Add a
scrollbar THUMB (position + proportional height) on the pane's right edge, and a JUMP-TO-BOTTOM button
shown while the pane is scrolled up. Reuses the R39 `viewport.rs` scroll model (`Viewport{following,top}`,
`visible`, `max_scroll`).

## Scope
### In
- **Pure seam** (viewport.rs, cov/MSI 100): `scrollbar_thumb(content, capacity, start) -> Option<(f32, f32)>`
  (top_fraction, height_fraction of the track; `None` when `content <= capacity`); `at_bottom(start, content,
  capacity) -> bool` (`start + capacity >= content` — the last row is visible).
- **Shim** (app.rs pane render): the thumb on the pane's right inner edge (position/height from the fractions
  × the pane content height), rendered only when `scrollbar_thumb` is `Some`; a jump-to-bottom affordance
  (bottom-right of the pane) shown when `!at_bottom`, whose click re-anchors the viewport to the bottom
  (reuse the tested `Viewport::scroll_down(content, content, capacity)` → re-anchors `following = true`).

### Out (explicitly deferred)
- Drag-to-scroll on the thumb (the wheel already scrolls; a draggable thumb is a follow-up refinement).
- A horizontal scrollbar (terminal rows don't scroll horizontally).
- Alt-screen panes (a full-screen program owns its own screen — no cooked scrollback thumb there).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — `scrollbar_thumb` returns FRACTIONS of the track (pure geometry), the shim multiplies by the real
  pixel height. `None` when all content fits (no thumb) — the sole scrollbar-visibility rule.
- **D2** — the jump re-anchors via the EXISTING `scroll_down` (a large `n` reaches max → `following=true`) —
  no new mutation-untestable setter.
- **D3** — tokens only (`border`/`muted` for the thumb; `surface`/`muted` for the button) — no new hsla; §20.
- **D4** — auto-approved (/work 195–222): document with the driven scroll capture.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a pane's content exceeds its viewport capacity, the pane shall render a scrollbar thumb whose top + height reflect the scroll position + the visible proportion. | Unit test (`scrollbar_thumb` exact fractions) + driven capture (thumb visible, tracks scroll) |
| REQ-002 | WHEN a pane's content fits its viewport (`content <= capacity`), the pane shall render NO scrollbar thumb. | Unit test (`scrollbar_thumb` = `None`) + capture (no thumb on a short pane) |
| REQ-003 | WHEN a pane is scrolled up (the latest output is not visible), the pane shall render a jump-to-bottom affordance. | Unit test (`at_bottom` false) + driven capture (button appears when scrolled up) |
| REQ-004 | WHEN the jump-to-bottom affordance is clicked, the pane shall re-anchor to the bottom (follow the latest output) and the affordance shall disappear. | Driven capture (click → back at bottom, button gone) |
| REQ-005 | `scrollbar_thumb`/`at_bottom` shall compute exact geometry/predicate for all boundaries (content≤capacity, start 0, start=max_scroll, content==capacity). | Unit tests (exact-value, cov/MSI 100) |

## Phase Plan
- **P2 Design** — the exact `scrollbar_thumb`/`at_bottom` signatures + clean-ratio test matrix; the thumb
  render geometry (right-edge inset, min thumb height?), the jump-button render + its click→scroll_down;
  where `start`/`content`/`capacity` come from at the render site; file manifest + test plan.
- **P3 Implement** — the 2 pure fns + the thumb + jump-button shim.
- **P3.5 Inspect** — critics: the fractions are correct (no div-by-zero — guarded by the `content<=capacity`
  None), the thumb clamps in-track, the jump reuses the tested follow, no alt-screen regression; clean-room.
- **P4 Validate** — the pure unit tests + a driven scroll capture (thumb tracks; jump returns to bottom); gate.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; close.
