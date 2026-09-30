---
pipeline_id: f22b97d1-2b4e-44f8-83df-e34f6fac5c4a
ticket: docs/planning/tickets/open/TICKET-600-rail-menu-and-projectless-groups.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The rail's right-click menu, and projectless groups"
type: feature
slice: workbench shell (the rail); first of two (#601 restores groups after a restart)
references: [docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md, docs/planning/pipeline/completed/453-rail-keyboard-and-reorder.spec.md, docs/planning/pipeline/completed/507-browser-context-per-project.spec.md]
---

## Title
A right-click menu on the rail's empty space with New Group… and projectless items, and the
projectless group itself: a named group backed by a workspace with no folder, listed in the rail
like a project, whose terminals and agent CLIs start in the home folder and whose Browser tabs use
a Chromium of the group's own.

## Scope
### In
- **The empty-space menu.** A right-click on the rail's blank area below its rows, or on the
  header's space beside PROJECTS, opens a menu at the pointer: **New Group…**, a separator, **New
  Terminal**, **New Browser Tab**, and under an "Agent CLIs" header each installed CLI (as the
  project + lists them). Rows keep their own right-click menus.
- **New Group….** Asks for a name in a small prompt (Enter makes it, Escape cancels; an empty name
  makes "Group", then "Group 2" and so on). Marley makes an empty workspace in this window (Zed's
  `Workspace::new_local` with no paths, `OpenMode::Add`), records it as a Marley group with its
  name, and the rail lists it.
- **Projectless items from the empty-space menu** go into the group named **Home**, which Marley
  makes the first time one is needed and reuses after (the one Home group per window; renaming it
  frees the name).
- **A group in the rail.** A header like a project's: a group icon in place of the project's
  folder icon, the name, the chevron, a muted count of its items when folded, and its `+`. Under
  it, its terminals (agent CLI rows included) and its Browser tabs. No lines changed, no pull
  request, no ports, no worktrees, no threads.
- **A group's `+`.** New Terminal, New Browser Tab, and the Agent CLIs. Not New Agent Thread, New
  Agent in Worktree or Launch, which need a folder.
- **Where items start.** Terminals and agent CLIs in the home folder (Zed's
  `default_working_directory` already falls back to it for a workspace with no worktree).
  Browser tabs on a Chromium and profile of the group's own, keyed by the group, not by the empty
  path list every folderless workspace shares.
- **A group's header menu (right-click).** Rename Group…, Remove Group. Remove closes the group's
  items as closing their tabs would (Zed's prompts and #550's question about a working agent
  apply), stops its Chromium, and drops the group.
- **Kept beside projects.** Adding or opening a project while a group is shown leaves the group
  in the window (Zed today closes an active folderless workspace when a project opens in it).
- **Order.** Groups list after the window's projects, in the order they were made. #602 lets them
  move.

### Out (explicitly deferred)
- Groups surviving a restart, with their names, items and order: #601.
- Dragging projects, groups and rows: #602.
- Agent Panel threads in a group (Zed archives threads with no folder), ports, and launch
  configs in a group.
- Groups in Zed's layout: Zed's Threads Sidebar lists no folderless workspace, and that stays.
- Moving an item from one group or project to another.
- The title bar's name for a folderless workspace ("Open Recent Project"), and the name the MCP
  tools give it ("Empty Workspace").

## Reference (§20)
Warp, from the observed capture of its vertical tab list
(`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`): an explicit tab group ("New
Group") draws a chevron, its name and a muted count ("1 tab") over the rows it holds. Marley keeps
the named group with its chevron and folded count, drawn as the rail's project headers are (#468
kept no band), and the right-click on the list's empty space that makes one, which is Chad's
request. Upstream Zed: the folderless workspace (`Workspace::new_local` with no paths, held by
`MultiWorkspace`), which Zed already opens in a new window and whose terminals start in the home
folder.

### Prior art
- **Behavior maps.** The Warp capture above; `docs/marley/workbench-shell.md` D3 (the row model)
  and D4 (one visible way to start things). `docs/zed_architecture/` has no multi-workspace notes
  beyond what the code says.
- **Published material.** None needed.
- **Code we already ship** (Explore, 2026-09-30; the notes hold the lines):
  - `Workspace::new_local(Vec::new(), app_state, Some(window), None, None, OpenMode::Add)` makes a
    folderless workspace in this window with a fresh `WorkspaceId`; `MultiWorkspace::add` holds and
    pins it.
  - Zed never makes a project group for a folderless workspace
    (`MultiWorkspace::ensure_project_group_state` returns early on an empty key; so do rekeying,
    restore and serialization), and every folderless workspace has the same key (an empty
    `PathList`). So a Marley group cannot be a Zed project group; the rail lists Marley's groups
    from its own record of the held workspaces.
  - `MultiWorkspace::open_project` closes an active workspace that has no visible worktree before
    opening a project (`multi_workspace.rs` around 1923-1970): the one Zed touch this ticket
    needs, an additive Marley hook that keeps a workspace Marley marks as a group (the precedent
    is `MarleyCloseGuard` in `crates/workspace/src/workspace.rs`).
  - `terminal_view::default_working_directory` falls back to `dirs::home_dir` with no worktree, and
    `agents::start_cli` uses it: no change for the home folder.
  - #507's `BrowserProject::of_group` keys a Chromium by the group key's paths; with none it would
    be `e3b0c44298fc1c14` for every folderless workspace, so a group keys its own.
  - `ui::right_click_menu` (`crates/ui/src/components/right_click_menu.rs`) and `ContextMenu`
    build the rail's existing row menus; `ui_prompt`/Zed's modal for the name, as the rail's
    rename runs Zed's own editor for a tab.
  - Marley's own SQLite tables keyed to `workspaces` with cascade delete (`MarleyTerminalIdsDb`,
    `MarleyBrowserTabsDb`) are the pattern for recording a group's id and name against its
    `WorkspaceId`.

