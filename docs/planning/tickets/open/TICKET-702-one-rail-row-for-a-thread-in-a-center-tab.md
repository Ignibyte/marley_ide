# TICKET-702 — One rail row for a thread in a center tab

- **Ticket:** LOCAL #702 (bug)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** found in #697's visual check; Chad, 2026-10-09, told the session to proceed on
  the default ("go ahead and work on the remaining items you have open as well").
- **Status:** open

## Summary
While an agent thread sits in a center tab (#697's `ThreadTab`), the rail lists it twice: as the
center tab's row (#674) and as the thread row ("hello · Marley · idle"). The thread row stays, since
it carries the agent, its status and the unread dot; the center-tab row for a `ThreadTab` goes.
Clicking the thread row already brings the tab forward (`show_thread` → `thread_tab::activate_for`).

## Acceptance
- While a thread is in a center tab, the rail lists one row for it, the thread row.
- Clicking that row brings the thread's tab forward.
