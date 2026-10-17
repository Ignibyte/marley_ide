# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-801](open/TICKET-801-component-templates-from-files.md) | feature | Component templates from `~/.config/marley/components/`, no recompile |
| [TICKET-802](open/TICKET-802-an-agents-html-page-in-a-browser-tab.md) | feature | `browser_show_page`: an agent's HTML page in a Browser tab, themed, reloaded on change |
| [TICKET-803](open/TICKET-803-a-blocks-output-opens-as-a-table.md) | feature | A finished block that reads as CSV, TSV, JSON or a markdown table gets a Table chip that opens Zed's table view in a tab |
| [TICKET-804](open/TICKET-804-a-blocks-table-in-the-terminal.md) | feature | The detected table drawn in the terminal over the block's rows, toggled per block; `marley.block_tables` |
| [TICKET-805](open/TICKET-805-show-a-file-or-stdin-from-a-terminal.md) | feature | `show <file\|->` through `marley-agent`: a table, a component, markdown or an image in a tab |
| [TICKET-794](open/TICKET-794-home-missing-after-a-restored-tab.md) | bug | Home goes missing when the Default group restores a tab on start (seen on the Mac with Onboarding) |
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | The first command in a new zsh on a Mac has no block header |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
