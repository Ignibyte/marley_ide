# TICKET-079 — richer agent status: waiting-for-input

- **Forge ticket:** #79 `ece6e649-2dbd-4540-bc85-14f58bfff70d` (feature, M2.E seq-2; sprint #13)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `f2380298-2fa3-428a-a819-532dc70d15ed`
- **Pipeline doc:** ../../pipeline/active/agent-waiting.spec.md
- **Status:** closed

## Summary
A Waiting(◔) agent status: a running agent quiet (no new output) for >WAITING_TICKS is at its prompt. PURE
`agent_status_from(exited, active, quiet_ticks)` + `AgentStatus::Waiting` + `AgentRun.quiet_ticks` (cov/MSI
100); the pump tracks quiet_ticks (reset on last_line change). Deps #67 + #78 + #66/#68.

## Acceptance
agent_status_from + glyph/label at cov/MSI 100 (incl. the 59/60 boundary); a quiet agent → ◔ (self-test/
engine); FULL gate GREEN. Full EARS in the spec.
