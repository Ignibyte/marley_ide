# TICKET-067 — drive AgentStatus live from the pane state

- **Forge ticket:** #67 `a12d6977-0e9a-4402-84fb-8f0509caa635` (feature, M2.C seq-2)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `277e763d-5448-4f71-be43-0cebdbe9a70d`
- **Pipeline doc:** ../../pipeline/active/agent-status-live.spec.md
- **Source ticket:** forge sprint #11 `fc38f0c0-0704-40e6-9530-3402f8b4821c` (M2.C — The Living Cockpit)
- **Status:** closed

## Summary
Make the #66 badge live: each pump tick recomputes an agent pane's status via `agent_status_from`
(is_command_running → ● Working / prompt → ○ Idle). SHIM-only (a mutants::skip `refresh_agent_statuses`
+ the pump-tick call); reuses #61's tested pure fn (no new unit tests). Also fixes the #62 agents-map
leak on pump-close. Deps #66 + #61 + #40.

## Acceptance
The badge shows ● while an agent runs vs ○ at its prompt (self-test capture of ○→●); the agents entry is
removed when a pane auto-closes; FULL gate GREEN (app shim excluded). Full EARS in the spec.
