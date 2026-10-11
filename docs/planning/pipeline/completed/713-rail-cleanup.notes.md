# Rail cleanup - one sidebar's subscriptions, refreshes on row changes, a silent swap, saves after the restore — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-713-rail-cleanup.md
- **Pipeline spec:** 713-rail-cleanup.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, the finish-line list's row 8 (the rail cleanup among the small
  fixes): "8.) yes to all". Built in the Queue after the tabs batch (#743 to #749).
- **Classification:** chore. One Zed file (`multi_workspace.rs`, three hunks), the rest in
  `marley_workbench`. Size M: four independent items, each small; they share the rail and the one
  Zed file, so they ship as one slice.
- **Recall (§18.3):**
  - #438's inspect is the source of all four (`docs/planning/pipeline/completed/438-marley-layout-and-rail.notes.md`):
    C9 (telemetry, line 263), S5 (a window restored in the Marley layout saves a partial state
    first, 307-308), S6 (two subscriptions per swap, 309-312), G10 (the snapshot rebuilt on each
    `Wakeup` and `UpdateTab`, 356-357). #442 carried them; the 2026-10-09 audit left these four
    (`docs/planning/intake/rail-internals.md:19-29`).
  - AD-claude-438-the-marley-layout-swaps-the-sidebar-and-two-defaults-001 and
    F-claude-438-b-a-layout-swap-dropped-zeds-sidebar-and-its-state-001: the rail keeps Zed's
    sidebar alive across a swap and answers the window's persistence with its state. D1 must not
    drop that kept sidebar, only the stale subscription pairs.
  - AD-claude-442-the-rail-adds-its-fields-to-zeds-sidebar-blob-001 and
    F-claude-442-a-close-deferred-on-the-rail-ran-inside-the-rails-update-001: the restore runs
    inside the `MultiWorkspace`'s update, and the restored close defers with `window.defer`. The
    hold's release and the silent close follow the same rule.
  - AD-claude-601-groups-are-restored-from-two-stores-001: the window's groups and order live in
    the sidebar blob; `groups::reopen` brings the rest back after Zed restores the shown workspace.
  - F-claude-504-a-favicons-arrival-changed-nothing-the-rail-compares-001 and its prevention rule:
    a redraw gate must compare everything the row draws. D3's per-row comparison is such a gate.
  - L-claude-572-an-observed-global-written-at-every-wakeup-redraws-the-rail-001: nothing written
    per wakeup may sit in a global the rail observes.
  - L-claude-670-a-restore-scenario-relaunches-with-no-path-001 and
    L-claude-601-a-launch-with-a-path-restores-no-session-001: the restore step clears `OPEN`
    before each relaunch.
  - Brain: not consulted (`/spec` drafts; the promotion runs the loop).
- **Discovery** (line numbers at `19dda499a2`; #743's Code phase is editing `rail.rs` and
  `groups.rs` as this is written, so they will move):
  - `crates/workspace/src/multi_workspace.rs`: fields `sidebar` and `_subscriptions` (329, 334);
    `new` puts the release and settings subscriptions in `_subscriptions` (395);
    `register_sidebar` pushes an observe and a `SerializeNeeded` subscription (400-412);
    `open_sidebar` with its event (500-507), the silent `restore_open_sidebar`, `pub(crate)`
    (509-513), `apply_open_sidebar`, which serializes and pins the active workspace through
    `retain_active_workspace` (515-526, 1384-1391); `close_sidebar` with its event (528-551);
    `serialize` (1456-1463), `serialize_now` (1465-1486), `flush_serialization` (1490-1494). The
    one Marley hunk today is `MarleyKeptWorkspaces` (289-300, and `open_project` at 1944), ledger
    row `docs/marley/zed-touchpoints.md:98`.
  - `crates/workspace/src/workspace.rs:10495-10512`: the restore's `restore_open_sidebar`, then the
    sidebar's `restore_serialized_state`, then `serialize`.
  - `crates/zed/src/zed.rs:539-547`: the deferred `marley_workbench::register_sidebar` at window
    creation.
  - `crates/marley_workbench/src/marley_workbench.rs:971-1040`: `register_sidebar`; Zed's arm opens
    (1007) or closes (1010), Marley's arm opens (1028); the settings observer swaps every window
    (1099-1123).
  - `crates/marley_workbench/src/rail.rs`: `SETTLE` (119); `StartupGroups` (262-281);
    `groups_changed` serializes (1082-1086); `group_ids` from the snapshot's live groups
    (1090-1096); `refresh` (1107-1154; the redraw comparison at 1143); `note_output` (2387-2399);
    the workspace subscription (about 2425-2429), the terminal subscriptions, `Wakeup` to
    `note_output` and every `ItemEvent` to `refresh` (2437-2455); `follow_panes`, every pane event
    to `refresh` (2586-2596); `terminal_snapshot` (7758); `serialized_state` (9366-9375);
    `restore_serialized_state`, which defers `groups::reopen` and the close (9377-9417).
  - `crates/marley_workbench/src/rail_order.rs:445-454`: `order_changed` prunes the saved rows to
    the listed groups.
  - `crates/marley_workbench/src/groups.rs:217-262`: `reopen` and `forget_pending`.
  - `crates/marley_workbench/src/home_page.rs:106`, `192-206`: the page caches a strong
    `Entity<Rail>` and its observe the first time it finds one.
  - `crates/terminal_view/src/terminal_view.rs:1565-1570`: a `Wakeup` also emits `UpdateTab`;
    `Bell` and Marley's notification re-emit `Wakeup` (1588-1604); `TitleChanged` reaches the rail
    as `UpdateTab` (1625-1627). `crates/workspace/src/item.rs:891-922`: `UpdateTab` becomes the
    pane's `ChangeItemTitle`, which carries no item; `workspace.rs:6251-6259`: the active pane's
    becomes `ActiveItemChanged`.
  - What a terminal row reads that changes without an event of its own: the bell, the command's
    running state and exit (a block's end notifies the terminal with no event,
    `running_errors.rs:113`, `command_watch.rs:64`), the folder, and the quiet time. All are inside
    the row `terminal_snapshot` builds, so the per-row comparison sees them.

### Design (draft; confirmed at promotion)
- **`multi_workspace.rs` (Zed crate), three hunks with `// Marley:` lines:**
  - a `sidebar_subscriptions: Vec<Subscription>` field, set (not pushed) in `register_sidebar`;
  - `close_sidebar`'s body moved into `apply_close_sidebar(window, cx)`; `close_sidebar` keeps the
    event and calls it; a `pub fn restore_closed_sidebar(window, cx)` calls it silently;
    `restore_open_sidebar` becomes `pub`;
  - a `serialize_held` flag with `hold_serialization(held, cx)`: `serialize` and
    `flush_serialization` skip the write while it is set and note one was asked; clearing it
    serializes once.
- **`marley_workbench.rs` (Marley crate):** `register_sidebar` calls the silent open and close in
  both arms.
- **`rail.rs` (Marley crate):**
  - The output gate: `note_output` stamps the time, builds the view's row with the inputs its last
    refresh used (kept per terminal), and calls `refresh` only on a difference; the terminal's
    `ItemEvent` subscription goes through the same check; the quiet timer stays. The pane and
    workspace subscriptions check `ChangeItemTitle` and `ActiveItemChanged` against a per-pane tab
    fingerprint (ids, titles, dirty, active) kept from the last refresh; other events refresh.
  - The hold: `Rail::new` with no kept sidebar (built with its window) asks the `MultiWorkspace` to
    hold, deferred (F-442-a); `refresh` releases once the restore is done (every saved group held
    or no longer pending), or once `SETTLE` has passed without `restore_serialized_state`; a 10 s
    timer releases regardless. The release is deferred with `window.defer` and logged.
  - The silent close in the restored-closed path (9403-9415).
  - `log::debug!("rail {:?} refresh", entity id)` at the top of `refresh`.
- **`rail_order.rs` (Marley crate):** `order_changed` keeps the rows of groups still in
  `saved_order.groups` that have not come back.
- **`home_page.rs` (Marley crate):** the page keeps a `WeakEntity<Rail>` and its subscription;
  `rail()` drops them and finds the window's rail again when the cached one is gone or is not the
  window's sidebar.
- **File manifest:** `crates/workspace/src/multi_workspace.rs` (Zed crate);
  `crates/marley_workbench/src/{marley_workbench.rs, rail.rs, rail_order.rs, home_page.rs}`
  (Marley crate); `script/e2e/713-rail-cleanup.sh`; the architecture note, the ledger row and the
  intake at Complete.
- **Ledger row:** the existing `crates/workspace/src/multi_workspace.rs` row (98) extended with the
  three hunks, before the edits: what changed, why (swaps, Marley's silent opens, saves after the
  restore), and at a merge: keep the field set at register, the silent close beside upstream's
  silent open, and the hold in front of `serialize_now`.

### Visual check plan
| REQ | The scenario does | Evidence |
|---|---|---|
| 003, 004 | A terminal in `repo` runs a 300-line print over six seconds, ending with a bell | `713-01-printing` (the running row), `713-02-finished` (the finished row and the bell's dot); the count of `rail … refresh` lines between two markers written to the terminal, ten or fewer |
| 004 | The stand-in agent prints for three seconds, then waits | `713-03-agent-working`, `713-04-agent-waiting` |
| 001, 002 | Home's page open; the two layout commands from the palette; a second stand-in agent | `713-05-home-after-swap` (the page lists it); after the swap back every refresh line names the new rail |
| 006 | Two groups from the rail's empty space; quit; `open_path ""`; launch; SIGKILL as the window maps; launch | `713-06-restored` (both groups back, in order); the killed launch's log holds the saves with no release; the last launch's logs one release |
| 005, 007 | — | Review of the diff |

- **Before the change, for contrast:** the Test phase may run the print step once on the installed
  build with the same log level, to record how many refreshes the print made before.
- **Out of a scenario's reach:** the telemetry event (off since #514, nothing sent) and a reorder
  timed inside the restore.

### Risks
- **#743 is active and changes `refresh`:** it adds a `retire_rusty_groups` step after
  `groups::adopt` that removes an earlier build's Rusty group through `remove_group`, which saves
  through `groups_changed`. Under the hold those saves wait for the release, which is right, and
  the retire is structural, so the output gate never skips it. The promotion re-reads `refresh`,
  `ensure_groups` and the retire step as #743 left them.
- **Passes that rode the output refreshes:** ended runs and the command row (inside the terminal's
  row), the inbox's ages (its own 30 s timer), drift (scheduled at a refresh when tips moved; git
  events still refresh), Claude Code's hook events (their own global), the cwd subtitle (inside the
  row). The promotion lists every pass in `refresh` with the trigger it keeps.
