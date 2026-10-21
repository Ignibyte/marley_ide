# TICKET-827 — Rail rows say their state by color

- **Ticket:** LOCAL #827 (feature, the Marley layout's look)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [827-rail-rows-say-their-state.spec.md](../../pipeline/completed/827-rail-rows-say-their-state.spec.md)
- **Source ticket:** Chad, 2026-10-20: "the idea is i think we would design the agent profile and
  things to be more chat gpt like. having panels around the conversations and stuff like that.
  lets go ahead and mock it up. and i feel like warp like". The mockup is the design canvas
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; this ticket is one of its boards. Chad filed the whole plan on 2026-10-20.
- **Boards:** Rail, RailMenus
- **Size:** M
- **Status:** closed (queued in wave 2 of [the Marley look plan](../../../marley/look-plan.md), 2026-10-20)

## Summary
Board 3, "Rail rows: kinds and states": a tile per kind (agent thread, agent in a terminal, shell, Browser tab, tmux session) with a state dot and a colored second line for working (amber), needs you (orange), done (blue) and idle (gray); a folded group header still says what needs the user; the colors as a small set of tokens.

## Acceptance
Every rail row and group header drawn as the board shows, the colors from the theme. Drawn in Marley Night and Marley Day once #828 lands.
