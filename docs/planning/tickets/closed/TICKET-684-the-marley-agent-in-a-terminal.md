# TICKET-684 — The Marley agent in a terminal

- **Ticket:** LOCAL #684 (feature, prong 2 C)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** ../../pipeline/completed/684-the-marley-agent-in-a-terminal.spec.md
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 1, item 6
- **Status:** closed

## Summary
The same Marley agent as TICKET-683, in a Marley terminal tab for someone who prefers Claude
Code's own interface: `claude --append-system-prompt-file <instructions> --disallowedTools Edit
Write NotebookEdit Bash --mcp-config <Marley's server>`, opened from the command palette and from
the Agent Panel's Marley entry. It uses TICKET-683's detection and instructions and opens only
while `marley.assistant` is on.

## Acceptance
"Open the Marley agent in a terminal" starts Claude Code in a new terminal tab with Marley's tools
and instructions; it answers from Marley's docs, proposes a setting change through the diff, and
is refused when it tries to edit a file or run a command.
