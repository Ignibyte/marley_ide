# TICKET-073 — broadcast a prompt to all agents

- **Forge ticket:** #73 `a466ec24-4271-409f-b932-9f380753edb5` (feature, M2.D seq-2)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `3f9e50e2-fc4c-4d1d-9a9e-1714a5d65d9d`
- **Pipeline doc:** ../../pipeline/active/broadcast-agents.spec.md
- **Source ticket:** forge sprint #12 `c93f9693-5569-4208-bb6a-20d38afec99b` (M2.D — The Controlling Cockpit)
- **Status:** closed

## Summary
cmd-shift-g broadcasts the focused pane's composed line to every running agent (the fleet counterpart of
#72). PURE: `agent_pane_ids` (sorted targets) + the keymap. SHIM: send_payload to each agent's session;
clear the prompt only on confirmed delivery to ≥1 (the #72 F1 lesson). cov/MSI 100 on agent_pane_ids +
keymap; the broadcast masked + self-test-verified. Deps #72 + #68.

## Acceptance
agent_pane_ids sorted (cov/MSI 100, 3-scrambled) + keymap cmd-shift-g→broadcast-to-agents; 2 agents both
receive + prompt clears (self-test); FULL gate GREEN. Full EARS in the spec.
