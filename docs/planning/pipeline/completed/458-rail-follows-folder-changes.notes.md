# The rail follows a project's folders — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-458-rail-follows-folder-changes.md
- **Pipeline spec:** 458-rail-follows-folder-changes.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-458, found in #453's Code phase; autonomous per Chad's goal "lets
  continue working on the remaining tickets".
- **Classification / tier:** bug, small; `marley_workbench` only.
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Recall (§18.3).**
  - `F-claude-453-the-rail-missed-a-projects-last-folder-going-001`: the cause, the Zed lines
    behind it, and the workaround in #453's test.
  - Brain: consultation `5fdad61b0d114ce6bf613404fa05369c`, nothing on this seam.
- **Discovery:** the seam was known from #453; no sweep beyond checking `rail.rs`'s `Watched`,
  `resubscribe` and `sync_subscriptions`.

### Design
- `Watched` gains `projects`, each listed workspace's project.
- `Rail` gains `project_subscriptions: HashMap<EntityId, Subscription>`. `sync_subscriptions`
  keeps it with `resubscribe`, like the others, and each subscription refreshes on the four
  worktree events and returns on every other event.
- **File manifest.** Marley only: `crates/marley_workbench/src/rail.rs` and `rail_tests.rs`.
  Docs at Complete: `CHANGELOG.md`, `docs/marley_architecture/marley_workbench.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: `enter_with_no_row_highlighted_does_nothing` without its extra terminal (the removal alone takes beta's row away); a new test adds a second folder to a project and removes it, and the row's name follows each change with nothing else happening |
| 002 | (added in Code) driven: the add-and-remove test also reads the switcher's order after each change |
| 003 | `script/gates.sh --diff` |

### Risks
- **Event volume.** Projects emit many events; the filter keeps the rail's rebuilds to folder
  changes.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] recall · [x] marker present · [x] `rail.rs` · [x] `rail_tests.rs` · [x] fmt
  and clippy · [x] review.
- **Built.**
  - `Watched::projects`, each listed workspace's project; the agent servers now come from it.
  - `Rail::project_subscriptions`, kept by `resubscribe` in `sync_subscriptions`, one
    `follow_folders` subscription per project.
  - `changes_the_folders`, the four events, as a `const fn` beside `changes_the_row`.
- **Deviation: the rebuild is deferred (D2).** The Phase 1 design refreshed in the handler.
  Reading `Project::on_worktree_store_event` showed the order: `WorktreeAdded` or
  `WorktreeRemoved` first, then `WorktreePathsChanged` from `emit_group_key_changed_if_needed`,
  and the `MultiWorkspace` rekeys the group only on the second
  (`multi_workspace.rs:587-604`). A refresh on the first would read a workspace whose live key
  matches no stored group, so `build_snapshot` would drop the project for one rebuild;
  `note_window_row` would forget its rows' recency and `note_ended_runs` its threads' dots. The
  handler now calls `cx.defer_in(window, Self::refresh)`, which runs after both events. Zed's
  sidebar avoids the same window with `schedule_update_entries`, a spawned update.
- **Deviation: `follow_folders`.** Written inline, the new subscription took
  `sync_subscriptions` to 102 lines (`clippy::too_many_lines`); it moved into its own method.
- **Tests grown with the code** (`rail_tests.rs`): the helpers `add_folder` and `remove_folder`;
  `a_project_that_loses_its_last_folder_leaves_the_rail`;
  `a_folder_added_or_removed_renames_the_row_and_keeps_its_rows_recency`;
  `only_a_change_of_folders_rebuilds_the_rail`; and `enter_with_no_row_highlighted_does_nothing`
  without its extra terminal.
- **Review.**
  - Re-entrancy: the handler runs while the project's event is delivered, not inside the rail's
    update, and the deferred refresh runs from the effect queue, outside every update.
  - A released rail: `defer_in` holds it weakly and skips the refresh.
  - Duplicate refreshes: one folder change queues two deferred refreshes and the
    `MultiWorkspace`'s notify adds a third. Each reads the same settled state, and `refresh`
    notifies only when the snapshot differs.
  - Provenance: Zed's sidebar was read for the event set; no code came from its body.
  - Upstream: no Zed path changed.

## Phase 3 — Test (2026-09-23)
- **Checklist:** [x] REQ-001 tests · [x] REQ-002 test · [x] negative checks · [x] crate suites ·
  [x] live drive (skipped, reason below) · [x] gate.
- **Tests** (`rail_tests.rs`, written in Code):
  - `a_project_that_loses_its_last_folder_leaves_the_rail` (REQ-001);
  - `enter_with_no_row_highlighted_does_nothing`, now with no extra terminal (REQ-001);
  - `a_folder_added_or_removed_renames_the_row_and_keeps_its_rows_recency` (REQ-001, REQ-002):
    `/gamma` added to alpha renames its row "alpha, gamma", and removed renames it back; after
    each, the switcher still lists server, build, logs;
  - `only_a_change_of_folders_rebuilds_the_rail`: the four events, and two others that are
    not (D1).
- **Negative checks** (`PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001`),
  each restored by sha256 checksum:
  - N1, no rebuild on the folder events: both last-folder tests fail. The rename test passes,
    as it should, since the `MultiWorkspace`'s notify renames a row on its own;
  - N2, a rebuild inside the handler instead of deferred: the rename test fails on the
    switcher's order, `[server, logs, build]` against `[server, build, logs]`;
  - N3, `WorktreeOrderChanged` dropped from the filter: the filter test fails;
  - N4, every project event rebuilds: the filter test fails.
- **Suites:** `cargo nextest run -p marley_workbench -p marley_rail`: 158 passed.
- **Live drive: not run.** It needs a folder removed from a running Marley, which is input.
  Chad was at the desk: `IdleHint=no`, and his windows were on ws3. Owed to the next headless
  capture with the drives for W3 to W6.
- **Gate:** `script/gates.sh --diff`: GATE GREEN [diff], all 18 steps passing; coverage at 100%
  of lines (2592) and functions (394). The receipt matches the tree.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist:** [x] document · [x] knowledge · [x] close the ticket · [x] archive · [x] commit.
- **Docs:** `CHANGELOG.md` (Fixed); `docs/marley_architecture/marley_workbench.md` (what the
  rail follows; the known limit removed; the live drive owed);
  `docs/marley/workbench-shell.md` (#458 fixed). No Zed path changed, so no ledger row.
- **Knowledge:** `L-claude-458-a-projects-folder-events-come-before-its-group-is-rekeyed-001`,
  `AD-claude-458-the-rail-follows-each-projects-folders-with-a-deferred-rebuild-001`; the #453
  failure block now points at the fix. No `F-` block: the timing flaw was found by reading the
  event order before any code shipped with it.
- **Brain:** consultation `5fdad61b0d114ce6bf613404fa05369c` closed with
  `decisions/marleys-rail-follows-each-projects-folders-with-a-deferred-rebuild`, follow-up by
  2026-10-07.
- **Ticket:** #458 closed; no BACKLOG row left.
