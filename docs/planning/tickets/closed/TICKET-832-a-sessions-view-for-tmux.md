# TICKET-832 — A Sessions view for every tmux session

- **Ticket:** LOCAL #832 (feature, the Marley look, W1.3)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [832-a-sessions-view-for-tmux.spec.md](../../pipeline/completed/832-a-sessions-view-for-tmux.spec.md)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W1.3. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** TmuxSessions
- **Size:** M
- **Status:** closed

## Summary

A third choice at the top of the rail, Projects · Threads · Sessions: every tmux session by host, its windows, where it is placed and whether it is attached, with Attach or Show and Close. Hosts come from `ssh_connections`, each toggled into `marley.tmux_hosts` from the view. A Sessions tile in the shrunk strip and a count in the bottom row (#833). Builds on `tmux_sessions.rs` (#808, #821).

## Acceptance

Sessions lists every session on every listed host with its placement and state; Attach, Show and Close work from it; a host can be added to `tmux_hosts` from it. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.
