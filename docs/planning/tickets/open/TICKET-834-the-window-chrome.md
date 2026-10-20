# TICKET-834 — The window chrome: title bar, tabs and menus

- **Ticket:** LOCAL #834 (feature, the Marley look, W2.1)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W2.1. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** Chrome
- **Size:** M–L
- **Status:** open

## Summary

Title bar: the project mark and branch, the palette field, agent pills, Rusty and Marley buttons. Tabs as pills showing their kind and state; the + menu with Marley's real shortcuts; the split menu drawn as pictures; the tmux close prompt in the theme. Touches Zed's `title_bar` and the workspace's tab drawing (ledger rows, additive).

## Acceptance

The title bar, tabs and menus look as the board shows in the Marley layout; Zed's layout is unchanged. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.
