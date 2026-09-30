# Restart a service from its port row, and its state and logs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-615-restart-a-service-from-its-port-row.md
- **Pipeline spec:** 615-restart-a-service-from-its-port-row.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-603: Stop stops a listener's service; a refusal is a toast with the reason and Copy Command.
  - A row's key is its port and pid, which change on a restart; the row must be kept across the gap.
  - More hover buttons would clip the row's text further (#618), so the actions go in a menu.
  - `unit_state` covers user units only; a system unit needs `systemctl show` without `--user`.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.
