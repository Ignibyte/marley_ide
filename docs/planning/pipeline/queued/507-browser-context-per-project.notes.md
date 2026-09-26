# A Chromium and a profile per project — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-507-browser-context-per-project.md
- **Pipeline spec:** 507-browser-context-per-project.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25, item 4 (second half) of the list after the browser waves; the
  same day he answered the Orca survey's open question 2: one Chromium per project, worktrees
  sharing their project's profile. The ticket doc written that morning planned CDP contexts and a
  cookie jar; this plan replaces it, and the ticket doc is revised to match.
- **Classification / tier:** feature, prong 3. Marley crates and the e2e fixture only; no Zed
  path. Size M to L (report 03 §3 item 3 says the same).
- **Recall (§18.3):**
  - AD-claude-488-marleys-browser-is-a-transient-unit-streamed-into-a-tab-001: one app-wide hub
    owns the connection so the user and agents share pages; a fixed port and a hub per tab were
    rejected. D4 keeps the single hub and puts the per-project part inside it.
  - AD-claude-494-browser-tabs-reattach-or-reopen-001: a restored tab claims its saved page id,
    or opens its saved URL when the Chromium did not live on. The profile move and the restart in
    the scenario both land on that path.
  - L-claude-494-zed-item-ids-change-at-each-launch-001: the tab table is keyed by workspace and
    item; the workspace names the project, so the table needs no project column.
  - L-claude-493-headless-chromium-lives-on-with-no-pages-001: a project's Chromium keeps
    running with no tab open. The stop is tied to the project leaving, not to its last tab.
  - F-claude-488-a-signal-skipped-the-e2e-cleanup-001: a run that ends early must still stop its
    units; with several units per run, `browser_teardown` stops all of them.
  - L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001: the rail's context menu
    opens on its first entry, so the scenario counts Downs from Move Project Up and takes each
    menu's shot before Enter.
  - Brain: `decisions/marleys-browser-a-transient-chromium-unit-streamed-into-a-gpui-tab-over-cdp`
    (one unit per data directory, one hub); nothing on per-project profiles. Consult with
    `brain_ask` at promotion.
- **Discovery:**
  - `crates/marley_browser/src/service.rs`: the module comment (lines 1 to 8: one unit per data
    directory, it outlives Marley, it ends at logout); `unit_name` (66, the first twelve hex
    digits of the SHA-256 of the profile path); `chromium_args` (81: `--headless`,
    `--remote-debugging-port=0`, `--user-data-dir=<profile>`, `--no-first-run`,
    `--no-default-browser-check`, `--password-store=basic`, `--no-startup-window`);
    `systemd_run_args` (98: `--collect`, `KillMode=mixed`, `TimeoutStopSec=10`); `start` (132);
    `unit_state` (176); `endpoint_in` (212); `remove_endpoint_in` (240). There is no stop.
  - `crates/marley_workbench/src/browser.rs`: `BrowserHub` (398: state, connection, pages,
    attaching, closing, focused, picks, next pick and annotation numbers, generation, run task);
    `global` (448, the `HubHandle` global at 432, created and started on the first ask); `start`
    (614) with the one profile at 627; `fail` (678); `attach` (695); `create_page_task` (809);
    `record` (1387); `iframe_attached` (1691); `new_page` (2174); `open_browser` (2209: connect,
    else start the unit, then 150 polls 100 ms apart); `try_connect` (2245); `follow` (2264);
    `recordings_dir` (2641, `<data>/browser/recordings`, which stays app-wide); `open_tab`
    (2754); `place_tab` (2798); `open_page_in` (2960); `BrowserView` (2988, holding the hub and
    its workspace); the `SerializableItem` impl (4843) whose `deserialize` (4863) takes the hub
    from `BrowserHub::global`; the tab table (`persistence`, 4907: workspace, item, target id,
    URL, title); `init` (5385); `show_for_agent` (5681); `open` (5693); `new_tab` (5744);
    `untabbed` (5758). `BrowserHub::global` is called at 4877, 5686, 5694, 5745 and in
    `browser_tools.rs:54`.
  - `crates/marley_workbench/src/browser_tools.rs`: `answer` (53) takes the global hub and
    restarts a failed one; `run` (63); `page_of` (106: the named tab, else the focused one);
    `tabs` (134); `navigate` (423).
  - `crates/marley_workbench/src/rail.rs`: `new_browser_tab` (607); `render_project_row` (1016)
    and its context menu (1084), which holds only Move Project Up and Down (1094);
    `render_project_menu` (1113), the + with New Browser Tab (1151); `group_names` (1989).
  - Zed: `ProjectGroupKey` (`crates/project/src/project.rs:6590`, the main worktree paths and
    the host), `from_project` (6604), `path_list` (6623); `WorktreeStore::paths`
    (`crates/project/src/worktree_store.rs:1481`), where a linked worktree's main path comes from
    `root_repo_common_dir`, which a local worktree finds when it is made
    (`crates/worktree/src/worktree.rs:515`), so the key is right from the worktree's first
    moment; `MultiWorkspaceEvent` (`crates/workspace/src/multi_workspace.rs:108`,
    `ProjectGroupsChanged` at 114); `project_group_keys` (842); `remove_project_group` (945);
    Zed's sidebar's Remove (`crates/sidebar/src/sidebar.rs:3155`); the recent-projects picker's
    remove (`crates/recent_projects/src/recent_projects.rs:796`).
  - The e2e: `script/e2e/browser-fixture.sh` (`browser_profile` is `$E2E_PROFILE/browser/profile`,
    `browser_unit` hashes it with `sha256sum`, `agent` hands the profile to `agent.mjs`, which reads
    its `DevToolsActivePort`, and `browser_teardown` stops one unit). Every browser scenario from
    488 to 501 opens `$E2E_WORK/repo`. `script/e2e.sh` copies only `config/settings.json`, `db`
    and `threads` into the profile copy (508 to 516), so a run starts with no `browser/` folder.
  - The live box, read only on 2026-09-25: one unit, `marley-browser-eda58f78db16`, running
    Chromium 152.0.7977.82 with the flags above, 287 MB (`MemoryCurrent`), 127 tasks, a 145 MB
    profile; `browser/` and `profile/` are mode 0700; the Chromium's working directory is the
    home directory; its DevTools port listens on 127.0.0.1 only.
