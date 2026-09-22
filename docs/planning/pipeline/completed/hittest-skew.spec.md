---
pipeline_id: 10b16f2f-2b14-4bfd-ab3e-5a3f7e47f259
ticket: forge#179 (61a841ee-2a48-4bf5-8c30-143e7386954c) · local docs/planning/tickets/open/TICKET-179-hittest-skew.md
aar_id: db594748-d203-4ac1-86e3-299498838ffa
status: Phase 5 — Complete PASS
title: M12 — fix the bottom-anchor remainder skew in pane_grid_pos
type: bug
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_app/src/viewport.rs (PURE: bottom_anchored_row; retire row_at/row_hit)
  - crates/marley_app/src/app.rs (SHIM: pane_grid_pos + the #175 hit-test compute y-from-bottom)
---

## Title
A click/drag in the bottom `r.h mod cell_h` sub-cell band of a row currently selects/menu-targets the NEXT
row's block — the hit-test maps from the pane TOP using the pty row count, ignoring the 24px title bar +
the paint remainder. Fix: map from the pane BOTTOM (rows paint flush there), matching the render exactly.

## Scope
### In
- PURE `viewport.rs`: `bottom_anchored_row(y_from_bottom, cell_h, start, end) -> Option<usize>` — the row
  `end-1-floor(y_from_bottom/cell_h)`, `None` when the pointer is ABOVE the visible content (the empty pad
  + title bar) or the window is empty; a below-bottom pointer clamps to the last row. Retire the
  top-anchored `row_at` + `row_hit` (both were the bug's source).
- SHIM: `pane_grid_pos` + the #175 right-click hit-test compute `y_from_bottom = (r.y + r.h - 1.0) -
  position.y` (the `-1` is the pane div's bottom border) and call the pure fn — selection clamps a `None`
  (above content) to `start`; the menu treats `None` as "not on a block" → the split menu.

### Out
- Changing the col math (horizontal is correct); the title-bar height / border constants (shim geometry).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `bottom_anchored_row` shall map the bottom row to `end-1`, each cell up to `end-2…`, the empty pad above content to `None`, and an empty window to `None` — the boundary + pad mutants killed. | unit |
| REQ-002 (visual) | A right-click on a block's LAST output row (its bottom sub-cell band) shall open THAT block's menu, not the next block's. | driven capture |
| REQ-003 | Terminal text selection shall still anchor at the pointer's row (no regression). | driven capture |
| REQ-004 | gate GREEN; the pure fn cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the pure fn + both shim call sites + retire row_at/row_hit. P3.5 1 critic (the boundary
arithmetic; the two consumers' clamp-vs-None; no col regression; the border offset). P4 unit + driven
(the boundary-band right-click + a selection drag) + gate. P5 docs.
