---
pipeline_id: 47c29107-2a58-4728-9106-58a36cbbda8d
ticket: forge#152 (43343e48-3d19-4a04-b921-4bdd0c565325) · local docs/planning/tickets/open/TICKET-152-rail.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: M9 seq-3 — the rail (Workspace→Project→Tab tree, click-to-switch)
type: feature
milestone: M9 — Workspace / Project / Tab model
references:
  - crates/marley_app/src/tabs.rs (PURE: RailLevel, RailRow, rail_rows)
  - crates/marley_app/src/app.rs (SHIM: render the rail rows + click→switch + highlight; retire group_sessions here)
---

## Title
The left rail becomes the real switcher — a Workspace → Project → Tab tree with the active tab highlighted;
click a tab to bring it full-screen.

## Scope
### In
- PURE `tabs.rs`: `RailLevel`, `RailRow`, `rail_rows(&Workspace<S>)` (the tree projection).
- SHIM `app.rs`: render `rail_rows` (indented headers + tab rows, active highlighted); a tab row click →
  `switch_project` + `switch_tab`. Keep the "Search tabs" box + dock chrome.

### Out
- Nested split-pane rows under a terminal tab (seq-6). Multi-project *opening* (seq-7 — but rail_rows already
  renders ≥2 projects). Live tab titles / real branch (seq-8). The right-dock cockpit (seq-4).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `rail_rows` emits: one Workspace row (`ws.name`), then for each project a Project row + its Tab rows.
  `active`: a Project row is active iff it is the active project; a Tab row is active iff its project is active
  AND it is that project's active tab.
- D2 — clicking a Tab row switches to it (`switch_project(p)` then `active_project_mut().switch_tab(t)`); the
  seq-2 render then shows that tab full-screen.
- D3 — generic over `S` (reads only names/titles/active indices).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `rail_rows` runs on a 1-project/2-tab workspace, it shall emit `[Workspace, Project(active), Tab0(active), Tab1(inactive)]` in order. | unit |
| REQ-002 | WHEN there are 2 projects, only the ACTIVE project's active tab shall be flagged active (the other project's tabs are inactive). | unit |
| REQ-003 (visual) | WHEN the rail renders, it shall show the Workspace/Project headers + one row per tab, the active tab highlighted. | driven capture |
| REQ-004 (visual) | WHEN a tab row is clicked, that tab shall become active and fill the main area. | driven capture |
| REQ-005 | gate GREEN, cov/MSI 100 on rail_rows; the render/click masked. | gate |

## Phase Plan
- **P2** — RailRow/RailLevel + rail_rows; the render + click wiring; test plan.
- **P3** — implement (tabs.rs + app.rs).
- **P3.5** — 2 critics (the active-flag logic; the render replaced the session list cleanly / click routing).
- **P4** — rail_rows tests (cov/MSI 100) + driven captures (rail tree; click switches) + gate GREEN.
- **P5** — docs, AAR, archive, close #152.
