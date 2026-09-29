# TICKET-525 — An agent reads and types into a running program

- **Ticket:** LOCAL #525 (feature, prong 2 with prong 1: plan D9's terminal tools)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/525-agent-drives-a-running-program.spec.md
- **Source ticket:** Chad, 2026-09-25, on Warp's Full Terminal Use: "love this idea lets do it" (`docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 1; the Orca survey's receipted input, `docs/orca_architecture/06-cli-automations-skills.md` §2.6 and item 2)
- **Status:** closed

## Summary
Marley's terminal tools only read blocks, so an agent cannot see what psql, gdb, a Python REPL
or a dev server shows right now, or answer its prompt. Two tools on Marley's MCP server fix that.
`terminal_screen` reads a terminal's live screen: its rows, the cursor, the program in the
foreground and who controls the terminal. `terminal_type` types text and keys into the program
running there, behind a `terminal.write` grant and Marley's own approval: asked on the first
write to each program by default, on every write, or never, a setting on the Marley page. A bar
under the terminal shows what the agent typed, and Ctrl-I there takes over: the agent's writes
are refused until Chad hands back. Each write names the generation it read, as rustal-harness's
managed input does, so a write meant for a screen that has since changed hands, or for a
program that has exited, types nothing. The shell's own prompt and agent CLIs are refused: those
are `terminal.run` and the session verbs, later tickets.

## Acceptance
An agent reads sqlite3's screen in a terminal and types a query that runs there once Chad allows
the first write; with the terminal taken over, its writes are refused until he hands back.
