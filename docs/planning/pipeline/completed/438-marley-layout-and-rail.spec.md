---
pipeline_id: 48974663-8cf1-4cb3-89cb-5ab08c6dd49d
ticket: docs/planning/tickets/closed/TICKET-438-marley-layout-and-rail.md
status: Phase 5 — Complete PASS
title: The Marley layout switch and the first rail (projects, center terminals, New Terminal)
type: feature
slice: workbench shell W2
references: [docs/marley/workbench-shell.md, docs/planning/design-notes/workbench-shell-shelf.md, docs/planning/design-notes/session-tabs-vs-sidebar.md, docs/planning/design-notes/simple-rail-shelf.md]
---

## Title
Give the fork a Marley layout beside Zed's: one setting, `marley.layout`, switches every
window between Zed's Threads Sidebar and the Marley rail without a restart. The first rail
lists each project with its center terminals under it, opens a terminal at the project root
from a visible `+`, and highlights exactly one row.

## Scope
### In
- **The setting (Zed touchpoints):** `crates/settings_content/src/marley.rs` (new) with
  `MarleySettingsContent { layout: Option<MarleyLayout> }` and `enum MarleyLayout { Zed,
  Marley }`; the `marley` field and module line in `settings_content.rs`; `marley: None` in the
  exhaustive literal in `crates/settings/src/vscode_import.rs:183-245`.
- **The crate:** `crates/marley_workbench` (`MIT OR Apache-2.0`, `[lib] path =
  "src/marley_workbench.rs"`):
  - `MarleySettings` (a `Settings` impl; `layout` defaults to `Zed`);
  - actions `marley::{UseMarleyLayout, UseZedLayout}` that write `marley.layout` to
    the user settings file;
  - `register_sidebar(multi_workspace, window, cx)`, which builds the rail or Zed's
    `sidebar::Sidebar` for the current layout, and a settings observer that swaps every
    window's sidebar when the layout changes;
  - the Marley defaults (`terminal.button: false`, `agent.dock: right`) applied with
    `SettingsStore::update_default_settings` in the Marley layout and restored from the
    embedded `default.json` when leaving it;
  - the rail opened when a window is created in the Marley layout;
  - `Rail`, a `workspace::Sidebar`: a header of the title bar's height with the window
    controls and an Add Project `+` (the recent-projects popover); one row per project group
    with its display name and collapse chevron; one row per center `TerminalView` of
    the group's workspaces with its title, a cwd subtitle and a bell dot; a project `+` menu
    with New Terminal; click to switch; exactly one selected row;
  - a pure, gpui-free row module that turns a window snapshot into rows and the selection.
- **Wiring (Zed touchpoints):** the member and `[workspace.dependencies]` entry in
  `Cargo.toml`; the dependency in `crates/zed/Cargo.toml`; `marley_workbench::init` as the
  first line of `initialize_workspace` and sidebar construction in
  `crates/zed/src/zed.rs:536-546` handed to `marley_workbench::register_sidebar`, both in
  `crates/zed/src/zed.rs`; `"marley"` in `test_action_namespaces`.
- Ledger rows and `Marley:` comments for every touchpoint.

