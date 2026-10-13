# TICKET-777 — Archive that sticks, and an archived thread that comes back when opened

- **Ticket:** LOCAL #777 (bug)
- **Owner:** 9623da6e-a5cd-4dcb-b048-40009324e9f0
- **Pipeline doc:** [777-archive-that-sticks.spec.md](../../pipeline/completed/777-archive-that-sticks.spec.md)
- **Source ticket:** Chad, 2026-10-12: "I cant click archive on the left panel when a agent panel
  is over there. I can close it but the archive button doesnt work", and "the threads on the top
  left show the threads on the actual claude / codex stored locally. But we dont see the threads
  from the agents that run from inside the agent panel (marley etc)".
- **Status:** closed

## Summary
The rail's Archive on a thread row archives the thread, but the row comes straight back while the
thread's center tab is open: `group_threads` lists every tabbed thread from `store.entry`, which
ignores `archived`, and nothing closes the tab. A restart restores the tab too. No Marley opener
unarchives a thread (the Threads page, Reopen Thread…, Talk to Marley/Rusty, a restored tab), so a
thread archived this way never shows in the Threads view again. Chad's own store has every Marley,
Rusty and Claude Agent thread with a message archived. Delete has the same open-tab gap.

## Acceptance
Archive closes the thread's tab and takes its row away, and neither comes back after a restart;
opening an archived thread unarchives it, so the Threads view lists it again; Talk to Marley or
Rusty skips an archived conversation; deleting a thread closes its tab.
