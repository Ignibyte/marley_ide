# TICKET-779 — The permission settings reach every agent, and a thread switches its mode

- **Ticket:** LOCAL #779 (feature)
- **Owner:** 9623da6e-a5cd-4dcb-b048-40009324e9f0
- **Pipeline doc:** none yet
- **Source ticket:** Chad, 2026-10-12: "we need to allow --dangerously-skip-permissions (on
  claude) and full permissions (on codex)"; then "if the default settings allow bypass then the
  agents start with it. but the zed agent panel needs the ability to toggle through and change
  these permissions."
- **Status:** open

## Summary
`marley.claude_code_permissions` and `marley.codex_permissions` (#532) reach only the agent CLIs
Marley types into a terminal. With bypass or full access set, Claude Agent and Codex threads in
tabs start in that mode too, and so do the Marley and Rusty agents, which today refuse bypass
(`allowDangerouslySkipPermissions: false`, Codex read-only). A thread's mode switch cycles
through every mode its agent offers, bypass and full access included.

## Acceptance
With bypass set, a new Claude Agent thread, a Marley and a Rusty conversation start in bypass;
with full access set, a Codex thread starts in full access; a thread's mode switch reaches those
modes; with the settings at ask, every agent starts as today.
