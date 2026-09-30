# TICKET-615 — Restart a service from its port row, and its state and logs

- **Ticket:** LOCAL #615 (feature, workbench shell, the rail's ports (after #603))
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/615-restart-a-service-from-its-port-row.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** open

## Summary
#603 made a port row served by a systemd user unit name the unit and stop it. It deferred restarting it and seeing its state and logs. The row gains Restart, a state word (active, failed, restarting), and Show Logs, which opens `journalctl --user -u <unit> -f` in a terminal.

## Acceptance
WHEN the user chooses Restart on a service port row, Marley shall restart its unit and the row shall show the unit's state.
