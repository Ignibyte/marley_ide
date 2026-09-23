# TICKET-454 — A switcher over recent terminals and threads

- **Ticket:** LOCAL #454 (feature, workbench shell W6e)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (split from #442 at its promotion)
- **Source ticket:** ../../pipeline/completed/442-rail-persistence.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
`ctrl-tab` in the Marley layout opens a switcher over the window's terminals and threads in
most-recently-used order, through the rail's `toggle_thread_switcher` (the `Sidebar` trait's
hook, a no-op today) and a picker. While the rail is registered, Zed's `NextProject`,
`NextThread` and the thread switcher reach the trait's no-op defaults; this slice gives them
the rail's meaning. The switcher under a hovering pointer needs
`PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001`.

## Acceptance
`ctrl-tab` opens the switcher with the most recent entry after the current one selected; each
further press moves down the list, and releasing the modifier shows the selection. The EARS
criteria come at promotion (the draft is REQ-007 of #442's queued spec, in git history).
