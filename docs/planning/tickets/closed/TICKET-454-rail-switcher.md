# TICKET-454 — A switcher over recent terminals and threads

- **Ticket:** LOCAL #454 (feature, workbench shell W6e)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/454-rail-switcher.spec.md
- **Source ticket:** ../../pipeline/completed/442-rail-persistence.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
`ctrl-tab` in the rail and in the Agent Panel opens a switcher over the window's terminals and
threads, most recently shown first, through the rail's `toggle_thread_switcher` (the `Sidebar`
trait's hook, a no-op today, so `ctrl-tab` in the Agent Panel does nothing in the Marley
layout). The center panes keep Zed's tab switcher, as in Zed's layout. Narrowed at promotion
(2026-09-23): Next and Previous Project and Thread are TICKET-459, and the draft's Warp
citation is dropped, since the design note it cited describes clicks only. The switcher under a
hovering pointer needs
`PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001`.

## Acceptance
`ctrl-tab` opens the switcher with the most recent entry after the current one selected; each
further press moves down the list, and releasing the modifier opens the selection. Full EARS
in the active spec.
