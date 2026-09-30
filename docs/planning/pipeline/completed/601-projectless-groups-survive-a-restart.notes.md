# Projectless groups survive a restart — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-601-projectless-groups-survive-a-restart.md
- **Pipeline spec:** 601-projectless-groups-survive-a-restart.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** the restore half of Chad's projectless groups (#600).
- **Classification / tier:** feature; `marley_workbench`, maybe one Zed hook.
- **Recall (§18.3):** #575 (terminal ids across restore), #576 (Browser tabs keyed by workspace
  and item), #577 (a restored terminal in its folder: Zed's terminal panel once deleted other
  terminals' saved rows), #494 (Browser tabs restored), #486 (the size of a launch's first
  terminals).
- **Discovery (Explore, 2026-09-30):** see the spec's Prior art; the restore order in
  `crates/zed/src/main.rs:1447+` (`read_serialized_multi_workspaces` then
  `restore_multiworkspace` then `apply_restored_multiworkspace_state`). An upstream commit,
  `1b467a0b1c` ("Restore saved local session members by workspace identity"), is not in this
  tree; read it at promotion, since it may restore more than the active workspace and change
  what Marley has to do.

### Design (to settle at promotion)
- The record: #600's `MarleyGroupsDb` rows plus an order column, or a list in the rail's blob in
  `sidebar_state` (per window, written when the `MultiWorkspace` serializes). The blob is per
  window, which is what the restore needs; the table survives the blob.
- The reopen: after `restore_multiworkspace`, for each recorded group whose workspace is not the
  active one, `open_workspace_by_id` (or `Workspace::new_local` with that id) with
  `OpenMode::Add` into the window, then mark it as a group again (#600's hook).

### Visual check plan
- The scenario quits and restarts Marley on its scratch profile; the runner supports a restart
  (see `script/e2e/494-browser-restore.sh` and `577-*` for the pattern). Shots as in the spec.

### Risks
- The active group is restored by Zed and must not be reopened a second time.
- A window restored with no folder at all (only groups) must not trip Zed's "empty window gets
  the Launchpad" path.

### Promotion (2026-09-30, `/pipeline:plan 601`)
- **Run mode:** back to back, as Chad chose; no wait at the plan.
- **Pre-flight:** no active pipeline; README marker present; cargo idle; `/mnt/fast` at 91%.
- **Brain (`rusty-cli brain ask`, consultation 9ea3c0e803594700b3cb6b0016bd0c39):** nothing on this
  seam.
- **Recall:** #600's records and F-claude-600 (lookups must read no entity; a registry mutation
  through `default_global` notifies observers, so a read path must use `try_global`).
- **Seams verified (2026-09-30):**
  - `workspace::open_workspace_by_id(id, app_state, Some(window), cx)` builds the workspace with
    its saved id in the window, `multi_workspace.add`s it (held, not shown), restores its items
    (`open_items`) and serializes it, then answers the window, not the workspace; a missing row
    fails with "Workspace … not found".
  - The rail's saved blob: `Rail::serialized_state` runs inside the `MultiWorkspace`'s update and
    reads only the rail's fields; `restore_serialized_state` runs inside it too and already defers
    its close through `window.defer` (`rail.rs` 6531-6566); `write_rail_state` and
    `read_rail_state` keep Zed's sidebar's fields.
  - `Workspace::database_id() -> Option<WorkspaceId>`; `WorkspaceId(i64)` is `Copy`, `Eq`,
    `Hash` and serde.
  - The Browser tab's `Item::deserialize(project, workspace, workspace_id, item_id, …)`
    (`browser.rs` around 6945) calls `BrowserProject::of(&project)` before the workspace is
    adopted as a group; the workspace id is at hand there.
  - `KeyValueStore::global(cx).scoped(..).read(key)` is synchronous (as `terminal_size::init`
    uses it at startup); `write` and `delete` are async.
  - `marley_workbench::init` runs once, at startup, before Zed restores windows.

