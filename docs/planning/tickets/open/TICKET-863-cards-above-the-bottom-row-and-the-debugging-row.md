# TICKET-863 — Cards above the bottom row, and the row while debugging

- **Ticket:** LOCAL #863 (feature, the Marley look, W1.4 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-833 while planning it (2026-10-21), from the design
  canvas's BottomRow board.
- **Boards:** BottomRow
- **Size:** M
- **Status:** open

## Summary

Each item of the bottom row opens a small card above it, never a dialog: language servers (each server's state, Restart, Open log, Ask an agent), problems in the project (errors, warnings, hints, Open in the drawer), panels (where each opens and its keys), and edit predictions (provider, never in .env files). And the debugging focus: Paused at file:line with Continue, Step over, Step in, Step out and Stop in the row, the selected frame and the breakpoint count.

## Acceptance

Each card opens from its item and acts; while a debug session is paused the row shows its place and controls; shot in Marley Night and Marley Day.
