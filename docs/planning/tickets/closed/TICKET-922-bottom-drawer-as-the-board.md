# TICKET-922 — The bottom drawer as the board draws it

- **Ticket:** LOCAL #922 (feature, the Marley look, the bottom drawer)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [922-bottom-drawer-as-the-board.spec.md](../../pipeline/completed/922-bottom-drawer-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/D-panels-editor.md, board BottomPanel), build order line 9.
- **Boards:** BottomPanel
- **Size:** L
- **Status:** closed

## Summary

#838 made the bottom dock a drawer with Problems and Debug tabs, but Problems is still Zed's
multibuffer of excerpts and the drawer looks like Zed's flush dock. The board draws Problems as a
list: a "Filter problems" field, then one row per problem with a severity dot, the message, the
place and a button that asks an agent to fix it. Each tab carries a pill (the problem count, red
when there are errors; "paused" or "running" for Debug). The strip ends with the drawer's own
Move to the side, Maximize and Close buttons. The drawer is a rounded card inset from the window's
edges, with a grab handle on its top edge. This ticket builds all of that. The Output and Tasks
tabs and the debugger's own panes stay out, as #838 decided.

## Acceptance

Problems lists one row per problem with a filter, opens a row's place and asks an agent to fix
it; the tabs carry pills; Move to the side, Maximize and Close act on the drawer; the drawer is an
inset card. Shot in Marley Night and Marley Day.
