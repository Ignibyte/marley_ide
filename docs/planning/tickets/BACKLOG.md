# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-794](open/TICKET-794-home-missing-after-a-restored-tab.md) | bug | Home goes missing when the Default group restores a tab on start (seen on the Mac with Onboarding) |
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | The first command in a new zsh on a Mac has no block header |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-825](open/TICKET-825-agent-thread-with-panels.md) | feature | Design board 1 of the Warp-like canvas; Chad picks which board goes first |
| [TICKET-826](open/TICKET-826-terminal-blocks-with-an-agent-beside.md) | feature | Design board 2 of the Warp-like canvas; Chad picks which board goes first |
| [TICKET-827](open/TICKET-827-rail-rows-say-their-state.md) | feature | Design board 3 of the Warp-like canvas; Chad picks which board goes first |
