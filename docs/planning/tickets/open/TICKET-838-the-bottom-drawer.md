# TICKET-838 — The bottom drawer: Problems and Debug

- **Ticket:** LOCAL #838 (feature, the Marley look, W3.2)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W3.2. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** BottomPanel
- **Size:** L
- **Status:** open

## Summary

Problems and Debug as tabs of one drawer in the bottom dock, drawn as the board shows; Output and Tasks come in a later ticket. Debug is Zed's debugger panel (#774 keeps it in the bottom dock); Problems is Zed's diagnostics. Touches Zed's dock code (ledger rows).

## Acceptance

The drawer opens on Problems or Debug from the bottom row and shows each as the board does. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.

## Decision

Decision taken by default: Problems and Debug first (open decision 4). Chad can change it before the ticket is planned.
