---
pipeline_id: 8da5840b-ccf1-4383-8ac4-5e58dc3d4fde
ticket: docs/planning/tickets/open/TICKET-845-rusty-screens.md
status: Phase 4 — Complete PASS
title: Rusty's Tasks tab as the Tasks board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Rusty's Tasks tab (#658) reads as the Tasks board draws it: each list with a coloured dot and its
open count, the chosen list's title over "N open · M done", its open tasks as rows and its done
ones under a Done heading, and a Rusty column at the right with the decisions due (Follow Up, Open
the page) and Rusty's newest memories.

## Scope
### In
- The lists column: a dot per list (a fixed palette by position), the open count.
- The header: the list's name, "N open · M done", Show archived and Refresh kept.
- The tasks: open first; done ones under "Done"; behaviour (check, rename, drag, menu) unchanged.
- A Rusty column (≥ 1100 px wide windows): Decisions due (`brain_due`, overdue first; Follow Up
  and Open the page as the Decisions tab's), and Memory (the three newest of `list_memories`).

### Out (explicitly deferred)
- The board's due chips and notes: Rusty's tasks have neither (`rusty-core` `UserTask`: title,
  completed, archived, order, created). "Ask an agent" on a row, and the Today card (the daily note
  has no read tool in Marley yet).
- The other Rusty screens: Decisions and Follow Up (TICKET-874); the Brain tab, pages, Knowledge
  and the Graph (TICKET-875); Memory, Skills, Secrets, Capture, Import and Rusty's Server
  (TICKET-876).

## Reference (§20)
N/A — Marley-specific: the Tasks tab is Marley's (#658) over Rusty's tools (`list_task_groups`,
`list_tasks`, `brain_due`, `list_memories`); the Tasks board.

### Prior art
- The code we ship: `tasks_tab.rs` (`render_lists`, `render_header`, `render_task`),
  `decisions_tab.rs` (`brain_due` and its Follow Up), `memory_tab.rs` (`list_memories`).
- The brain (consultation b417c36c): nothing on this seam.

## UI proof
Scenario `script/e2e/845-tasks-tab.sh` (sway), #658's stand-in Rusty (lists Home, Work, Someday;
a done task in Home) with #659's decision pages (one overdue) and three memories.
- `845-01-tasks`: Home chosen: the dots and counts, "N open · 1 done", the open tasks, Done with
  its task, the Rusty column with the overdue decision and the memories (REQ-001, REQ-002).
- `845-02-work`: Work chosen: its tasks and counts (REQ-001).
- `845-03-day`: Marley Day (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — Counts come from one `list_tasks` per list at each read (no archived).
- D2 — The Rusty column shows only while Rusty answers and the tab is at least 1100 px wide.
- D3 — The palette of list dots is fixed by the list's position (accent, eyes, warning, info, …).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Tasks tab shall show each list with a dot and its open count, the chosen list's "N open · M done", its open tasks and then its done tasks under Done, in Marley Night and Marley Day. | `845-01-tasks`, `845-02-work`, `845-03-day` |
| REQ-002 | WHILE the tab is wide enough, it shall show a Rusty column with the decisions due and Rusty's newest memories. | `845-01-tasks`, `845-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code** (`tasks_tab.rs`), **P3 Test**, **P4 Complete**.
