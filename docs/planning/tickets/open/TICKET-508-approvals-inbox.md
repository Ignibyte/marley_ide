# TICKET-508 — One approvals inbox in the rail

- **Ticket:** LOCAL #508 (feature, prong 2 (attention))
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/508-approvals-inbox.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 6 of the list after the browser waves)
- **Status:** open

## Summary
Agents ask for permission in different places: Claude Code in a terminal, the Zed Agent and external agents in the Agent Panel. Marley gathers every pending permission prompt into one list at the top of the rail, each naming its agent and project, and a click goes to the prompt.

## Acceptance
A Claude Code permission prompt in a terminal and a pending tool call in the Agent Panel both show in the rail's inbox; clicking an entry shows its prompt; an answered prompt leaves the list.
