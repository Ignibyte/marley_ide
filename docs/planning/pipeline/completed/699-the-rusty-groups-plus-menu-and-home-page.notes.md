# The Rusty group's + menu and its home page — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-699-the-rusty-groups-plus-menu-and-home-page.md
- **Pipeline spec:** 699-the-rusty-groups-plus-menu-and-home-page.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09, from daily use of the installed Marley: quick links for Rusty in
  the Rusty group's +, and the home page instead of Zed's "open project" screen when the group is
  empty ("lets default it to basically the home page. or have the home page open all the time").
- **Classification:** feature, Marley layout. Marley crate, plus one small additive hook in Zed's
  `workspace/src/pane.rs`.
- **Recall (§18.3):**
  - AD-claude-679: Chad asked to "force a home page"; `home_tab::ensure` keeps it first in the
    group, but only when something opens there through `in_rusty_group`. Nothing puts it back once
    the group is empty, and it is not restored after a restart.
  - AD-claude-675: one Rusty group per window, a folderless workspace, left out while Rusty is off.
  - Nothing in the ledgers on the + menu or the empty pane. The brain's decision page for #679
    says the same as AD-679.
- **Discovery:**
  - The empty state: `Pane::render` draws `welcome::WelcomePage` (Open Project, recent projects)
    in a pane with no item when the project has no visible worktree, as in every projectless
    group. That is what Chad saw.
  - The + menu: `default_render_tab_bar_buttons` in `pane.rs` builds it inline (New File, Open
    File, Search Project, Search Symbols, New Terminal, New Center Terminal), with no hook.
    `Pane.workspace` (`pub(crate)`) names the pane's workspace.
  - `workspace::Event::ItemRemoved` follows every pane item's removal;
    `MultiWorkspaceEvent::ActiveWorkspaceChanged` follows `MultiWorkspace::activate`.
  - `groups::adopt` runs from the rail's refresh after a restart and writes `Groups`, whose
    observers then run.
  - Actions in reach: `rusty::OpenPage { slug: None, preview: false }`, `CaptureToToday`,
    `CaptureToInbox`, `CaptureUrl`; screens through `brain::open_screen`, home through
    `home_tab::open_later`.

### Design
- **Zed (`crates/workspace/src/pane.rs`):**
  - `pub struct MarleyNewItemMenu(pub Arc<dyn Fn(ContextMenu, &WeakEntity<Workspace>, &mut
    Window, &mut App) -> ContextMenu>)`, a `Global`, and `fn marley_new_item_menu(menu,
    workspace, window, cx)`, which passes the menu through the global when set.
  - In `default_render_tab_bar_buttons`, the + menu's builder starts from
    `marley_new_item_menu(menu, &workspace, window, cx)`, with `pane.workspace` cloned in.
  - Each hunk carries `// Marley:`; the existing `pane.rs` row in `docs/marley/zed-touchpoints.md`
    gains the hook before the edit.
- **Marley (`crates/marley_workbench/src/rusty/home_tab.rs`):**
  - `init` sets `MarleyNewItemMenu` to `rusty_links`: for a Rusty group's workspace while Rusty is
    on, it adds Home (`RUSTY_ICON`), each `Screen::ALL` entry with its icon (handlers calling
    `open_screen` with the window's `MultiWorkspace`), a separator, Open Page… and the three
    captures (as actions), then a separator before Zed's entries. Any other workspace gets the
    menu back unchanged.
  - The empty-group rule, `fill(workspace, window, cx)`: while Rusty is on, if the workspace is
    the Rusty group's and holds no item, `ensure` adds the home page. Callers:
    - each workspace's own `ItemRemoved` (`cx.subscribe_in(&cx.entity(), window, …)` in the
      existing `observe_new`);
    - each `MultiWorkspace`'s `ActiveWorkspaceChanged` and its window's `Groups` changes
      (`observe_global_in`), on the workspace shown.
- **Marley (`crates/marley_workbench/src/groups.rs`):** `is_rusty_workspace(id: EntityId, cx)`,
  whether a live Rusty group holds that workspace.
