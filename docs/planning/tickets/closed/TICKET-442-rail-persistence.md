# TICKET-442 — Rail persistence: a closed rail stays closed, one width for both layouts

- **Ticket:** LOCAL #442 (feature, workbench shell W6a)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/442-rail-persistence.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
A rail Chad closes opens again at every restart, since the Marley layout opens the rail when a
window is built, and its width is lost. The rail now records a close and its width in the
window's saved sidebar state, next to the fields of Zed's own sidebar, and a restored window
gets both back; the width is shared with Zed's sidebar across a layout switch. Split at
promotion (2026-09-23): W6's other features are TICKET-451 to TICKET-454, and the rail's
smaller internals are `docs/planning/intake/rail-internals.md`.

## Acceptance
A rail closed before a restart stays closed after it, and one left open stays open; a width set
in either layout holds in the other and through a restart. Full EARS in the completed spec.
Shipped 2026-09-23.
