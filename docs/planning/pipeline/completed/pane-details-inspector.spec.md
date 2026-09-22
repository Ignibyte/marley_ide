---
pipeline_id: 9338a94e-3a8e-41eb-a6aa-f8492a17733b
ticket: forge#93 (a734efda-d9f1-46cb-a174-41c99471b33b) · local docs/planning/tickets/open/TICKET-093-pane-details-inspector.md
aar_id: 729d1893-0aee-4497-b7af-d0786824c8eb
status: Phase 5 — Complete PASS
title: rich focused-pane Details inspector
type: feature
milestone: M2.F — The Persistent Cockpit
references:
  - crates/marley_app/src/pane_details.rs (NEW PURE: DetailRow, agent/remote/terminal_details)
  - crates/marley_app/src/lib.rs (mod pane_details)
  - crates/marley_app/src/app.rs (SHIM: the Details-section dispatch)
---

## Title
The #90 Details tab becomes a live inspector of the FOCUSED pane, projecting the right rows per pane kind —
a terminal's running/last block, an agent's kind/status/ticket, or a remote's host/connection.

## Scope
### In
- NEW pure `pane_details.rs`: `DetailRow { label, value }` + `agent_details` + `remote_details` +
  `terminal_details` (each → `Vec<DetailRow>`).
- SHIM: the Details section dispatches by focused pane kind (agent / remote / terminal / empty) + renders
  the rows, recomputed each frame.

### Out
- Editing from the inspector. History of past blocks (just the last). Non-focused panes. Forge/Agents tabs.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — dispatch order: agents map → remotes map → focused last block → empty ("No command selected").
- D2 — the projectors reuse block_details (#58) / launch_command + agent_status_label (#66) / RemoteStatus
  (#85); conditional rows (ticket/last/exit/cwd/git) appear only when their field is set.
- D3 — `DetailRow.label` is `&'static str` (a fixed vocabulary); the value is the formatted String.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_details(run)` runs, it shall list agent+status, plus ticket/last when set. | unit |
| REQ-002 | WHEN `remote_details(host, status)` runs, it shall list the host + connected/disconnected. | unit |
| REQ-003 | WHEN `terminal_details(d)` runs, it shall list command+status, plus exit/cwd/git when set. | unit |
| REQ-004 (visual) | WHEN a pane of each kind is focused, the Details tab shall show that kind's rows. | self-test (static live + engine) |
| REQ-005 | gate GREEN, cov/MSI 100 on pane_details.rs; the shim masked. | gate |

## Phase Plan
- **P2** — pane_details.rs API; the app.rs Details dispatch/render; test plan.
- **P3** — implement (pane_details.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: each projector MSI (rows/pushes/match arms); the dispatch order; block_details reuse.
- **P4** — the 3 projector tests (cov/MSI 100) + gate GREEN + static live capture (synthetic focus env-blocked).
- **P5** — docs, AAR, archive, close #93.
