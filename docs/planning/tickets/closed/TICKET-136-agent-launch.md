# TICKET-136 — launch an agent session from the new-session affordance [M7 seq-5]

- **Forge ticket:** #136 `a5c4b8e1-548f-4df0-8ef4-34fd6126c2d4` (feature, M7; sprint #18)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `2f7c8955-0e7a-4bee-9274-8a8de5f309f1`
- **Pipeline doc:** ../../pipeline/active/agent-launch.spec.md
- **Status:** closed

## Summary
A 🧠 agent-launch icon in the top bar (next to the "+") → `dispatch_action("new-agent")` — reuses the tested
M2.D agent launch. Launch a terminal OR an agent from the top bar. Shim-only. **Closes M7 sprint #18.**
Deps #135 + M2.D.

## Acceptance
The 🧠 icon renders next to the "+" (live capture); click launches an agent (code-reviewed; new-agent path
tested); FULL gate GREEN. Full EARS in the spec.
