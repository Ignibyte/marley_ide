# TICKET-546 — Block reads while a full-screen program shows

- **Ticket:** LOCAL #546 (bug, prong 1 T0 with prong 2's terminal tools)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** (none yet)
- **Source ticket:** found in #544's Plan, 2026-09-26, by the sweep of the block anchors
- **Status:** open

## Summary
Blocks live on the terminal's main screen. While a full-screen program (vim, less, htop) holds
the alternate screen, alacritty's active grid is the alternate one, and the main screen is kept
aside. `absolute_lines_text` (behind `Terminal::block_output`) and `Terminal::block_output_kept`
read the active grid, so `terminal_read` and `terminal_blocks` answer from the alternate screen's
rows while such a program runs: an agent reading an earlier block's output while the user sits in
`less` gets the wrong text, or none. #544 adds `Term::main_grid()` to the vendored alacritty; the
two readers should use it.

## Acceptance
While a full-screen program shows, `terminal_read` answers each finished block's output as it
was, and `terminal_blocks` reports whether it is still in the scrollback from the main screen.
