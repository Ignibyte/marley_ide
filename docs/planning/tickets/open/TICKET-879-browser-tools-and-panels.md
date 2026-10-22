# TICKET-879 — The Browser's panels and tools as the boards draw them

- **Ticket:** LOCAL #879 (feature, the Marley look, W5)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-850 while planning it (2026-10-21): #850 kept the toolbar.
- **Boards:** Browser, BrowserTools
- **Size:** M
- **Status:** open

## Summary

The picks panel ("Picks for the agent", a note, what goes with it), the console drawer (Console,
Network, Recordings, Send to the agent), held clicks, dialogs, Playwright scripts, recordings,
take over and clearing a project's data, as the boards draw them.

## Acceptance

Each part looks as its board shows and keeps its behavior; shot in Marley Night and Marley Day.

## Notes

- #850 made Pick, Annotate and Record labelled buttons, which are wider than the icons were. The
  scenarios that click toolbar buttons at fixed points (496, 505, 508, 518, 523) need those
  points measured again at the regression.
