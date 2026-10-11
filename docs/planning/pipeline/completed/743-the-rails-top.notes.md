# The rail's top - the Rusty group leaves, Home becomes Default — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-743-the-rails-top.md
- **Pipeline spec:** 743-the-rails-top.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-11: "The rusty panel shouldnt be there anymore on start up"; "i cant
  manually remove it. but rusty now is in the bottom panel which is what we want"; the Home group
  renamed Default (his idea, agreed). Shelf: `docs/planning/design-notes/tabs-everywhere-2026-10-11.md`.
- **Classification / tier:** feature, Marley crate only (`marley_workbench`), no Zed hunk.
- **Checklist (no TaskCreate in this harness):** pick ✓, pre-flight ✓ (cargo idle, no active
  pipeline, README marker present, 355 GB free), recall ✓, mint ✓, prior art ✓, spec ✓, design ✓.
- **Recall (§18.3):**
  - AD-claude-675 and AD-claude-700: the Rusty group was made on first Rusty open, then from the
    start; AD-675 had rejected the start-time group as "a rail with a group nobody used", which is
    where Chad has come back to.
  - AD-claude-676 rejected "a new Default group beside Home" (two catch-alls); this renames Home,
    so there is still one.
  - L-claude-675: every Rusty opener funnels through `rusty::in_rusty_group`; route there, and
    check the palette actions call `open_later`.
  - F-claude-679: an open must be deferred, since openers run inside the workspace's update;
    F-claude-702/708: making a group in the background must not take the focus (`with_group` is
    the path that already does it right).
  - Brain (`rusty-cli brain ask`): only due follow-ups came back; nothing on this seam.
- **Discovery:** `groups.rs` (GroupKind, `make`, `adopt`, `reopen`, `in_group`, `claim`, `rename`),
  `rail.rs` (`ensure_groups` 1196, `rusty_changed` 959, `leave_rusty_group` 977, `refresh` 1107,
  `rail_groups` 8055, `header_icon` 495, header render 4799, `group_context_menu` 4964,
  `remove_group` 4084), `rusty.rs` `in_rusty_group` 703, the openers in `rusty/{brain_tab,
  decisions_tab,graph_tab,memory_tab,skills_tab,secrets_tab,tasks_tab,page,home_tab}.rs`, and
  `home_tab.rs` (`fill`, `fill_shown`, `rusty_links`, `open_later`, `ensure`, `render_screens`).

### Design
- **groups.rs.** `GroupKind` keeps `Named` and `Home`. `HOME` becomes `DEFAULT = "Default"`; `make`,
  `claim` and `adopt` name the Home kind Default (adopt overrides a saved "Home"). `Group.rusty`
  stays, documented as an earlier build's Rusty group awaiting retirement; `make` and `claim` never
  set it. New `rusty_groups(multi_workspace, cx) -> Vec<(Uuid, Entity<Workspace>)>` lists them, and
  `demote(id, cx)` clears the flag and saves (REQ-006). `is_rusty_workspace`, `rusty_group`,
  `is_rusty`, `pending_kind`'s and `group_of_kind`'s Rusty arms go. New `in_shown(asked_from, window,
  cx, open)`: `window.defer`, then in the Marley layout the root `MultiWorkspace`'s shown workspace,
  else `asked_from`; runs `open` in it.
- **rail.rs.** `ensure_groups` wants Home only. `rusty_changed` keeps `ensure_groups` (Home) and drops
  `leave_rusty_group`. `refresh` calls a new `retire_rusty_groups` after `adopt`: for each live Rusty
  group not already retiring (a `retiring: Vec<Uuid>` on the rail), defer; if the window shows it,
  `with_group(Home, …)` and activate Default first; then `remove_group`-style removal whose refusal
  calls `groups::demote`. `remove_group` gains that refusal branch, or the retire path inlines the
  same three steps with it. `rail_groups` drops the Rusty sort arm and filters `group.rusty` out of
  the rows always. `header_icon` loses its `rusty` parameter; `group_context_menu` loses the Rusty
  branch.
- **rusty.rs.** `in_rusty_group` becomes `in_shown_group`, calling `groups::in_shown`, with no
  `home_tab::ensure`. Every opener's call is renamed.
