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
