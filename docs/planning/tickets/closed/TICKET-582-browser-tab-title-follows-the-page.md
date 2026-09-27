# TICKET-582 — A Browser tab's title follows a title its page's script sets

- **Ticket:** LOCAL #582 (bug, prong 3)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/582-browser-tab-title-follows-the-page.spec.md
- **Source ticket:** found in #523's Plan, 2026-09-27: the release build's golden set failed #507's
  and #581's scenarios, whose page set its title after an IndexedDB read
- **Status:** closed

## Summary
Marley reads a page's title when its main frame fires DOMContentLoaded or load, or moves within
its document, and from `Target.targetInfoChanged`. Chromium sends no event when a script sets
`document.title` later: a scratch Chromium 152 with discovery on reported the first title and
nothing when a timer changed it (2026-09-27). So a page that retitles itself after it loads, an
inbox's unread count, a dev server's build state or a page that sets its title after fetching
data, keeps its old title in its Browser tab, in the rail's row and in `browser_tabs`. The
debug build usually read the title after such a page had set it; the faster release build reads
it before, which is how the golden set found it. #523 changed the scenarios' login site to move
within its document after setting its title, so they no longer depend on this.

## Acceptance
WHEN a page's script changes its `document.title` after the page loaded, the system shall show
the new title in the page's Browser tab, its rail row and `browser_tabs` within a second.
