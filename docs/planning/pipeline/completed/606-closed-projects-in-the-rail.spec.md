---
pipeline_id: 3b01562e-2e93-4bb9-a54e-2480e4971fac
ticket: docs/planning/tickets/open/TICKET-606-closed-projects-in-the-rail.md
status: Phase 4 — Complete PASS
title: "List a window's closed projects in the rail"
type: feature
slice: workbench shell (the rail); after #601 and #602
references: [docs/planning/pipeline/completed/601-projectless-groups-survive-a-restart.notes.md, docs/planning/pipeline/completed/602-drag-to-reorder-the-rail.spec.md, docs/planning/pipeline/completed/507-browser-context-per-project.spec.md]
---

## Title
A project the window lists but holds no workspace for (after a restart Zed reopens only the shown
one) stays in the rail as a dimmed header, and a click on it opens it.

## Scope
### In
- **Listed.** Every project group of the window is a header, whether a workspace of it is open or
  not. A group with none is *closed*: its header's name and icon are dimmed, it has no rows, no
  chevron (its room is kept, so names line up) and no `+`, and its tooltip says it is not open
  and that a click opens it.
- **Opening.** A click on a closed header, or Enter while the keyboard is on it, opens the project
  as Zed's Threads Sidebar opens a group with no workspace
  (`MultiWorkspace::find_or_create_workspace`, activating it; a remote project connects through
  `remote_connection::connect_with_modal`). Zed restores the workspace saved for those folders,
  so its terminals and tabs come back, and the header lists as any open project's.
- **Its menu.** A closed header's right-click menu keeps Move Project Up and Down (#602's order) and
  Remove Project (#507), which drops the group; Clear Browser Data… is disabled, since it acts on
  an open project's browser.
- **Order.** Closed headers take their place in the rail's order like any other (#602's saved
  order, else the window's), and can be dragged.

### Out (explicitly deferred)
- Reopening every project at startup: each would start its language servers, terminals and
  browser; the rail opens one when asked.
- Rows under a closed header (its threads from the metadata store, its ports).
- Projectless groups (#601 reopens them itself).

## Reference (§20)
Upstream Zed: the Threads Sidebar (`crates/sidebar/src/sidebar.rs`) lists every project group of
the `MultiWorkspace`, including those with no open workspace, and opens one on a click
(`activate_or_open_workspace_for_group` → `open_workspace_for_group` →
`MultiWorkspace::find_or_create_workspace` with `OpenMode::Activate` and
`remote_connection::connect_with_modal` for a remote group). The rail keeps that behavior, and
dims the closed header so the difference shows.

### Prior art
- **Behavior maps.** #601's Test notes (the gap found: `repo` not listed after a restart);
  #507 (Remove Project through `MultiWorkspace::remove_project_group`); #602 (header order).
- **Published material.** None needed.
- **Code we already ship.**
  - `MultiWorkspace::project_groups` keeps closed groups' keys (`restore_project_groups`,
    `multi_workspace.rs:832`) and `find_or_create_workspace` (`:1105`) opens one.
  - `remove_project_group` (`:958`) removes a key with no workspace.
  - `remote_connection::connect_with_modal` and `dismiss_connection_modal`.
  - `gpui::WeakEntity::new_invalid` for a closed group's absent workspace.
  - The rail's filter of groups with no workspace is in `rail_groups` and `build_snapshot`
    (`listed_workspace`).

## UI proof
The scenario `script/e2e/606-closed-projects-in-the-rail.sh` (sway) opens `repo` and hands over
`repo-b` and `repo-c` (#513), gives `repo-b`'s terminal the folder `sub`, shows `repo`, quits,
and starts Marley with no path. It shoots the rail with `repo` open and the other two dimmed
(`restart.png`) and a dimmed header's tooltip (`tooltip.png`); clicks `repo-b` and shoots it open
with its terminal back in `sub` (`opened.png`); opens `repo-c`'s right-click menu
(`menu.png`) and chooses Remove Project (`removed.png`).

## Locked-In Decisions
- D1 — A closed project is listed, not reopened: nothing starts until the user opens it.
- D2 — Opening is Zed's own `find_or_create_workspace`, so the saved workspace is restored as
  opening the project any other way restores it.
- D3 — Dimmed, header only: no rows, chevron or `+` until it is open.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the window holds a project group with no open workspace, the rail shall list its header, dimmed, with no rows, chevron or `+`. | Shot `restart.png` |
| REQ-002 | WHILE the pointer is on a closed header, its tooltip shall say the project is not open and a click opens it. | Shot `tooltip.png` |
| REQ-003 | WHEN the user clicks a closed header, Marley shall open the project in the window, and the rail shall list it as an open project with its restored rows. | Shot `opened.png` |
| REQ-004 | WHEN the user presses Enter with the keyboard on a closed header, Marley shall open it as a click does. | Review of the diff |
| REQ-005 | WHEN the user chooses Remove Project on a closed header, the rail shall list it no more. | Shots `menu.png`, `removed.png` |

## Phase Plan
- **P1 Plan** — mint, recall, the design in the notes.
- **P2 Code** — `marley_rail`'s closed flag; the rail's closed groups, header, open, menu; a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, architecture notes, the guide page, guide.md and the walkthrough
  (§21); the ledger; close, archive, commit.
