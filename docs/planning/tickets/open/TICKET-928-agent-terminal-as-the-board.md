# TICKET-928 — An agent CLI's terminal as the board draws it

- **Ticket:** LOCAL #928 (feature, the Marley look, terminal)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [928-agent-terminal-as-the-board.spec.md](../../pipeline/queued/928-agent-terminal-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/A-core.md, board AgentTerminal)
- **Boards:** AgentTerminal
- **Size:** M
- **Status:** open

## Summary

When an agent asks Marley to run a command in a terminal, the user gets a one-row bar with Run,
Refuse and "<who> wants to run <cmd>". It does not say where the command runs, whether it is on
the ask list or on neither list, or which keys answer it, and it offers no way to stop being
asked. When an agent types into a program, the bar repeats what it typed and shows Take Over
without its key. The rich input has no row saying which keys do what. This ticket draws the
waiting command as the AgentTerminal board's card, with Always run … here, words the typing bar
as the board does with its key, and adds the rich input's key hints.

## Acceptance

A waiting command shows a card with who asks, the folder, the list status, the command on its
own line, Run · Enter, Refuse · Esc and Always run <prefix> here, which stops the asking for that
prefix in that folder; the typing bar reads "<agent> is typing into this terminal's program" with
Take Over · Ctrl I; the rich input shows its key hints. Shot in Marley Night and Marley Day.
