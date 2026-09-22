# TICKET-441 — Terminal routing and keys in the Marley layout

- **Ticket:** LOCAL #441 (feature, workbench shell W5)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/441-marley-terminal-routing.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
In the Marley layout nothing should open the bottom Terminal Panel. A replacement TerminalProvider sends tasks to the center. New Terminal and Open in Terminal are caught before Zed's panel handlers. A Marley keymap, loaded from one line in load_default_keymap, binds terminal keys to Marley actions that do Zed's own thing in the Zed layout. A project opened with no terminal gets one at its root.

## Acceptance
In the Marley layout, tasks, New Terminal, Open in Terminal and ctrl-` all land in center terminals; in the Zed layout each behaves exactly as upstream. Full EARS in the queued spec.
