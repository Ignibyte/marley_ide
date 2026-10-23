# TICKET-927 — Thread states as the board draws them

- **Ticket:** LOCAL #927 (feature, the Marley look, agent thread)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [927-thread-states-as-the-board.spec.md](../../pipeline/queued/927-thread-states-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/C-threads.md, board ThreadStates)
- **Boards:** ThreadStates
- **Size:** M
- **Status:** open

## Summary

An agent thread in a tab has the states the ThreadStates board draws, but several look or read
differently. The width button counts in percent where the board uses fractions. A tool call that
waits for permission has no ASKS label, and its buttons carry the agent's own words. An agent's
HTML page shows as a plain tool call. New Thread with Context puts its preamble in the message box
with no counts. A Zed Agent with no model shows a callout above the thread, not a centred card.
The thread tab's right-click menu lacks Rename, Review Last Reply, Archive and Delete Thread…,
which only the rail row offers. This ticket draws and words each as the board does.

## Acceptance

The width button reads ⅓, ½, ⅔ or Whole window; a waiting tool call leads with ASKS and a tool
chip over Allow and Deny; an agent's page is a card with Show; New Thread with Context shows a
framed notice with its counts; a Zed Agent with no model shows a centred card; the tab's menu
offers Rename, Review Last Reply, Archive and Delete Thread…. Shot in Marley Night and Marley
Day.
