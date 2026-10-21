# TICKET-865 — An agent panel beside a terminal, and a running block's progress line

- **Ticket:** LOCAL #865 (feature, the Marley look, W2 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-826 while planning it (2026-10-21): the AgentTerminal board
  shows an agent panel at the terminal's right that reads a failed block and proposes a fix to
  apply, and a running block with a progress line, which #826 leaves out.
- **Boards:** AgentTerminal
- **Size:** L
- **Status:** open

## Summary

A `MarleyTerminalSide` hook in `TerminalView::render` (a `// Marley:` hunk, the root in an
`h_flex`), drawn by Marley: an agent column beside the terminal that takes a block (from its Ask
or Explain), shows the agent's answer and a proposed command or patch with Apply. And a thin
progress line along a running block's top while its command runs (indeterminate, a gpui
animation that stops under reduced motion).

## Acceptance

From a failed block, the panel opens beside its terminal with the block and the agent's answer,
and Apply runs the proposed command; a running block shows the moving line. Shot in Marley Night
and Marley Day.
