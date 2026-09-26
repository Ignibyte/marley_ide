---
pipeline_id: 6b4cc8ee-5ad6-42b0-94df-ec19b491b1d9
ticket: docs/planning/tickets/closed/TICKET-575-terminal-id-across-restore.md
status: Phase 4 — Complete PASS
title: "A terminal keeps its MARLEY_TERMINAL_ID across a restore"
type: feature
slice: prong 2 (C0 follow-on); #520's third piece, after #574
references: [docs/planning/pipeline/completed/520-terminal-identity.spec.md, docs/planning/pipeline/completed/520-terminal-identity.notes.md, docs/planning/pipeline/completed/494-browser-restore.notes.md, docs/orca_architecture/06-cli-automations-skills.md]
---

## Title
#520 gives each Marley terminal an id, `MARLEY_TERMINAL_ID`, that an agent's bridge hands
Marley's tools, but the id lasts one launch: Zed restores the terminal at the next launch with a
new shell, and the builder mints a new id. An agent resumed in the restored terminal, or a script
that kept the id, then names a terminal Marley no longer has. Here a terminal Zed restores keeps
the id it had: Marley keeps each terminal item's id in a table of its own, and the restore hands
it back to the terminal builder. A split of a restored terminal, a new terminal and a task still
get no one else's id.

## Scope
### In
- **Marley's table** (`crates/marley_workbench/src/terminal_ids.rs`, new): `MarleyTerminalIdsDb`,
  its own database domain after Zed's `WorkspaceDb`, as #494's `MarleyBrowserTabsDb`; the table
  `marley_terminal_ids(workspace_id, item_id, terminal_id)`, keyed by the pair, its rows deleted
  with their workspace. The workbench sets the hook below at `init`.
- **The hook** (`crates/terminal_view/src/terminal_view.rs`): a global,
  `MarleyTerminalIdentity`, of four functions (the saved id of an item, save an item's id, follow
  an item to a new workspace id, clean up the items not loaded), called where Zed saves, restores,
  re-homes and cleans up its own terminal rows. With no hook set (Zed's own tests), nothing
  changes.
- **The handoff** (`crates/project/src/terminals.rs`, `crates/terminal/src/terminal.rs`):
  `Project::create_terminal_shell_restoring(cwd, terminal_id)` puts a saved id into a local
  terminal's environment under a private key, `MARLEY_RESTORED_TERMINAL_ID`; the builder takes the
  key out before anything reads the environment and uses a well-formed id in place of a new one.
- The terminal panel's terminals come back through the same `TerminalView::deserialize`, so they
  keep their ids too.

### Out (explicitly deferred)
- A terminal's process surviving a restart (Orca's daemon); resuming the agent is #540's.
- The id of a terminal Zed does not restore (a closed terminal, a task, a remote terminal).
- #494's `marley_browser_tabs` keeps `item_id UNIQUE`, the constraint Zed dropped from its own
  `terminals` table because item ids repeat across launches: TICKET-576, not this slice.
- A restored center terminal's saved folder, which the terminal panel's cleanup can delete
  before the restore reads it (found in Test): TICKET-577.

## Reference (§20)
- **Orca:** a terminal's identity rides the environment (`ORCA_TERMINAL_HANDLE`), which a
  long-lived shell keeps across an app restart, so the CLI checks the handle live and remints a
  stale one from a pane key that survives (report 06 §2.5, `terminal-identity.ts`). Marley's
  shells do not outlive Marley; the restored terminal's new shell is given the id its terminal
  had, which is the pane key's role.
- **Upstream Zed:** `terminal_view`'s persistence (the `terminals` table, `TerminalView`'s
  `serialize`, `deserialize`, `cleanup` and `added_to_workspace`, `delete_unloaded_items`) and the
  terminal builder's copy template, kept as they are; Marley's rows follow Zed's through the hook.
- **Warp:** N/A. Plumbing under Marley's tools; no Warp behavior to match.