- **File manifest:**
  - `crates/workspace/src/pane.rs`: Zed crate, the hook (two small hunks).
  - `docs/marley/zed-touchpoints.md`: the `pane.rs` row.
  - `crates/marley_workbench/src/rusty/home_tab.rs`: Marley crate, the links and the rule.
  - `crates/marley_workbench/src/groups.rs`: Marley crate, `is_rusty_workspace`.
  - `docs/marley/guide.md`: the Rusty section's line on the + and the home page.
  - `script/e2e/699-the-rusty-groups-plus-menu-and-home-page.sh`: the scenario (Test).

### Visual check plan
Under `compositor sway`, with the stand-in `rusty-mcp` (`MARLEY_RUSTY_MCP`, `RUSTY_STAND_IN_STATE`
over a scratch vault and `tasks.json`), as #679's scenario does.

| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Clicks the rail's Rusty button (home page in the Rusty group), then the pane's + | 699-01-plus-menu: Home, the eight screens, Open Page… and the captures above New File … New Center Terminal |
| 002 | Chooses Tasks in that menu | 699-02-from-menu: the Tasks tab in the Rusty group, after the home page |
| 003 | Closes every tab of the group (`workspace: close all items and panes` from the palette, or ctrl-w per tab) | 699-03-closed-all: the home page alone, no Welcome page |
| 004 | Shows the project, quits Marley (`quit_marley`), relaunches, clicks the Rusty group's header in the rail | 699-04-shown-empty: the home page in the Rusty group |
| 005 | Shows the project and opens its + menu | 699-05-project-plus: Zed's six entries only |
| 006 | — | The gate |

The rail's Rusty group header and the + button's position are read from the first run's shots.

### Risks
- **Closing the home page beside other tabs.** It closes; the rule acts only on an empty group, so
  it comes back when the group empties or anything opens there. Re-adding it on every close would
  make its × do nothing visible.
- **A Welcome page flash.** The fill runs on the event after the pane empties, so the Welcome page
  may draw for one frame.