- **Decisions:** D1 to D9 in the spec.

### Design
- **`service.rs`.** Pure: `project_key(paths, host) -> String`; `project_dir_in(data, key)`
  (`<data>/browser/projects/<key>`); `profile_of(project_dir)` (`…/profile`);
  `legacy_profile_in(data)` (`<data>/browser/profile`). Adapters, each taking its directory
  (§14's `*_in(dir)` rule): `write_project_file_in(project_dir, paths, host)`, written to a
  temporary name and renamed; `move_legacy_profile_in(data, project_dir)`, answering `Moved`,
  `NoLegacy` or `KeptBecauseAProjectHasOne`, for a caller that has already stopped the legacy
  unit; `async fn stop(unit)`, `systemctl --user stop`, where a unit that is not there is not an
  error.
- **The hub.** `ProjectBrowser { key, name, paths, state, connection, generation, run }` in a map
  on the hub; each `PageState` gains `project: String`. `BrowserHub::global` stays the app's hub
  and starts nothing; `browser_for(key, name, paths, cx)` starts a project's browser when it is
  absent or failed. Every async result carries its (key, generation), so a superseded start of
  one project cannot touch another's pages. `follow` runs per browser, and a dropped connection
  fails that project's browser only. `BrowserView` gains `project` (the key), set from its
  workspace when it is made, and draws that browser's state.
- **The project of a workspace.** `ProjectGroupKey::from_project` gives the paths and host, and
  the key follows (D3). The display name is the paths' last components, as `group_names` in the
  rail makes it. An agent's page gets its tab in a workspace of the page's project: the anchor
  tab's workspace, else the active workspace of a window that holds the project.
- **Start.** `open`, `new_tab`, `deserialize`, `open_page_in` and the tools' new-page path call
  `browser_for` with their project. Before the first `open_browser` of the process, the legacy
  move runs once under a flag: stop `unit_name(legacy)` when it is up, wait until inactive,
  remove its `DevToolsActivePort`, rename the folder, write `project.json`, log where it went.
- **Stop.** `init` observes each new `MultiWorkspace` and subscribes to `ProjectGroupsChanged`
  and `WorkspaceRemoved`. The handler gathers the live keys of every window (groups that still
  hold a workspace) and, for each running browser whose key is gone, schedules a stop in two
  seconds; an event that finds the key live again drops the task. `cx.on_app_quit` sets a flag
  that cancels pending stops and refuses new ones. A stop closes the connection, forgets the
  project's pages (their tabs left with their workspaces) and runs `service::stop`.
- **Clear Browser Data.** The action and the rail's entry ask with `window.prompt`
  (`PromptLevel::Warning`, "Clear the browser data of <name>?", detail: its Browser tabs close
  and every site in the project signs out; Clear and Cancel). Then the hub closes the project's
  tabs, drops the connection, stops the unit, polls `unit_state` until it is down (up to 15 s),
  and removes `profile/` off the main thread; a toast says it is done or why not.
- **The tools.** `TabSummary` gains `project`; pages stay app-wide, so `page_of` is unchanged;
  `answer` restarts the failed browser of the named tab's project, else the focused tab's, else
  the active workspace's; `navigate` with `new_tab` opens in that project.
- **The rail.** After Move Project Down: "Clear Browser Data…" (the action, in the group's
  workspace) and "Remove Project" (`remove_project_group(&key, window, cx)` with
  `detach_and_log_err`, as sidebar.rs does).
- **The fixture.** `browser_profile [root]` takes `realpath "${1:-$E2E_WORK/repo}"`, looks for
  the `project.json` under `$E2E_PROFILE/browser/projects/` whose paths hold it and prints that
  folder's `profile`; before the first start it prints the path the key gives
  (`printf '%s\n' "$root" | sha256sum | cut -c1-16`). `browser_unit [root]` hashes that path as
  today. `browser_teardown` stops the unit of every `browser/projects/*/profile` and of the
  legacy path. `agent` takes an optional `--profile <dir>`, which setup uses to sign in on the
  legacy profile.
- **File manifest.** Marley crates: `crates/marley_browser/src/service.rs`;
  `crates/marley_workbench/src/browser.rs`, `browser_tools.rs`, `rail.rs`,
  `marley_workbench.rs`. Scripts (Marley-owned): `script/e2e/browser-fixture.sh`,
  `script/e2e/507-browser-context-per-project.sh`. At Complete: `CHANGELOG.md`,
  `docs/marley/three-prong-plan.md` (D16: one unit per project; the slice table),
  `docs/marley_architecture/marley_browser.md`.
- **Ledger rows.** None in `docs/marley/zed-touchpoints.md`: Zed's `ProjectGroupKey`,
  `MultiWorkspace::remove_project_group` and `MultiWorkspaceEvent` are used through their public
  API and no Zed file changes. At Complete, an AD that the browser is a unit per project, which
  refines AD-claude-488's "one per data directory".
- **If it runs long.** Split after the stop: Clear Browser Data and the rail's two entries become
  a follow-up ticket, and REQ-007 is then shown through the recent-projects picker's remove.

### E2E plan
Setup as the spec's UI proof. The scratch Chromium for the legacy profile is the offline
wrapper, started by setup on `$E2E_PROFILE/browser/profile`; the stand-in agent signs it in as
`legacy`; setup stops it before Marley starts.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | alpha opens; `marley: open browser`; Ctrl+L, `whoami.html`, Enter | `507-01-migrated`; the log's `systemctl --user list-units 'marley-browser-*'` and `find browser -maxdepth 3` |
| REQ-008 | the same step: the legacy profile's `legacy` login shows in alpha's tab | `507-01-migrated`; the listing shows no `browser/profile` and one project folder |
| REQ-002 | beta added through the rail's add-project popover, Open Local Folders and Zed's prompt; beta's + New Browser Tab; `whoami.html` (signed out), then `signin.html?as=beta`; alpha's tab clicked and reloaded | `507-02-beta-signed-in`, `507-03-alpha-kept` |
| REQ-003 | `alpha-wt` added the same way (it joins alpha's group); with its workspace active, `marley: new browser tab`, `whoami.html` | `507-04-worktree-shares`; the unit list still shows two |
| REQ-009 | `mcp_agent tabs`; `mcp_agent --tab <beta's id> look` | the log: each tab with its project; the look names beta's URL and title |
| REQ-004 | `quit_marley`, then the unit list | the log: both units active |
| REQ-005 | both units stopped with `systemctl --user stop`; `launch_marley`; alpha's tab, then beta's | `507-05-alpha-after-restart`, `507-06-beta-after-restart` |
| REQ-006 | right-click beta's row, Clear Browser Data…, Enter on Clear; beta's + New Browser Tab, `whoami.html` | `507-07-cleared`; the log lists beta's folder with `project.json` and a new `profile/` |
| REQ-007 | right-click beta's row, Remove Project; settle five seconds | `507-08-removed`; the log: beta's unit inactive, alpha's active |

Not reachable by a scenario: the move of Chad's own profile on his first launch after the
update. Test records which project received it, from `Marley.log` and the folder listing.

### Risks
- Memory: about 290 MB per open project's Chromium, so five open projects cost about 1.5 GB.
  This slice sets no cap (Out).
- A project's key changes when a folder is added to it, which gives it a clean profile; the old
  one stays on disk. Recorded in the ticket's summary of what a project is (D3).
- The quit race: windows closing during a quit look like projects leaving. The two-second grace
  and the `on_app_quit` flag guard it, and REQ-004 checks it. If a quit took longer than the
  grace, a unit would stop and its tabs would reopen their URLs still signed in: page state
  lost, logins kept.
- Restore order: a tab deserialized before its project's worktrees are added would compute an
  empty key. Zed opens the worktrees before it deserializes items as far as the calls read;
  confirm in `Workspace`'s load path at P2, and make a tab with an empty key wait for the
  project's worktrees before it picks a browser.
- Two projects in one window is new ground for a scenario. Zed's own path prompt is the route.
  If it will not take typed input under sway, a second window through `workspace: new window`
  with the same prompt shows REQ-002 to REQ-005, and closing that window shows REQ-007.
- The earlier browser scenarios depend on the fixture's profile lookup; the default root keeps
  them working, and Test reruns #494's to show it.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
