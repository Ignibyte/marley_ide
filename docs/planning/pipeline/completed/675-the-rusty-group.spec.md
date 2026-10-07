---
pipeline_id: 2f90c9b6-cbb5-4694-b6e6-8a6bcd75525b
ticket: docs/planning/tickets/open/TICKET-675-the-rusty-group.md
status: Phase 4 — Complete PASS
title: "The Rusty group: where every Rusty screen opens"
type: feature
slice: the rail; Rusty in Marley (decision 3 of docs/planning/intake/rail-and-center-tabs.md)
references: [docs/planning/intake/rail-and-center-tabs.md, docs/planning/pipeline/completed/600-rail-menu-and-projectless-groups.spec.md]
---

## Title
Every Rusty screen and page opens in a Rusty group in the rail, whatever project the window shows.
Chad, 2026-10-06: "anthing that opens there should be in a group called Rusty (have a placeholder
for an icon when i get it)"; on 2026-10-07: "rusty group makes sense".

## Scope
### In
- **The group**: a folderless group (#600) named Rusty, made in a window the first time a Rusty
  screen or page opens there, with a placeholder icon on its header. Its tabs are its rows (#674).
- **Routing**: Today, Graph, Tasks, Decisions, Memory, Skills, Secrets and every page, from the
  header's buttons, the palette, the Brain view's tree, the page picker, the Knowledge panel and
  capture, open in the window's Rusty group, which the window then shows.
- **Kept**: no Rename or Remove in its menu; it comes back after a restart as #601's groups do.
- **Off**: while Rusty is off the rail does not list it (#661).
- **The Zed layout**: screens open in the shown workspace, as before.

### Out (explicitly deferred)
- **Chad's own icon**: a placeholder until he has one; one constant to change.
- **The fleet's Agent tab and System One calls**: they wait for the fleet talk.

## Reference (§20)
N/A — Marley-specific: Rusty's screens are Marley's, and folderless groups are Marley's own (#600).
The nearest outside shape is Orca's floating workspace, a place outside every worktree with tabs of
its own (`docs/orca_architecture/05-terminal-and-workspace.md`).

### Prior art
- **Behaviour maps:** Orca's floating workspace (above). Recorded in the intake note.
- **Published material:** none applies.
- **The code we ship:** `groups::make`, `groups_of`, the Home group's `in_home` (#600), its saved
  record (#601); `MultiWorkspace::activate`; each Rusty tab's `open_later`; #674's tab rows.

## UI proof
`script/e2e/675-the-rusty-group.sh` (`compositor sway`), Rusty's stand-in over a scratch vault.
Shots: `675-01-before` (a project with its terminal, no Rusty group); `675-02-graph` (the header's
Graph clicked: a Rusty group with its icon, Graph its row, the window showing it);
`675-03-project` (the project clicked: its terminal back in front, Graph still under Rusty);
`675-04-page` (a page opened with `rusty: open page` while the project shows, the path the Brain
view's tree shares through `page::open_later`: under Rusty, in front); `675-05-menu` (the group's right-click menu: no Rename, no Remove); `675-06-off` (Rusty
turned off: no Rusty group).

## Locked-In Decisions
- D1 — **Made on first use**, not at start: a window that never opens a Rusty screen keeps a rail
  without it.
- D2 — **One per window**, as the Home group is.
- D3 — **Routing in each opener's `open_later`**, and the palette actions go through it, so no
  path opens a Rusty tab in a project.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Rusty screen or page opens in the Marley layout, the system shall open it in the window's Rusty group, making the group when the window has none, and show the group. | `675-02-graph`, `675-04-page` |
| REQ-002 | The Rusty group's header shall carry its own icon. | `675-02-graph` |
| REQ-003 | WHEN the window shows a project again, the Rusty group shall keep its tabs. | `675-03-project` |
| REQ-004 | The Rusty group's menu shall offer no Rename or Remove. | `675-05-menu` |
| REQ-005 | WHILE Rusty is off, the rail shall not list the Rusty group, and a window showing it when Rusty turns off shall show its first project. | `675-06-off` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `groups.rs`, `rusty.rs` and the Rusty tabs' openers, `rail.rs`; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide, the architecture notes, ledger capture, close, archive,
  commit.
