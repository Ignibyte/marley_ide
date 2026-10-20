# TICKET-835 — Home: one prompt, then what needs you

- **Ticket:** LOCAL #835 (feature, the Marley look, W2.2)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W2.2. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** Home
- **Size:** M
- **Status:** open

## Summary

`home_page.rs`: a prompt box with agent chips that starts the picked agent in the picked project, then the cards: agents at work, tasks, follow-ups, Rusty, recent projects, agent activity, and Configure as links.

## Acceptance

Home looks as the board shows; typing a task and picking an agent starts it in the chosen project. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.
