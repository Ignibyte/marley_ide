# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-895](open/TICKET-895-rail-shrink-motion-and-focus-ring.md) | feature | The rail's shrink in motion, and one focus ring (W6, from #858) |
| [TICKET-860](open/TICKET-860-a-remote-projects-logo-over-ssh.md) | feature | A remote project's logo, read once over SSH (W1.2 follow-up) |
| [TICKET-901](open/TICKET-901-the-debugging-row-and-ask-an-agent-about-a-server.md) | feature | The bottom row while debugging, and Ask an agent about a language server (W1.4, from #863) |
| [TICKET-794](open/TICKET-794-home-missing-after-a-restored-tab.md) | bug | Home goes missing when the Default group restores a tab on start (seen on the Mac with Onboarding) |
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | The first command in a new zsh on a Mac has no block header |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