### Prior art
- **Behavior maps.** Orca's report 06 §2.5 above.
- **Published material.** None applies: SQLite through Zed's `sqlez`.
- **The code we already ship.** Zed's `terminals` table keys rows by `(workspace_id, item_id)`
  and dropped `UNIQUE(item_id)` in a later migration, since item ids are entity ids that repeat
  across launches (L-claude-494); `TerminalView::serialize` saves only while `needs_serialize`,
  which a restored terminal's first event sets, so it saves again under its new item id;
  `added_to_workspace` moves a row to a new workspace id (`update_workspace_id`); `cleanup` is
  `delete_unloaded_items`. #494's `MarleyBrowserTabsDb` (`browser.rs`, `mod persistence`) is a
  Marley table after `WorkspaceDb` with the same foreign key. #477's `MarleyTerminalFooter` and
  #484's `MarleyTerminalSuggestion` are hook globals in `terminal_view` that the workbench sets.
  #520's builder hunk mints the id beside #474's nonce; `clone_builder` rebuilds a split from the
  template's environment, saved after that hunk. Does a crate we build own this seam? Zed owns
  the restore and its table; a column there was rejected in AD-claude-520 (Marley migrations would
  sit in upstream's list), so the id gets Marley's own table, reached through a hook.

## UI proof
UI-AFFECTING: the id a restored terminal's programs see, and Marley's tools with it.
`script/e2e/575-terminal-id-across-restore.sh` (`compositor sway`; a foreign id exported under
the private key before the launch; a scratch HOME whose `ids` prints the pane's label,
`MARLEY_TERMINAL_ID`, `MARLEY_PROJECT` and the private key, and whose `agent` runs the stand-in
through the plugin's bridge). Shots:
- `575-01-before`: two terminals in a split, each with its `ids` line.
- `575-02-restored`: after a quit and a launch, both terminals back, each `ids` line with the id
  it had, and the stand-in's `terminals` marking the right one `self` under that id.
- `575-03-restored-again`: after a second quit and launch, the same ids.
- `575-04-split-and-new`: a split of a restored terminal and a new terminal, each with a new id.

## Locked-In Decisions
- D1: The id lives in Marley's own table, `marley_terminal_ids(workspace_id, item_id,
  terminal_id)`, keyed by the pair with no `UNIQUE(item_id)` (Zed's `terminals` dropped it: item
  ids repeat across launches), its rows deleted with their workspace, in its own domain after
  `WorkspaceDb` (AD-claude-520 rejected a column in Zed's table).
- D2: Zed's `TerminalView` reaches the table through one hook global, `MarleyTerminalIdentity`,
  which the workbench sets (AD-claude-477's pattern); Marley's rows follow Zed's own: saved in
  `serialize`, read in `deserialize`, moved in `added_to_workspace`, cleaned up in `cleanup`.
  (Test: the restore reads the ids from memory, the table as `init` read it and each id saved or
  moved since, because the terminal panel's own cleanup, which lists the panel's items alone, can
  delete a row between two terminals' restores.)
- D3: The saved id reaches the builder under a private key, `MARLEY_RESTORED_TERMINAL_ID`, which
  the builder takes from every terminal's environment, leaving the key empty, before any other
  use; a local interactive terminal takes a well-formed id in place of a new one. The id goes
  only into a restored local terminal's environment (a remote shell's environment leaves the
  machine), and the project sets the key empty for every other terminal, since a program
  inherits Marley's own environment besides the map
  (PR-claude-empty-a-variable-the-child-must-not-inherit-001), so an inherited value is never
  taken for an id.
- D4: A split of a restored terminal, a new terminal and a task get a new id or none, as in
  #520: the template a split is rebuilt from is saved after the key is taken out.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley quits and launches again, each terminal it restores shall have the `MARLEY_TERMINAL_ID` it had before the quit, after each of two relaunches. | Shots `575-02-restored`, `575-03-restored-again`; the run log compares each pane's ids |
| REQ-002 | WHEN a restored terminal's agent calls `terminal_list`, the system shall mark that terminal `self` under its restored id. | Shot `575-02-restored`; the run log |
| REQ-003 | WHEN a restored terminal is split, or a new terminal opens, the new terminal shall get an id no other terminal has. | Shot `575-04-split-and-new`; the run log |
| REQ-004 | No terminal's programs shall see an id under the private key, an inherited value included, and no terminal shall take an inherited value for its id. | The run log: a foreign id exported before the launch; every `ids` line prints the key empty, and no id is the foreign one |
| REQ-005 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec, and the design in the notes.
- **P2 Code:** the ledger rows first; the key and the builder's take; the project's restoring
  function; the hook and its four calls; Marley's table and `init`. fmt and clippy clean; a
  review.
- **P3 Test:** write and run the scenario, read every shot, `just regress`,
  `script/gates.sh --diff`.
- **P4 Complete:** CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the ledger rows,
  a follow-up ticket for the browser table's constraint, close, archive, commit.
