# TICKET-452 — Rename and close terminals from the rail

- **Ticket:** LOCAL #452 (feature, workbench shell W6c)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/452-rail-rename-and-close.spec.md
- **Source ticket:** ../../pipeline/completed/442-rail-persistence.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
A terminal row's context menu and a double-click rename the terminal through
`TerminalView::set_custom_title`, which Zed persists. A row's `×` and its context menu close
the terminal item, with Zed's usual prompt while a process runs. Warp's session list renames
and closes the same way (`docs/planning/design-notes/session-tabs-vs-sidebar.md`).

## Acceptance
A renamed terminal shows the new title in its row and tab, and keeps it through a workspace
reload; closing a row closes its terminal, prompting first while a process runs. The EARS
criteria come at promotion (the draft is REQ-003 and REQ-004 of #442's queued spec, in git
history before its promotion).