### Out (explicitly deferred)
- The git branch on a project header, and listing project groups that have no open workspace
  (#442, which also adds the git-store refresh the branch needs).
- Zed thread rows and New Agent Thread (#439); agent CLIs (#440).
- Routing tasks, New Terminal and Open in Terminal to the center, the Marley keymap, and a
  terminal on project open (#441).
- Closed-rail memory, rename, close from the rail, keyboard navigation, the filter, the
  switcher, project reorder (#442).
- A Marley entry in the title bar's Panel Layout menu, the AI gate, the status-bar toggle's
  label (deferred in the plan).

## Reference (§20)
- **Warp (the rail):** Warp's left panel beside the center pane group
  (`docs/warp_architecture/subsystems/00-overview.md` §2) holds the session list grouped by
  project, each row a live session, the active one highlighted, with no separate tab strip
  (observed on Chad's own Warp, recorded in
  `docs/planning/design-notes/session-tabs-vs-sidebar.md`). The rail grammar Chad picked is in
  `docs/warp_architecture/observed/beautifului-2026-08-12-notes.md`: quiet rows, small-caps
  headers, one selected row, an accent `+`. Marley reimplements that behavior on Zed's
  primitives; no Warp source is involved.
- **Upstream Zed (the frame):** `crates/workspace` `MultiWorkspace` and its `Sidebar` trait
  (resizing, open state, persistence, project order, the displayed workspace) and
  `crates/terminal_view` center terminals, kept as Zed does them. Zed's own layouts
  (`PanelLayout::AGENT`/`EDITOR`) stay intact in the Zed layout.

### Prior art
- **Behavior maps:** `docs/warp_architecture/subsystems/00-overview.md`; the observed rail
  grammar above; `docs/zed_architecture/subsystems/07-workspace-panes-palette.md`; the
  gpui-era rail (Marley's own MIT code): `rail_rows`, `RailSelection`, `RailDot` in
  `/srv/stacks/marley/crates/marley_app/src/tabs.rs:827-1256`, whose single-selection model
  this ticket keeps.
- **Published material:** Zed's docs on the Threads Sidebar, Terminal Threads and layouts
  (`docs/src/ai/parallel-agents.md`, `docs/src/ai/terminal-threads.md`,
  `docs/src/windows-and-projects.md`).
- **Code we already ship:** `workspace::Sidebar` and `MultiWorkspace::register_sidebar`
  (`multi_workspace.rs:121-160`, `:387`); the forty-line `TestWorkspaceSidebar`
  (`crates/agent_ui/src/test_support.rs:159-225`); `MultiWorkspace::project_groups`,
  `activate`, `find_or_create_workspace`; `SidebarRecentProjects::popover`
  (`crates/recent_projects/src/sidebar_recent_projects.rs:30-95`);
  `platform_title_bar::render_left_window_controls` / `render_right_window_controls`;
  `Project::create_terminal_shell`, `TerminalView::new`, `Workspace::add_item_to_active_pane`,
  `Workspace::items_of_type`, `Workspace::activate_item`; `terminal::Event` (bell, title,
  close); `SettingsStore::update_default_settings`, `settings::update_settings_file`;
  `sidebar::Sidebar::new` for the Zed layout; `ui` list and disclosure components. Zed's
  sidebar crate is read for behavior only: the new crate is `MIT OR Apache-2.0`, so none of
  its GPL code is copied (CONSTITUTION §20).

## UI proof
UI-AFFECTING.
- **Driven tests** (in-crate `#[cfg(test)]`, gpui `VisualTestContext`, the
  `.agents/skills/gpui-test` harness): the layout swap in both directions; defaults applied and
  restored; rows for two projects with display-only terminals in their center panes; a click
  on a terminal row switching the displayed workspace and focusing the item; New Terminal
  adding a row; the bell dot; a closed terminal's row leaving; exactly one selected row across
  those sequences.
- **Live drive:** `cargo run` (the `marley` binary from #437), run "marley: use Marley layout"
  from the command palette, open two projects, open two terminals, screenshot the rail through
  `dev-box-desktop` and read it; click a row in the other project and screenshot again; switch
  back to the Zed layout and screenshot Zed's sidebar.

## Locked-In Decisions
- D1 — The rail implements `workspace::Sidebar` in `crates/marley_workbench`
  (`MIT OR Apache-2.0`), written fresh against public APIs.
- D2 — The switch is `marley.layout` in Zed's settings tree, defaulting to `zed`
  (workbench-shell D0; Chad's reading of "zed can be used as default").
- D3 — `zed.rs` hands sidebar construction to the crate inside Zed's existing deferred
  callback, so restore still finds a registered sidebar.
- D4 — The `sidebar` crate stays linked; the Zed layout builds it with `Sidebar::new`.
- D5 — Terminals are ordinary center `TerminalView` items; the rail builds them from the
  project and keeps the view.
- D6 — Rows come from a pure function over a snapshot; the one selected row comes from one
  selector (the displayed workspace's active terminal, else its project header).
- D7 — `is_threads_list_view_active` returns `false` in this ticket (the rail shows no threads
  yet, so Zed must not suppress thread notifications).
- D8 — Defaults are patched in memory; Zed's own values are read from the store before the
  first patch and put back when the layout returns to `zed`; user values always win.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.layout` is `zed` or unset, each window shall register Zed's Threads Sidebar and Zed's default values shall stand | driven test |
| REQ-002 | WHEN `marley.layout` changes to `marley`, every open window shall register the rail without a restart | driven test flips the setting |
| REQ-003 | WHEN `marley.layout` changes back to `zed`, every window shall register Zed's sidebar and the defaults for `terminal.button` and `agent.dock` shall return to Zed's values | driven test |
| REQ-004 | WHILE the layout is `marley`, the default `terminal.button` shall be `false` and `agent.dock` shall be `right`, and a user value for either shall win | driven test with a user override |
| REQ-005 | WHEN `UseMarleyLayout` or `UseZedLayout` runs, the user settings file shall gain the matching `marley.layout` value | driven test on a fake fs; round-trip on the non-default value |
| REQ-006 | WHEN a window is created while the layout is `marley`, its rail shall be open | driven test |
| REQ-007 | The rail shall show one header row per project group, in the `MultiWorkspace` order, labelled with the group's display name | unit test on the row module; driven test |
| REQ-008 | The rail shall show each center `TerminalView` of a group's workspaces as a row under that group's header, and no row for a collapsed group's terminals | unit + driven tests |
| REQ-009 | WHEN a terminal row is clicked, its workspace shall become the displayed workspace and that terminal shall become the active, focused item | driven test |
| REQ-010 | WHEN New Terminal is chosen from a project's `+`, a terminal whose working directory is the project root shall open in that project's center pane and appear as a row | driven test |
| REQ-011 | The rail shall mark exactly one row selected: the displayed workspace's active terminal if it has one, else that workspace's project header | unit tests on the selector over mixed sequences |
| REQ-012 | WHEN a listed terminal rings its bell, its row shall show an attention dot and `has_notifications` shall be true until that bell clears (a keystroke in the terminal, as Zed's tab indicator does, or activating its row) | driven test |
| REQ-013 | WHEN a terminal item closes, its row shall leave the rail | driven test |
| REQ-014 | The rail's `is_threads_list_view_active` shall return `false` | unit test |
| REQ-015 | WHEN the rail restores a serialized blob it cannot parse (Zed's sidebar's), it shall keep its defaults and raise no error | unit test |
| REQ-016 | Every touchpoint shall have a ledger row and a `Marley:` comment, and the diff gate shall be green | `script/gates.sh --diff`, gate:16 |

## Phase Plan
- **P2 Design** — the crate layout (pure row module vs gpui side), the snapshot type, the
  terminal factory seam that lets tests use display-only terminals, the swap mechanics, the
  settings content derives, the test plan per requirement.
- **P3 Implement** — settings block, crate, wiring, ledger rows.
- **P3.5 Inspect** — critics on correctness (entity re-entrancy on swap and restore, focus),
  upstream discipline (the size of each Zed hunk), provenance (nothing pasted from
  `crates/sidebar`), data integrity (settings round-trip, blob parse).
- **P4 Validate** — driven and unit tests, `script/gates.sh --diff`, the live drive.
- **P5 Complete** — CHANGELOG, the crate note under `docs/marley_architecture/`, ledger,
  close, archive.
