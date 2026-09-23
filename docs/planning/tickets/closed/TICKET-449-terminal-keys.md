# TICKET-449 — Terminal keys in the Marley layout

- **Ticket:** LOCAL #449 (feature, workbench shell W5b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/449-terminal-keys.spec.md
- **Source ticket:** ../../pipeline/completed/441-marley-terminal-routing.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
The terminal keys still reach the bottom Terminal Panel after #441: `` ctrl-` `` is Zed's
`terminal_panel::Toggle` and `ctrl-j` its `workspace::ToggleBottomDock`. In the Marley layout
`marley_workbench` catches those actions, and `terminal_panel::ToggleFocus`, in the capture
phase as #441 catches New Terminal, and switches between the code and the project's center
terminals instead. `ctrl-~` is Zed's `workspace::NewTerminal`, which #441 already routes.
Split at promotion (2026-09-23): catching the actions needs no keymap, so the Marley keymap of
workbench-shell D7 moved to TICKET-450 with the New Agent chord, the one binding with no Zed
action to catch.

## Acceptance
In the Marley layout `` ctrl-` `` focuses the project's last center terminal, or opens one,
and from a terminal goes back to the code; `ctrl-~` opens a new center terminal; `ctrl-j` does
what `` ctrl-` `` does while it would otherwise show the Terminal Panel. The bottom panel stays
closed throughout. In the Zed layout all three do what Zed's own bindings do. Full EARS in the
completed spec. Shipped 2026-09-23.
