# A thread on a remote seat — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-742-a-thread-on-a-remote-seat.md
- **Pipeline spec:** 742-a-thread-on-a-remote-seat.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - #694 (the Manager entry through `rh acp`) and rustal-harness TICKET-111.
  - The harness: a non-manager seat has no thread; it has a fleet record, its retained output (`session_read`) and a message path (`session_send` → `deliver`, `delivery_state`). TICKET-116 decides where its replies come from.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.
