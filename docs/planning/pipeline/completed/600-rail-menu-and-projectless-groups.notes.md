# The rail's right-click menu, and projectless groups — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-600-rail-menu-and-projectless-groups.md
- **Pipeline spec:** 600-rail-menu-and-projectless-groups.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30 (quoted in the ticket). His choice: a projectless group (a named
  group with no folder, like Warp's New Group, whose + makes terminals, agent CLIs and Browser
  tabs in the home folder), over a heading that files projects, or both.
- **Classification / tier:** feature, large; split in two: this ticket (the session) and #601
  (restore). #602 (drag) follows both.
- **Recall (§18.3):** AD-claude-453 (header menus reorder through `MultiWorkspace`);
  AD-claude-452 (the rail starts Zed's own rename and close; the group's rename is Marley's own,
  since a group is no Zed item); #507's per-project Chromium and its stop rule; #550's question
  before a working agent's terminal closes.
- **Discovery (Explore, 2026-09-30):**
  - Keys: `ProjectGroupKey { paths, host }`, `crates/project/src/project.rs:6590-6611`;
    `display_name` says "Empty Workspace" with no paths (6626-6651).
  - `MultiWorkspace` (`crates/workspace/src/multi_workspace.rs`): `project_groups` (308) holds
    only keyed groups; `ensure_project_group_state` skips empty keys (664-667), as do
    `rekey_project_group` (701-703), `handle_project_group_key_change` (625-628) and
    `restore_project_groups` (826-828). `add` (1295-1307) and `activate` (1310-1366) hold and pin.
    `open_project` (1915-1993) closes an active workspace with no visible worktrees (1923-1935,
    1962-1970).
  - The rail's `build_snapshot` (`crates/marley_workbench/src/rail.rs:5125-5232`) walks
    `project_groups(cx)` and drops groups with no open workspace (5136-5140); names from
    `group_names` (`marley_workbench.rs:728-742`).
  - Menus: the header's `right_click_menu` in `render_project_row` (3235-3347) and
    `project_context_menu` (3351-3395); terminal rows (3761-3788); the rows container
    `v_flex().id("marley-rail-rows")` (6286-6308) has no handlers and is the only element over the
    blank space; the header's spacer (3124-3158). A `right_click_menu` around the whole container
    would also fire over thread, Browser and port rows, which stop no propagation: put the
    handler on a filler element after the last row, and on the header spacer.
  - The + menu: `render_project_menu` (3397-3463); `new_terminal` (2111-2132) through
    `terminal_view::default_working_directory` (`terminal_view.rs:2571-2594`, home fallback);
    `new_browser_tab` → `browser::new_tab` (`browser.rs:8189-8215`) → `BrowserProject::of`;
    `new_agent` → `agents::start_cli` (`agents.rs:278-318`, `launch_mode` reads the key's paths).
    Threads with empty `worktree_paths` are archived by default
    (`thread_metadata_store.rs:1325-1331`), one reason groups offer no Agent Panel thread.
  - Browser keying: `BrowserProject::of_group` (`browser.rs:498-542`) →
    `marley_browser::service::project_key` (`service.rs:52-72`); `live_projects`
    (`browser.rs:7545-7574`) keeps a key's Chromium while any held workspace has it.
  - Persistence patterns for #601: `MarleyTerminalIdsDb` (`terminal_ids.rs:100-171`),
    `MarleyBrowserTabsDb` (`browser.rs:6986-7070`), the rail's blob in `sidebar_state`
    (`rail.rs:4159-4202`, 6161-6199).
- **Decisions:** see the spec's D1 to D6.

### Design (to settle at promotion)
- `groups.rs`: `Group { id: Uuid, name, workspace: WeakEntity<Workspace> }`, a per-window list in
  the rail (or a global keyed by window), and a `MarleyGroupsDb` table
  `(workspace_id → group_id, name)` written now so #601 can read it.
- The Zed hook: a global predicate the workspace crate asks in `open_project` before it closes an
  empty active workspace ("is this workspace kept by Marley?"), set by `marley_workbench`.
- The rail: `build_snapshot` adds a `GroupEntry` per Marley group after the project groups;
  `marley_rail` learns a group section kind (no ports, threads or worktrees); rendering reuses
  `render_project_row` with the group icon and the group's menu.
- Chromium: `BrowserProject` gains a group variant whose key is `service::project_key` over a
  synthetic, never-a-path entry (`group:<uuid>`), so `project.json` and the profile folder follow
  the existing layout; `live_projects` counts it while the group's workspace is held.
- Name prompt: a small modal with a single-line editor (Enter makes, Escape cancels), as the
  worktree prompt (#510) is built.

### Visual check plan
- One shot per criterion as listed in the spec's UI proof; `pwd` output read from the shot; the
  unit count read with `systemctl --user list-units 'marley-browser-*'` into `units.txt` (the
  runner's own Chromium units are scoped to its data dir's hash).

### Risks
- `open_project`'s close of an empty active workspace also fires for the fresh window's own empty
  workspace; the hook must keep only Marley groups, or first launches keep a stray empty
  workspace.
- `assert_project_group_key_integrity` (test-only) may fail for a pinned folderless workspace; the
  tests are not run per ticket (§7), but note it.
- The title bar reads "Open Recent Project" while a group is shown: out of scope, but visible in
  the shots; say so in the Test notes.

### Promotion (2026-09-30, `/pipeline:plan 600`)
- **Run mode:** back to back, as Chad chose; no wait at the plan.
- **Pre-flight:** no active pipeline; README marker present; cargo idle; `/mnt/fast` at 91%.
- **Brain (`rusty-cli brain ask`, consultation 34cd483d9d4f442d8525d40c62862b8d):** nothing on this
  seam.
- **Seams re-verified (2026-09-30):**
  - `build_snapshot` (`rail.rs:5127`) walks `multi_workspace.project_groups(cx)`, keeps those with
    a workspace, names them with `group_names`, and for each builds terminals and Browser tabs from
    every member workspace, threads from `group_threads`, ports from `port_snapshots(&group.key)`,
    worktrees from `group_worktrees`, then pushes a `ProjectSnapshot` and a `GroupEntry { key,
    workspace, source, git }`. `note_focus` finds the shown project as the group whose
    `workspaces` holds the displayed workspace.
  - `workspace::ProjectGroup { key, workspaces, expanded }` has public fields: Marley can build one
    for each of its groups (an empty key, the group's workspace) and append it to the list the rail
    walks, so rows, focus and the attention order work unchanged.
  - Fold goes through `toggle_expanded(key)` on `MultiWorkspace::group_state_by_key_mut`; a Marley
    group keeps its own `expanded`.
  - `MultiWorkspace::open_project` (`multi_workspace.rs:1915`) takes the displayed workspace as
    `empty_workspace` when it has no visible worktree, asks it to close (`prepare_to_close`,
    `ReplaceWindow`) and detaches it: the hook goes in that condition.
  - `MultiWorkspace::remove(workspaces, RemovalIntent, window, cx)` runs `prepare_to_close` (and so
    #550's guard) for each, detaches them and picks a replacement for the displayed one.
  - `Workspace::new_local(Vec::new(), app_state, Some(window), None, None, OpenMode::Add, cx)` makes
    a folderless workspace in the window.
  - `BrowserProject::of(&Project, cx)` keys by the group key; `live_projects` adds
    `BrowserProject::of_workspace` for every held workspace, so a group key returned by `of` keeps
    the group's Chromium alive with no other change. `of` is called in `browser.rs` only (five
    calls).
  - The runner's `click x y right` right-clicks (`script/e2e.sh` `click`'s third argument).
  - `uuid` is a workspace dependency (`Cargo.toml:933`), not yet `marley_workbench`'s.

### Design
- **`crates/marley_workbench/src/groups.rs`** (new):
  - `Group { id: Uuid, name: String, workspace: WeakEntity<Workspace>, expanded: bool, home: bool }`
    in a global `Groups`, for every window; `groups_of(multi_workspace, cx)` gives those whose
    workspace this window holds, in the order made.
  - `make(multi_workspace, name, home, then, window, cx)`: `Workspace::new_local` with no paths and
    `OpenMode::Add` into this window, then records the group and runs `then` with its workspace
    (the Home path uses it to open the item). Names: an empty name makes "Group", "Group 2", and so
    on; the Home group is named "Home".
  - `rename`, `set_expanded`, `forget` (on Remove), `group_of_project(project_id, cx)` for the
    browser, and after every change the kept set for the Zed hook.
  - `GroupNamePrompt`: a modal (`ModalView`, key context `MarleyGroupName`) with a one-line editor,
    for New Group… and Rename Group…; Enter is `marley::MakeGroup` (a new action, bound in
    `keymap.json` under `MarleyGroupName > Editor`), Escape `menu::Cancel`.
- **`crates/marley_workbench/src/rail.rs`:**
  - `build_snapshot`: after Zed's groups, a `ProjectGroup` per Marley group of this window (empty
    key, its workspace, its `expanded`); its name from the record; no threads, ports, worktrees or
    git read for it; `GroupEntry` gains `group: Option<Uuid>`.
  - The header: a group draws `IconName::ListTree` before its name; its right-click menu is Rename
    Group… and Remove Group; fold toggles the record.
  - The `+` for a group: New Terminal, New Browser Tab and the Agent CLIs (no New Agent Thread;
    the worktree and Launch entries already hide with no folder).
  - The empty space: a filler element after the last row (`flex_1`, the rest of the column) and
    the header's spacer carry a `right_click_menu` with New Group…, New Terminal, New Browser Tab
    and the Agent CLIs; the three item entries find or make the window's Home group, show it, and
    open the item there.
  - Remove Group: `MultiWorkspace::remove([workspace], RemovalIntent::CloseProject)`, then
    `groups::forget` when it went through.
- **`crates/marley_workbench/src/browser.rs`:** `BrowserProject::of` takes `&Entity<Project>`; a
  group's project gets `marley_browser::service::group_key(id)`, the group's name, no paths.
- **`crates/marley_browser/src/service.rs`:** `group_key(id)`, the first sixteen hex digits of the
  SHA-256 of `group\n<id>\n` (a path list never hashes to it: every path ends in a newline after a
  path, never after the word `group`).
- **`crates/workspace/src/multi_workspace.rs`** (Zed crate): `MarleyKeptWorkspaces(pub
  HashSet<EntityId>)`, a `Global`, and in `open_project` the empty workspace is taken only when
  the global does not hold it. One `// Marley:` hunk; its row in `zed-touchpoints.md` first.
- **`crates/marley_workbench/Cargo.toml`:** `uuid.workspace = true`.

### File manifest
- `crates/marley_workbench/src/groups.rs` (Marley, new); `rail.rs`, `browser.rs`,
  `marley_workbench.rs` (the action, the module, `groups::init`), `keymap.json`, `Cargo.toml`
  (Marley); `crates/marley_browser/src/service.rs` (Marley); `crates/workspace/src/multi_workspace.rs`
  (Zed); `docs/marley/zed-touchpoints.md`; `script/e2e/600-rail-menu-and-projectless-groups.sh`.

### Risks (and what Test watches)
- A restart brings back an active group's workspace as a plain folderless workspace the rail does
  not list (#601 restores groups): say so in the Test notes; #601 follows at once.
- Opening a file from outside every project while a group is shown could give the group's
  workspace a folder; out of scope, noted.
- Removing the shown group: `remove` picks another held workspace with the same (empty) key, which
  may be another group; acceptable.

### Checklist (no TaskCreate in this harness)
- [x] Pick · [x] pre-flight · [x] recall · [x] promote · [x] prior art · [x] spec · [x] design ·
  [x] presented (autonomous run)

## Phase 2 — Code
- **Built:**
  - `crates/marley_workbench/src/groups.rs` (new): `Group` and the global `Groups`; `groups_of`
    (a window's groups), `group_of_project` (for the browser), `make` (a folderless workspace
    through `Workspace::new_local` with `OpenMode::Add`, recorded once it exists, then a follow-up
    run through the window alone), `rename`, `toggle_expanded`, `forget`, `keep` (the
    `MarleyKeptWorkspaces` set after every change), and `GroupNamePrompt` for New Group… and
    Rename Group… (`MakeGroup` on Enter, `menu::Cancel` on Escape).
  - `rail.rs`: `rail_groups` appends a `ProjectGroup` per group (the empty key, its workspace, its
    `expanded`) after the window's projects; `listed_workspace` takes a group's own workspace;
    `GroupEntry.group`; `group_threads` returns nothing for a key with no folder; the header's
    icon (`header_icon`: `IconName::ListTree` for a group), its fold (the record's), and its menu
    (`HeaderMenu`: Rename Group…, Remove Group for a group, the project menu otherwise); the `+`
    without New Agent Thread for a group; `new_group`, `rename_group`, `remove_group`
    (`MultiWorkspace::remove` with `RemovalIntent::CloseProject`, then `forget`), `in_home`, and
    the empty space's menu (`deploy_empty_menu`, drawn anchored and deferred at the pointer) on a
    filler under the rows (`render_rows`) and on the header's spacer. The rail observes `Groups`.
  - `browser.rs`: `BrowserProject::of(&Entity<Project>)`, a group's project keyed by
    `service::group_key(id)`; its five callers.
  - `marley_browser/src/service.rs`: `group_key`.
  - `marley_workbench.rs`: `pub mod groups`, the `MakeGroup` action; `keymap.json`: the
    `MarleyGroupName > Editor` block; `Cargo.toml`: `uuid`.
  - Zed: `crates/workspace/src/multi_workspace.rs` (`MarleyKeptWorkspaces`, `marley_kept`, the
    `open_project` clause) and `crates/workspace/src/workspace.rs` (the re-export); their rows in
    `zed-touchpoints.md` (a new row, and the `workspace.rs` row extended).
- **Deviations:** the empty space's menu is deployed by hand (a right mouse-down on a filler, the
  menu kept on the rail) rather than through `ui::right_click_menu`, whose wrapper lays out with a
  default style and would collapse a `flex_1` filler.
- **Review of the diff, and what changed because of it:**
  - *Re-entrancy (a real bug, fixed before the gate):* `make`'s follow-up ran inside
    `WindowHandle<MultiWorkspace>::update`, which leases the `MultiWorkspace`; the Home path's
    action (`rail.new_terminal` → `activate_workspace`) updates that same entity and would panic.
    It now runs through `AnyWindowHandle::update`, which takes only the window.
  - Clippy's line limits split `build_snapshot`, `render` and `render_project_row`
    (`rail_groups`, `listed_workspace`, `render_rows`, `header_icon`, `HeaderMenu`); the name
    search is bounded (`1..=taken.len() + 1`); `make` takes `&Window`.
- **Gate:** `just gate-diff` (scope `workspace`, `marley_browser`, `marley_workbench`): every gate
  PASS, receipt written (`gate-600.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/600-rail-menu-and-projectless-groups.sh` (sway) on the debug build: a
  scratch folder `repo`; `repo-b` handed over by a second launch on the same profile (#513's
  path), then removed so Add Project's list offers it. Five runs; the last, all checks passing,
  is the one recorded.
- **`menu.png`** (REQ-001): a right-click under the rows opens New Group… (selected), a
  separator, New Terminal, New Browser Tab, then "Agent CLIs" with Claude Code, Codex, Gemini CLI
  and OpenCode. PASS.
- **`group.png`** (REQ-002): Scratch listed after `repo`, with the list-tree icon, its chevron and
  its `+`. PASS.
- **`group-plus.png`** (REQ-009): Scratch's `+` holds New Terminal, New Browser Tab and the four
  CLIs: no New Agent Thread, New Agent in Worktree or Launch. PASS.
- **`home.png`** (REQ-003): Scratch's terminal (`cpeppers — bash`, folder `~`) prints
  `/home/cpeppers` for `pwd`; its row reads `pwd · done · 0 s`. PASS.
- **`home-group.png`** (REQ-004): the empty space's New Terminal made a Home group (the icon, the
  name Home) with a terminal in `~`. PASS.
- **`browser.png`, `units.txt`** (REQ-005): Scratch's New Browser Tab adds an `about:blank` row
  under Scratch, and exactly one new unit started (`marley-browser-5129fab59150.service`); the
  scenario's `expect` passed. PASS.
- **`repo-b.png`, `repo-b-menu.png`, `repo-b-removed.png`:** the handed-over `repo-b` lists at the
  top; its menu opens on Move Project Down (Move Project Up disabled); Remove Project takes it
  away.
- **`scratch-shown.png`, `add-project.png`, `kept.png`** (REQ-006): with Scratch shown (title
  "Open Recent Project", its Browser tab in front), Add Project filtered to `repo-b` opens it,
  and the rail then lists repo-b, repo, Scratch (its terminal and Browser tab) and Home: the
  displayed folderless workspace was not replaced. PASS.
- **`renamed.png`** (REQ-007): Rename Group… on Scratch's header makes it Tools. PASS.
- **`removed.png`, `removed-unit.txt`** (REQ-008): Remove Group takes Tools and its rows away
  (repo-b, repo and Home remain), and its browser unit reads `inactive`; the `expect` passed.
  PASS.
- **Found and fixed in Test:**
  1. *The menu lost the keyboard.* Run 1's keys went to the terminal and the menu stayed open:
     the rail's root, which tracks focus, focused itself on the mouse-down after the filler's
     handler opened the menu. `empty_space_menu` now stops the event and prevents its default,
     as `ui::right_click_menu` does. (A test bug came with it: the menu opens with its first
     entry selected, so the scenario's `menu_entry N` presses Down N−1 times.)
  2. *A panic opening a group's Browser tab.* Run 2 died with "cannot read workspace::Workspace
     while it is already being updated": `browser::new_tab` runs inside the group workspace's
     update and asks `BrowserProject::of`, whose `groups::group_of_project` read each group's
     workspace for its project. The group now records its project's entity id when it is made,
     and the lookup reads no workspace.
  3. *Scenario points:* Remove Project is the menu's third selectable entry at the top; Scratch's
     header moves to y 280 while repo-b lists above it.
- **Known and out of scope:** a shown group's title bar reads "Open Recent Project" (Zed's name
  for a folderless workspace); groups do not survive a restart until #601.
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and
  did not reload it"; the sway run stopped with its Marley; the runner stopped the run's browser
  units.
- **Gate after the Test fixes:** `just gate-diff` every gate PASS (`gate-600b.log`); run again at
  Complete for the commit's receipt.

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Unreleased, Added: #600); `docs/marley_architecture/marley_workbench.md`
  (a section for `src/groups.rs` and the rail's part); the `workspace` rows in
  `docs/marley/zed-touchpoints.md` checked against the hunks shipped (`MarleyKeptWorkspaces`, the
  `open_project` clause, the re-export); `docs/marley/workbench-shell.md` (the slice line names
  #600); `docs/marley/guide.md` (a Groups with no folder section); `docs/marley/walkthrough.md`
  (stop 2.10); the guide page (`crates/marley_workbench/guide/index.html`, a new article in the
  rail's section, per AD-claude-599).
- **Ledger:** `F-claude-600-a-registry-lookup-read-the-workspace-its-caller-was-updating-001`,
  `F-claude-600-a-hand-deployed-menu-lost-the-keyboard-to-the-rails-own-focus-001`,
  `F-claude-600-a-follow-up-inside-the-windows-root-update-would-update-it-again-001` (failures);
  `PR-claude-600-a-lookup-others-call-inside-updates-reads-no-entity-001` (prevention rules);
  `L-claude-600-a-hand-deployed-menu-stops-and-prevents-its-mouse-down-001` (lessons);
  `AD-claude-600-projectless-groups-are-marley-records-over-folderless-workspaces-001` (decisions).
- **Brain:** consultation 34cd483d9d4f442d8525d40c62862b8d closed with `brain decide`.
- **Ticket:** closed; archived with this pair; committed with the change.
