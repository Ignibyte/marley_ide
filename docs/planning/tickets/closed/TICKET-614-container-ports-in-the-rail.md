# TICKET-614 — Container ports in the rail

- **Ticket:** LOCAL #614 (feature, workbench shell, the rail's ports (after #521, #603))
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/614-container-ports-in-the-rail.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** closed

## Summary
A port a Docker or Podman container publishes belongs to root's `docker-proxy` or a rootless helper, which the rail does not list today (#603 deferred it). The rail lists such ports under the project whose folder the container mounts, or under a Containers section, named by the container, and Stop stops the container.

## Acceptance
WHILE a container publishes a port on this machine, the rail shall list the port named by its container, and Stop shall stop the container.