- **Rusty off.** The rule and the links do nothing while Rusty is off; the group is not listed
  then (#675).
- **The hook's cost to upstream merges:** one global type and one call in a function upstream
  rewrites now and then; the ledger row says how to carry it.

## Phase 2 — Code
- **Built:**
  - `crates/workspace/src/pane.rs` (Zed): `MarleyNewItemMenu`, a `Global` holding
    `Fn(ContextMenu, &WeakEntity<Workspace>, &mut Window, &mut App) -> ContextMenu`;
    `marley_new_item_menu`; and in `default_render_tab_bar_buttons` the + menu's builder starts
    from it, with `pane.workspace` cloned in. Each hunk carries `// Marley:`. The `pane.rs` row in
    `docs/marley/zed-touchpoints.md` was extended first.
  - `crates/marley_workbench/src/groups.rs`: `is_rusty_workspace(id, cx)`.
  - `crates/marley_workbench/src/rusty/home_tab.rs`:
    - `init` (now `&mut App`) sets the hook to `rusty_links`: for a Rusty group's workspace while
      Rusty is on, Home (`RUSTY_ICON`, `open_later`), each `Screen::ALL` entry with its icon
      (`RustyHome::open_screen`), a separator, Open Page… (`FileMarkdown`), Capture to Today… and
      Capture to Inbox… (`Plus`), Capture a URL… (`Link`), and a separator before Zed's entries.
      The four forms dispatch their actions from their handlers: the + menu sets no action
      context, so this is the dispatch Zed's own `action` entries make there, with an icon added.
    - `fill(workspace, …)`: while Rusty is on, a Rusty group's workspace with no item gets
      `ensure`'s home page. Each workspace hears its own `ItemRemoved`
      (`subscribe_in(&cx.entity(), window, …)`); each `MultiWorkspace` its
      `ActiveWorkspaceChanged` and the `Groups` global (`fill_shown`, on the shown workspace).
  - `docs/marley/guide.md`: "The Rusty group and its home page", which the guide lacked since
    #675 and #679, with #699's two lines.
- **Deviations:** none from the design.
- **Review of the diff:**
  - The hook runs while the `ContextMenu` is being built: it reads the `Groups` and `Rusty`
    globals and a weak handle's id, and updates no entity. Its entries' opens defer
    (`in_rusty_group`'s `window.defer`), and the actions dispatch as Zed's own entries do.
  - `fill` runs from subscriptions, after the update that emitted, so the pane it adds to is not
    being updated. A tab dragged out of an only-tab pane is re-added in the same effect cycle, so
    the group is not empty when `fill` looks. `ensure` adds the home page only when the workspace
    has none, so no path makes two.
  - A close-all closes the ids it took at its start, so a home page `fill` adds meanwhile stays.
  - Rusty off: neither the links nor the fill act, and the group is not listed (#675).
  - The Zed hunk is additive: one type, one function, one clone and the builder's first call.
- **Gate (`699-gate-1.log`):** GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/699-the-rusty-groups-plus-menu-and-home-page.sh`, under
  `compositor sway`, on the debug build, with the stand-in `rusty-mcp` over a scratch vault and
  `tasks.json` (two made-up lists), as #679's scenario does.
- **First run (`shots-699a`), positions and a relaunch fix:**
  - The Rusty group has no Agent Panel, so its + is at (1530, 50), not where #697's was; the
    project's + is at (1290, 50), left of its project panel. The rail lists the project's header
    at y 89 and the Rusty group's at y 182.
  - The relaunch passed the scratch path, which Zed takes as an open request instead of restoring
    the session, so no group came back. `open_path ""` before `launch_marley`, as #601's scenario
    does, fixed it. Neither was a fault in the change.
- **Final run (`shots-699c`): every shot shows its criterion.**
  - **699-00-home:** the Rusty button opened the home page as the group's one tab.
  - **699-01-plus-menu (REQ-001):** the + menu lists Home, Brain, Today's Note, Graph, Tasks,
    Decisions, Memory, Skills and Secrets with their icons; then Open Page… (with its Ctrl-Alt-U),
    Capture to Today…, Capture to Inbox… and Capture a URL…; then Zed's New File, Open File,
    Search Project, Search Symbols, New Terminal and New Center Terminal.
  - **699-02-from-menu (REQ-002):** Home, four Downs and Return chose Tasks: the Tasks tab (lists
    Home and Work) beside the home page in the Rusty group; the rail lists Rusty and Tasks under the
    group.
  - **699-03-closed-all (REQ-003):** after `pane: close all items` the Tasks tab is gone and the
    group shows the home page as its only tab, not Zed's Welcome page.
  - **699-05-project-plus (REQ-005):** the project's + lists Zed's six entries only.
  - **699-04a-restored:** after the quit and relaunch, the rail lists the Rusty group's header with
    no row under it: the group came back empty, as Chad found it.
  - **699-04-shown-empty (REQ-004):** the group's header clicked: the home page, the group's only
    tab.
  - **REQ-006:** the gate (Phase 2, and again at Complete).
  - The run reports Chad's Hyprland untouched: one Marley window before and after, no rule added.
- **Seen, not in scope (pre-existing):** the home page's Recent Pages card lists page names from
  the run's copy of the user's profile (Marley's own store of recent slugs), not the scratch
  vault's page. No Rusty data is read or written there (R-D8 holds), and shots stay in the
  scratchpad, but the card shows the user's recent page names in a test run.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md` (Added: The Rusty group's + menu and its home page);
  - `docs/marley_architecture/marley_workbench.md`, "The Rusty group's + menu and its never-empty
    rule";
  - the slice status on R4 in `docs/marley/rusty-in-marley.md`. R9 there is marked moot: a
    rusty-v3 session reported that Chad retired Rusty's Qt app today (Rusty TICKET-053);
  - the guide's "The Rusty group and its home page" came with Phase 2;
  - the `pane.rs` row in `docs/marley/zed-touchpoints.md`, checked against what shipped: the struct,
    the function and the builder's first call.
- **Knowledge appended:** AD-claude-699-the-rusty-group-is-never-empty-and-its-plus-lists-rustys-screens-001.
  No product bug was found in Code or Test. The scenario's first relaunch missed the session
  restore. That trap is already recorded (lessons.md, the `open_path ""` lesson near line 4981),
  and recall did not search for it. This is the second time it has caught a scenario.
- **Brain:** this repository's sessions have no `rusty` MCP server, so no brain loop ran; the
  decision is in the ledger.
- **Ticket:** closed (`tickets/closed/`); its BACKLOG row left at promotion.
- **Gate at commit:** `699-gate-2.log`, GATE GREEN [diff], 17 passed (the scenario added since `699-gate-1.log`).
