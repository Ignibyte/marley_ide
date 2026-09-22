# TICKET-082 — per-agent activity age ("quiet 5s") in the Fleet

- **Forge ticket:** #82 `3172ec5c-98c7-4b46-b32c-7906a0d08d30` (feature, M2.E seq-5 FINALE; sprint #13)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `090a234f-45ce-4539-9a08-814eb8df1a77`
- **Pipeline doc:** ../../pipeline/active/agent-quiet-age.spec.md
- **Status:** closed

## Summary
"quiet Ns"/"quiet Nm" age per agent in the Fleet, from #79's quiet_ticks. PURE `quiet_age(quiet_ticks)` +
`AgentRow.age` (cov/MSI 100); the Fleet row shows it. Deps #79 + #68. Closes M2.E.

## Acceptance
quiet_age at cov/MSI 100 (0/<60/>=60 + boundaries); the age flows through agent_rows; FULL gate GREEN.
Full EARS in the spec.
