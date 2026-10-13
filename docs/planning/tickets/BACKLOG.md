# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-778](open/TICKET-778-rustys-agent-in-the-title-bar.md) | feature | The title bar's Rusty button opens Rusty's agent; Rusty's home page back in the status bar |
| [TICKET-779](open/TICKET-779-permission-modes-for-every-agent.md) | feature | Bypass and full access reach threads and the Marley and Rusty agents; a thread cycles its modes |
| [TICKET-780](open/TICKET-780-marley-and-rusty-folders.md) | feature | Marley and Rusty start in folders set in settings, never asked |
| [TICKET-781](open/TICKET-781-zed-agent-blank-in-the-agent-panel.md) | bug | Zed Agent shows blank; Select Model opens the old right panel |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
