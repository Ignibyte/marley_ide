# TICKET-061 — marley_agent: the agent-run model

- **Forge ticket:** #61 `57ebab5f-adab-4d76-a202-0ab33ab3e56b` (feature, M2.B seq-3)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `0e27da39-7932-47a1-b222-f98ef51c28d6`
- **Pipeline doc:** ../../pipeline/active/agent-model.spec.md
- **Source ticket:** forge sprint #10 `4c988c99-56be-4434-8017-6909db864935` (M2.B — The Agent Cockpit)
- **Status:** closed

## Summary
A new pure `marley_agent` crate: `AgentKind` + `agent_kind_of` (recognize claude/codex from a command),
`AgentStatus` + `agent_status_from` (exited-first projection), `AgentRun { kind, label, status }`. The
cockpit foundation — #62 launches + tags a pane with an AgentRun. cov/MSI 100. No UI.

## Acceptance
agent_kind_of + agent_status_from + AgentRun::new at cov/MSI 100 (claude/codex/path/args/None;
exited-first; new→Idle); FULL gate GREEN. Full EARS in the pipeline spec.
