---
pipeline_id: 9a3effb1-f95a-4876-996f-994989c29b01
ticket: forge#82 (3172ec5c-98c7-4b46-b32c-7906a0d08d30) · local docs/planning/tickets/open/TICKET-082-agent-quiet-age.md
aar_id: 090a234f-45ce-4539-9a08-814eb8df1a77
status: Phase 5 — Complete PASS
title: per-agent activity age ("quiet 5s") in the Fleet
type: feature
milestone: M2.E — The Observing Cockpit (FINALE)
references:
  - crates/marley_app/src/agent_view.rs (PURE: quiet_age; AgentRow.age)
  - crates/marley_app/src/app.rs (SHIM: the Fleet row age render)
---

## Title
Show how long each agent has been quiet ("quiet 5s" / "quiet 1m") in the Fleet, so you can tell a
briefly-thinking agent from a long-stalled one. Pairs with #79's Waiting. Closes M2.E.

## Scope
### In
- PURE `quiet_age(quiet_ticks) -> String` (secs = ticks/62; 0→""; <60→"quiet Ns"; else "quiet Nm").
- `AgentRow.age: String` (= quiet_age(run.quiet_ticks), computed in agent_rows).
- SHIM: the Fleet row shows the age in the status segment.

### Out
- Wall-clock time (Date::now banned — tick count only). A live-ticking display beyond the pump cadence.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — ~62 ticks/sec (16ms pump); secs==0 → "" (active/<1s); <60s → seconds; >=60s → minutes.
- D2 — the age is computed in agent_rows (reuses #79's quiet_ticks) → `AgentRow.age`.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `quiet_age(quiet_ticks)` runs, it shall be "" (<1s), "quiet Ns" (<60s), or "quiet Nm". | unit |
| REQ-002 | WHEN quiet_ticks crosses 62 / 3720, the bucket shall switch ""→s and s→m. | unit |
| REQ-003 (visual) | WHEN an agent sits quiet, its Fleet row shall show "quiet Ns" climbing. | self-test (env-blocked → engine) |
| REQ-004 | gate GREEN, cov/MSI 100 on the pure surface; the shim masked. | gate |

## Phase Plan
- **P2** — quiet_age + AgentRow.age (agent_rows) + the render; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: quiet_age MSI (divisor, 0/60 boundaries, s-vs-m); the age copy.
- **P4** — quiet_age + age-copy tests (cov/MSI 100) + gate GREEN + self-test (env-blocked → engine).
- **P5** — docs, AAR, archive, close #82. **M2.E COMPLETE.**
