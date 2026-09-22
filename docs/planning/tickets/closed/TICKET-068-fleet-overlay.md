# TICKET-068 — Fleet overlay (cmd-shift-e)

- **Forge ticket:** #68 `0690368e-6fbe-419d-aa20-13f51977a0f4` (feature, M2.C seq-3)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `a577e1a8-9793-4265-8678-6c2f1aa0b05a`
- **Pipeline doc:** ../../pipeline/active/fleet-overlay.spec.md
- **Source ticket:** forge sprint #11 `fc38f0c0-0704-40e6-9530-3402f8b4821c` (M2.C — The Living Cockpit)
- **Status:** closed

## Summary
A ⌘⇧E Fleet overlay listing every launched agent + its status. PURE: `agent_status_label` + `AgentRow` +
`agent_rows` (sorted by pane id) + the keymap. SHIM: the overlay (mirrors the forge overlay). cov/MSI 100
on agent_view + keymap; the overlay is masked + self-test-verified. Deps #61 + #62 + #66.

## Acceptance
agent_status_label + agent_rows + keymap at cov/MSI 100 (each status→label; 2 agents → rows sorted by id;
empty→empty; cmd-shift-e→toggle-fleet); ⌘⇧E lists the fleet (self-test); FULL gate GREEN. EARS in the spec.
