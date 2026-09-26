# TICKET-553 — Whether an agent's commands enter the shell history, as a setting

- **Ticket:** LOCAL #553 (feature, prong 1 T7 with prong 2: the Warp second pass, "Smaller"; after #556)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/553-agent-commands-in-history.spec.md
- **Source ticket:** docs/planning/design-notes/warp-second-pass-2026-09-25.md ("Agent commands in the user's history"; Chad, 2026-09-26: every remaining finding gets built)
- **Status:** open

## Summary
Once `terminal_run` (#556) types an agent's commands at Chad's prompt, they enter his shell
history and Marley's autosuggestions like his own. Warp has a setting for it,
`include_agent_commands_in_history`, off by default. Marley gets
`marley.agent_commands_in_history`, on by default: when off, Marley types the agent's command
with a leading space and its shell integration keeps space-prefixed lines out of the history
(bash drops the entry after Marley's hook has read it, zsh refuses it in `zshaddhistory`), and
the autosuggestions skip blocks an agent ran. The shell rule is installed at spawn, so it applies
to terminals opened after the setting changes.

## Acceptance
With the setting off, a command `terminal_run` typed is absent from `history` and from the
history file in bash and zsh, and never shows as a suggestion, while the block keeps its command
and pill; with the setting on (the default), it is in both; a user's own space-prefixed line
behaves as the shell's own settings say when the setting is on. The full EARS criteria live in
the pipeline spec.
