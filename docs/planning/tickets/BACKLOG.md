# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-673](open/TICKET-673-containers-panel-on-the-right.md) | feature | the machine's containers leave the rail for a right-dock panel with its button in the status bar |
| [TICKET-674](open/TICKET-674-every-center-tab-in-the-rail.md) | feature | every center tab gets a rail row; a project's files fold under a Files row above its containers |
| [TICKET-675](open/TICKET-675-the-rusty-group.md) | feature | a Rusty group in the rail, never closed, where every Rusty screen and page opens |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then). Chad, 2026-10-01: wait, so bugs found before the first release do not mean cutting several |
