---
pipeline_id: ab430a81-634b-4385-a367-427d724de8c6
ticket: forge#94 (111a4867-649d-462f-86b5-f84b6b3e04c6) · local docs/planning/tickets/open/TICKET-094-cockpit-status-bar.md
aar_id: 58ee5207-3dc8-47d5-a184-8db01dd3f58b
status: Phase 5 — Complete PASS
title: the cockpit status bar (sprint · agents · focus)
type: feature
milestone: M2.F — The Persistent Cockpit
references:
  - crates/marley_app/src/status_bar.rs (NEW PURE: agent_summary, sprint_summary, cockpit_status)
  - crates/marley_app/src/lib.rs (mod status_bar)
  - crates/marley_app/src/app.rs (SHIM: the footer strip render)
---

## Title
A thin persistent footer strip under the panes that always shows the global cockpit state at a glance:
the active sprint + ticket count, the agent-fleet summary, and the focused pane.

## Scope
### In
- NEW pure `status_bar.rs`: `agent_summary` ("N agent[s] · M working" / "no agents"), `sprint_summary`
  ("{name} · {n} tickets" / "no sprint"), `cockpit_status(sprint, agents, focused)` → 3 ordered segments.
- SHIM: a bottom footer strip fed each frame from forge_sprint + agents + the focused pane label.

### Out
- Clickable segments. Per-segment icons. A config to hide it. Dock-state persistence (#95).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `agent_summary`: empty → "no agents"; else "{n} agent[s] · {m} working" (pluralize on n==1; m =
  Working count #67).
- D2 — `cockpit_status` fixed order: sprint, agents, focus.
- D3 — focused_label = the focused pane's agent label / remote host / "terminal".

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_summary(agents)` runs, it shall be "no agents" (empty) or "{n} agent[s] · {m} working". | unit |
| REQ-002 | WHEN `sprint_summary(sprint)` runs, it shall be "{name} · {n} tickets" (Some) or "no sprint" (None). | unit |
| REQ-003 | WHEN `cockpit_status(...)` runs, it shall return [sprint, agents, focus] in order. | unit |
| REQ-004 (visual) | The footer strip shall show the cockpit status at the bottom. | self-test (static live) |
| REQ-005 | gate GREEN, cov/MSI 100 on status_bar.rs; the shim masked. | gate |

## Phase Plan
- **P2** — status_bar.rs API; the footer render + focused_label; test plan.
- **P3** — implement (status_bar.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: the 3 fns MSI (0/pluralization/working-count/Some-None/order); the footer + label.
- **P4** — the 3 fn tests (cov/MSI 100) + gate GREEN + LIVE static capture (the footer renders on launch).
- **P5** — docs, AAR, archive, close #94.
