# TICKET-602 — Drag to reorder the rail

- **Ticket:** LOCAL #602 (feature, workbench shell: the rail)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/602-drag-to-reorder-the-rail.spec.md
- **Source ticket:** Chad, 2026-09-30: "We should add the ability to drag and drop the items on the left". He chose reorder only for the first version: projects and groups up and down, and rows within their own group; the order is saved.
- **Status:** closed

## Summary
The rail reorders projects only through a header's right-click Move Project Up and Down (#453),
one step at a time, and rows not at all: terminals follow their tab order and threads their last
activity. Dragging makes the order Chad's: a project's or group's header drags up or down among
the headers, and a terminal, Browser tab or thread row drags up or down within its own group, with
a line showing where it will land. The order is saved and comes back after a restart. Under the
attention order (#542), the order Chad set breaks ties within each attention class.

## Acceptance
Drag the second project above the first, and a terminal row above the one before it: both stay
there, and both are in that order after a restart. A row dragged onto another group goes back to
its place.
