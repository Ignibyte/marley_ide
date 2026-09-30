# TICKET-618 — A port row's clipped lines, and a header tooltip over its menu

- **Ticket:** LOCAL #618 (bug, workbench shell, the rail (polish after #603, #606))
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/618-port-row-lines-and-header-tooltip.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** open

## Summary
Two small faults from Test notes: a service port row's URL and unit lines clip at the default rail width while its hover buttons keep their room (#603), and a project header's tooltip, showing when the header is right-clicked, stays over the just-opened menu's first entry until the pointer moves (#606).

## Acceptance
WHILE a port row shows at the default rail width, its URL and unit shall read whole or end in an ellipsis with the whole text in its tooltip; WHEN a header's menu opens, its tooltip shall close.
