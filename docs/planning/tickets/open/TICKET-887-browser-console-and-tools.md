# TICKET-887 — The Browser's console drawer and tools as the boards draw them

- **Ticket:** LOCAL #887 (feature, the Marley look, W5)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-879 while planning it (2026-10-22): #879 made the picks a
  panel.
- **Boards:** Browser, BrowserTools
- **Size:** M
- **Status:** open

## Summary

The console drawer under the page (Console with its count, Network, Recordings, Send to the
agent), held clicks, dialogs, Playwright scripts, recordings, take over and clearing a project's
data; and a pick's "Goes with it" and "To", as the boards draw them.

## Acceptance

Each part looks as its board shows and keeps its behavior; shot in Marley Night and Marley Day.

## Notes

- The browser scenarios that click at fixed points (496, 505, 508, 518, 523) count tray rows
  above the page; since #879 the picks are a panel at the page's right and since #850 the toolbar's
  buttons are wider. Their points are measured again at the regression.
