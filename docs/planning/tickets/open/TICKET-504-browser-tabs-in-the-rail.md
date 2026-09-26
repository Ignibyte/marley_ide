# TICKET-504 — Browser tabs as rows of their project in the rail

- **Ticket:** LOCAL #504 (feature, prong 3 with the rail)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/504-browser-tabs-in-the-rail.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 4 (first half) of the list after the browser waves)
- **Status:** open

## Summary
The rail lists a project's terminals and agent threads but not its Browser tabs, so a page an agent opened is found only by hunting through the center's tabs. Each Browser tab becomes a row under its project, with the page's title, its host and its loading state; a click shows the tab.

## Acceptance
A project with Browser tabs lists each as a row with its title and host; clicking the row activates the tab; a closed tab's row goes.
