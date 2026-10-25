# TICKET-907 — The bottom row for a thread

- **Ticket:** LOCAL #907 (feature, the Marley look, the bottom row)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [907-bottom-row-for-a-thread.spec.md](../../pipeline/completed/907-bottom-row-for-a-thread.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/A-core.md, board Main; docs/marley/design-review-2026-10-23/B-rail-chrome.md, board BottomRow)
- **Boards:** Main, BottomRow
- **Size:** M
- **Status:** closed

## Summary

When a thread tab is in front, the bottom row now says only the agent's name, a status word and
the thread's title ("Marley idle What changed in the tmux rows?"). The Main and BottomRow boards
draw much more: the agent's tile and name, the model and its effort, the mode, what the agent
does and for how long, a context meter, the files it changed with lines added and removed, and a
Full width toggle. This ticket draws that row. It also adds `thread_activity`, the one helper
that says what a thread's agent does or asks and since when; #910 reuses it for the rail's
thread rows.

## Acceptance

With a thread tab in front, the bottom row shows the agent tile and name, model and effort, mode,
what the agent does and for how long, the context meter, files changed (+/−) and a Full width
toggle that works; shot in Marley Night and Marley Day.
