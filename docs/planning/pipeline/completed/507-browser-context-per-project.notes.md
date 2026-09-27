# A Chromium and a profile per project — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-507-browser-context-per-project.md
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

### Promotion (2026-09-27, at f21b8aea6b)
- **Split.** Clear Browser Data (D6, REQ-006, the action and the rail's entry) is #581, minted
  now and queued first. #507 keeps D1 to D5, D7 to D9, REQ-001 to REQ-005 and REQ-007 to REQ-009.
- **Landed on the hub since the draft:** #503 (terminal URLs open in a Browser tab:
  `open_url_tab`), #504 (the rail's Browser rows read the hub through `try_global`, and
  `PageStatusChanged`), #505 (`check_pick`, the comparison card), #506 (`watch_actions`,
  `action_reported`, `record(project)`), #518 (the fuller pick), #561 (`browser_open_url`), #574
  (the caller's `Scope`, `focus_history`, `placements`, `focused_among`, `tab_workspaces`), #575
  and #576 (the tab table keyed by workspace and item), #579 (the link menus). None of them adds a
  second browser or a second profile; all of them go through the one `connection`, `state` and
  `generation` this ticket splits.
- **Seams re-verified** (an Explore sweep, then the code read here): `BrowserHub`
  (`browser.rs:450`): `state`, `connection`, `generation` and `run` belong to the browser;
  `pages`, `attaching`, `closing` and `placements` are reset at each start; `focus_history`,
  `picks` and the pick and annotation numbers are the app's. `global` (583) creates the hub and
  starts it; `start` (779) holds the one profile (793); 27 functions carry the generation, 20 of
  them guarding with `generation != self.generation`. `open_browser` (2557) and `try_connect`
  (2593) already take the profile. `showing` (2499), `shown` (2534) and `wait_for_start_pages`
  (2541) read the one state; `new_page` (2521), `create_page_task` (980) and `close_page` (1008)
  the one connection. `open_tab` (3141) hands a listed page to the first adopting tab of any
  project; `place_tab` (3186) anchors on the tab the user focused last anywhere. `render` (5371)
  and `message` (5342) draw the one state; `deserialize` (5556) gets the `Project` it restores
  into. `open` (6517), `new_tab` (6568), `open_url_tab` (6582) and `show_for_agent` (6505) call
  `global`. `service.rs`: as the draft read it, and still no stop. `browser_tools.rs`: `answer`
  (87) restarts the one browser; `run` (204) waits on its state; `caller_scope` (101) gives the
  caller's `home` workspace, which `navigate` (1004) places the new page in. `rail.rs`: the
  project row's menu (1236) holds Move Project Up and Down only; `move_project` (1021) is the
  pattern for Remove. Zed: `MultiWorkspace::remove_project_group` (`multi_workspace.rs:945`)
  drops the group, then emits `ProjectGroupsChanged`, then runs the removal (which can be
  refused at a save prompt); `rekey_project_group` (691) changes a key with no event; a window
  restores only its active workspace at launch (`restore_multiworkspace`,
  `workspace.rs:10284`), the others loading when opened; `Workspace::project_group_key`
  (`workspace.rs:2505`) reads the project's; `new_local` adds the worktrees before it builds the
  workspace and deserializes its items (`workspace.rs:2255`), so a restored tab reads a full key
  (the draft's restore-order risk is closed). On Linux the last window closing quits Zed
  (`zed.rs:351`).
- **Recall at promotion:** AD-claude-488 (one hub, shared pages: kept, D4); AD-claude-494 (a
  restored tab reattaches or reopens its URL: the move and the restart land there);
  AD-claude-574 (a call naming no tab acts in the caller's project; a named tab is the caller's
  choice: D8 builds on it); F-claude-574 (a placement goes with a failed attach, at a start and
  with its page: placements move into the per-project browser, so a project's start clears its
  own); L-claude-493, L-claude-494, F-claude-488 and L-claude-500 as in the draft;
  PR-claude-a-view-that-redraws-on-a-changed-snapshot-keeps-what-it-draws-in-it-001 (the rail
  adds no drawn value here, only a menu entry). Brain consultation
  27120e8923d74913ac0086e0db14de93: nothing on per-project profiles.

### Design (revised at promotion)
- **`service.rs`** (pure, then adapters taking their directory, §14):
  `project_key(paths, host) -> String` (16 hex digits of the SHA-256 of the sorted paths, each
  followed by a newline, then `host` and a newline for a remote project);
  `project_dir_in(data, key)` (`<data>/browser/projects/<key>`); `profile_in(project_dir)`;
  `legacy_profile_in(data)` (`<data>/browser/profile`). `write_project_file_in(project_dir,
  paths, host)`: makes the folder (0700) and writes `project.json` (`paths`, `host`, `made` in
  Unix seconds) through a temporary name, unless it is there. `move_legacy_profile_in(data,
  project_dir) -> LegacyMove` (`Moved`, `NoLegacy`, `KeptBecauseAProjectHasOne`), for a caller
  that stopped the legacy unit. `async fn stop(unit)`: `systemctl --user stop`; a unit that is
  not loaded is already stopped. The module comment: one unit per project.
- **The hub.** `BrowserProject { key, name, paths, host }`, made from a `ProjectGroupKey` (the
  name is Zed's `display_name`; the host line is the connection's type and host).
  `ProjectBrowser { project, state, connection, attaching, closing, placements, generation, run,
  stopping }` in `BrowserHub::browsers`; `BrowserHub` keeps `pages` (each `PageState` gains the
  `generation` of the start that attached it), `focus_history`, `picks` and the numbers, plus a
  hub-wide start counter, the legacy move's shared task and a `quitting` flag. Each start takes
  the counter's next number, so a generation names its browser: the 20 guards become
  `!self.is_current(generation)` and the 27 signatures stay. `attach`, `attached`, `fail` and
  `page_gone` find their browser by generation; `create_page_task` and `new_page` take the
  project's key; `close_page` finds the page's browser. `global` makes the hub and starts
  nothing; `browser_for(project, cx)` starts a project's browser when it is absent or failed.
  `state_of(key)` gives a project's state; a project with no browser reads as stopped, with the
  `marley: open browser` hint.
- **Start.** Every start awaits the legacy move, which the process's first start makes: stop
  `unit_name(legacy)` when it is up, wait (up to 15 s) until it is not, remove its
  `DevToolsActivePort`, move the folder to that first project, log the outcome. Then
  `write_project_file_in`, then `open_browser(profile)` as today.
- **Tabs.** `BrowserView` gains `project` (the key): the page's project for a tab made for a page,
  the workspace's for one made to wait for a page. `render`, `message` and `render_toolbar` read
  `state_of(project)`; `shown`, `wait_for_start_pages`, `showing`, `open_page_in` and the
  adopting tab's wait take the key. `open_tab` hands a listed page only to an adopting tab of its
  project. `place_tab` anchors among the page's project's tabs and, with no anchor, opens in a
  workspace of the page's project: the placement's, else the active window's active workspace
  when it is the project's, else the first workspace of the project in any window, else the
  active workspace. `open`, `new_tab`, `open_url_tab` and `deserialize` read the project from
  their workspace (`deserialize` from the `Project` Zed hands it) and call `browser_for`.
  `untabbed` in `open` lists the workspace's project's pages only.
- **Stop.** `init` observes each new `MultiWorkspace`: its `WorkspaceAdded`, `WorkspaceRemoved`
  and `ProjectGroupsChanged`, and its release (a window closing), each ask the hub to review. A
  project is live while any window lists its group or holds a workspace of it, or while a
  Browser tab of it sits in a workspace a window holds (a rekeyed workspace's tabs keep their
  browser). A running browser whose project is not live gets a stop in two seconds; a review that
  finds it live again drops the stop; the stop checks once more before it acts. `on_app_quit`
  sets `quitting` and drops every pending stop. A stop forgets the browser and its pages (their
  tabs close if any is left) and runs `service::stop` on its unit.
- **The tools.** `answer` makes the hub and starts nothing. `browser_tabs` waits while a browser
  starts or attaches (20 s at most), then lists every project's pages; a page whose tab sits in
  no workspace is named by its browser's project. The page tools wait the same way, then find
  their page as today; a named tab acts in whichever browser holds it (D8). `navigate`'s new page
  goes to the caller's project's browser (the scope's `home`), else the active workspace's,
  starting it when it is not running.
- **The rail.** After Move Project Down: a separator and "Remove Project", which calls
  `remove_project_group(&key, window, cx)` with `detach_and_log_err`, as Zed's sidebar does.
  The rail's comments that asking the hub starts the browser now say it makes the hub.
- **The fixture.** `browser_profile [root]` prints `<data>/browser/projects/<key>/profile` for
  the project whose one main path is `realpath ${1:-$E2E_WORK/repo}` (the key computed as D3
  says); `browser_unit [root]` hashes that path as before; `browser_teardown` stops the unit of
  every `browser/projects/*/` folder and of the legacy profile. `agent` stays on the default
  root.
- **File manifest.** Marley crates: `crates/marley_browser/src/service.rs`;
  `crates/marley_workbench/src/browser.rs`, `browser_tools.rs`, `rail.rs`. Scripts:
  `script/e2e/browser-fixture.sh`, `script/e2e/507-browser-context-per-project.sh`. At Complete:
  `CHANGELOG.md`, `docs/marley/three-prong-plan.md` (D16 and the slice row),
  `docs/marley_architecture/marley_browser.md`, `marley_workbench.md`, `marley_mcp.md` if the
  tools' text changes.
- **Ledger rows.** None: Zed's `ProjectGroupKey`, `MultiWorkspace` and its events are used through
  their public API, and no Zed file changes.

### E2E plan
`compositor sway`, the offline Chromium, a loopback site (`signin.html?as=<name>` sets a `login`
cookie, `localStorage.login` and an IndexedDB record; `whoami.html` prints all three). Setup
makes `alpha` and `beta` (git repositories) and `alpha-wt` (`git worktree add` from alpha), and
the legacy profile at `$E2E_PROFILE/browser/profile`: a scratch Chromium on it, signed in as
`legacy` by the stand-in agent, then stopped. Marley opens alpha; beta and `alpha-wt` reach the
running Marley by a second launch, as #574's scenario hands a project over (#513).

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-008 | alpha: `marley: open browser`, Ctrl+L, `whoami.html` | `507-01-migrated` (signed in as `legacy`); the log's unit list and `find browser -maxdepth 3`: one project folder with `project.json`, no `browser/profile` |
| REQ-002 | beta handed over; in beta, `marley: new browser tab`, `whoami.html` (signed out), then `signin.html?as=beta`, `whoami.html`; back in alpha, reload | `507-02-beta-signed-in`, `507-03-alpha-kept` |
| REQ-003 | `alpha-wt` handed over (it joins alpha's group); in its workspace, `marley: new browser tab`, `whoami.html` | `507-04-worktree-shares`; the unit list still shows two |
| REQ-009 | `mcp_agent tabs`; `mcp_agent --tab <beta's id> look` | the log: each tab with its project; the look names beta's page |
| REQ-004 | `quit_marley`, then the unit list | the log: both units active |
| REQ-005 | both units stopped; launch on alpha; beta handed over again | `507-05-alpha-after-restart`, `507-06-beta-after-restart` |
| REQ-007 | right-click beta's row, Remove Project; settle five seconds | `507-07-removed`; the log: beta's unit inactive, alpha's active |

Not reachable by a scenario: the move of Chad's own profile on his first launch after the update;
Test records which project took it, from `Marley.log` and the folder listing, once installed.

### Risks
- Memory: about 290 MB per open project's Chromium; this slice sets no cap (Out).
- A project's key changes when a folder is added to it, which gives it a clean profile; the old
  one stays on disk (D3). The rekeyed workspace's open tabs keep their browser until they close.
- The quit race: windows closing during a quit look like projects leaving. The two-second grace
  and the `on_app_quit` flag guard it, and REQ-004 checks it.
- A removal refused at a save prompt: the group is gone from the window's list though its
  workspace stays; the workspace keeps the project live, so its browser keeps running.
- The earlier browser scenarios depend on the fixture's profile lookup; the default root keeps
  them working, and Test reruns #494's and #574's.

## Phase 2 — Code
- **Checklist** (no task tool in this session): service.rs ✓; the hub's per-project browsers ✓;
  the tabs, restore and open paths ✓; the stop review ✓; the tools ✓; the rail ✓; the fixture ✓;
  fmt ✓; clippy on marley_browser, marley_mcp and marley_workbench with `--all-targets` ✓; the
  review of the diff ✓.
- **Built.** `service.rs`: `project_key`, `project_dir_in`, `profile_in`, `legacy_profile_in`,
  `write_project_file_in` (0700 folder, `project.json` with `paths`, `host` and `made` in Unix
  seconds, through `project.json.new`), `LegacyMove` and `move_legacy_profile_in`, `stop` (a unit
  that is not loaded is stopped already); the module comment says a unit per project.
  `browser.rs`: `BrowserProject` (`of`, `of_group`, `of_workspace`); `ProjectBrowser` holding
  what was the hub's `state`, `connection`, `attaching`, `closing`, `placements`, the start's
  `generation` and its task (`_run`, held for its drop) and `stop_pending`; the hub keeps the
  pages (each with the generation that attached it), the focus history, the picks and numbers,
  a hub-wide start counter, the legacy move's `Shared` task and `quitting`. The 20 guards read
  `is_current(generation)`; `attach`, `attached`, `fail` and `page_gone` find their browser by
  generation; `create_page_task`, `new_page`, `showing`, `shown`, `wait_for_start_pages`,
  `open_page_in` and `close_page` take the project's key. `global` starts nothing;
  `browser_for` starts an absent or failed browser and drops a pending stop;
  `restart_if_failed` restarts a failed one. `state_of(key)` reads an absent browser as
  "This project's browser is not running." with the open-browser hint. `BrowserView.project`;
  `open_tab` hands a listed page only to its project's adopting tab; `place_tab` anchors among
  its project's tabs (`anchor_tab`) and otherwise opens in `workspace_of_project`. `open`,
  `new_tab`, `open_url_tab` and `deserialize` (from the `Project` Zed hands it) call
  `browser_for`; `untabbed` lists the project's pages. `init` observes each `MultiWorkspace`
  (`WorkspaceAdded`, `WorkspaceRemoved`, `ProjectGroupsChanged`, and its release) and defers
  `review_browsers`; `live_projects` gathers each window's groups, its held workspaces'
  projects, and the projects of Browser tabs in held workspaces; `review` numbers a stop per
  browser that is not live and a detached task stops it after `STOP_GRACE` if it is still the
  pending stop and the project is still not live; `on_app_quit` calls `quit`, which sets
  `quitting` and drops the pending stops. `browser_tools.rs`: `answer` restarts only the
  caller's project's failed browser; `run` waits with `settled` (while any browser starts or
  connects, 20 s at most); `navigate`'s new page goes to `caller_project` (the scope's `home`,
  else the active window's workspace) through `browser_for`; `browser_tabs` names a page with no
  tab by its browser's project. `rail.rs`: Remove Project after a separator, through
  `remove_project` and `remove_project_group(..).detach_and_log_err`. `browser-fixture.sh`:
  `browser_project_dir`, `browser_profile [root]`, `browser_unit_of`, `browser_unit [root]`;
  `browser_teardown` stops every `browser/projects/*/` unit and the legacy one.
- **Deviations from the plan, and why.**
  - `marley_mcp`'s `browser_tabs` and `browser_navigate` descriptions now say each project has
    its own browser and logins: agents read them, and "one per page of its browser" was no
    longer true.
  - A pending stop is a number, not a stored task: the stop's own task would have dropped
    itself when it cleared the field.
  - The legacy move stops the old unit only when `is-active` says it is up; `systemctl stop`
    answers once the unit has stopped, and one more `is-active` confirms it before the rename.
  - The tools wait with `settled` instead of the one browser's `showing`: a tool no longer
    answers with a start's error unless it opens a page, where `new_page` still does.
  - `anchor_tab` came out of `place_tab`, which clippy found too long.
  - The fixture computes the key from the root instead of reading `project.json`: the key is
    the file's name, and a scenario may ask for a profile before the first start writes it.
- **Review of the diff.**
  - REQ-001, REQ-008: the first start awaits the move, then writes `project.json`, then opens
    Chromium on `projects/<key>/profile`. REQ-002, REQ-003: a browser per key, the key from Zed's
    group, which maps a linked worktree to its main path. REQ-004: gpui's `shutdown` runs the
    quit observers before it clears the windows, so every review after it sees `quitting`; a
    last window closed by hand quits within the grace. REQ-005: restore starts the project's
    browser from `deserialize`, and a missing page opens its saved URL (#494). REQ-007:
    `remove_project_group` drops the group, then detaches its workspaces after the close
    prompts; the project stays live while a window holds them, and its stop follows the
    `WorkspaceRemoved`. REQ-009: a named tab's `Page` carries its own browser's connection.
  - Re-entrancy: `subscribe_self` calls back inside the `MultiWorkspace`'s update, so the
    review is deferred; the review's liveness is computed before the hub's update; the stop
    task computes it in `cx.update` before `this.update`.
  - Fixed here: a module-doc line past 100 columns, `settled`'s doc (it said it waited on
    attaching), and a log line past 100 columns.
  - No Zed path changed; nothing derived from Warp.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario, a row per REQ ✓; three runs, the third green ✓;
  every shot read ✓; 488 and 494 rerun ✓; the golden set with 507 added ✓; the gate (below).
- **The scenario.** `script/e2e/507-browser-context-per-project.sh`, `compositor sway`, the
  offline Chromium. A loopback site: `signin.html?as=<name>` keeps the login as a cookie with a
  `Max-Age`, in `localStorage` and in an IndexedDB record, then goes to `whoami.html`, which
  shows all three and puts them in its title (`whoami: cookie=… local=… idb=…`), so the stand-in
  agent's `browser_tabs` reads them. `alpha` and `beta` (an empty commit each) and `alpha-wt`
  (`git worktree add`). Setup leaves an earlier build's unit running on
  `$E2E_PROFILE/browser/profile`, started with Marley's pre-#507 name and flags on
  `signin.html?as=legacy`, and waits for the page's title through `/json/list`. beta, alpha and
  alpha-wt reach the running Marley by a second launch (`hand_over`, #513). The restart step
  closes both units over CDP (`browser_close`, new in the fixture), as Marley closes one.
- **Run 1, red:** alpha's moved profile read `cookie=none`, with `localStorage` and IndexedDB
  `legacy`. A probe on scratch profiles (stopped at once, closed over CDP, the network service
  in process, stopped after 35 seconds) showed Chromium losing the cookies it had not written
  (F-claude-507, below). Fixed in `browser.rs`: `stop_chromium` sends `Browser.close`, waits up
  to five seconds for the unit to stop, then runs `service::stop`; `stop_browser` and the legacy
  move use it.
- **Run 2, red:** every check through REQ-005 passed. The removal right-clicked y 136, #574's
  constant, which is beta's terminal row (Rename and Close); End and Enter closed the scratch
  terminal. The project header is at y 95: `TOP_ROW_Y=95`.
- **Run 3, green:** 30 checks pass. The shots, read:
  - `507-01-migrated`: alpha's tab, "Who is signed in": cookie `legacy`, localStorage `legacy`,
    IndexedDB `legacy`; the rail shows alpha, its terminal and the page's row. With the log (the
    earlier build's unit inactive, `browser/profile` gone, `projects/<key>/profile` and a
    `project.json` naming alpha's folder, alpha's unit active): REQ-001 and REQ-008.
  - `507-02a-beta-signed-out`: beta's new tab reads none, none, none while alpha's row still
    says legacy; `507-02-beta-signed-in`: beta, beta, beta; `507-03-alpha-kept`: alpha's tab
    after F5, legacy, legacy, legacy. REQ-002.
  - `507-04a-worktree-handed-over`: Zed's Unrecognized Project prompt over alpha-wt's workspace,
    and the rail already lists `alpha-wt — bash` under alpha; `507-04-worktree-shares`: the
    worktree's new tab reads legacy three times; the log shows two project folders and two units,
    none for alpha-wt's own path. REQ-003.
  - The log after `quit_marley` and five seconds: both units active. REQ-004.
  - `507-05-alpha-after-restart` and `507-06-beta-after-restart`: after both Chromiums closed and
    a launch on alpha, alpha's restored tab reopened `whoami.html` in a new Chromium, legacy
    three times; beta, handed over again, came back beta three times. REQ-005.
  - `507-07b-remove-project`: beta's project menu, Move Project Up greyed, Move Project Down, a
    separator, Remove Project selected; `507-07-removed`: only alpha in the rail, its tab still
    legacy; five seconds later beta's unit is inactive and alpha's active, and `Marley.log` says
    "beta left Marley's windows, so marley-browser-… stops". REQ-007.
  - The log: `browser_tabs` lists alpha's two tabs, the worktree's among them, and beta's, each
    with its project; `--tab <beta's id> look` answers beta's page and title. REQ-009.
- **Regressions.** 488 passes on the fixture's default profile. 494 failed first in its own
  `report()`, which read the first `db.sqlite` `find` listed: the copied profile holds `0-dev`
  and `0-global`, and `0-global` has no `items` table; independent of #507, and the report also
  printed every saved tab of the copy. It now reads each database read-only, only this run's
  workspaces' rows, as #576 does, and passes: its tabs reattach while Chromium runs, and reopen
  their saved URLs after `browser_unit`'s unit stops. 576's `quit_all` stopped the unit of
  `$E2E_WORK/repo`, a project it never opens; it stops repo-a's and repo-b's now. The golden set
  with 507 added: 29 of 29 pass.
- **Focus.** Every run in a headless sway: "0 Marley windows before the run, 0 after; the run
  added no rule and did not reload it".
- **Not reachable by a scenario.** The move of Chad's own profile at his first launch of the
  installed build (`Marley.log` says which project took it); a stop at logout, which still loses
  a login made in its last 30 seconds (recorded in the plan's risks).
- **The gate.** The first `script/gates.sh --diff` was red on two stages. gate:21 (dylint,
  Zed's lints in the Marley crates) found four `async` blocks with no `.await`: three
  `background_spawn(async move { … })` of blocking work and `on_app_quit`'s `async {}`; they are
  `futures::future::lazy` and `futures::future::ready(())` now. gate:11 (shellcheck) flagged each
  call of `browser_unit` and `browser_profile` with no project (SC2119: they take one now) and
  the `${…}` inside `browser_close`'s single-quoted script (SC2016); every call names
  `$E2E_WORK/repo`, and the script joins its strings. The second run: `GATE GREEN [diff]`, 16 of
  16, the receipt on the tree committed. After it, a build and 488, 494, 504 and 507 again: all
  four pass.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Documented.** `CHANGELOG.md` (Added: a Chromium and a profile per project);
  `docs/marley/three-prong-plan.md` (D16: a unit per project; the B7b row, shipped; two risks: the
  memory per open project, and a stop at logout losing a login made in its last 30 seconds);
  `docs/marley_architecture/marley_browser.md` (the service: the project's folder and key,
  `project.json`, the legacy move, `stop`); `marley_workbench.md` (the hub's per-project
  browsers, the stop review, tab placement per project, Remove Project, the tools);
  `marley_mcp.md` (the two tool descriptions). No path outside the Marley-owned set changed;
  gate:16 found every touchpoint recorded.
- **Knowledge appended:** F-claude-507-a-stopped-unit-lost-the-cookie-set-before-the-stop-001;
  PR-claude-close-chromium-over-cdp-before-stopping-its-unit-001;
  L-claude-507-a-window-restores-only-its-active-workspace-001,
  L-claude-507-subscribe-self-calls-back-inside-the-entitys-update-001,
  L-claude-507-the-rails-project-row-is-its-header-001,
  L-claude-507-sync-work-for-the-background-executor-goes-in-future-lazy-001;
  AD-claude-507-a-chromium-and-a-profile-per-project-001.
- **Brain:** consultation 27120e8923d74913ac0086e0db14de93 closed with
  `decisions/marleys-browser-a-chromium-and-a-profile-per-project`, a follow-up due 2026-10-27
  (the memory of a Chromium per project, and whether an idle one should stop).
- **Closed:** TICKET-507 to `tickets/closed/`, its Clear Browser Data lines pointing at #581;
  #581's doc names `stop_chromium` and `stop_browser` for the clear to build on. The pair is
  archived to `completed/`.
