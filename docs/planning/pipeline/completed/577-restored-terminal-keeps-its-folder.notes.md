# A restored center terminal keeps its saved folder — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-577-restored-terminal-keeps-its-folder.md
- **Pipeline spec:** 577-restored-terminal-keeps-its-folder.spec.md

## Phase 1 — Plan
- **Request:** found in #575's Test (2026-09-26) and queued first; Chad's goal of 2026-09-26, "lets
  continue completing tickets".
- **Classification / tier:** bug, prong 1. Two Zed files, small additive hunks (a query, and a
  wider kept list in one call). Size S.
- **Checklist (no TaskCreate in this harness):** pick · pre-flight (clean at #575's commit) ·
  recall · mint · prior art · spec · design · present: all done here.
- **Recall (§18.3):**
  - F-claude-575-the-terminal-panels-cleanup-deleted-the-center-terminals-rows-001: the panel's
    cleanup, with the panel's items alone, deleted a center terminal's row between two restores.
  - PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001: #575's ids read from memory;
    Zed's folder rows are read at `deserialize`, so here the delete itself narrows.
  - L-claude-494-zed-item-ids-change-at-each-launch-001: rows stay one per live item only while
    a cleanup deletes the rest.
  - Brain consultation 821a9c0e70b74de79a3cb480a7190646: nothing on this seam.
- **Discovery (at `6b441aba60`):**
  - `crates/terminal_view/src/terminal_panel.rs:311-404` (`restore_serialized_state`: the panel's
    restore when a serialized panel exists, then, always, `TerminalView::cleanup(workspace_id,
    alive_item_ids)` with the panel's items, then the panel's items not in that list marked for a
    new save).
  - `crates/workspace/src/workspace.rs:7937-8081` (`load_workspace`: the center's items restored,
    `item_ids_by_kind` built from them, the cleanup per kind, then
    `serialize_workspace_internal`, which rewrites `items`).
  - `crates/terminal_view/src/persistence.rs:456-470` (`TerminalDb`, `static_connection!(TerminalDb,
    [WorkspaceDb])`: the same database as the workspace's `items` table), and the `items` table
    (`workspace_id`, `item_id`, `kind`, `pane_id`, …; #575's dump read it).
- **Decisions:** D1 to D3 in the spec.

### Design
- **Approach.**
  1. `persistence.rs`: `TerminalDb::marley_saved_terminal_items(workspace_id) -> Result<Vec<ItemId>>`,
     `SELECT item_id FROM items WHERE workspace_id = ? AND kind = 'Terminal'`.
  2. `terminal_panel.rs`, in the cleanup's update: `kept` = the panel's items plus that query's
     items (read in the same update; a failed read keeps the panel's items alone, as before);
     `TerminalView::cleanup(workspace_id, kept, …)`; the returned `alive_item_ids` for the
     re-save filter stays the panel's alone.
- **File manifest.** Zed: `crates/terminal_view/src/persistence.rs`,
  `crates/terminal_view/src/terminal_panel.rs` (new ledger rows first). Marley:
  `script/e2e/577-restored-terminal-keeps-its-folder.sh` (Test).

### E2E plan
Fixtures: a scratch repository with `alpha/` and `beta/`; a HOME whose `.bashrc` defines
`here <label>` (prints the label and `pwd`, teed to `here.log`).

| REQ | Scenario part | Shot or log |
|---|---|---|
| — | the left terminal `cd alpha`, a split, the right `cd beta`, `here` in each | `577-01-before` |
| REQ-001 | quit, launch; a click in each pane, `here`: `alpha` and `beta`, pane by pane | `577-02-restored`; the run log |
| REQ-002 | a third terminal opened and closed before the quit; after the launch, `terminal_list` counts two | the run log |
| — | the same scenario on the build before (the installed release at `d0939a6fc4`): red on REQ-001 | the run log |
| REQ-003 | the golden set with 577 added; the diff gate | `just regress`; `script/gates.sh --diff` |

### Risks
- The race is timing: the panel's cleanup landed between the two center restores in each of
  #575's runs, so the red on the old build is expected but not certain; the fix does not depend
  on the timing.

## Phase 2 — Code
- **Built.** `TerminalDb::marley_saved_terminal_items(workspace_id, kind)` (the `items` table's
  items of that kind); in `restore_serialized_state`, `kept_item_ids` = the panel's items plus that
  query's for `TerminalView::serialized_item_kind()`, passed to `TerminalView::cleanup` (so #575's
  hook cleanup gets it too); `alive_item_ids`, the panel's alone, still drives the re-save filter.
- **Deviations:** the kind is a bound parameter rather than the literal `'Terminal'`, which the
  `query!` macro's Rust tokens read as a character literal; `TerminalDb` named by its path in the
  hunk, so the panel's imports stay as upstream has them.
- **Review:** REQ-001 (the center's rows survive the panel's cleanup, whether it runs before or
  after the center's restores: the table lists the old ids until the workspace saves itself after
  its load, the new ones after); REQ-002 (a closed terminal leaves the saved layout, so its row
  still goes); the workspace's own cleanup is unchanged. A failed read keeps upstream's list.
- **Checks:** `cargo fmt`, `cargo clippy -p terminal_view --all-targets -- -D warnings`: clean.

## Phase 3 — Test
- **Checklist (no TaskCreate in this harness):** 577-01 · REQ-001 · REQ-002 · the red on the old
  build · golden set · gate.
- **Scenario:** `script/e2e/577-restored-terminal-keeps-its-folder.sh` (`compositor sway`; a
  scratch repository with `alpha/` and `beta/`; the HOME's `here <label>` prints the label and
  `pwd`, teed to `here.log`). Steps: `cd alpha && here left-0`; a split, `cd ../beta && here
  right-0`; a new terminal, `here third`, closed with `ctrl-shift-w`; quit and launch; a click in
  each pane and `here left-1`, `here right-1`; `mcp_agent terminals` counts the terminals.
- **Red on the build before the fix** (`E2E_BINARY=~/.local/bin/marley`, `d0939a6fc4`): the left
  terminal came back in `alpha`, the right one in the repository's root instead of `beta`, and the
  run failed there: the bug.
- **Green on the fix:** both terminals came back in their folders, and the tools listed two
  terminals, the closed one not restored.
- **Shots, read:**
  - `577-01-before`: the split, tabs `alpha — bash` and `beta — bash`, each terminal's `here`
    line with its folder; the rail lists the two terminals (the third is closed).
  - `577-02-restored`: after the relaunch, the same two tabs, `here left-1` in `alpha` and `here
    right-1` in `beta`; the rail lists two terminals.
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule
  and did not reload it".
- **Golden set:** 577 added; `just regress`: all 18 passed.
- **Gate:** `script/gates.sh --diff`: 16 passed, 0 failed, `GATE GREEN [diff]`, receipt written.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Fixed: a restored terminal opens in the folder it was in);
  `docs/marley/zed-touchpoints.md` (the two new rows, `terminal_panel.rs` and `persistence.rs`,
  checked against what shipped). No Marley crate changed, so no per-crate note; no plan row.
- **Knowledge appended:** F-claude-577-a-restored-center-terminal-opened-in-the-projects-folder-001.
- **Brain:** `rusty-cli brain decide 821a9c0e70b74de79a3cb480a7190646` →
  `decisions/zeds-terminal-panel-keeps-the-center-terminals-rows-in-its-cleanup`.
- **Ticket:** closed; the pipeline archived to `completed/`.