- **home_tab.rs.** `fill` and the Rusty half of `fill_shown` go (Default still gets the Home page,
  #701). `rusty_links` and its `MarleyNewItemMenu` install go. `open_later` runs `open_here` through
  `in_shown_group`. `ensure` goes. `render_screens` gets a second row: Open Page…, Capture to Today…,
  Capture to Inbox…, Capture a URL…, each dispatching its action as the + menu did.
- **Manifest:** all Marley crate: `crates/marley_workbench/src/{groups.rs, rail.rs, rusty.rs,
  rusty/home_tab.rs, rusty/brain_tab.rs, rusty/brain.rs (doc), rusty/decisions_tab.rs,
  rusty/graph_tab.rs, rusty/memory_tab.rs, rusty/skills_tab.rs, rusty/secrets_tab.rs,
  rusty/tasks_tab.rs, rusty/page.rs}`; `script/e2e/743-the-rails-top.sh`; docs at Complete. No Zed
  crate, so no ledger row; the `pane.rs` row's "the Rusty group's panes list Rusty's screens there"
  gets a note that the hook has no user since #743 (Complete).

### Visual check plan
- `old` (REQ-003's starting point): installed build, Rusty on by stand-in, repo open; after the 3 s
  settle the rail lists Home, Rusty, repo; the palette's `rusty: open home` shows the Rusty group;
  then a quit.
- `restored` (REQ-001, REQ-003, REQ-004): the debug build launched with no path restores the session
  that showed the Rusty group; after the settle, the rail's first row reads Default, no Rusty row,
  and Default is shown.
- `brain-here` (REQ-002): click the repo's row, then the palette's `rusty: brain`; the Brain tab is in
  the repo's tab bar and the repo is still the highlighted row.
- `rusty-home` (REQ-005): the status bar's Rusty button; the home page shows the second row of
  buttons.
- Not reachable: REQ-006's cancelled close needs a running process in a group the old build made;
  reviewed in the diff instead.

### Risks
- The removal is async; a refresh during it must not ask twice (the `retiring` list).
- The installed build in the scenario writes the run's profile in its own format; both builds share
  the record format (the `rusty` field exists in both), so the debug build reads it.
- A window with only the Rusty group restored: Default is made first through `with_group`, which
  keeps the focus (#702/#708), then the Rusty group goes.

## Phase 2 — Code
- **Built:** `groups.rs`: the Rusty kind goes (`GroupKind` is `Named` and `Home`); the Home kind is
  named `DEFAULT` ("Default") in `make`, `claim` and `adopt` (a record saved as "Home" comes back as
  Default); `rusty_groups` lists an earlier build's Rusty groups; `demote` keeps a group whose close
  was cancelled as a named "Rusty"; `in_shown` runs an open in the shown workspace (deferred; the
  asking workspace in Zed's layout); `names_but` is shared by `rename` and `demote`. `rail.rs`:
  `retire_rusty_groups` runs after `adopt` in every refresh and `close_retired` closes through
  `MultiWorkspace::remove(CloseProject)`, forgetting the group on a close and demoting it on a
  refusal or an error; a `retiring` list keeps a refresh during a close from asking twice;
  `ensure_groups` wants Home only; `rusty_changed` no longer makes or leaves a group;
  `rail_groups`, `header_icon` and the group menu lose their Rusty arms. `rusty.rs`:
  `in_rusty_group` became `in_shown_group` over `groups::in_shown`, and every opener calls it.
  `home_tab.rs`: the Rusty group's fill, + menu links and forced-first page go; `open_later` opens
  or brings forward the home page through `open_here`; the RUSTY card gains a row of Open Page… and
  the three captures, dispatched from the page's focus handle, deferred.
- **Deviation:** the e2e runner gained `use_binary <path>` (one line, documented in the header
  comment's style) so a scenario can relaunch with another build; setting `marley` directly in a
  scenario trips shellcheck's SC2034.
- **Review:** one real defect, fixed before the gate. A Rusty group restored as the shown
  workspace would have run `with_group(Home)` before the window's saved Default came back, making
  a second Default (#700's duplicate trap). A shown Rusty group now waits until the window holds
  Default or is making it (`has_kind`); hidden ones close at once. Re-entrancy: the retire step
  defers before it reads the `MultiWorkspace` or removes a workspace; the home page's buttons defer
  their dispatch. No Zed crate touched; `MarleyNewItemMenu` in `pane.rs` now has no user (#744
  decides).
- **Gate:** the first run was red: five clippy findings in the new code (`Rail::new` one line over
  100, `into_iter` on one item, a bool-to-int `if`, an unused `self`, a needless `&mut App`) and a
  receipt mismatch from the e2e helper written mid-run. Fixed at the source: `ensure_groups` takes
  its one kind directly, Default sorts by `!group.home`, the `retiring` list moved into
  `StartupGroups`, `render_forms` and `init` lost what they did not use. `just gate-diff` then
  GREEN, all 16 gates and the receipt.

## Phase 3 — Test
- **Scenario:** `script/e2e/743-the-rails-top.sh`, `compositor sway`. The installed build (19dda499a2)
  runs on the run's profile with Rusty on through its stand-in, shows its Rusty group with
  `rusty: open home` and quits; `use_binary` and `open_path ""` then relaunch the debug build with
  no path, so it restores that session. Run twice: the first run's `tasks-here` used `multi
  workspace: next project`, which skips a project restored closed, so Tasks opened in Default (REQ-002
  held, but not in a project); the second run clicks the repo's row instead and adds a click on Open
  Page…. Both runs exit 0.
- **Shots (second run):**
  - `743-01-old`: the installed build's rail lists Home, Rusty (shown, its home tab open), then
    repo. The starting point REQ-003 needs.
  - `743-02-restored`: the debug build after the restore: the rail's first row reads Default and is
    shown with the Home page; no Rusty row; repo below it, closed. REQ-001, REQ-003, REQ-004 (the
    status bar's Home button still reads Home).
  - `743-03-tasks-here`: after a click on repo's row and `rusty: open tasks`, Tasks is a tab in
    repo's tab bar beside `repo — bash`, repo's row is highlighted, and no group appeared. REQ-002.
  - `743-04-rusty-home`: Rusty's home page as a tab in repo; under the eight screen buttons, the row
    Open Page…, Capture to Today…, Capture to Inbox…, Capture a URL…. REQ-005.
  - `743-05-open-page`: a click on Open Page… opens the page picker ("Open a page of Rusty's brain,
    or name a new one…"): the deferred dispatch from the page's focus handle reaches the workspace.
- **Not reached by a scenario:** REQ-006 (a cancelled close keeps the group as a named Rusty group)
  needs a running process in the old build's Rusty group; reviewed in the diff.
- **Pre-existing, not in scope:**
  - The Rusty home page's Recent Pages shows page names from the profile the run copied (noted
    after #701; already on the list to raise with Chad).
  - Repo restored with a right-side project panel open from the old build's dock state: #747 stops
    the right dock opening.
  - The run log's errors are the headless sway's keymap, the Vulkan loader, a llama.cpp provider in
    the copied settings, and the quit's timeout.
- **Focus:** the scenario's keys went to the run's Marley only; the user's session was not touched
  (sway's report: 1 Marley window before and after).
- The scenario changed after the gate's green, so Complete runs the gate again before the commit.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Changed: no Rusty group; Home is Default); the architecture note
  (`marley_workbench.md`, a new section ahead of the Rusty group's, which now reads as retired);
  the slice line in `workbench-shell.md`; the guide and the walkthrough came with Phase 2. No Zed
  path is touched; `pane.rs`'s `MarleyNewItemMenu` row stays as it is until #744 decides its use.
- **Knowledge appended:** F-claude-743-a-shown-rusty-group-would-make-a-second-default-001,
  PR-claude-743-make-a-window-group-only-when-none-can-still-come-back-001,
  L-claude-743-test-a-migration-by-running-the-installed-build-first-001,
  AD-claude-743-rustys-screens-open-where-the-user-is-and-home-is-default-001.
- **Brain:** consultation a547b58a closed with `brain decide`:
  `decisions/marley-rustys-screens-open-where-the-user-is-and-the-home-group-is-called-default`,
  follow-up by 2026-10-25.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate:** run again on the tree committed, since the scenario changed after Phase 2's green:
  GATE GREEN [diff], 17 passed, 0 failed.
