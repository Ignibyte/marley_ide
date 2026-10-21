# TICKET-829 — The cat mark: app icon and title bar

- **Ticket:** LOCAL #829 (feature, the Marley look, W0.2)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [829-the-cat-mark.spec.md](../../pipeline/completed/829-the-cat-mark.spec.md)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W0.2. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** Theme
- **Size:** S
- **Status:** closed

## Summary

Marley is named for Chad's cat, a solid orange cat with green eyes. The Theme board's cat head becomes the app icon at every size (`crates/zed/resources/app-icon*.png`, the desktop entry's icon) and the mark in the title bar.

## Acceptance

The desktop entry, the window and the title bar show the cat mark; About shows it with the line about Marley. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.

## Decision

Decision taken by default: the Theme board's line, "Named for Marley: a solid orange cat with green eyes", until Chad changes it (open decision 2). Chad can change it before the ticket is planned.
