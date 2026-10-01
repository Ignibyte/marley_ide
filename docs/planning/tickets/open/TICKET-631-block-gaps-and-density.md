# TICKET-631 — Gaps between blocks and a header density setting

- **Ticket:** LOCAL #631 (feature, prong 1 T5, stage two's last slice)
- **Owner:** claude-opus-5-5, 2026-10-01
- **Pipeline doc:** none yet (Deliberate)
- **Source ticket:** split from #630 at its promotion (`docs/planning/pipeline/completed/630-block-density-and-two-line-headers.spec.md`)
- **Status:** open

## Summary
#628 to #630 draw a block's header in exactly the rows its prompt took, so no row moves. What
is left of the Warp look needs rows that are no grid line: a gap between blocks, a header taller
than its prompt (two lines over a one-row prompt), and `marley.block_density` (`compact`,
`comfortable`). That takes a display-row map through every place `terminal_element.rs` turns a
row into a y (thirteen) and every place a pixel becomes a row (four, `grid_point_and_side` among
them), with the view scrolling past the grid by pixels, as `block_below_cursor` does. It waits
for Chad to decide the look is worth that much diff in Zed's terminal crates, which the fork keeps
small so upstream merges stay cheap.

## Acceptance
With comfortable density, blocks stand half a row apart and a one-row prompt's header shows two
lines; the live prompt stays on the last row; scrolling, selection and the mouse land on the rows
the user sees.
