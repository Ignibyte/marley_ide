---
pipeline_id: e82dc7ba-ca2a-4c3a-98c3-4d43b16ab144
ticket: docs/planning/tickets/closed/TICKET-507-browser-context-per-project.md
status: Phase 4 — Complete PASS
title: "A Chromium and a profile per project"
type: feature
slice: prong 3, after the browser waves (item 4, second half); Chad's answer to the Orca survey's open question 2
references: [docs/orca_architecture/03-browser-and-design-mode.md, docs/planning/pipeline/completed/488-browser-pane.spec.md, docs/planning/pipeline/completed/494-browser-tabs-restored.spec.md]
---

## Title
Each project gets its own Chromium, a transient user unit on a profile directory of its own,
keyed on the project's main worktree paths, so a project's logins (cookies, `localStorage`,
IndexedDB) are its own, survive restarts, and are shared by the project's linked worktrees. A
project's Chromium starts with its first Browser tab and stops when the project is removed from
Marley. The single profile of earlier builds becomes the first project's. Clearing one
project's data is #581's, split off at promotion.

## Scope
### In
- `crates/marley_browser/src/service.rs`: the project key (16 hex digits of the SHA-256 of the
  project's main worktree paths, sorted, one per line, and a host line for a remote project);
  the project's folder `<data>/browser/projects/<key>/` with `profile/` and `project.json` (the
  paths, the host, when it was made); `stop(unit)` through `systemctl --user stop`; the one-time
  move of `<data>/browser/profile`, the profile every earlier build used. `unit_name(profile)`
  and `chromium_args(profile)` stay as they are: a second profile is a second unit.
- `crates/marley_workbench/src/browser.rs`: the app's one `BrowserHub` holds a browser per
  project (its key, name, state, connection, generation and start task) and records each page's
  project. `open`, `new_tab`, `open_page_in`, `place_tab`, `show_for_agent` and the restore's
  `deserialize` find the project from their workspace. A tab shows its own project's browser
  state. The hub stops a project's browser when the project leaves every window while Marley
  runs, and never during a quit.
