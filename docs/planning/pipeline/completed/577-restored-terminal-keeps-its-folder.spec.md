---
pipeline_id: c8e74530-2e34-4cb8-9ea6-4f9af994dbfc
ticket: docs/planning/tickets/closed/TICKET-577-restored-terminal-keeps-its-folder.md
status: Phase 4 — Complete PASS
title: "A restored center terminal keeps its saved folder"
type: bug
slice: prong 1 (the Marley layout's terminals); found in #575's Test
references: [docs/planning/pipeline/completed/575-terminal-id-across-restore.notes.md, docs/planning/knowledge/failures.md]
---

## Title
At a launch the terminal panel restores its own terminals and then cleans up the workspace's
terminal rows, keeping only the panel's items. In the Marley layout the terminals are the
center's, so that cleanup deletes their rows, and when it runs before a center terminal's
restore reads its row, the terminal comes back in the project's folder instead of the one it was
in (#575's Test caught it deleting the second terminal's row between two restores). Here the
panel's cleanup also keeps the terminal items of the workspace's saved layout, so a center
terminal's row lasts until its restore reads it and the workspace's own cleanup replaces it.

## Scope
### In
- `crates/terminal_view/src/terminal_panel.rs`: the cleanup after the panel's restore keeps, besides
  the panel's items, the terminal items the workspace's saved layout lists; the filter that marks
  the panel's late items for a new save keeps reading the panel's items alone.
- `crates/terminal_view/src/persistence.rs`: a query for those items, read from the workspace's
  `items` table (the same database as `terminals`).

### Out (explicitly deferred)
- The workspace's own cleanup, which deletes the panel's rows when the center holds terminals; the
  panel saves its restored terminals again after its cleanup, as upstream already does.
- A terminal's folder across a remote project's restore (Zed saves none for a remote shell).

## Reference (§20)
- **Upstream Zed:** `terminal_view`'s persistence: `TerminalPanel::restore_serialized_state`
  (the panel's restore and its cleanup, "Since panels/docks are loaded outside from the workspace,
  we cleanup here"), `TerminalView::cleanup` (`delete_unloaded_items`), `TerminalView::deserialize`
  (the saved folder read at the restore), and `Workspace::load_workspace` (the center's restore and
  the workspace's cleanup, which runs only for the kinds the center holds). Marley keeps both
  cleanups and narrows what the panel's deletes.
- **Warp:** N/A. Zed's persistence, no Warp behavior to match.

### Prior art
- **Behavior maps.** None apply (a Zed persistence race).
- **Published material.** None.
- **The code we already ship.** F-claude-575-the-terminal-panels-cleanup-deleted-the-center-terminals-rows-001
  and PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001 (#575 keeps its ids out of
  the race through memory); `load_workspace` builds `item_ids_by_kind` from the center's items, so
  a workspace whose center holds no terminal never cleans up the `terminals` rows, which is why
  the panel's cleanup exists; the workspace's `items` table holds the center layout's items by
  kind until the workspace saves itself again after its load. Does a crate we build own this seam?
  Zed does; the fix is a narrower list in Zed's own call.

## UI proof
UI-AFFECTING: the folder a restored terminal opens in.
`script/e2e/577-restored-terminal-keeps-its-folder.sh` (`compositor sway`; a scratch repository
with two subfolders, `alpha` and `beta`). Shots:
- `577-01-before`: two terminals in a split, the left in `alpha`, the right in `beta`, each
  printing its folder.
- `577-02-restored`: after a quit and a launch, each restored terminal printing the folder it was
  in.

## Locked-In Decisions
- D1: The panel's cleanup keeps the panel's items and the workspace's saved terminal items; it still
  deletes every other row, so a closed terminal's row does not stay.
- D2: The saved terminal items come from the workspace's `items` table, which lists the center's
  items until the workspace saves itself after the load; by then the center's restores have read
  their rows.
- D3: Upstream's own lines stay: the panel's list of its items still drives the new saves of the
  panel's late items.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley quits and launches again, each restored center terminal shall open in the folder it was in at the quit. | Shot `577-02-restored`; the run log compares each pane's folder |
| REQ-002 | WHEN a center terminal closes before a quit, the system shall not restore it at the next launch. | The run log: the terminal count after the launch |
| REQ-003 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec, and the design in the notes.
- **P2 Code:** the ledger rows first; the query; the panel's kept list. fmt and clippy clean; a
  review.
- **P3 Test:** write and run the scenario (red on the build before, green after), read every
  shot, `just regress`, `script/gates.sh --diff`.
- **P4 Complete:** CHANGELOG (Fixed), the ledger rows, knowledge, close, archive, commit.
