# TICKET-604 — A click on a port row opens nothing

- **Ticket:** LOCAL #604 (feature, workbench shell: the rail's port rows)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/604-port-row-click-does-not-open.spec.md
- **Source ticket:** Chad, 2026-09-30: "Whats odd about the playwright though is that it opens as a service and a browser. What causes this to automatically open in marley?" A click on a port row's name or URL opens its URL in a Browser tab (#521); reaching for the row's Stop button that way opened Playwright's server, which has no page, three times.
- **Status:** open

## Summary
A port row opens its URL in a Browser tab on any click of its body, which is easy to do by
accident while aiming at its hover buttons, and wrong for a listener that serves no page. A single
click now only marks the row, as the rail's keyboard cursor; Open (the row's button), a
double-click and Enter open it. Nothing else about port rows changes.

## Acceptance
A single click on a port row starts no Browser tab and marks the row; a double-click, Enter on the
marked row, or the Open button opens its URL as before.
