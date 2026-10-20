# TICKET-837 — Side panels as one inspector

- **Ticket:** LOCAL #837 (feature, the Marley look, W3.1)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W3.1. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** SidePanels
- **Size:** L
- **Status:** open

## Summary

Files, Changes and Outline in one column beside the pane, with a switcher at its top, offered as a choice next to #747's full-width panel tabs (`panel_tabs.rs`).

## Acceptance

With the inspector chosen, the three panels open in the column as the board shows; #747's tabs still work when it is not. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.

## Decision

Decision taken by default: the inspector sits beside #747's tabs as a choice, not a replacement (open decision 1). Chad can change it before the ticket is planned.
