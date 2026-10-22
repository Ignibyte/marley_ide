# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-890](open/TICKET-890-port-rows-containers-collab.md) | feature | Port rows, the Containers panel and Collab (W5, from #883) |
| [TICKET-900](open/TICKET-900-inline-assist-offers-the-agent.md) | feature | Inline assist offers to send the selection to an agent (W6, from #899; needs an LLM stand-in) |
| [TICKET-895](open/TICKET-895-rail-shrink-motion-and-focus-ring.md) | feature | The rail's shrink in motion, and one focus ring (W6, from #858) |
| [TICKET-860](open/TICKET-860-a-remote-projects-logo-over-ssh.md) | feature | A remote project's logo, read once over SSH (W1.2 follow-up) |
| [TICKET-901](open/TICKET-901-the-debugging-row-and-ask-an-agent-about-a-server.md) | feature | The bottom row while debugging, and Ask an agent about a language server (W1.4, from #863) |
| [TICKET-866](open/TICKET-866-tabs-as-pills-and-themed-menus.md) | feature | Tabs as pills, and the menus in the theme (W2.1 follow-up) |
| [TICKET-867](open/TICKET-867-home-prompt-picks-its-project.md) | feature | Home's prompt picks its project (W2.2 follow-up) |
| [TICKET-794](open/TICKET-794-home-missing-after-a-restored-tab.md) | bug | Home goes missing when the Default group restores a tab on start (seen on the Mac with Onboarding) |
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | The first command in a new zsh on a Mac has no block header |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