- **Building one row per wakeup** reads the foreground command from the terminal's cached process
  info; it is far less than the full snapshot, but it runs per chunk.
- **The hold and a crash:** a crash during the hold loses at most the first 10 s of the session's
  changes; a quit during it keeps the previous saved state.
- **Re-entrancy:** the hold is set and released outside the `MultiWorkspace`'s update
  (F-claude-442-a), and the rail's comparison reads only its own fields and the terminal.
- **Upstream merges:** `multi_workspace.rs` grows from one Marley hunk to four; the moved close body
  is the one change that is not purely added, and the row says so.

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress.
- **Seams checked again** at `0593fb97b3` (an Explore pass). `multi_workspace.rs`:
  `register_sidebar` 400-412 still pushes two subscriptions into `_subscriptions`;
  `open_sidebar` 500-507 (event at 505), `restore_open_sidebar` 511-513 `pub(crate)`,
  `apply_open_sidebar` 515-526, `close_sidebar` 528-551 (event at 533); `serialize` 1456-1463,
  `serialize_now` 1465-1486, `flush_serialization` 1490-1494; no flag holds a save.
  `workspace.rs`: `apply_restored_multiworkspace_state` at 10445, its order 10491 (groups), 10496
  (`restore_open_sidebar`), 10504 (the sidebar's state), 10510 (`serialize`). `rail.rs` moved with
  today's tickets: `SETTLE` 120, `StartupGroups` 263-285, `refresh` 1203-1251 (the comparison
  gates only `cx.notify()`), `ensure_groups` 1300-1329, `sync_subscriptions` 2484 (the terminal
  subscriptions 2520-2538, `note_output` 2470-2482, the workspace subscription 2504-2514),
  `follow_panes` 2681-2689, `terminal_snapshot` 7954, `serialized_state` 9555-9564,
  `restore_serialized_state` (sets `restored` at 9576, the deferred close at 9601). `groups.rs`:
  `adopt` 169, `reopen` 223-264, `forget_pending` 267-272, `pending_kind` 427. `rail_order.rs`:
  `drop_header` 371-389, `order_changed` 445-457. `home_page.rs`: the strong `rail` cache at 106 and
  `rail()` 192-206. `terminal_view.rs`: `Wakeup` emits `UpdateTab` 1565-1571; `Bell` and
  `MarleyNotification` re-emit `Wakeup` without it.
