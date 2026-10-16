# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-789](open/TICKET-789-marleys-keys-use-cmd-on-macos.md) | feature | Marley's own keys use Cmd on macOS through `secondary-`; shell keys stay Ctrl |
| [TICKET-791](open/TICKET-791-rustys-vault-when-rusty-runs-on-another-machine.md) | feature | Rusty's vault when the service runs on another machine: page files and the agent's folder |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
