---
pipeline_id: cfc5c372-e1f0-4c39-b01f-161f600580be
ticket: forge#79 (ece6e649-2dbd-4540-bc85-14f58bfff70d) · local docs/planning/tickets/open/TICKET-079-agent-waiting.md
aar_id: f2380298-2fa3-428a-a819-532dc70d15ed
status: Phase 5 — Complete PASS
title: richer agent status — waiting-for-input
type: feature
milestone: M2.E — The Observing Cockpit
references:
  - crates/marley_agent/src/lib.rs (AgentStatus::Waiting, agent_status_from(quiet_ticks), WAITING_TICKS, AgentRun.quiet_ticks)
  - crates/marley_app/src/agent_view.rs (glyph/label for Waiting)
  - crates/marley_app/src/app.rs (SHIM: quiet_ticks tracking in refresh)
---

## Title
A running agent that's been quiet (no new output) for a while is probably at its prompt waiting for you —
show a WAITING status (◔), distinct from actively Working (●).

## Scope
### In
- PURE (marley_agent): `AgentStatus::Waiting`; `agent_status_from(exited, active, quiet_ticks)` (new 3rd
  param) → Waiting when active + quiet_ticks >= `WAITING_TICKS` (=60, ~1s); `AgentRun.quiet_ticks: u32`.
- agent_view.rs: glyph ◔ + label "waiting" for Waiting.
- SHIM (app.rs): track quiet_ticks per agent (reset when last_line changes, else +1); feed to agent_status_from.

### Out
- Distinguishing "waiting for approval" vs "waiting for a prompt" (both Waiting). Tuning is heuristic.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — arm order: Exited → (!active) Idle → (quiet_ticks >= WAITING_TICKS) Waiting → Working.
- D2 — `WAITING_TICKS = 60` (~1s of silence at the 16ms pump tick — reuses #77's tick clock).
- D3 — quiet detection reuses #78's last_line: quiet_ticks resets to 0 when the freshly-read last_line
  differs (output changed → active), else saturating_add(1).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_status_from(exited, active, quiet_ticks)` runs, it shall be Exited / Idle(!active) / Waiting(active & quiet>=WAITING_TICKS) / Working. | unit |
| REQ-002 | WHEN quiet_ticks is 59 vs 60, it shall be Working vs Waiting (the boundary). | unit |
| REQ-003 | WHEN status is Waiting, its glyph shall be ◔ and label "waiting". | unit |
| REQ-004 (visual) | WHEN an agent goes quiet, its Fleet/badge shall show ◔ waiting; on output, ● working. | self-test (env-blocked → engine) |
| REQ-005 | gate GREEN, cov/MSI 100 on the pure surface; the shim masked. | gate |

## Phase Plan
- **P2** — AgentStatus::Waiting + agent_status_from(quiet_ticks) + WAITING_TICKS + glyph/label; AgentRun.
  quiet_ticks + the pump reset; caller updates; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: agent_status_from MSI (4-arm order + boundary), the glyph/label, the reset heuristic,
  all callers updated.
- **P4** — the pure tests (cov/MSI 100 incl. 59/60) + gate GREEN + self-test (env-blocked → engine).
- **P5** — docs, AAR, archive, close #79.
