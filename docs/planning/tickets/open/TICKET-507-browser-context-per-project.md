# TICKET-507 — A browser context per project

- **Ticket:** LOCAL #507 (feature, prong 3)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/507-browser-context-per-project.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 4 (second half) of the list after the browser waves)
- **Status:** open

## Summary
Every Browser tab shares one Chromium profile, so a login made for one project is a login for all of them, and two projects on localhost trample each other's cookies. Each project gets its own browser context, created over CDP, and its cookies are kept per project across restarts.

## Acceptance
Tabs of two projects hold separate cookies; a project's login survives a restart of Marley.
