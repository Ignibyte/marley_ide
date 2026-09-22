# TICKET-104 — the agent-diff view ("what did this agent change?")

- **Forge ticket:** #104 `9acb6d71-1661-4727-ab57-ca07ab288f29` (feature, M4 seq-8; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `7bea4cd9-8d1f-492b-a3d5-107bc18f573f`
- **Pipeline doc:** ../../pipeline/active/agent-diff.spec.md
- **Status:** closed

## Summary
⌘-click an agent (Agents dock section) → the working-tree diff + a `N files · +A −R` summary. PURE
`agent_diff_summary(files)` (cov/MSI 100); the ⌘⇧D header uses it. v1 = the current working diff. Deps
#102 + #103 + #91 + #67.

## Acceptance
agent_diff_summary at cov/MSI 100 (empty/len/counts); ⌘-click an agent opens the diff (engine/live git);
FULL gate GREEN. Full EARS in the spec.
