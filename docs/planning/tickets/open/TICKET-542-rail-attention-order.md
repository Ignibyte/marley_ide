# TICKET-542 — The rail puts what needs Chad first

- **Ticket:** LOCAL #542 (feature, prong 2 attention)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/542-rail-attention-order.spec.md
- **Source ticket:** Chad, 2026-09-25, on the Orca survey: "we will be taking what it does well and bring it in here" (docs/orca_architecture/README.md); the attention order is report 01 §2.4 and §3 item 5, and report 05 §2.10 and §3 item 4.
- **Status:** open
- **Backlog:** Queue, after #519. For agents in terminals the states it sorts on (waiting on Chad, working, failed, and when the agent last reported) come from #519's events; before them a terminal can only guess from two seconds of quiet.

## Summary
With several projects open, the rows that need Chad should come first without expanding every
project. The rail orders its projects, and the rows under each, by attention: needs you (a
permission or a question waiting, or a failed run not yet seen), then done and not yet seen, then
working, then not reporting (a working agent silent for 30 minutes), then idle. Ties keep today's
order, so Move Project Up and Down still order projects of the same class. A collapsed project's
header says what its agents are doing ("1 waiting, 2 working"). While the pointer is over the rail
the order holds still, so a row never moves under the pointer. A setting brings back the window's
own order.

## Acceptance
With a project whose agent waits on a permission, one whose agent finished unseen and one whose
agent works, the rail lists them in that order; a collapsed project reads "1 waiting"; an agent
that starts waiting while the pointer is over the rail moves up only once the pointer leaves.
