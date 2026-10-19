# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-812](open/TICKET-812-a-restored-blocks-header-takes-the-panes-click.md) | bug | A click on a block header in another pane focuses that pane |
| [TICKET-814](open/TICKET-814-a-new-assistant-agent-reaches-new-conversations-without-a-restart.md) | bug | A changed `marley.assistant.agent` reaches new Marley and Rusty conversations without a restart |
| [TICKET-809](open/TICKET-809-install-keeps-the-running-builds-remote-server.md) | bug | An install keeps the remote server of the Marley still running |
| [TICKET-806](open/TICKET-806-triage-and-repair-the-golden-set.md) | chore | The golden set: 19 of 29 stale scenarios repaired (parked); the 10 left, then the whole set |
| [TICKET-794](open/TICKET-794-home-missing-after-a-restored-tab.md) | bug | Home goes missing when the Default group restores a tab on start (seen on the Mac with Onboarding) |
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | The first command in a new zsh on a Mac has no block header |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
