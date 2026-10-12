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
| [TICKET-726](open/TICKET-726-plan-review-in-a-tab.md) | feature | after the finish, the review batch: plan review in a center tab (in slices) |
| [TICKET-727](open/TICKET-727-plan-versions-and-diff.md) | feature | the review batch: plan versions and a diff between them |
| [TICKET-728](open/TICKET-728-question-cards-in-plans.md) | feature | the review batch: question cards in plans |
| [TICKET-729](open/TICKET-729-notes-on-markdown-and-last-reply.md) | feature | the review batch: notes on any Markdown file or the agent's last reply |
| [TICKET-730](open/TICKET-730-agent-findings-as-diff-notes.md) | feature | the review batch: agent findings as diff notes over MCP |
| [TICKET-731](open/TICKET-731-viewed-marks-in-the-diff.md) | feature | the review batch: viewed marks in the project diff |
| [TICKET-733](open/TICKET-733-guided-review.md) | feature | the review batch: guided review |
| [TICKET-757](open/TICKET-757-claude-code-remote-control-switch.md) | feature | shelved (Chad: "yes and shelve"): a switch for Claude Code's Remote Control |
| [TICKET-719](open/TICKET-719-scheduled-runs.md) | feature | moved to rustal-harness (Chad: "harness yes"); Marley shows the runs once the harness has a scheduler |
