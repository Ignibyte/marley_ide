# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-933](open/TICKET-933-tools-page-as-the-board.md) | feature | MCP Servers and Agent Activity as the board draws them (design review 2026-10-23) |
| [TICKET-934](open/TICKET-934-agent-activity-lists-rustys-calls.md) | feature | Agent Activity lists Rusty's calls (design review 2026-10-23) |
| [TICKET-935](open/TICKET-935-fleet-as-a-centre-tab.md) | feature | The Fleet tab with the snapshot beside the list (design review 2026-10-23) |
| [TICKET-936](open/TICKET-936-component-library-as-the-board.md) | feature | The Component Library as the board draws it (design review 2026-10-23) |
| [TICKET-937](open/TICKET-937-system-one-page.md) | feature | A System One page (design review 2026-10-23) |
| [TICKET-938](open/TICKET-938-agent-access-as-the-board.md) | feature | What agents may do, as the board draws it (design review 2026-10-23) |
| [TICKET-939](open/TICKET-939-rusty-tasks-knowledge-brain-as-the-board.md) | feature | Rusty's Tasks and brain pages as the board draws them (design review 2026-10-23) |
| [TICKET-940](open/TICKET-940-rusty-server-and-forms-as-the-board.md) | feature | Rusty's Server, Secrets and Skills as the board draws them (design review 2026-10-23) |
| [TICKET-941](open/TICKET-941-settings-and-keymap-as-the-board.md) | feature | Settings and the keymap as the board draws them (design review 2026-10-23) |
| [TICKET-942](open/TICKET-942-search-and-editor-popups-as-the-board.md) | feature | Search and the editor's pop-ups as the board words them (design review 2026-10-23) |
| [TICKET-943](open/TICKET-943-states-and-theme-as-the-board.md) | feature | Empty states and theme details as the board draws them (design review 2026-10-23) |
| [TICKET-944](open/TICKET-944-open-and-connect-as-the-board.md) | feature | Remote folders, layouts and About as the board draws them (design review 2026-10-23) |
| [TICKET-945](open/TICKET-945-system-pages-as-the-board.md) | feature | The system pages as the board draws them (design review 2026-10-23) |
| [TICKET-946](open/TICKET-946-forge-as-the-board.md) | feature | Forge's board and online agents as the board draws them (design review 2026-10-23) |
| [TICKET-794](open/TICKET-794-home-missing-after-a-restored-tab.md) | bug | Home goes missing when the Default group restores a tab on start (seen on the Mac with Onboarding) |
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | The first command in a new zsh on a Mac has no block header |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
