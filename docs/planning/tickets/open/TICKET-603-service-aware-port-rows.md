# TICKET-603 — Port rows that know a service

- **Ticket:** LOCAL #603 (feature, workbench shell: the rail's port rows)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/603-service-aware-port-rows.spec.md
- **Source ticket:** Chad, 2026-09-30: "When i stop the web servers they come back open ... I wonder if we treat these differently". The Playwright server on :3101 was a system unit with `Restart=always`; Stop's SIGTERM ended it and systemd started it again five seconds later, twice.
- **Status:** open

## Summary
A port row (#521) treats every listener alike, and Stop sends SIGTERM. A listener that a systemd
service runs comes back when the unit restarts it, and one whose unit does not restart it stays
down with systemd none the wiser. The row now names the service behind a listener, and Stop stops
the service: a user service through `systemctl --user stop`, a system service through `systemctl
stop` with the desktop's authorization prompt, and when that cannot be done, a message naming the
unit with the command to copy.

## Acceptance
A listener run by a user service shows the unit's name on its row; Stop stops the unit and the row
goes and stays gone. A listener run by a system service shows its unit; Stop asks for
authorization, and refused or unavailable, says so and copies `sudo systemctl stop <unit>`.
