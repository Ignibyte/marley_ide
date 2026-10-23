# TICKET-919 — The Threads page as the board draws it

- **Ticket:** LOCAL #919 (feature, the Marley look, Threads)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [919-threads-page-as-the-board.spec.md](../../pipeline/queued/919-threads-page-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/C-threads.md, board Threads), build order line 7.
- **Boards:** Threads
- **Size:** M
- **Status:** open

## Summary

#836 built the Threads page's filters, day groups and preview, but the page still lists sparse
rows and says nothing about what each conversation is doing. The board draws each conversation
as a card: a 32 px agent tile with a state dot, the title and time on one line, then the agent
and the folder as pills. Its header counts the conversations and how many are open, and has a New
Agent button. The search has a magnifier and shorter words. Each WHO filter row has a tile and a
count. The preview says the conversation's state under its title and shows a grid of facts:
Folder, Files changed, Started and Mode. This ticket draws all of that. The conversation excerpt
and a snippet on every card stay out, as #836 decided.

## Acceptance

The Threads page shows cards with state dots, the header's count and New Agent, the new search
field, WHO rows with tiles and counts, and a preview with a state line and the facts grid; shot
in Marley Night and Marley Day.
