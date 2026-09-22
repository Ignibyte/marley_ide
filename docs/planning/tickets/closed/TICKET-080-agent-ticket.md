# TICKET-080 — agent ↔ ticket association (who's on what)

- **Forge ticket:** #80 `1b383541-41e3-44f6-9f56-956dafc8ffa1` (feature, M2.E seq-3; sprint #13)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `0b0b6422-1e04-43a9-b3b9-65140962cf2d`
- **Pipeline doc:** ../../pipeline/active/agent-ticket.spec.md
- **Status:** closed

## Summary
Remember which ticket an agent is on: `extract_ticket_ref(line)` (first #N) stored on the target
AgentRun when a ticket-mentioning line is sent/broadcast; shown in the Fleet row + badge. PURE
extract_ticket_ref + AgentRun.ticket (cov/MSI 100). Deps #72/#73 + #68 + #66.

## Acceptance
extract_ticket_ref + badge/row ticket at cov/MSI 100; a sent "#77" → the agent's row shows #77 (self-test/
engine); FULL gate GREEN. Full EARS in the spec.
