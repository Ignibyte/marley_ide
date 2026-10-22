# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-886](open/TICKET-886-harness-rail-runs-and-seat-form.md) | feature | The rail's Harness section, scheduled runs and the seat form (W5, from #878) |
| [TICKET-887](open/TICKET-887-browser-console-and-tools.md) | feature | The Browser's console drawer and tools (W5, from #879) |
| [TICKET-881](open/TICKET-881-plan-review-turns-and-diff-review.md) | feature | Plan review, turns and the diff review (W5, from #851) |
| [TICKET-882](open/TICKET-882-launch-remote-and-block-tools.md) | feature | The Launch menu, remote terminals and the block tools (W5, from #852) |
| [TICKET-883](open/TICKET-883-switcher-ports-containers-collab.md) | feature | The Ctrl Tab switcher, port rows, Containers and Collab (W5, from #853) |
| [TICKET-884](open/TICKET-884-search-branches-history-extensions-guide.md) | feature | Project search, branches, history, extensions and the Guide (W5, from #854) |
| [TICKET-885](open/TICKET-885-more-system-pages.md) | feature | The other system pages (W5, from #855) |
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
