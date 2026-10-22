# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-847](open/TICKET-847-system-one.md) | feature | System One restyled (W5) |
| [TICKET-848](open/TICKET-848-fleet-and-harness.md) | feature | Fleet, harness sessions and scheduled runs (W5) |
| [TICKET-849](open/TICKET-849-components.md) | feature | The Component Library and Component tabs (W5) |
| [TICKET-850](open/TICKET-850-browser-tab-and-tools.md) | feature | The Browser tab and its tools (W5) |
| [TICKET-851](open/TICKET-851-worktrees-and-review.md) | feature | Worktrees, plan review, turns and diff review (W5) |
| [TICKET-852](open/TICKET-852-workflows-and-launch.md) | feature | Workflows, launch configs and remote terminals (W5) |
| [TICKET-853](open/TICKET-853-splits-and-ports.md) | feature | Splits, the switcher, ports, containers and collab (W5) |
| [TICKET-854](open/TICKET-854-search-and-more.md) | feature | Project search, recent, branches, history, extensions, the guide (W5) |
| [TICKET-855](open/TICKET-855-system-pages.md) | feature | New pages: this build, data folders, troubleshooting (W5) |
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
