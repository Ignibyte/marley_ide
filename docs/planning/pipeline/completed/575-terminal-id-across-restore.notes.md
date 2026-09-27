# A terminal keeps its MARLEY_TERMINAL_ID across a restore — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-575-terminal-id-across-restore.md
- **Pipeline spec:** 575-terminal-id-across-restore.spec.md

## Phase 1 — Plan
- **Request:** cut from TICKET-520 at its promotion (2026-09-26); #520's design, items 2 to 5, its
  D3 and D4 and its queued REQ-004, is the start. Chad's goal of 2026-09-25 covers it ("The rest
  go ahead and begin implementing it now"), and so does 2026-09-26's "The rest of the orca / warp
  findings i want built".
- **Classification / tier:** feature, prong 2 (C0 follow-on). Three Zed files widen by small
  additive hunks (`terminal.rs`, `terminals.rs`, `terminal_view.rs`); the table is Marley's. Size
  M.
- **Checklist (no TaskCreate in this harness):** pick · pre-flight · recall · mint · prior art ·
  spec · design · present: all done here.
- **Recall (§18.3):**
  - L-claude-494-zed-item-ids-change-at-each-launch-001: an item deserializes under its saved id
    and saves again under its new one; a Marley table keyed by item id stays right only if it saves
    when Zed does and deletes the rest in `cleanup`.
  - AD-claude-477: code Marley runs inside Zed's `TerminalView` goes through a hook global the
    workbench sets (`MarleyTerminalFooter`); this hook is the same kind.
  - AD-claude-520: a column in Zed's `terminals` table was rejected, so no Marley migration sits in
    upstream's list; a caller's id scopes defaults and is no authority.
  - PR-claude-a-marley-crate-writes-from-the-contract-not-the-gpl-body-001: the Marley table
    follows `MarleyBrowserTabsDb`'s shape, not Zed's terminal queries.
  - Brain consultation d2e7b3bdb03742d083c9e08764bc2401: nothing on this seam (it listed other
    projects' follow-ups).
- **Discovery (at `93b87ee807`):**
  - `crates/terminal/src/terminal.rs:1155` (`TerminalBuilder::new`, `env` by value), `:1214`
    (#474's nonce), `:1229-1240` (#520's id: minted for a local interactive terminal, the id and
    the project emptied otherwise), `:1451-1453` (the `CopyTemplate` keeps `env` after those
    hunks), `:3418-3436` (`clone_builder` rebuilds a split from `self.template.env`), `:1872`
    (`Terminal::marley_terminal_id`).
  - `crates/project/src/terminals.rs:284` (`create_terminal_shell` → the internal function with
    `force_local` false), `:295` (`create_local_terminal`, `force_local` true), `:312`
    (`create_terminal_shell_internal`), `:385-395` (after `env.extend(settings.env)`, #520's
    `MARLEY_PROJECT`), `:416-425` (`create_remote_shell` takes `env` for a remote client, so the
    key must not go into a remote terminal's map).
  - `crates/terminal_view/src/terminal_view.rs:130-154` (`MarleyFooterContext`,
    `MarleyTerminalFooter`, `MarleyTerminalSuggestion`), `:1158-1166` (a terminal's first event
    sets `needs_serialize`, so a restored terminal saves again under its new item id), `:1895-1915`
    (`added_to_workspace`: `update_workspace_id` when the workspace id changes), `:1927-1935`
    (`cleanup`: `delete_unloaded_items` over `terminals`), `:1937-1968` (`serialize`: the working
    directory and the title, while `needs_serialize`), `:1974-2030` (`deserialize`: both read in a
    foreground update, then `project.create_terminal_shell(cwd)`).
  - `crates/terminal_view/src/persistence.rs:417-445` (the `terminals` table, and the migration that
    dropped `UNIQUE(item_id)`), `:316-335` (the panel's `deserialize_terminal_views` calls
    `TerminalView::deserialize`).
  - `crates/marley_workbench/src/browser.rs:4946-5065` (`SerializableItem for BrowserView` and
    `mod persistence`: `MarleyBrowserTabsDb`, `static_connection!(…, [WorkspaceDb])`, `query!`
    reads and writes, `delete_unloaded_items` in `cleanup`; its table keeps `item_id UNIQUE`).
  - `crates/marley_terminal/src/identity.rs` (`TERMINAL_ID_VARIABLE`, `PROJECT_VARIABLE`,
    `new_terminal_id`, `is_terminal_id`).
- **Decisions:** D1 to D4 in the spec.

### Design
- **Approach.**
  1. `marley_terminal::identity`: `RESTORED_ID_VARIABLE = "MARLEY_RESTORED_TERMINAL_ID"`, the
     private handoff key, documented as never reaching a program.
  2. `TerminalBuilder::new`, in #520's hunk: first `env.remove(RESTORED_ID_VARIABLE)` for every
     terminal, kept when `is_terminal_id`; a local interactive terminal's id is that one, else a
     new one. The removal comes before the template keeps `env`, so a split never sees it.
  3. `crates/project/src/terminals.rs`: `create_terminal_shell_restoring(cwd, terminal_id)` beside
     `create_terminal_shell`; the internal function takes the id as a new last parameter (the two
     existing callers pass `None`) and, after #520's `MARLEY_PROJECT` line, inserts it under the
     key only when the terminal is local (no remote client once `force_local` is applied).
  4. `terminal_view`: `MarleyTerminalIdentity`, a global of four `Arc<dyn Fn>`s: `saved(workspace,
     item) -> Option<String>`, `save(workspace, item, id) -> Task<Result<()>>`, `moved(new, old,
     item) -> Task<Result<()>>` and `cleanup(workspace, alive) -> Task<Result<()>>`. `deserialize`
     reads `saved` in the update that reads the folder and calls `create_terminal_shell_restoring`;
     `serialize` adds `save` of `Terminal::marley_terminal_id` to its background task;
     `added_to_workspace` calls `moved` beside `update_workspace_id`; `cleanup` awaits the hook's
     cleanup after Zed's. With no hook, each call is skipped.
  5. `marley_workbench::terminal_ids` (new): `MarleyTerminalIdsDb`, table
     `marley_terminal_ids(workspace_id INTEGER, item_id INTEGER, terminal_id TEXT NOT NULL,
     PRIMARY KEY(workspace_id, item_id), FOREIGN KEY(workspace_id) REFERENCES
     workspaces(workspace_id) ON DELETE CASCADE) STRICT`, `static_connection!(…, [WorkspaceDb])`;
     `save_id` (`INSERT OR REPLACE`), `saved_id`, `move_id`; `cleanup` through
     `workspace::delete_unloaded_items`. `init` sets the hook; `crate::init` calls it.
- **File manifest.**
  - Marley: `crates/marley_terminal/src/identity.rs`,
    `crates/marley_workbench/src/terminal_ids.rs` (new),
    `crates/marley_workbench/src/marley_workbench.rs` (the module and `init`),
    `script/e2e/575-terminal-id-across-restore.sh` (Test).
  - Zed: `crates/terminal/src/terminal.rs` (the key's take in #520's hunk),
    `crates/project/src/terminals.rs` (`create_terminal_shell_restoring`, the parameter, the key),
    `crates/terminal_view/src/terminal_view.rs` (the hook and its four calls). Ledger rows first:
    the three rows exist and widen.

### E2E plan
Fixtures: a scratch repository; a foreign, well-formed id exported under the private key before
the launch (Phase 2); a HOME whose `.bashrc` defines `ids <label>` (echoes the label,
`MARLEY_TERMINAL_ID`, `MARLEY_PROJECT` and `MARLEY_RESTORED_TERMINAL_ID` in brackets, or `unset`,
teed to `ids.log`) and `agent` (the stand-in through the plugin's bridge, as #520's scenario
does). The repository opens at launch; `quit_marley` and `launch_marley` relaunch on the same
profile.

| REQ | Scenario part | Shot or log |
|---|---|---|
| — | `ids left-0`; split right; `ids right-0` | `575-01-before` |
| REQ-001 | quit, launch; a click in each pane, `ids left-1`, `ids right-1`: the ids of `-0`, pane by pane | `575-02-restored`; the run log |
| REQ-002 | in the right pane, `agent terminals`: the `(self)` row's id is `right-0`'s | `575-02-restored`; the run log |
| REQ-001 | quit, launch again; `ids left-2`, `ids right-2`: still the ids of `-0` | `575-03-restored-again` |
| REQ-003 | in the left pane, split right, `ids split`; `workspace: new terminal`, `ids new`: two new ids, none of them another's | `575-04-split-and-new` |
| REQ-004 | every `ids` line | the run log: the key empty (`[]`), and no id the foreign one |
| REQ-005 | the golden set with 575 added; the diff gate | `just regress`; `script/gates.sh --diff` |

Not reached: the terminal panel's terminals (the Zed layout). They come back through the same
`TerminalView::deserialize`, so the scenario's center terminals exercise the path.

### Risks
- A restored terminal saves under its new item id only after its first event; a quit before any
  event would lose the row at the next `cleanup`. The shell's prompt is an event, and Zed's own
  folder row has the same timing.
- #494's `marley_browser_tabs` keeps `UNIQUE(item_id)`: a tab saved in one workspace can replace
  another workspace's row for the same item id in a later launch. Out of scope; a follow-up ticket
  at Complete.
- The hook's `saved` reads the table in the foreground update where Zed reads its own row; the
  dylint gate's `blocking_io_on_foreground` decides whether that read must move.

## Phase 2 — Code
- **Checklist (no TaskCreate in this harness):** the ledger rows (three, widened) ·
  `marley_terminal::identity` · the builder's take · the project's restoring function · the hook
  and its four calls · `marley_workbench::terminal_ids` and `init` · fmt, clippy, rustdoc ·
  review: all done.
- **Built.**
  - `marley_terminal::identity::RESTORED_ID_VARIABLE` (`MARLEY_RESTORED_TERMINAL_ID`).
  - `TerminalBuilder::new`, in #520's hunk: `env.insert(key, "")` takes the handed-over value and
    leaves the key empty for every terminal; a local interactive terminal's id is that value when
    `is_terminal_id` holds, else a new one. It runs before the template keeps `env`, so a split's
    rebuild finds the key empty and mints its own.
  - `Project::create_terminal_shell_restoring(cwd, terminal_id)`; the internal function's new last
    parameter (`None` from `create_terminal_shell` and `create_local_terminal`) sets the key after
    #520's `MARLEY_PROJECT`: the saved id for a local terminal, empty otherwise.
  - `terminal_view::MarleyTerminalIdentity { saved, save, moved, cleanup }` (`Arc<dyn Fn>`s):
    `deserialize` reads `saved` in a foreground update of its own after Zed's read and opens the
    shell with `create_terminal_shell_restoring`; `serialize` makes the `save` task of
    `Terminal::marley_terminal_id` and awaits it inside its own background task, after Zed's saves;
    `added_to_workspace` calls `moved` beside `update_workspace_id`, detached with its error
    logged; `cleanup` runs the hook's cleanup, detached with its error logged, before Zed's.
  - `marley_workbench::terminal_ids`: `MarleyTerminalIdsDb` (domain after `WorkspaceDb`), table
    `marley_terminal_ids` keyed by `(workspace_id, item_id)` with the workspace's cascade and no
    `UNIQUE(item_id)`; `save_id`, `saved_id` (filtered to well-formed ids), `move_id`, and the
    cleanup through `workspace::delete_unloaded_items`; `init` sets the hook, called from
    `crate::init`.
- **Deviations from the design, and why.**
  - The key is emptied, not removed, and the project sets it for every terminal it builds (the
    saved id or empty): a program inherits Marley's own environment besides the map
    (PR-claude-empty-a-variable-the-child-must-not-inherit-001), and the directory environment Zed
    captures carries an inherited value into the map, where the builder would have taken a
    well-formed one for a new terminal's id. The spec's D3 and REQ-004 say so now, and the
    scenario exports a foreign id before the launch.
  - `cleanup` and `moved` run detached beside Zed's own calls, so Zed's lines stay as they are;
    `save` is awaited inside `serialize`'s task, since Zed flushes those at a quit and a detached
    save could be lost.
- **Review.**
  - Criteria: REQ-001 (saved on the terminal's first event under its item id, read at the
    restore before Zed's cleanup runs, handed to the builder); REQ-002 (`terminal_list` reads
    `Terminal::marley_terminal_id`, now the restored id); REQ-003 (a split finds the key empty; a
    new terminal gets it empty from the project); REQ-004 (every map ends with the key empty).
  - The other ways a terminal is built: tasks (`create_terminal_task`) get no id, and the builder
    empties the key; `clone_builder` rebuilds from the template; the rest are tests.
  - Re-entrancy: the hook's four functions read a database global and spawn background tasks;
    none touches an entity.
  - Provenance: `terminal_ids.rs` follows #494's `MarleyBrowserTabsDb`; the one `UPDATE` is the
    statement the schema allows.
  - Found and fixed in the review: the inherited-value path above (before any build).
- **Checks:** `cargo check`, `cargo fmt --check`, `cargo clippy -p marley_terminal -p terminal -p
  project -p terminal_view -p marley_workbench --all-targets -- -D warnings` (a first run red on a
  long first doc paragraph in `identity.rs`, fixed), `RUSTDOCFLAGS="-D warnings" cargo doc
  --no-deps` on the two Marley crates: clean.
- **The release install** (`just install`, started after #574's commit) overlapped this phase's
  first edits; `strings` found none of #575's names in its binary (the constant's literal among
  them), so the installed build is `93b87ee807`'s code.

## Phase 3 — Test
- **Checklist (no TaskCreate in this harness):** 575-01 · REQ-001 (first relaunch) · REQ-002 ·
  REQ-001 (second relaunch) · REQ-003 · REQ-004 · golden set · gate.
- **Scenario:** `script/e2e/575-terminal-id-across-restore.sh` (`compositor sway`). Setup exports
  a foreign, well-formed id (`00000000-0000-4000-8000-000000000575`) under the private key; the
  scratch HOME's `ids <label>` prints the label, both variables and the key in brackets (`[]`
  when set and empty, `unset` when unset), teed to `ids.log`; `agent` runs the stand-in through
  the plugin's bridge. Steps: `ids left-0`, a split, `ids right-0`; quit and launch; `ids left-1`
  and `ids right-1` by a click in each pane, `agent terminals` in the right; quit and launch
  again, `ids left-2`, `ids right-2`; a split of the left, `ids split`, a new terminal, `ids new`.
- **A red, and the bug it found.** The first run failed REQ-001 for the right terminal alone: it
  came back with a new id. A dump of the profile's rows after the quit showed both terminals'
  ids saved; a temporary log of the hook's calls at the relaunch (L-claude-544) showed the order:
  the left terminal's id read, then a `cleanup` of workspace 7 with no items, then the right's
  read finding nothing, then the workspace's own cleanup with both new items. The empty call is
  the terminal panel's: `TerminalPanel` restores its own terminals and then runs
  `TerminalView::cleanup` with the panel's items alone (`terminal_panel.rs:359-378`), none in
  the Marley layout, which deletes every terminal row of the workspace, and it runs between two
  center terminals' restores. Zed's own `terminals` rows meet the same delete, so a restored
  center terminal can lose its saved folder too (unseen here, where the folder is the project's,
  which is also the fallback).
- **The fix (Code, reopened):** `terminal_ids` keeps the ids in memory (`KnownIds`: the table's
  rows read at `init`, which runs before any window restores, and each id saved or moved since),
  and `saved` answers from there, then from the table. The hook's `save` and `moved` take
  `&mut App` for that; `serialize` computes the id, then clones the hook out of the global, so
  the terminal's borrow ends before the call. The temporary logs and the row dumps are gone.
  clippy (`-p terminal_view -p marley_workbench --all-targets -D warnings`) and fmt clean.
- **The final run: every check passes.** Both terminals came back with their ids after the first
  relaunch and after the second; the stand-in's `terminals` marked the right terminal `self`
  under its restored id; the split and the new terminal got ids no other terminal had; no `ids`
  line showed the foreign value or anything but an empty key.
- **Shots, read:**
  - `575-01-before`: repo's two terminals in a split, the right one's `ids right-0` line with
    its id and `restored=[]`; the rail lists both terminals.
  - `575-02-restored`: after the relaunch, the split back; `ids left-1` and `ids right-1` print
    the ids of `-0`; `agent terminals` in the right pane lists the left terminal under its id and
    the right one `(self)` under its restored id.
  - `575-03-restored-again`: after the second relaunch, `ids left-2` and `ids right-2` print the
    same ids.
  - `575-04-split-and-new`: three panes, the middle one the left's split holding a new terminal
    tab whose `ids new` line shows an id of its own; the rail lists four terminals.
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule
  and did not reload it".
- **Not reached:** the terminal panel's own terminals (the Zed layout); they restore through the
  same `TerminalView::deserialize` and the same memory.
- **Golden set:** 575 added (`script/e2e/golden`); `just regress`: all 17 passed.
- **Gate:** `script/gates.sh --diff`: 16 passed, 0 failed, `GATE GREEN [diff]`, receipt written.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: a terminal keeps its id across a restart);
  `docs/marley_architecture/marley_workbench.md` (the new section, "Terminal ids across a
  restore"); `docs/marley/zed-touchpoints.md` (the three rows, widened in Code, checked against
  what shipped: the terminal view's row now says the id is read in an update of its own).
- **Knowledge appended:** F-claude-575-the-terminal-panels-cleanup-deleted-the-center-terminals-rows-001
  (medium; found in Test); PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001;
  L-claude-575-log-a-hooks-calls-in-order-to-find-a-race-001;
  AD-claude-575-a-restored-terminal-keeps-its-id-through-marleys-table-001.
- **Follow-ups minted:** TICKET-577 (a restored center terminal's folder, which the panel's
  cleanup can delete before its restore reads it) and TICKET-576 (`marley_browser_tabs`'
  `UNIQUE(item_id)`), queued first.
- **Brain:** `rusty-cli brain decide d2e7b3bdb03742d083c9e08764bc2401` →
  `decisions/marley-keeps-a-restored-terminals-marley-terminal-id-in-its-own-table`.
- **Ticket:** closed; the pipeline archived to `completed/`.
