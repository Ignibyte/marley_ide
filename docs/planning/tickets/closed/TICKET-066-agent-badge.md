# TICKET-066 — agent status badge on its pane

- **Forge ticket:** #66 `b48ace07-d215-4994-8cdb-da70c75da4c1` (feature, M2.C seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `d567948c-1da4-4503-9362-86498d9aa0e2`
- **Pipeline doc:** ../../pipeline/active/agent-badge.spec.md
- **Source ticket:** forge sprint #11 `fc38f0c0-0704-40e6-9530-3402f8b4821c` (M2.C — The Living Cockpit)
- **Status:** closed

## Summary
Surface the invisible RootView.agents tag: a corner badge on each agent pane showing its kind + a status
glyph. PURE: `agent_status_glyph` (●/○/✓) + `agent_badge` (reusing launch_command). SHIM: the corner
badge in the pane render. cov/MSI 100 on agent_view; the render is masked + self-test-verified. Deps
#61 + #62.

## Acceptance
agent_status_glyph + agent_badge at cov/MSI 100 (each status→glyph; badge format label-before-glyph);
cmd-shift-a → the agent pane shows a "claude ○" badge (self-test capture); FULL gate GREEN. Full EARS in
the spec.