- **What the pass added:**
  - `drop_header` also rebuilds `saved_order.headers` from the groups listed now, so REQ-007 keeps
    the headers of groups still reopening as well as `order_changed`'s rows.
  - `serialized_state` writes `group_ids` from the snapshot (live groups only); the hold is what
    keeps an early save from writing the window's groups without the ones still reopening.
  - A rail built with its window is the one with no kept sidebar: at a window's opening no sidebar
    is registered yet (`marley_workbench::register_sidebar`, Marley's arm), while a swap from Zed's
    layout passes Zed's sidebar as `kept`. The hold keys on `kept.is_none()`.
  - #743's `retire_rusty_groups` sits in `refresh` after `groups::adopt`; it is structural, so the
    output gate never stands in its way, and its saves wait for the hold's release.
- **Brain:** consultation c1a5a5cc25e74fde9393f1934d04b161.

## Phase 2 — Code
- **Built:**
  - `multi_workspace.rs` (Zed, the row extended first): `marley_sidebar_subscriptions`, set by
    `register_sidebar`; `apply_close_sidebar` (the moved body), `restore_closed_sidebar`,
    `restore_open_sidebar` made `pub`; `marley_hold_serialization` with two flags, checked first
    in `serialize` and `flush_serialization`.
  - `marley_workbench.rs`: `register_sidebar` opens and closes silently in both arms, and holds
    the window's saves for a rail built with its window (`kept.is_none()`), before its first open.
  - `rail.rs`: `StartupGroups` holds the saves (`saves_held`, a `SAVES_HELD_AT_MOST` 10 s release
    task); `release_saves_once_restored` at the end of `refresh` (restored: no saved group still
    pending, `groups::is_pending`; else the settle) and `release_saves` (logged at info, deferred
    out of any update); `OutputGate` (the output times and quiet timers moved into it, each
    terminal's row as built before the passes, each pane's and workspace's tabs, the window's
    groups and workspaces, a refresh-due flag); `row_changed`, `workspace_event`,
    `terminal_item_event`, `window_notified`, `refresh_soon`; the silent restored close; the
    debug line in `refresh`.
  - `groups.rs`: `is_pending`. `rail_order.rs`: during the hold `drop_header` lays the moved
    headers into the saved places (`keep_unlisted`) and `order_changed` keeps every row.
    `home_page.rs`: the rail held weakly and found again when the window's is another.
- **Deviations:**
  - The window's own observer is gated too (`window_notified`, `WindowSeen`): a terminal's
    retitled tab makes its workspace notify, which the `MultiWorkspace` passes on, and the rail
    refreshed on each. The plan gated only the terminal, pane and workspace events.
  - Gated paths refresh at the end of the effect cycle (`refresh_soon`): one change reaches the
    rail as a terminal's, a pane's, a workspace's and the window's event, which made three or four
    refreshes.
  - The output times and quiet timers moved into `OutputGate`, to keep `Rail::new` and
    `sync_subscriptions` under clippy's 100 lines; the gated handlers became methods for the same
    reason.
- **Review of the diff:** the hold and the release run outside the `MultiWorkspace`'s update
  (`window.defer`); the gate's fingerprints read only the panes, workspaces and window the event
  names; a row compares everything `terminal_snapshot` builds except `matched` and `worktree`,
  which come from the last row; every event other than the four gated kinds refreshes as before.
- **Clippy:** `too_many_lines` (twice), `used_underscore_binding`, `trivially_copy_pass_by_ref`,
  `needless_pass_by_ref_mut`; each fixed at the source.

## Phase 3 — Test
- **Scenario:** `script/e2e/713-rail-cleanup.sh` (`compositor sway`, `ZED_LOG=info,marley_workbench=debug`).
- **What the runs found:**
  - The first run counted 103 refreshes over a six-second print. A temporary backtrace on each
    refresh (removed after) named the callers: a shell loop retitles the tab for every command it
    runs (bash's per-command hook), which is a change the row shows, so the print became one
    `python3` process; and the `MultiWorkspace` observer refreshed on every workspace notify. That
    led to the window gate and `refresh_soon` above. With both: 0 refreshes in two seconds
    mid-print.
  - Scenario only: the fake `claude` ran as `/bin/sh …/claude`, not read as an agent (now `exec -a
    claude` from bash); the empty-space menu selects no entry as it opens, so New Group… is
    clicked (the queued draft's `menu_entry 1` pressed Return on nothing).
- **Shots** (the last run):
  - `713-01-printing` (REQ-003, REQ-004): the row "repo — python3 -c import…", `· running`;
    `refreshes during the print: 0` between 1.5 s and 3.5 s.
  - `713-02-finished` (REQ-004): "repo — bash", `· done · 6 s`, the bell's dot on the row and the tab.
  - `713-03-agent-working`, `713-04-agent-waiting` (REQ-004): the fake agent's row "Claude Code ·
    working", then "Claude Code · waiting" once quiet (crops read at 3×).
  - `713-05-home-after-swap` (REQ-001, REQ-002): after Zed's layout and back, a second fake agent:
    two Claude Code rows, and Home's AGENTS AT WORK lists both; `rails refreshing after the swap:
    rail EntityId(201v5) refresh`, one rail.
  - `713-06-restored` (REQ-006): Alpha and Beta made from the empty space; quit; a launch killed
    as its window mapped; a launch again: Default, repo, Alpha, Beta, in order. The log: the first
    launch released its saves 4 s after it started; the killed launch (12:54:32) wrote no release
    before the kill; the last (12:54:35) released at 12:54:36.
- **REQ-005, REQ-007** by the review: telemetry is off (#514); a reorder timed inside the restore
  is not reliable in a scenario.
- **Gate:** `just gate-diff` GREEN, 17 passed, over the code and the scenario.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Fixed: the rail's cleanup); `docs/marley_architecture/marley_workbench.md`
  (a #713 paragraph at the head of the rail's section); `docs/marley/workbench-shell.md` (the
  slice line); `docs/marley/zed-touchpoints.md` (the `multi_workspace.rs` row, written before the
  hunks, matches what shipped); `docs/planning/intake/rail-internals.md` (the four items shipped;
  the restart check remains).
- **Ledgers:** F-claude-713-the-window-observer-refreshed-the-rail-on-every-workspace-notify-001,
  L-claude-713-a-shell-loop-retitles-its-terminal-for-each-command-001,
  L-claude-713-find-a-hot-paths-callers-with-a-temporary-backtrace-001,
  L-claude-713-the-empty-space-menu-selects-no-entry-001,
  AD-claude-713-the-rail-redraws-on-row-changes-and-holds-saves-until-its-restore-001.
- **Brain:** `brain decide` on c1a5a5cc….
- Ticket closed, pipeline archived, committed.
