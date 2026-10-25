# TICKET-909 — The bottom row for the other screens

- **Ticket:** LOCAL #909 (feature, the Marley look, the bottom row)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [909-bottom-row-for-other-screens.spec.md](../../pipeline/completed/909-bottom-row-for-other-screens.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/C-threads.md,
  F-tools.md and E-rusty.md; boards Threads, Browser, Tools, Components, Tasks, Knowledge,
  BrainPage)
- **Boards:** Threads, Browser, Tools, Components, Tasks, Knowledge, BrainPage
- **Size:** M
- **Status:** closed

## Summary

The bottom row says what is in front for a file, a terminal and a thread. For every other screen
it says nothing. The boards give each screen its own short row: a chip naming the screen, a few
facts, and a pink or red chip for what needs the user. This ticket adds that row for the Threads
page, a Browser tab, MCP Servers, the Component Library, and Rusty's Tasks, Graph and brain page
tabs. The Fleet is a dock panel, not a tab, so its row is a follow-up.

## Acceptance

With each of the seven screens in front, the bottom row shows the board's chips for it, read from
what the screen already holds; shot per screen, and once in Marley Day.
