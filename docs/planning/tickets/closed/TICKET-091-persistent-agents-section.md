# TICKET-091 — persistent Agents section in the dock

- **Forge ticket:** #91 `09e43d8c-d681-4353-b32a-3ace356981e3` (feature, M2.F seq-2; sprint #14)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `2890437b-be54-4b15-abe8-d8692206dfd0`
- **Pipeline doc:** ../../pipeline/active/persistent-agents-section.spec.md
- **Status:** closed

## Summary
The Fleet always-visible in the #90 Agents tab. PURE `agent_row_text(&AgentRow)` (extracted from the masked
Fleet render) + `agents_empty_hint()` (cov/MSI 100); the Agents-section renders clickable agent_rows →
focus (reuse #81), or the hint. Deps #90 + #68/#78-82 + #81 + #77.

## Acceptance
agent_row_text + agents_empty_hint at cov/MSI 100; the Agents tab lists agents/the hint (static live +
engine); FULL gate GREEN. Full EARS in the spec.
