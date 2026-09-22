---
pipeline_id: fd023ad0-d8a5-4442-97db-02bb60e78204
ticket: forge#162 (a1b62026-cb96-4d82-8ebb-56b8de86600f) · local docs/planning/tickets/open/TICKET-162-project-close.md
aar_id: 3af93b17-5174-4795-95b6-b91e1c26d0f7
status: Phase 5 — Complete PASS
title: M10 — project close (a rail × per project row; refuse the last)
type: feature
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/app.rs (SHIM: close_project_at + the Project-row ×; mirrors #161's tab-× pattern)
---

## Title
Projects can be closed from the rail — an × per project row; the last project refuses with a flash; the
surviving project's Files/branch/cwd resync.

## Scope
### In
- SHIM `app.rs` only: `close_project_at(idx)` (close_project → thread-drop the whole removed Project +
  `sync_active_project()` + persist; `Err(LastProject)` → a flash); the Project row gains the #161-style ×
  (flex_row, label keeps the switch click, the × stop_propagation's then closes).

### Out
- New pure fns (Workspace::close_project + adjust_active are already cov/MSI 100 from #150).
- Confirm-before-close; reopen; persisting the multi-project shape (that's #163).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — mirror #161's × pattern EXACTLY (stop_propagation before the action; thread-drop reap; a refusal flash)
  — one affordance language across the rail.
- D2 — `sync_active_project()` runs after a successful close so Files/branch/cwd follow the surviving active
  project (chosen by the tested `adjust_active`).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN a project row's × is clicked and another project exists, that project shall close — its rail subtree disappears and the Files/branch/titlebar resync to the survivor. | driven capture (picker-permitting; see the #156 harness note) |
| REQ-002 (visual) | WHEN the ONLY project's × is clicked, a refusal flash shall show and nothing closes. | driven capture |
| REQ-003 | The × shall not trigger the row's switch click (stop_propagation), and the removed project's PTYs shall be dropped off the UI thread. | critic + code |
| REQ-004 | gate GREEN (no new pure surface; the shim masked). | gate |

## Phase Plan
- **P2** — the two shim pieces (folded into this doc — the pattern is #161's, already inspected).
- **P3** — implement. **P3.5** — 1 critic (reap scope: a whole Project drop; sync ordering; × routing).
- **P4** — gate + driven (the refusal solo-provable; 2-project close if the picker cooperates).
- **P5** — docs, AAR, close #162, archive.
