# TICKET-442 — Rail persistence and polish

- **Ticket:** LOCAL #442 (feature, workbench shell W6)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/442-rail-polish.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
The finish on the rail. A rail Chad closes stays closed across restarts, and terminal rows can be renamed and closed from the rail. The rail gains keyboard navigation and a filter field, and ctrl-tab switches recently used terminals and threads. Projects can be reordered from the header menu, and the rail's width is shared with Zed's sidebar blob.

## Acceptance
The rail's closed state and width survive a restart; rename, close, keyboard navigation, the filter and the switcher each work from the keyboard and the pointer. Full EARS in the queued spec.
