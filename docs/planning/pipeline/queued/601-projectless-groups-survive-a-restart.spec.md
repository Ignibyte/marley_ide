---
pipeline_id: 2ef22390-9dd7-45f7-a267-4ee0dc97c930
ticket: docs/planning/tickets/open/TICKET-601-projectless-groups-survive-a-restart.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Projectless groups survive a restart"
type: feature
slice: workbench shell (the rail); second of two, after #600
references: [docs/planning/pipeline/queued/600-rail-menu-and-projectless-groups.spec.md, docs/planning/pipeline/completed/575-terminal-id-across-restore.spec.md, docs/planning/pipeline/completed/576-browser-tabs-table-item-id-not-unique.spec.md]
---

## Title
Bring #600's projectless groups back after a restart: each window's groups return with their
names, their order and their items, as its projects do.

## Scope
### In
- **The record.** #600's per-window list of groups (group id, name, `WorkspaceId`, order) is saved
  with the window: in Marley's group table and the rail's own blob in the window's
  `sidebar_state`, whichever the design at promotion settles.
- **The restore.** When a window is restored, Marley reopens each recorded group's workspace by its
  `WorkspaceId` into that window without activating it (Zed restores the active one itself), then
  lists the groups by the record. Zed's own workspace restore brings back each workspace's items:
  terminals in their folders with their ids (#575, #577), Browser tabs (#494, #576).
- **What cannot come back.** A group whose workspace row Zed no longer has (its garbage collection
  drops a folderless workspace absent from the last two sessions) is dropped from the record,
  with a line in the log.
- **The Home group** comes back as any group does.

### Out (explicitly deferred)
- Groups in a window that is not restored (Zed's own rule for windows is kept).
- Moving a group to another window.
- The group order across projects and groups under a drag: #602.

## Reference (§20)
Upstream Zed: a window's workspaces and their items come back through Zed's session restore
(`crates/zed/src/main.rs` restore path, `workspace::restore_multiworkspace`,
`open_workspace_by_id`, each item's `deserialize`), which Marley keeps and extends to the
folderless workspaces it records. Warp: its tab groups last across restarts
(`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md` shows a saved "New Group").

### Prior art
- **Behavior maps.** The Warp capture; `docs/zed_architecture/` for session restore.
- **Published material.** None needed.
- **Code we already ship** (Explore, 2026-09-30):
  - `read_serialized_multi_workspaces` (`crates/workspace/src/persistence.rs:327-374`) keeps one
    active workspace per window; `restore_multiworkspace` (`workspace.rs:10284-10369`) reopens a
    folderless active workspace through `open_workspace_by_id` (10294-10297, 11056+); other
    groups come back only as keys, and empty keys are skipped (10390-10392, and
    `restore_project_groups` 826-828). So the folderless workspaces other than the active one are
    Marley's to reopen, by id.
  - Folderless workspaces are stored with `paths = ""` and found only by id; garbage collection
    keeps those of the current and last session (`persistence.rs:1590-1607`, 2045-2057,
    2164-2213).
  - The rail already saves per-window state in Zed's blob (`Rail::serialized_state` and
    `restore_serialized_state`, `rail.rs:6161-6199`; `write_rail_state`/`read_rail_state`,
    4182-4202). Marley's own tables keyed to `workspaces` (`MarleyTerminalIdsDb`,
    `MarleyBrowserTabsDb`) cascade on delete.

## UI proof
The scenario `script/e2e/601-projectless-groups-survive-a-restart.sh` (sway) makes two groups
through #600's menu, opens two terminals in one (one `cd`s to `/tmp`) and a Browser tab on a local
page in the other, shoots the rail (`before.png`), quits Marley through the palette, starts it on
the same profile, and shoots the rail again (`after.png`) and each terminal after `pwd`
(`folders.png`).

## Locked-In Decisions
- D1 — Marley reopens the recorded groups' workspaces itself, by `WorkspaceId`, after Zed's
  restore of the window; it changes no Zed restore code unless the design at promotion finds a
  hook is needed, which then gets its touchpoint row.
- D2 — A group's order is the record's order, after the window's projects, until #602.
- D3 — A group whose workspace is gone is dropped quietly, with a log line.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley restores a window that had projectless groups, the rail shall list each group again with its name, in its saved order. | Shots `before.png`, `after.png` |
| REQ-002 | WHEN a group comes back, its terminals shall come back in the folders they were in, and its Browser tabs on their pages. | Shot `folders.png`; `after.png` |
| REQ-003 | WHEN the window's active workspace was a group, that group shall come back shown, and the others listed. | Shot `after.png` |
| REQ-004 | IF a recorded group's workspace no longer exists, THEN Marley shall drop it from the record and log it, and restore the rest. | Review of the diff |

## Phase Plan
- **P1 Plan** — promote this pair, recall, the design in the notes: where the record lives, when
  the reopen runs, and how it avoids a second copy of the active group.
- **P2 Code** — the record's save and the restore in `marley_workbench` (and a Zed hook only if the
  design needs one); a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario (it restarts Marley), read every shot.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the guide page and
  the walkthrough (§21); the ledger; close, archive, commit.
