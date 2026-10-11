# TICKET-772 — Reopen Thread…, and every thread opens in a tab

- **Ticket:** LOCAL #772 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** not started; the design is drafted in [744's queued spec](../../pipeline/queued/744-one-new-agent.spec.md) (its Reopen Thread… and "threads open in tabs" parts, REQ-007 to REQ-012)
- **Source ticket:** split from #744 on 2026-10-11, at its promotion, to keep each slice one
  shippable change. Chad, 2026-10-11, of "New Agent from Thread": "my thought is that you would re
  open it". Plan: [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** open

## Summary
Reopen Thread… (`marley: reopen thread`) is a picker over the saved conversations, newest first,
with title, agent, folder and time; Enter opens the conversation in a tab of the group it was asked
from, or brings its tab forward. In the Marley layout every opener (Reopen Thread…, the Threads
page, the rail's thread rows and Archived Threads, the Needs you inbox's Manager entry, a
notification's click) goes through one path that never shows Zed's Agent Panel: the tab forward,
else the thread moved out of a panel into a tab, else a tab from the store. Move Thread to Panel
and Open in Center leave the Marley layout's menus.

## Acceptance
Reopen Thread… lists a closed conversation and reopens it in a tab with its earlier turns; chosen
again, it brings that tab forward; a click on its rail row opens it in a tab with the right dock
closed.
