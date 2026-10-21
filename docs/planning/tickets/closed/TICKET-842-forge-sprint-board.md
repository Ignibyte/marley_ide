# TICKET-842 — Forge's sprint board

- **Ticket:** LOCAL #842 (feature, the Marley look, W4.2)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [842-forge-board.spec.md](../../pipeline/completed/842-forge-board.spec.md)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W4.2. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** ForgeBoard
- **Size:** L
- **Status:** closed

## Summary

A tab with columns for Open, In progress, In review, QA (qa-review, qa-failure, qa-pass) and Done; each card a ticket with its kind, phase and agent; a ticket's detail on the right. Claim, move and comment through forge's tools; "Start here in a worktree" hands the ticket to New Agent in Worktree (#510). Depends on #841.

## Acceptance

The board shows the current sprint's tickets by status as the board does; moving a ticket writes it to forge. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.
