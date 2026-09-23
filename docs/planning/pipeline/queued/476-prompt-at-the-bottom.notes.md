# The prompt at the bottom of the terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-476-prompt-at-the-bottom.md
- **Pipeline spec:** 476-prompt-at-the-bottom.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "the terminals sessions in warp command line stays at the
  bottom which is nice. is that doable". First of the six queued that day, under the goal "lets
  continue and spec these tickets out and run them end to end".
- **Recall.** #470 moved the block decorations onto `dimensions.bounds.origin`, so they follow
  one origin; L-claude-470-the-alternate-grid-counts-its-own-evicted-lines-001 (the alternate
  screen is its own frame of reference); PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001.
- **Seams read:** `terminal_element.rs` prepaint (the origin, `should_anchor_to_bottom`, the
  padding, `TerminalBounds::new`, `set_size` then `sync`); `Terminal::set_size` (resizes only on
  a line, column or cell-size change); `alacritty.rs` `make_content` (`bottom_row_occupied`).