## UI proof
The scenario `script/e2e/600-rail-menu-and-projectless-groups.sh` (sway) opens a scratch project,
right-clicks the rail below its rows (`menu.png`), makes a group "Scratch" (`group.png`), opens a
terminal from the group's `+` and runs `pwd` (`home.png`), chooses New Terminal from the
empty-space menu (`home-group.png`), opens a Browser tab in Scratch and checks the systemd unit
list for a second `marley-browser-` unit (`units.txt`), adds a second scratch project with Add
Project (`kept.png`), renames Scratch (`renamed.png`) and removes it (`removed.png`).

## Locked-In Decisions
- D1 — A group is a Marley record over a folderless workspace held in the window: an id Marley
  makes, the name, and the `WorkspaceId`. It is not a Zed project group.
- D2 — Items made from the empty-space menu go to the window's Home group, made on first use.
- D3 — A group's Browser tabs get a Chromium keyed by the group's id.
- D4 — One Zed touch: `MultiWorkspace::open_project` keeps a workspace Marley marks as a group.
  Additive, behind a Marley hook, with its row in `zed-touchpoints.md`.
- D5 — In the Marley layout only; Zed's layout is unchanged.
- D6 — Groups last for the session in this ticket; #601 restores them.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user right-clicks the rail's empty space, the rail shall open a menu with New Group…, New Terminal, New Browser Tab and each installed agent CLI. | Shot `menu.png` |
| REQ-002 | WHEN the user chooses New Group… and enters a name, the rail shall list a group with that name, a group icon, a chevron and a `+`. | Shot `group.png` |
| REQ-003 | WHEN the user opens a terminal or an agent CLI from a group's `+`, it shall start in the home folder and list under the group. | Shot `home.png` (`pwd` prints the home folder) |
| REQ-004 | WHEN the user chooses New Terminal, New Browser Tab or an agent CLI from the empty-space menu, the item shall open in the window's Home group, which the rail shall make if the window has none. | Shot `home-group.png` |
| REQ-005 | WHEN a group opens its first Browser tab, Marley shall start a Chromium keyed by the group, apart from every project's and every other group's. | `units.txt`: one more `marley-browser-` unit |
| REQ-006 | WHEN the user adds or opens a project while a group is shown, the group shall stay in the window with its items. | Shot `kept.png` |
| REQ-007 | WHEN the user chooses Rename Group… on a group's header, the rail shall show the new name. | Shot `renamed.png` |
| REQ-008 | WHEN the user chooses Remove Group…, Marley shall close the group's items with Zed's and #550's prompts, stop its Chromium, and drop the group from the rail. | Shot `removed.png` |
| REQ-009 | A group's `+` shall offer no New Agent Thread, New Agent in Worktree or Launch entries. | Shot `home.png` (the menu before the choice) or review |

## Phase Plan
- **P1 Plan** — promote this pair, recall, the design in the notes: the group record and its
  table, the Zed hook, the Chromium key, where the rail draws groups.
- **P2 Code** — the record (`groups.rs` in `marley_workbench`), the rail's menu and group rows,
  `BrowserProject` keyed by group, the `open_project` hook and its touchpoint row; a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the guide page and
  the walkthrough (§21), `zed-touchpoints.md` checked; the ledger; close, archive, commit.
