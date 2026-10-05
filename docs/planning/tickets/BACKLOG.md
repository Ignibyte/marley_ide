# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-655](open/TICKET-655-knowledge-panel-project-view.md) | feature | Rusty in Marley R6 · the Knowledge panel's project view: the project's brain page, follow-ups and task group, linked when none matches |
| [TICKET-656](open/TICKET-656-brain-page-outline-and-property-edits.md) | feature | Rusty in Marley R2b · the Page tab's outline, and the title, name and properties edited in place |
| [TICKET-657](open/TICKET-657-brain-graph-groups-and-forces.md) | feature | Rusty in Marley R5b · the Graph tab's colour groups, sliders and arrows, restored after a restart |
| [TICKET-658](open/TICKET-658-rusty-tasks-tab.md) | feature | Rusty in Marley R7 · the Tasks tab over Rusty's task tools |
| [TICKET-659](open/TICKET-659-rusty-decisions-tab.md) | feature | Rusty in Marley R7 · Rusty's Decisions tab; Marley's System One tab renamed "System One calls" |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-548](open/TICKET-548-system-one-via-cloudflare.md) | feature | prong 2 · Jev through Cloudflare Workers AI (stated zero retention) as a provider setting, per project; waits for the System One layer itself |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then). Chad, 2026-10-01: wait, so bugs found before the first release do not mean cutting several |
