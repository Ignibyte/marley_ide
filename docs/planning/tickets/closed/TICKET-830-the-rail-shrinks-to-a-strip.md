# TICKET-830 — The rail shrinks to a strip of project marks

- **Ticket:** LOCAL #830 (feature, the Marley look, W1.1)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [830-the-rail-shrinks-to-a-strip.spec.md](../../pipeline/completed/830-the-rail-shrinks-to-a-strip.spec.md)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W1.1. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** RailStates, States
- **Size:** M–L
- **Status:** closed

## Summary

A button beside the rail's search, and `marley.rail_shrunk` (proposed), fold the rail to a 60 px strip: each project's mark with a needs-you badge, a flyout of the project's rows on hover, and the search kept at the top as a button that opens the filter as a popover. Ctrl F still filters. The width eases over 180 ms; with reduced motion it jumps. Code: `rail.rs`, `rail_switcher.rs`.

## Acceptance

The rail folds to the strip and back from its button and the setting; the search opens from the strip; a project that needs you shows its badge; the setting survives a restart. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.