### Design
- **`groups.rs`:**
  - `SavedGroup { database_id: WorkspaceId, id: Uuid, name, home, expanded }` (serde).
    `Groups` keeps `live: Vec<Group>` (as #600, plus `database_id`) and `pending:
    Vec<SavedGroup>` (records read at startup whose workspace is not open yet).
  - `init(cx)`: reads `marley-groups`/`groups` (a JSON array) into `pending`.
  - `save(cx)`: writes every live group that has a database id, and every pending record, to the
    same key on the background executor; called after `make`, `rename`, `toggle_expanded`,
    `forget` and `forget_pending`.
  - `adopt(multi_workspace, cx) -> bool`: moves each pending record whose database id a workspace
    of this window holds into `live`, with that workspace and its project's id; reads through
    `try_global` and writes only when there is something to move (a `default_global` read would
    notify the rail's observer, which calls `adopt`: a loop).
  - `group_of_workspace_id(id, cx)`: the id and name of the live or pending group with that
    database id, for a Browser tab while it deserializes.
  - `forget_pending(id, cx)`: drops a pending record whose workspace could not be reopened.
- **`browser.rs`:** the Browser tab's `deserialize` asks `groups::group_of_workspace_id(workspace_id)`
  first and keys by the group; `BrowserProject::of_group_id(id, name)` builds that project.
- **`rail.rs`:**
  - `refresh` calls `groups::adopt` first, and `keep` runs when it moved anything.
  - `rail_groups` orders a window's groups by `saved_groups` (the rail's list of database ids),
    any not in it after, as made.
  - The rail keeps `group_ids: Vec<WorkspaceId>` from its last snapshot; `serialized_state` writes
    them as `marley_groups` beside the rail's fields; the `Groups` observer also asks the
    `MultiWorkspace` to serialize, so the blob follows a new, renamed or removed group.
  - `restore_serialized_state` reads `marley_groups` into `saved_groups` and defers the reopen:
    each id the window does not hold is opened by `open_workspace_by_id` into this window; a
    failure drops the pending record and logs it (D3). The workspace the window shows (restored by
    Zed) is adopted by `refresh` like the rest.
- **No Zed change.**

### File manifest
- `crates/marley_workbench/src/groups.rs`, `rail.rs`, `browser.rs`, `marley_workbench.rs`
  (`groups::init`) (Marley); `script/e2e/601-projectless-groups-survive-a-restart.sh`.

### Visual check plan
- As the spec's UI proof; the restart follows `script/e2e/494-browser-restore.sh` (quit through
  the palette, start again on the same profile). The Browser tab's page is a local HTML file
  opened with `browser_open_url`'s path or typed as a `file://` address.

### Risks
- A record whose window never comes back stays in the store; harmless, and dropped when its
  reopen fails.
- Zed may show its launchpad in a window restored with only groups (no folder); note it if the
  scenario shows it.

### Checklist (no TaskCreate in this harness)
- [x] Pick · [x] pre-flight · [x] recall · [x] promote · [x] prior art · [x] spec · [x] design ·
  [x] presented (autonomous run)

## Phase 2 — Code
- **Built:**
  - `groups.rs`: `SavedGroup` and `Groups { live, pending }`; `init` reads `marley-groups`/`groups`
    at startup (called from `marley_workbench::init` after `terminal_size::init`); `save` writes
    every live group with a workspace id and every pending record after each change; `adopt`
    moves a pending record into `live` once a workspace of the window holds its id (reading
    through `try_global`, writing only when there is one to move); `reopen` opens each saved id the
    window does not hold through `workspace::open_workspace_by_id`, and a failure drops the record
    with a warning (`forget_pending`); `group_of_workspace_id` for tabs that deserialize before
    their group is adopted; `Group.database_id`.
  - `browser.rs`: the Browser tab's `deserialize` asks `group_of_workspace_id(workspace_id)` first;
    `BrowserProject::of_projectless(id, name)` builds a group's project for both paths.
  - `rail.rs`: `refresh` adopts first; `rail_groups` orders a window's groups by `saved_groups`,
    new ones after as made; `GroupEntry.database_id`; the `Groups` observer is `groups_changed`
    (refresh, then `MultiWorkspace::serialize`); `serialized_state` adds `marley_groups` (the
    groups' workspace ids) through `write_rail_groups`; `restore_serialized_state` reads them
    (`read_rail_groups`) into `saved_groups` and defers `groups::reopen` past the
    `MultiWorkspace`'s update. `RailState` is untouched, so the rail's tests stand as they are.
- **Deviations:** the window's list is a separate `marley_groups` field beside `RailState`'s rather
  than in it (`RailState` is `Copy` and built field by field in `rail_tests.rs`).
- **Review of the diff:** REQ-001 through `reopen`, `adopt` and the sort; REQ-002 through Zed's own
  item restore in `open_workspace_by_id` and the tab's lookup by workspace id; REQ-003 (a shown
  group): Zed restores the shown workspace, and `adopt` makes it the group; REQ-004 through
  `forget_pending`. `adopt` reads the `MultiWorkspace`, which `refresh` already reads, so it never
  runs inside that entity's update; `reopen` runs from `window.defer`, outside it.
- **Gate:** `just gate-diff`: every gate PASS, receipt written (`gate-601.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/601-projectless-groups-survive-a-restart.sh` (sway): a scratch project
  `repo`; the empty space's New Group… makes Web (a Browser tab on a local `page.html`) and
  Scratch (a terminal moved to `/tmp`, and one at home); Marley quits through the palette and
  starts again on the same profile with no path, as the app menu starts it.
- **First run, and the fix to the scenario:** the runner's `launch_marley` passes the scenario's
  folder, and Zed answers a launch with a path as an open request instead of restoring the last
  session (`crates/zed/src/main.rs`, the `restore_task` match), so nothing but `repo` came back.
  The scenario clears `OPEN` before the relaunch, which is how Chad starts Marley. No app change.
- **`before.png`:** repo; Web with its "Group page" row; Scratch with `tmp — bash` (`/tmp`) and
  `cpeppers — bash` (`~`), the latter shown.
- **`after.png`** (REQ-001, REQ-002, REQ-003): Web and Scratch back, in that order and with those
  names; Web's row reads "Group page" (its tab on its page); Scratch's rows read `tmp — bash` in
  `/tmp` and `cpeppers — bash` in `~`; Scratch, shown at the quit, is shown again (its two
  terminals in the tab bar). PASS.
- **`units.txt`:** Web's unit before the quit (`marley-browser-943b3588e222.service`) and no new
  unit after the restart: Web's tab went back to Web's own Chromium, found by its workspace id
  before the group was adopted. The scenario's two `expect`s passed. PASS.
- **REQ-004** (a group whose workspace is gone): by review of `reopen`/`forget_pending`.
- **Pre-existing — not in scope:** `repo` is not listed after the restart. Zed reopens only a
  window's shown workspace and brings its other project groups back as keys with no workspace,
  which Zed's Threads Sidebar lists and the rail has always left out (`build_snapshot`'s filter
  of groups with no open workspace, from before #600): before this ticket a window's second
  project was hidden the same way. A follow-up ticket should list those projects (dimmed, opened
  on a click, as Zed's sidebar does).
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and
  did not reload it"; the sway run stopped with its Marley.

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Unreleased, Added: #601); `docs/marley_architecture/marley_workbench.md`
  (the groups section's restore paragraph); `docs/marley/workbench-shell.md` (the slice line);
  `docs/marley/guide.md` and the guide page (groups come back); `docs/marley/walkthrough.md`
  (stop 2.10's restart check). No Zed path touched.
- **Ledger:** `L-claude-601-a-launch-with-a-path-restores-no-session-001` (lessons),
  `AD-claude-601-groups-are-restored-from-two-stores-001` (decisions). No app bug found.
- **Brain:** consultation 9ea3c0e803594700b3cb6b0016bd0c39 closed with `brain decide`.
- **Follow-up to raise with Chad:** list the window's other projects after a restart (see Phase
  3's pre-existing note).
- **Ticket:** closed; archived with this pair; committed with the change.
