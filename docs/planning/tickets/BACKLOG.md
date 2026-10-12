# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-716](open/TICKET-716-delegation-tools.md) | feature | after the finish: agents that start agents, tracked and never lost; waits on a harness request (seat labels, an ended state) |
| [TICKET-757](open/TICKET-757-claude-code-remote-control-switch.md) | feature | shelved (Chad: "yes and shelve"): a switch for Claude Code's Remote Control |
| [TICKET-719](open/TICKET-719-scheduled-runs.md) | feature | moved to rustal-harness (Chad: "harness yes"); Marley shows the runs once the harness has a scheduler |
