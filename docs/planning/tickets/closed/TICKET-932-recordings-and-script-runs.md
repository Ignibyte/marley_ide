# TICKET-932 — Recordings with a timeline and Draft a Test, and scripts' last run

- **Ticket:** LOCAL #932 (feature, the Marley look, Browser)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [932-recordings-and-script-runs.spec.md](../../pipeline/completed/932-recordings-and-script-runs.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/F-tools.md, board BrowserTools)
- **Boards:** BrowserTools
- **Size:** L
- **Status:** closed

## Summary

Record This saves the page's last minute and says so in a toast. The user never sees what the
minute holds. The BrowserTools board draws the minute instead: a bar chart of its activity, the
steps at their times before the save (−0:41 Open…, −0:22 Click…), then "Draft a test" and "Send
to the agent". Drafting a test exists today only as the agent tool `browser_draft_test` (#506).
The board's Playwright scripts also say when each one last ran and how it went, and Marley keeps
no run history. This ticket shows a recording's minute in the drawer's Recordings tab, with a
Draft a test button that opens the same draft the agent gets, and keeps each script's last run
and result in the tray.

## Acceptance

Record This opens the minute in the drawer: bars, steps, Draft a test and Send to the agent;
Draft a test opens the drafted test in an unsaved editor tab; each script in the tray shows when
it last ran and passed, failed or not run; shot in Marley Night and Marley Day.
