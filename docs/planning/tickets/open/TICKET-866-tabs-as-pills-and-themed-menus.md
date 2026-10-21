# TICKET-866 — Tabs as pills, and the menus in the theme

- **Ticket:** LOCAL #866 (feature, the Marley look, W2.1 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-834 while planning it (2026-10-21): the Chrome board's tabs
  and menus, which #834 leaves out.
- **Boards:** Chrome
- **Size:** M
- **Status:** open

## Summary

In the Marley layout, center tabs as pills showing their kind (an icon tile, as #827's rail rows)
and their state (a dot); the + menu listing Marley's real shortcuts beside each entry; the split
menu drawn as small pictures of each layout; the tmux close prompt (#821) in the theme's colors
rather than the system's. Touches the workspace's tab drawing (`pane.rs`, a ledger row, additive)
and Marley's menus.

## Acceptance

Tabs, the + menu, the split menu and the tmux close prompt look as the Chrome board shows in the
Marley layout; Zed's layout is unchanged. Shot in Marley Night and Marley Day.
