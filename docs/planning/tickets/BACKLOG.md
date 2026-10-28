# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-944](open/TICKET-944-open-and-connect-as-the-board.md) | feature | Remote folders, layouts and About as the board draws them (design review 2026-10-23) |
| [TICKET-945](open/TICKET-945-system-pages-as-the-board.md) | feature | The system pages as the board draws them (design review 2026-10-23) |
| [TICKET-946](open/TICKET-946-forge-as-the-board.md) | feature | Forge's board and online agents as the board draws them (design review 2026-10-23) |
| [TICKET-947](open/TICKET-947-pairing-card-as-the-board.md) | feature | The pairing card as the board draws it (split from #938) |
| [TICKET-948](open/TICKET-948-knowledge-panel-as-the-board.md) | feature | The Knowledge panel as the board draws it (split from #939) |
| [TICKET-794](open/TICKET-794-home-missing-after-a-restored-tab.md) | bug | Home goes missing when the Default group restores a tab on start (seen on the Mac with Onboarding) |
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | The first command in a new zsh on a Mac has no block header |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
