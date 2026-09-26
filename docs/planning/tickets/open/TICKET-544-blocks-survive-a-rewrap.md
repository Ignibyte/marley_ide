# TICKET-544 — Blocks keep their rows when a resize rewraps the terminal

- **Ticket:** LOCAL #544 (bug, prong 1 T0: the block terminal's anchors, the plan's D2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** (none yet)
- **Source ticket:** found in #516's Test, 2026-09-25: the Settings window, tiled beside the main
  window by sway, narrowed the terminal; its lines rewrapped, and every block lost its rows
- **Status:** open

## Summary
A block is a range of absolute lines in the one scrollback (`output_start`, `output_end`, the
plan's D2). When a resize changes the width, alacritty rewraps the grid: a long line becomes two
rows at the narrower width, or two rows become one at a wider one. The anchors are not moved with
the rows. In #516's run the terminal narrowed from about 120 columns to 30: the block's margin
bar and pill vanished, and `terminal_read` answered rows that began inside the command line
(`ake sh secrets.sh`) and stopped mid-line, so an agent reads the wrong text after any resize
that rewraps. The plan names this "the known weak spot" and defers the choice until there are
real resize traces; this is one. The candidates the plan lists: move each anchor through the
rewrap (count the wrapped rows above it), or tag each prompt's first cell with an id that
survives the rewrap and re-find the block from it.

## Acceptance
After a resize that rewraps a block's lines, the block's bar and pill sit on its rows again and
`terminal_read` answers the block's output as printed, at the new width and after the window
returns to the old one.
