# TICKET-441 — Terminal routing and keys in the Marley layout

- **Ticket:** LOCAL #441 (feature, workbench shell W5)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/441-marley-terminal-routing.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
In the Marley layout nothing should open the bottom Terminal Panel. A replacement TerminalProvider sends tasks to the center, and New Terminal and Open in Terminal are caught before Zed's panel handlers. Split at promotion (2026-09-22): the Marley keymap moved to TICKET-449, and the first-show terminal and the Panel Layout presets moved to W6 (TICKET-442).

## Acceptance
In the Marley layout, tasks, New Terminal and Open in Terminal land in center terminals; in the Zed layout each behaves exactly as upstream. Full EARS in the completed spec. Shipped 2026-09-22.
