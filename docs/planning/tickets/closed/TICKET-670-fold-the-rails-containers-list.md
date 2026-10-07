# TICKET-670 — Fold the rail's Containers list

- **Ticket:** LOCAL #670 (feature, the rail)
- **Owner:** claude-opus-5-5, 2026-10-06 (Chad's request)
- **Pipeline doc:** ../../pipeline/completed/670-fold-the-rails-containers-list.spec.md
- **Source ticket:** #614 (container ports in the rail), #669 (Containers in the Rail)
- **Status:** closed

## Summary
Chad, 2026-10-06, on his first session with the new build: "the containers showing up on the left
are huge in number. if containers are persistent there in that pane we need to put them in a
dropdown list that is collapsible". The dev box runs 13 containers, and the rail lists every one
under a CONTAINERS label after the projects, which pushes the Harness section down. The label
becomes a header with a chevron and the number of ports; a click folds or unfolds the list; it
starts folded, and the window remembers how it was left.

## Acceptance
The Containers header shows a chevron, CONTAINERS and the count; folded, no container row shows;
a click on the header or its chevron folds or unfolds it; a window starts with it folded, and a
restart brings it back as it was left.
