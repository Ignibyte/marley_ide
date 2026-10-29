# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-946](open/TICKET-946-forge-as-the-board.md) | feature | Forge's board and online agents as the board draws them (design review 2026-10-23) |
| [TICKET-948](open/TICKET-948-knowledge-panel-as-the-board.md) | feature | The Knowledge panel as the board draws it (split from #939) |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | Not seen on Linux (2026-10-29); needs the current build on the Mac |
| [TICKET-949](open/TICKET-949-a-zsh-given-with-arguments-loses-the-integration.md) | bug | Found checking #795; a choice of what a shell with arguments gets |
