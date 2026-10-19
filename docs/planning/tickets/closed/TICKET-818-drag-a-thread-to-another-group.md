# TICKET-818 — A thread's row drags to another group

- **Ticket:** LOCAL #818 (bug, the Marley layout's rail)
- **Owner:** 6bc788ef-5565-4849-9e73-e341a6348e9f
- **Pipeline doc:** ../../pipeline/completed/818-drag-a-thread-to-another-group.spec.md
- **Source ticket:** Chad, 2026-10-19: "i cant drag a rusty or agent across terminal groups. … i
  opened a rusty in one and tried to move it to default and couldnt". Asked which threads should
  move: "Every thread".
- **Status:** closed

## Summary
#613 lets a terminal's row dropped on another group's header move the terminal there; a thread's
row (Rusty, Marley, an agent's) only reorders inside its own group (`DraggedRailRow::moves_to`
takes terminals alone). Since #734 a thread in a center tab is listed under the group whose
workspace holds the tab, so moving the tab moves the row.

## Acceptance
Any thread's row dropped on another open group's header, Default included, moves the thread
there, and it is still there after Marley starts again. The agent keeps the folder it works in.
