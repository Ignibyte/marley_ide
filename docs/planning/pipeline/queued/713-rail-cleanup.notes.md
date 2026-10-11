# Rail cleanup - one sidebar's subscriptions, refreshes on row changes, a silent swap, saves after the restore — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-713-rail-cleanup.md
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
