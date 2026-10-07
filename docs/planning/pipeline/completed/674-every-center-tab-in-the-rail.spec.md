---
pipeline_id: 1e2bdc25-f5e5-4fec-b8eb-ace49ed5ff88
ticket: docs/planning/tickets/open/TICKET-674-every-center-tab-in-the-rail.md
status: Phase 4 — Complete PASS
title: "Every center tab has a row in the rail, a project's files under a Files row"
type: feature
slice: the rail (decisions 1 and 2 of docs/planning/intake/rail-and-center-tabs.md)
references: [docs/planning/intake/rail-and-center-tabs.md, docs/planning/pipeline/completed/504-browser-tabs-in-the-rail.spec.md]
---

## Title
Every center tab a project holds gets a row in the rail. Chad, 2026-10-06: "some things opened in
the main pane dont show up on the left pane ... if i open a file to edit, a knowledge page it doesnt
show up but only in the middle". Asked on 2026-10-07 whether to keep the top tabs: "yes keep them
and show up on the left"; on files: "Files makes sense to me put it above containers".

## Scope
### In
- **Tab rows**: a row for each center item that is neither a terminal nor a Browser tab (those
  have rows already): project search, diffs, settings, Marley's tabs and Rusty's, previews. Its
  tab's icon and title, a dot while it has unsaved changes, a close button on hover; a click (or
  Enter) brings the tab forward. After the project's Browser tabs.
- **Files**: a project's file editors under one **Files (n)** row, after its threads and before its
  ports, open to start; a click on it, or Enter on it, folds and unfolds it. A file
  row's second line is its folder in the project.
- **The highlight**: whatever is in front has its row highlighted; a file folded away under Files
  highlights the Files row.
- **Live**: a tab added, closed, renamed, made dirty or saved in any pane of the project changes
  its row at once.
- **Keys and filter**: the rows join the rail's walk, so Up and Down, Next and Previous, and the
  filter (by title) reach them.

### Out (explicitly deferred)
- **Dragging** tab and file rows to reorder: they keep the panes' order.
- **The Files fold across a restart**: kept per window in memory, as a terminal's turns fold is.
- **A preview tab's italic**: its row is a plain row.
- **Rusty's tabs** in a group of their own: #675.

## Reference (§20)
Upstream Zed — the pane's tab bar (`workspace::pane`), whose items, titles (`tab_content_text`),
icons (`tab_icon`), dirty marks (`is_dirty`) and close the rows mirror, and whose
`pane::Event`s keep them current. Warp's vertical tabs list every tab with its panes
(`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`).

### Prior art
- **Behaviour maps:** Warp's vertical tabs (a row per pane or per tab, one list); Orca's sidebar,
  which lists worktrees and agents and leaves tabs to the top bar
  (`docs/orca_architecture/05-terminal-and-workspace.md` §2.7). Recorded in the intake note.
- **Published material:** docs.warp.dev/terminal/windows/vertical-tabs/ ("View as: Panes / Tabs").
- **The code we ship:** `Workspace::panes`, `Pane::items`, `Pane::preview_item_id`,
  `ItemHandle::{tab_content_text, tab_icon, project_path, buffer_kind, is_dirty, item_id}`,
  `Workspace::activate_item`, `Pane::close_item_by_id`; the rail's Browser rows (#504) as the
  template for a row kind, and its `row_card`, `Disclosure` and turns fold (#509).

## UI proof
`script/e2e/674-every-center-tab-in-the-rail.sh` (`compositor sway`), a scratch project with two
files. Shots: `674-01-files` (two files opened: Files (2) above nothing, each with its folder);
`674-02-other` (a project search opened: its row after the terminal, highlighted);
`674-03-dirty` (a file typed in: its dot); `674-04-folded` (Files clicked: folded, the Files row
highlighted while a file is in front); `674-05-closed` (a file's tab closed from its row: the
row gone, Files (1)).

## Locked-In Decisions
- D1 — **Two kinds**: a file is an item with a project path and a single buffer; anything else
  is an Other tab.
- D2 — **Order**: terminals, worktrees, Browser tabs, Other tabs, threads, Files and its files,
  ports. Within a kind, the panes' order and each pane's tab order.
- D3 — **Pane subscriptions**: the rail subscribes to every pane of every workspace it follows,
  since a title or dirty change in a pane that is not active reaches no workspace event.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a project's center pane opens a tab that is neither a terminal nor a Browser tab, the rail shall list a row for it under the project. | `674-01-files`, `674-02-other` |
| REQ-002 | The rail shall list a project's file tabs under a Files row with their count, above the project's ports. | `674-01-files` |
| REQ-003 | WHILE a tab has unsaved changes, its row shall show a dot. | `674-03-dirty` |
| REQ-004 | WHEN the Files row is clicked, the rail shall fold or unfold its files. | `674-04-folded` |
| REQ-005 | The rail shall highlight the row of the tab in front, or the Files row when that tab is a file folded under it. | `674-02-other`, `674-04-folded` |
| REQ-006 | WHEN a tab row's close button is clicked, the system shall close that tab and the rail shall drop its row. | `674-05-closed` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_rail.rs` (the model), `rail.rs` (snapshot, subscriptions, rows), the
  tests in the tree that match on rows; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide, the architecture notes, ledger capture, close, archive,
  commit.
