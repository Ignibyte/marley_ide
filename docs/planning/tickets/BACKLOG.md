# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-892](open/TICKET-892-browser-recordings-send-and-downloads.md) | feature | The Browser drawer's Recordings and Send, a pick's options, downloads (W5, from #887) |
| [TICKET-888](open/TICKET-888-diff-review-and-turns.md) | feature | The diff review and a terminal's turns (W5, from #881) |
| [TICKET-889](open/TICKET-889-launch-menu-remote-and-block-tools.md) | feature | The Launch menu, remote terminals and the block tools (W5, from #882) |
| [TICKET-890](open/TICKET-890-port-rows-containers-collab.md) | feature | Port rows, the Containers panel and Collab (W5, from #883) |
| [TICKET-891](open/TICKET-891-search-branches-history-extensions.md) | feature | Project search, branches, history, extensions and the guide's Try it (W5, from #884) |
| [TICKET-856](open/TICKET-856-editor-pop-ups.md) | feature | The editor's pop-ups in the theme (W6) |
| [TICKET-857](open/TICKET-857-empty-and-error-states.md) | feature | Empty and error states (W6) |
| [TICKET-858](open/TICKET-858-motion-and-focus.md) | feature | Motion, attention and focus (W6) |
| [TICKET-859](open/TICKET-859-a-project-mark-in-notifications-and-the-switcher.md) | feature | A project's mark in its notifications and the switcher (W1.2 follow-up) |
| [TICKET-860](open/TICKET-860-a-remote-projects-logo-over-ssh.md) | feature | A remote project's logo, read once over SSH (W1.2 follow-up) |
| [TICKET-861](open/TICKET-861-a-sessions-page.md) | feature | The Sessions page: every tmux session as a table (W1.3 follow-up) |
| [TICKET-862](open/TICKET-862-the-panel-buttons-become-a-labeled-group.md) | feature | The panel buttons become one labeled group (W1.4 follow-up) |
| [TICKET-863](open/TICKET-863-cards-above-the-bottom-row-and-the-debugging-row.md) | feature | Cards above the bottom row, and the row while debugging (W1.4 follow-up) |
| [TICKET-864](open/TICKET-864-the-thread-plan-chips-terminal-peek-and-checks.md) | feature | The thread's plan, reply chips, terminal peek and checks (W2 follow-up) |
| [TICKET-865](open/TICKET-865-an-agent-panel-beside-a-terminal.md) | feature | An agent panel beside a terminal, and a running block's progress line (W2 follow-up) |
| [TICKET-866](open/TICKET-866-tabs-as-pills-and-themed-menus.md) | feature | Tabs as pills, and the menus in the theme (W2.1 follow-up) |
| [TICKET-867](open/TICKET-867-home-prompt-picks-its-project.md) | feature | Home's prompt picks its project (W2.2 follow-up) |
| [TICKET-794](open/TICKET-794-home-missing-after-a-restored-tab.md) | bug | Home goes missing when the Default group restores a tab on start (seen on the Mac with Onboarding) |
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | The first command in a new zsh on a Mac has no block header |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
