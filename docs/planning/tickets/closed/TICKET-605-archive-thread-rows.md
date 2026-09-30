# TICKET-605 — Archive an agent thread from the rail

- **Ticket:** LOCAL #605 (feature, workbench shell: the rail's thread rows)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/605-archive-thread-rows.spec.md
- **Source ticket:** Chad, 2026-09-30: "Is there any way to close the agent conversation on the left?" Thread rows (#439) have no close button and no menu; only Zed's layout's Threads Sidebar archives a thread today.
- **Status:** closed

## Summary
A Zed agent thread's row stays in the rail for as long as the thread exists. Pointing at a thread
row now shows an Archive button in the slot where a terminal row shows its close button, and a
right-click opens a menu with Archive Thread. Archiving is what Zed's own thread history does: the
thread leaves the rail and the Agent Panel's list, and stays in Zed's archive, from which Zed can
restore it.

## Acceptance
Point at a thread row and click Archive: the row goes and does not come back after a restart; the
thread is in the Agent Panel's archived threads.
