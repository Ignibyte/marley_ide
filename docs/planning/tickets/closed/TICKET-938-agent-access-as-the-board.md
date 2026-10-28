# TICKET-938 — What agents may do, as the board draws it

- **Ticket:** LOCAL #938 (feature, the Marley look, agent access)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [938-agent-access-as-the-board.spec.md](../../pipeline/completed/938-agent-access-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/F-tools.md, board AgentAccess)
- **Boards:** AgentAccess
- **Size:** M
- **Status:** closed

## Summary

When an agent asks to edit an open editor, Marley's question says "<who> wants to use
editor_edit" and "change <file>". The AgentAccess board shows the agent's tile, "Claude Code
wants to edit rail.rs", the task and project, the change itself as removed and added lines, and
the footnote "One undo step, left unsaved. Saving asks again." Nothing in Marley shows which
files are kept from agents or that keys in terminal output are redacted; the board has a "Kept
from agents" section for that. The Agent Control modes read "Ask Every" where the board says "Ask
Every Time", and Outside Clients' levels read Read / Write / Run where the board says Read / Read
and Write / Read, Write and Run. This ticket builds the consent question, the "Kept from agents"
section and the wording. The pairing card is a follow-up ticket.

## Acceptance

An agent's `editor_edit` asks with its tile, "<who> wants to edit <file>", the task and project,
the change as −/+ lines and the footnote; the Agent Activity tab shows "Kept from agents" with the
globs and the redaction note; the modes read "Ask Every Time" and the client levels read the
board's words; shot in Marley Night and Marley Day.
