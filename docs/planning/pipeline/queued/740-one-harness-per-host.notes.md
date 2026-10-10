# One harness per host — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-740-one-harness-per-host.md
- **Pipeline spec:** 740-one-harness-per-host.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - #534 (sessions in the rail), #632 (the embedded harness), #689 (writes), #691 (the seat form), #694 (the Manager), #710 (stop and remove).
  - rustal-harness D164: Marley reaches each other box over its own SSH connection; the harness does not join hosts into one fleet.
  - `Harness::seat_command` guesses the base from the position of `mcp`; the new entries name host, root and program instead.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.
