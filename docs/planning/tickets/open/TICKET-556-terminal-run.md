# TICKET-556 — `terminal_run`: an agent runs commands in the user's terminal

- **Ticket:** LOCAL #556 (feature, prong 2 with prong 1: plan D9's `terminal.run`; the Warp blocks note, recommendation 3)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/556-terminal-run.spec.md
- **Source ticket:** docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md (item 4 of "What Marley would do"; Chad, 2026-09-26: every remaining finding gets built)
- **Status:** open

## Summary
A new tool on Marley's MCP server, `terminal_run`, types a command at a terminal's shell prompt
and answers with the block it made: exit code, duration and output, under `terminal_read`'s caps
and #516's redaction. It runs only when that terminal's shell waits at a verified prompt with
nothing typed, and nobody has taken the terminal over. Warp's two regex lists gate it with Warp's
defaults: a denylisted command shows as a card in that terminal and runs only on Run or Enter; an
allowlisted one runs at once; the rest run on the agent's own permission prompt unless a setting
says to ask. Every block an agent ran carries an agent mark on its pill, and Ctrl-I in that
terminal takes it over, so the tool refuses until the user hands back. Claude Code, Codex and the
Zed agent then get Warp's loop, and every command they run is a block Chad can copy, rerun or
send back.

## Acceptance
`terminal_run` on an idle terminal runs the command and returns its block; a denylisted command
waits for the card's Run; the allowlist and the setting decide the rest; the block's pill carries
the agent mark; Ctrl-I refuses the tool until hand-back; a half-typed line, a running program, a
forged prompt frame and a taken-over terminal each refuse with a reason. The full EARS criteria
live in the pipeline spec.
