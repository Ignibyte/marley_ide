# TICKET-937 — A System One page

- **Ticket:** LOCAL #937 (feature, the Marley look, System One)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [937-system-one-page.spec.md](../../pipeline/completed/937-system-one-page.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/F-tools.md, board SystemOne)
- **Boards:** SystemOne
- **Size:** M
- **Status:** closed

## Summary

The SystemOne board draws one page for System One: its name, a line saying what it is, the
Provider and Today cards, then each use with its Off / Shadow / Suggest / Act mode, then the
calls under their own subheading. Marley has the cards and the calls in a tab titled "System One
calls" with no opening line, and the use modes only in the Settings window. A failed call is red
text, where the board tints its row. This ticket turns the calls tab into the System One page:
the title and opening line, the ten uses with their modes on the page, the calls subheading, and
failed calls tinted. The Settings window keeps its rows.

## Acceptance

`marley: open system one calls` opens a "System One" page with the opening line, the Provider and
Today cards, every use with its four modes (a click changes the mode), and the calls under
"System One calls · newest first · what left the machine is shown whole" with a failed call
tinted; shot in Marley Night and Marley Day.
