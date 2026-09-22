---
pipeline_id: 969a433c-6bd6-4844-859a-9ebbebb1227f
ticket: forge#110 (ee9a310e-a2bf-437e-aac5-625c2d8f955c) · local docs/planning/tickets/open/TICKET-110-workspace-groups.md
aar_id: 87fd8c72-820d-492b-85b5-e9ceee19174d
status: Phase 5 — Complete PASS
title: workspace groups in the sidebar
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/sessions.rs (PURE: SessionGroup, group_sessions; Session.workspace)
  - crates/marley_app/src/app.rs (SHIM: grouped sidebar render)
---

## Title
Group the sessions sidebar under named workspace headers (like Warp's Rusty / Forge / Dev N) — sessions in
the same workspace cluster under one header.

## Scope
### In
- PURE: `Session.workspace` + `SessionGroup` + `group_sessions(sessions, focused)` (first-seen group order,
  reusing `session_rows`).
- SHIM: the sidebar renders a header per group + its rows.

### Out
- Multiple real workspaces (all sessions are one project for now; multi-workspace arrives with multi-project M3).
- Collapsing/reordering groups. Renaming a workspace.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — group order is first-seen; within a group the session order is kept; `group_sessions` reuses
  `session_rows` per group (so #109's row mapping stays the single source of truth + live).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `group_sessions` runs, it shall bucket sessions by `workspace` in first-seen order, rows kept in order. | unit |
| REQ-002 | WHEN a session is focused, its row within its group shall be active. | unit |
| REQ-003 (visual) | WHEN the app renders, the sidebar shall show a named group header + its session rows. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on group_sessions; the shim masked. | gate |

## Phase Plan
- **P2** — Session.workspace + SessionGroup + group_sessions; the grouped render; test plan.
- **P3** — implement (sessions.rs + app.rs; update #109 test helper).
- **P3.5** — 1 critic: group_sessions MSI (grouping/first-seen/order/active); session_rows reused.
- **P4** — group_sessions tests (cov/MSI 100) + a live capture + gate GREEN.
- **P5** — docs, AAR, archive, close #110.
