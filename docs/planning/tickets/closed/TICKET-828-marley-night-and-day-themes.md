# TICKET-828 — Marley Night and Marley Day themes

- **Ticket:** LOCAL #828 (feature, the Marley look, W0.1)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [828-marley-night-and-day-themes.spec.md](../../pipeline/completed/828-marley-night-and-day-themes.spec.md)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W0.1. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** Theme, GpuiFoundations, MainDay
- **Size:** S–M
- **Status:** closed

## Summary

A theme family at `assets/themes/marley/marley.json` with a dark and a light variant, loaded as `ayu`, `gruvbox` and `one` are. GPUI's 25 `Color` variants and its elevation levels map to the Theme board's tokens: coat `#ff8c42` as the accent, eye green `#9be15d` for success and added lines, warm darks from `#0f0c0a` to `#2a221d`, honey `#f6c453` for warnings, pink `#ff6f91` for conflicts and needs-you, red `#ff6b5a` for errors. `assets/settings/default.json` sets `theme.dark` to Marley Night and `theme.light` to Marley Day (a ledger row). Following Omarchy's theme (#759) still overrides.

## Acceptance

Marley starts in Marley Night with the system dark and Marley Day with it light; the rail, tabs, terminal and editor take the board's colors; `component_preview` and the main screen are shot in both themes. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.

## Decision

Decision taken by default: Marley Day is the default whenever the system is light (open decision 5). Chad can change it before the ticket is planned.
