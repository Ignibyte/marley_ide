# TICKET-081 — jump to an agent from the Fleet overlay

- **Forge ticket:** #81 `8dbd37dc-4fa9-48db-942c-328855f717ac` (feature, M2.E seq-4; sprint #13)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `43b13570-fb50-4410-ba6f-49e9e0e39435`
- **Pipeline doc:** ../../pipeline/active/agent-jump.spec.md
- **Status:** closed

## Summary
Click a ⌘⇧E Fleet row → focus that agent's pane + close the overlay. PURE `AgentRow.pane` threaded through
agent_rows (cov/MSI 100); the clickable row → workspace.focus + fleet_open=false + flash. Deps #68 +
workspace.focus + #77.

## Acceptance
AgentRow.pane threads at cov/MSI 100; a Fleet-row click focuses the pane + closes (self-test/engine); FULL
gate GREEN. Full EARS in the spec.
