# TICKET-180 — M12: Agents cockpit output tail

- **Forge ticket:** #180 `81ab5a37-70df-4cea-8993-ee0bcb804f78` (feature; sprint #23)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `8d34ce20-8dd8-4e60-ac2a-952bc5da42f9`
- **Pipeline doc:** ../../pipeline/active/agent-tail.spec.md
- **Status:** closed

## Summary
pure agent_tail(output, n) + the Agents cockpit rows render a live 6-line output tail per agent (read via
grids().find_map, kept fresh by the #173 pump). The observe core. Deps #173, #174, #78.