- `crates/marley_workbench/src/browser_tools.rs`: `browser_tabs` lists every project's pages,
  each with its `project`; a named `tab` acts in its own project's browser; a new page opens in
  the caller's project's browser (#574's scope), else in the active workspace's.
- `crates/marley_workbench/src/rail.rs`: the project row's context menu gains "Remove Project"
  (`MultiWorkspace::remove_project_group`, as Zed's own sidebar offers it).
- `script/e2e/browser-fixture.sh`: `browser_profile [root]` and `browser_unit [root]` name a
  project's profile and unit (the scenario's `$E2E_WORK/repo` by default, which every browser
  scenario since #488 opens), found through `project.json`; `browser_teardown` stops every unit
  whose profile lies under the run's profile copy.
- `script/e2e/507-browser-context-per-project.sh`.

### Out (explicitly deferred)
- Clear Browser Data for one project (D6, REQ-006): #581, split off at promotion, next in the
  queue.
- Refusing a named tab of another project: #574 (with #520's terminal identity) made a call that
  names no tab act in the caller's project, and left a named tab the caller's choice
  (AD-claude-574). #507 keeps that and runs the named tab in its own project's Chromium.
- Named profiles inside a project, cloning a tab into another profile, cookie import (Orca's
  profile menu and importer, report 03 §2.8 and §2.9).
- A cap on how many project Chromiums run, and stopping an idle project's Chromium while the
  project stays open.
- A remote project's pages loading through its SSH host (report 03 §3 item 15). A remote
  project gets a profile of its own; its pages still load from this machine.
- A setting that gives a worktree a clean profile instead of its project's.
- Moving a Browser tab from one project to another.

## Reference (§20)
Upstream Zed's project groups: `project::ProjectGroupKey` (`crates/project/src/project.rs:6590`)
is what Zed's sidebar and Marley's rail group workspaces by, and it maps each linked worktree to
its main repository's path (`WorktreeStore::paths`, `crates/project/src/worktree_store.rs:1481`).
Marley keeps Zed's grouping as the definition of a project and gives each group one browser,
which is how "worktrees share their project's profile" needs no rule of its own. Orca's
persistent profiles (report 03 §2.8: an Electron `persist:` partition per profile, named
deterministically so it rebuilds at startup) are the behavior matched for what survives a
restart; Marley keys on the project where Orca keys on a named profile. Warp: N/A, Warp has no
browser.

### Prior art
- **Behavior maps and reports.** Report 03 §2.8 (Orca's profiles: `persist:` partitions keep
  cookies, `localStorage`, IndexedDB, service workers and cache on disk; every worktree shares the
  default profile unless the user switches; `src/shared/browser-workspace-types.ts`,
  `src/main/browser/browser-session-registry.ts`); §2.2 (storage keyed on the execution host's
  durable identity, after per-boot values in the partition name lost cookies and `localStorage`
  at every restart or reconnect; `browser-execution-host-storage-identity.ts`); §3 item 3 (CDP
  contexts are off the record; one unit per project with its own `--user-data-dir`; stop on
  close; a clear action; the worktree question Chad answered). The README's open question 2 and
  its #507 paragraph.
- **Published material.** The CDP spec on `Target.createBrowserContext`: "Creates a new empty
  BrowserContext. Similar to an incognito profile but you can have more than one." (Target.pdl,
  read 2026-09-25): the route this ticket first planned, dropped because an incognito-like
  context keeps no `localStorage` or IndexedDB across a restart. systemd's transient units
  (`systemd-run --user --collect`, `systemctl --user stop`), already Marley's (D16).
- **The code we already ship.** `service.rs` names each unit by its profile path's hash
  (`unit_name`, `service.rs:66`), so a profile per project is a unit per project with nothing new
  in the naming; `chromium_args` (`service.rs:81`) and `open_browser`/`try_connect`
  (`browser.rs:2557`, `2593`, at f21b8aea6b) already take the profile as an argument, and only
  `BrowserHub::start` hard-codes the one profile (`browser.rs:793`). The tab restore (#494) keys
  each saved tab by workspace and item (`MarleyBrowserTabsDb`, `browser.rs:5613`), and a tab's
  workspace names its project, so the saved rows need no project column. Zed already signals a
  project leaving (`MultiWorkspace::remove_project_group`, `multi_workspace.rs:945`, and
  `MultiWorkspaceEvent::ProjectGroupsChanged`, line 114) and offers Remove in its own sidebar
  (`crates/sidebar/src/sidebar.rs:3155`). Does a crate we build own the seam? Yes: `project` owns
  project identity, and `marley_browser::service` owns units and profiles; the ticket adds no
  second notion of either.

## UI proof
UI-AFFECTING (the Browser tabs' logins, the rail's menu). `script/e2e/507-browser-context-per-project.sh`
(`compositor sway`: it clicks the rail's menus). Setup: the offline Chromium; a loopback site whose
`signin.html?as=<name>` sets a `login` cookie, `localStorage.login` and an IndexedDB record, and
whose `whoami.html` prints all three; scratch repositories `alpha` and `beta` and a linked worktree `alpha-wt`
(`git worktree add`); the profile of earlier builds, made at `$E2E_PROFILE/browser/profile` by a
scratch Chromium that signs in as `legacy` and stops. Marley opens alpha; beta and `alpha-wt`
reach it by a second launch, as #574's scenario hands a project over (#513). Shots: alpha's tab signed in as
`legacy` after the move (`507-01-migrated`); beta signed in as `beta` in a tab of its own
(`507-02-beta-signed-in`); alpha's tab reloaded, still `legacy` (`507-03-alpha-kept`); a tab
opened from the worktree's workspace, `legacy` (`507-04-worktree-shares`); after a quit, both
units stopped and a launch, each project's tab back at its URL and signed in
(`507-05-alpha-after-restart`, `507-06-beta-after-restart`); beta removed from the rail
(`507-07-removed`). The run log
carries the unit list and `browser/` listing after each state, the stand-in agent's
`browser_tabs`, and beta's unit going inactive after the removal.

## Locked-In Decisions
- D1: One Chromium per project (Chad, 2026-09-25). A persistent profile keeps cookies,
  `localStorage` and IndexedDB across restarts, which CDP contexts cannot (report 03 §3 item 3).
  The price is memory: the unit running on the dev box on 2026-09-25 held 287 MB with Chad's
  pages.
- D2: A project is Zed's project group (`ProjectGroupKey`: the main worktree paths, and the host
  for a remote project). A linked worktree maps to its main repository's path, so it shares the
  project's Chromium and profile, as Chad decided, with no rule of Marley's own.
- D3: The key is durable, never per boot: 16 hex digits of the SHA-256 of the group key's
  sorted main paths, each followed by a newline, then a host line for a remote project. The
  project's folder `<data>/browser/projects/<key>/` holds `profile/` and `project.json`, which
  names the paths and host for anyone reading the folder, the scenario included. Adding a folder
  to a project makes a new key, and so a new profile; the old one stays on disk.
- D4: One hub, a browser per project inside it. AD-claude-488's single hub stays, so the user and
  agents keep sharing pages; pages, picks, annotation numbers, recordings and the focused tab
  stay app-wide, and each page records its project. Connection, state, generation and start task
  move into the per-project browser.
- D5: A project's Chromium starts on its first Browser tab: `marley: open browser`, Ctrl+T, the
  rail's New Browser Tab, a tab restored at launch, or an agent's new page in that project. It
  stops when the project leaves every window while Marley runs, after a two-second grace so that
  a workspace swap that adds the project back does not stop it. It keeps running at a quit,
  whether through the palette or the last window closing: the unit outlives Marley (plan D16),
  which is what lets #494's tabs take their pages back.
- D6: Clear Browser Data moved to #581 at promotion, with its design as drafted here: it asks
  first, naming the project; closes the project's Browser tabs; stops the unit and waits until
  systemd reports it inactive; deletes `profile/`, keeping `project.json`.
- D7: The old profile moves once, by rename, to the first project whose Chromium starts after the
  update, after its own unit has been stopped and its `DevToolsActivePort` removed. It is never
  deleted: when any project profile already exists, it stays where it is and the log says so.
  Tabs restored at that launch find their pages gone and open their saved URLs, the #494 path.
- D8: The agent tools span every project's browser. `browser_tabs` names each tab's project; a
  named tab acts in its own project's Chromium; with no tab named, the caller's project's tab the
  user focused last, as #574 made it; a new page opens in the caller's project's Chromium, in
  the caller's own workspace (#574's placement), and for a caller in no project, in the active
  workspace's project.
- D9: The rail's project menu gains Remove Project, the way Zed's sidebar offers it: without it
  the rail has no way to remove a project, and removing one is what stops its Chromium.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a project's first Browser tab opens, the system shall start a Chromium unit for that project on `<data>/browser/projects/<key>/profile` and write the project's `project.json`. | Shot `507-01-migrated`; the run log's unit list and `browser/` listing |
| REQ-002 | WHILE two projects have Browser tabs, the system shall keep a cookie, `localStorage` or IndexedDB value set in one project's tab out of the other project's tabs. | Shots `507-02-beta-signed-in`, `507-03-alpha-kept` |
| REQ-003 | WHEN a linked worktree of a project opens a Browser tab, the system shall run the tab in its project's Chromium, signed in as the project is, and start no new unit. | Shot `507-04-worktree-shares`; the unit list |
| REQ-004 | WHEN Marley quits, the system shall leave every project's Chromium unit running. | The run log after `quit_marley` |
| REQ-005 | WHEN Marley starts after the projects' Chromiums were stopped, the system shall open each restored tab's saved URL in its own project's new Chromium, still signed in through its cookie, its `localStorage` and its IndexedDB record. | Shots `507-05-alpha-after-restart`, `507-06-beta-after-restart` |
| REQ-006 | Moved to #581 at promotion: Clear Browser Data for one project. | — |
| REQ-007 | WHEN the user removes a project from the rail while Marley runs, the system shall stop that project's Chromium unit within five seconds and leave the other projects' running. | Shot `507-07-removed`; the run log's unit states |
| REQ-008 | WHEN the first project Chromium starts and `<data>/browser/profile` exists with no project profile yet, the system shall move that profile to the starting project, whose tabs then keep its cookies, `localStorage` and IndexedDB. | Shot `507-01-migrated`; the listing (no `browser/profile` left) |
| REQ-009 | WHEN an agent calls a browser tool, the system shall list every project's tabs with each tab's project in `browser_tabs`, and act on a named tab in that tab's own project's Chromium. | The run log: the stand-in agent's `browser_tabs` and `browser_look` on beta's tab |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. On promotion, recall again
  (the knowledge ledger moves daily) and check #503 and #504, which run first and touch the same
  hub.
- **P2 Code:** the key and folders in `service.rs`, the per-project browsers in the hub, the tools,
  the rail's Remove Project, the fixture; fmt and clippy clean; a review of the diff against
  each REQ, the gpui re-entrancy rules and errors reaching the tab.
- **P3 Test:** write and run the scenario and read every shot; rerun the earlier browser
  scenarios that read the profile (`494-browser-restore.sh`, `574-…`, and the golden set) to show
  the fixture's new profile lookup; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG, the prong's slice status in `docs/marley/three-prong-plan.md` and
  its D16 paragraph (one unit per project), the ledger capture, close the ticket, archive,
  commit.
