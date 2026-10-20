# TICKET-833 — The bottom row says what has the focus

- **Ticket:** LOCAL #833 (feature, the Marley look, W1.4)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W1.4. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** BottomRow
- **Size:** L
- **Status:** open

## Summary

New `StatusItemView` items in `marley_workbench` for five focuses: a file, a terminal, an agent thread, debugging, and an agent that needs you (with Allow and Open in the row). The panel buttons become one labeled group with a dot on a panel already open as a tab. Small cards open above the row: language servers, problems, panels, edit predictions. Replacing Zed's own items touches `crates/zed/src/zed.rs` (a ledger row).

## Acceptance

The row changes with the focus as the board shows in each of the five cases; each card opens from its item. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.
