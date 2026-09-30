# Container ports in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-614-container-ports-in-the-rail.md
- **Pipeline spec:** 614-container-ports-in-the-rail.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-603 defers containers: their ports belong to root's `docker-proxy`, which the rail does not list.
  - The trap: `service_of` would read `docker-proxy`'s cgroup as `docker.service`, and Stop would stop Docker itself.
  - L-claude-603-a-scenario-fakes-a-program-for-marley-through-setups-path-001: fakes first on the PATH reach Marley.
  - F-claude-521: the scan runs only while a rail watches.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.
