---
pipeline_id: 6471e72b-23b6-42fb-8ab1-863be04a95c1
ticket: forge#187 (ec3f3366-5130-4463-9157-eead7052982f) · local docs/planning/tickets/open/TICKET-187-agent-lifecycle.md
aar_id: 8ef178ef-0572-4e79-96b1-154eca183df6
status: Phase 5 — Complete PASS
title: M12 — agent lifecycle: exit code + duration + finish flash
type: feature
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_agent/src/lib.rs (AgentRun: run_ticks + exit_code)
  - crates/marley_app/src/agent_view.rs (PURE: fmt_duration, agent_finish_flash, the row ✓/✕+duration)
  - crates/marley_app/src/app.rs (SHIM: the pump captures ChildExited + marks Exited + flashes; run_ticks tick)
---

## Title
"Did my agent finish, and how?" — when an agent's shell exits, its Agents-tab row stays, marked ✓/✕ with
the run duration, and a "{label} finished (exit N, 2m14s)" flash fires once. Today a dead agent lingers
with a stale glyph.

## Scope
### In
- `marley_agent` AgentRun: `run_ticks: u32` (lifetime, +1/pump-tick while alive) + `exit_code: Option<i32>`
  (Some once exited). `new()` inits both.
- PURE `agent_view.rs`: `fmt_duration(secs) -> "45s"/"2m14s"/"1h03m"`; `agent_finish_flash(label, code, secs)`;
  AgentRow gains `exit: Option<(i32, u64)>` (populated from run); `agent_rows` picks ✕ for a non-zero exit
  (else the status glyph); `agent_row_text` appends " · exit N · DUR" when exited.
- SHIM `app.rs`: the pump extracts ChildExited's ExitCode; a dead AGENT pane is marked Exited + exit_code +
  the flash (once, guarded on exit_code.is_none()) and its row is KEPT (the automatic reap no longer removes
  an exited agent); run_ticks increments each tick for a live agent (in refresh_agent_statuses).

### Out
- A "clear finished agents" affordance (exited rows accumulate for the session — a follow-up); wall-clock
  time (tick-count only — Date::now is banned).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `fmt_duration` shall bucket secs → "Ns"/"NmSSs"/"NhMMm" at the 60/3600 boundaries (mutation-killed). | unit |
| REQ-002 | `agent_finish_flash` + the row's exit suffix + the ✕-on-failure glyph shall render from (label, code, secs). | unit |
| REQ-003 (visual) | WHEN an agent's shell exits, its Agents-tab row shall stay marked ✓/✕ + duration and a finish flash shall fire. | driven capture |
| REQ-004 | gate GREEN; the pure fns cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the AgentRun fields + pure fns + the pump. P3.5 1 critic (the exit-once guard + the borrow;
run_ticks lifecycle; the kept-row vs the reap; the ExitCode extraction). P4 unit + driven + gate. P5 docs.
