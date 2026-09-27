# TICKET-504 — Browser tabs as rows of their project in the rail

- **Ticket:** LOCAL #504 (feature, prong 3 with the rail)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/504-browser-tabs-in-the-rail.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 4 (first half) of the list after the browser waves), with the Orca survey's details for #504 folded in (`docs/orca_architecture/README.md`, "What it changes in the queued sprint"; report 03 §2.1 and item 8)
- **Status:** closed

## Summary
The rail lists a project's terminals and agent threads but not its Browser tabs, so a page an
agent opened is found only by hunting through the center's tabs. Each Browser tab becomes a row
under its project, after its terminals: the page's favicon (a spinner while it loads, the globe
when it has none), its title, its host and port, how many picks and annotations it holds, and a
mark when an agent acted in it since the user last looked. A click shows the tab; the row goes
when the tab closes. The rail never starts Chromium on its own.

## Acceptance
A project with Browser tabs lists each as a row with its favicon, title and host; a loading page
shows a spinner; picks, annotations and an unseen agent action show on the row; clicking the row
activates the tab and clears the mark; a closed tab's row goes.
