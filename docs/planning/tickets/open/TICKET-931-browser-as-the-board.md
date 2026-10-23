# TICKET-931 — The Browser tab as the board draws it

- **Ticket:** LOCAL #931 (feature, the Marley look, Browser)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [931-browser-as-the-board.spec.md](../../pipeline/queued/931-browser-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/F-tools.md, boards Browser and BrowserTools)
- **Boards:** Browser, BrowserTools
- **Size:** M
- **Status:** open

## Summary

The Browser tab works, but it is drawn differently from the Browser and BrowserTools boards in
many small places. A picked element loses its mark as soon as pick mode ends, so the user cannot
see on the page what they picked. The project is a muted word in the address box, not a chip.
The console drawer stays shut until asked for, its tab counts every entry, and an error row is
only a red word. A pick's card leads with "Pick 2", not the element's selector and source line,
and its Note is one line. "To" is two stacked rows, not a dropdown. Several bars and dialogs use
other words than the boards: the take-over bar, the page's own dialog, the clear-data dialog, the
new-script field and the annotate note. This ticket draws each of them as the boards do. The
recording timeline and the scripts' last run are #932.

## Acceptance

A pick keeps an accent mark with its selector badge on the page; the address box, console
drawer, pick card, To dropdown, take-over bar, page dialog, clear-data dialog, new-script field
and annotate note read and look as the boards draw them; shot in Marley Night and Marley Day.
