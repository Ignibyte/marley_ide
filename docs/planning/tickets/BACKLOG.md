# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-703](open/TICKET-703-agent-control-layer.md) | feature | The agent-control layer: tiers, consent, an activity log and a kill switch |
| [TICKET-704](open/TICKET-704-editors-read-and-open.md) | feature | Editors over MCP: list, read and open |
| [TICKET-705](open/TICKET-705-editors-edit.md) | feature | Editors over MCP: edit and save |
| [TICKET-706](open/TICKET-706-agent-panel-threads.md) | feature | Agent Panel threads over MCP |
| [TICKET-707](open/TICKET-707-palette-actions.md) | feature | Palette actions over MCP |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then). Chad, 2026-10-01: wait, so bugs found before the first release do not mean cutting several |
