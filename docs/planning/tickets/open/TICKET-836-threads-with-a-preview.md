# TICKET-836 — The Threads page with a preview

- **Ticket:** LOCAL #836 (feature, the Marley look, W2.3)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W2.3. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** Threads
- **Size:** M
- **Status:** open

## Summary

`threads_page.rs`: filters at the left (who, folder), cards grouped by day, and a preview of the selected thread with Open, Open in a Split and Archive.

## Acceptance

The page looks as the board shows; selecting a card fills the preview; its buttons work. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.
