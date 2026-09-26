# TICKET-529 — A long block's command stays in view

- **Ticket:** LOCAL #529 (feature, prong 1 T1: stage-one block rendering)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/529-sticky-command-header.spec.md
- **Source ticket:** Chad, 2026-09-25, approving all seven items of the Warp once-over (`docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 5, "Block filtering and the sticky command header")
- **Status:** open

## Summary
Scrolled back into the middle of a long block, the terminal shows lines with nothing to say which
command printed them. Warp pins that block's command at the top of the pane while you scroll its
output, and a click on it jumps to the block's start. Marley draws the same header over the
terminal's top row, only while the view is scrolled back and the block at the top started above
it, so it never covers the live screen. A click scrolls the view to the block's first line. A
setting on the Marley page turns it off.

## Acceptance
Scrolled back into `seq 1 400`'s output, the terminal shows `seq 1 400` pinned at its top; a click
on it brings the block's first line to the top; at the live screen, or with the setting off, no
header shows.
