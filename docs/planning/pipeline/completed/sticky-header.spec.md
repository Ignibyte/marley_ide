---
pipeline_id: d3b50a15-4bd3-4715-b458-cffd497b64bb
ticket: forge#185 (e0611622-933f-45bf-b53f-6d778b21f960) · local docs/planning/tickets/open/TICKET-185-sticky-header.md
aar_id: 13c90d9e-fced-4f82-9546-ca367094f0c1
status: Phase 5 — Complete PASS
title: M12 — sticky command header while scrolling a long block
type: feature
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_app/src/nav.rs (PURE: sticky_block over fold_visible_rows)
  - crates/marley_app/src/app.rs (SHIM: the sticky header overlay row at the pane top)
---

## Title
Warp/devtools feel — when you scroll deep into a long command's output, that command's header PINS to the
pane's top edge (a sticky overlay row) so you always know which command you're reading, until the next block's
real header scrolls into place.

## Scope
### In
- PURE `nav::sticky_block(output_line_counts, folds, viewport_top) -> Option<usize>` — over the #184
  fold-aware row sequence: `Some(block)` when the top visible row is that block's OUTPUT (its real header has
  scrolled ABOVE the top); `None` when the top row is a HEADER (already visible at the top) or past the last
  block row (the prompt / empty). Reuses `fold_visible_rows` (so it's fold-consistent) + complements #175.
- SHIM `app.rs`: after `(start, end) = viewport.visible(...)`, compute `sticky_block(..., start)`; when Some,
  render an ABSOLUTE overlay row at the pane's top edge — the block's status glyph + its command text on a
  surface bg (occluding the scrolled output beneath) — the compact sticky header.

### Out
- The sticky header's own chevron / copy / rerun actions (the real header keeps those; the sticky is a
  read-only label). A shadow/animation as it swaps. Sticky for the FILES/code panes (terminal blocks only).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `sticky_block` shall return `Some(block)` when `viewport_top` is one of that block's output rows (header scrolled above). | unit + mutation |
| REQ-002 | `sticky_block` shall return `None` when `viewport_top` is a block's header row (the boundary — already visible) or past the last block row. | unit + mutation |
| REQ-003 | `sticky_block` shall respect folds (a folded block above shifts the row→block mapping). | unit |
| REQ-004 (visual) | WHEN a tall command's output is scrolled so its header is off the top, that command's header shall ride the pane's top edge; scrolling to another block switches it. | driven capture |
| REQ-005 | gate GREEN; the pure fn cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the pure fn + the shim overlay. P3.5 1-2 critics (the header-exactly-at-top boundary → None; the
past-end case; fold-consistency; the overlay occlusion / no double-header when the real header IS at top). P4
unit + driven (scroll a tall block) + gate (staged --diff). P5 docs.
