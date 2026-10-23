# TICKET-916 — Terminal blocks as the board draws them

- **Ticket:** LOCAL #916 (feature, the Marley look, terminal blocks)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [916-terminal-block-look.spec.md](../../pipeline/queued/916-terminal-block-look.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/A-core.md, boards Terminal and TerminalSession: "Block
  frames", "Block header", "Failed block's action row", "Running block time", "Progress bar")
- **Boards:** Terminal, TerminalSession
- **Size:** M
- **Status:** open

## Summary

#826 and #865 framed each block and put Ask an agent on a failed one, but the Terminal board
draws more: each block a rounded card with a gap to the next, a state dot before the command and
the output set in under it, a neutral border once a block passes, a red border and a tinted header
band when it fails, honey while it runs. A failed block's actions sit in a row under its output,
labelled; a running block's time ticks in its header; its progress line runs along the block's
bottom. This ticket draws the blocks so. Rerun and Copy stay on hover, as #826 left them.

## Acceptance

Blocks are rounded, gapped frames with a state dot; a passed block's frame is neutral, a failed
one red with its header band tinted and its actions in a row under the output, a running one honey
with a ticking time and its progress line at the bottom; shot in Marley Night and Marley Day.
