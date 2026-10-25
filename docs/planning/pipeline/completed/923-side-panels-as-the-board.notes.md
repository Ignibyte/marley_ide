# The Inspector and Files as the SidePanels board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-923-side-panels-as-the-board.md
- **Pipeline spec:** 923-side-panels-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this". This
  is the design review's build order, line 10 (the side panels).
- **Classification / tier:** feature, M. Marley crate (`inspector.rs`, `marley_workbench.rs`) and
  two Zed-crate hunks (`workspace/src/pane.rs`, `project_panel/src/project_panel.rs`). The
  review's full list was M–L, so it is split (D1): the Changes panel's layout with Ask an agent,
  Outline's header and kind labels, and the breadcrumb go to a follow-up ticket.
- **Recall (§18.3):**
  - AD-claude-837: the Inspector is a center item narrowed to 320 px. Its count and its restore
    were left out.
  - AD-claude-868: Files is a left-dock pop-out set by `apply_defaults` per layout.
  - PR-claude-747: an item hosting a view answers from values it kept.
  - PR-claude-773: re-tell both panels a dock call touched. The button's open goes through
    `inspector::show`, which already does this.
  - PR-claude-793: before putting a value in Zed's defaults, find Zed's writers for the key.
    Here: `git_status_indicator` and `default_width`. The settings UI writes the user's file,
    which should win.
  - PR-claude-758: a defaults write re-enters every `SettingsStore` observer. The new defaults go
    in the same `update_default_settings` call #868's do.
  - L-claude-660, L-claude-664: `ToggleButtonGroup`'s traps (accent fill, takes its parent's
    width). The reason D2 draws the track itself.
  - F-claude-893: a toolbar item must not read its workspace in `set_active_pane_item`. Same
    spirit here: the Inspector keeps its project from when it is made.
  - F-claude-865: a hook a Zed view calls during its render never reads that view. The tab-bar
    hook runs inside the pane's render, so it takes the pane's workspace as an argument and never
    reads the pane.
  - F-claude-898: a toggle button reads the dock's or setting's state, not the focus. The
    Inspector button reads `marley.side_inspector`.
- **Discovery:**
  - `crates/marley_workbench/src/inspector.rs`: `Inspector` (fields, `new` :122, `render`'s
    switcher :247). `show` makes it with `cx.new(|_| …)`, which gets a context for the
    subscription.
  - `crates/marley_workbench/src/marley_workbench.rs`: `toggle_inspector` (:1438), `LayoutState`
    (:1062), `apply_defaults` (:1570), the `init` that sets the globals.
  - `crates/workspace/src/pane.rs`: `default_render_tab_bar_buttons` (:4472). The new global goes
    beside `MarleyNewItemMenu` (:4420).
  - `crates/project_panel/src/project_panel.rs`: `ProjectPanel::new` (:709), the
    `update_visible_entries` task (:4605), `render` (the list), `git_status_indicator` (:8393).
  - The left dock's width: `default.json:880` (`resize_all_panels_in_dock: ["left"]`),
    `dock.rs:395` (`dock_panel_size`), `:1174`.
- **Decisions:** D1–D8 in the spec.

### Design
- **Switcher** (`inspector.rs`, Marley crate): a `div` track with rounded corners and a fill a
  step above the panel's. In it, a segment per `Shown`, each `flex_1`, rounded, the chosen one
  filled with `element_selected` and its label in the default text colour, the others muted.
  Each segment's click keeps `show_index`.
- **Count** (`inspector.rs`): the Inspector gains `project: WeakEntity<Project>` and a
  `_subscription`. `show` builds it with `cx.new(|cx| …)` and subscribes to the project's
  `GitStore` (`RepositoryUpdated`, `ActiveRepositoryChanged`) to `cx.notify()`. The label is
  `format!("Changes {count}")` when the active repository's `status().count()` is above zero.
  The label becomes a `SharedString`, not `&'static str`.
- **Tab-bar hook** (`pane.rs`, Zed crate): `pub struct MarleyTabBarEnd(pub fn(&WeakEntity<Workspace>,
  &mut Window, &mut App) -> Option<AnyElement>)` and `impl Global`. `default_render_tab_bar_buttons`
  asks it after its focus check and puts the element first in `right_children`.
- **Button** (`inspector.rs`): `tab_bar_button(workspace, window, cx)`, `None` outside the
  Marley layout. A `Button::new("marley-inspector-button", "Inspector")` with a panel icon,
  `toggle_state` from `MarleySettings::side_panels`. Its click turns the setting with
  `toggle_inspector`. When the setting was off, it also updates the workspace from a deferred
  call to run `inspector::show` on the Git panel (`panel_tabs::workspace_panel(…, "GitPanel")`).
  `marley_workbench::init` sets the global.
- **Filter** (`project_panel.rs`, Zed crate): `pub struct MarleyFilesFilter;` and `impl Global`.
  `ProjectPanel` gains `marley_filter: Option<Entity<Editor>>`, made in `new` when the global is
  set, with the placeholder "Find a file" and a subscription that reruns
  `update_visible_entries` on edits. While its text is non-empty, the visible-entries task walks
  each worktree snapshot's files (the panel's own include rules), keeps the paths that contain
  the text ignoring case, and adds their ancestors. `expanded_dir_ids` is not touched. `render`
  draws the field, with a magnifier, above the list. Escape in it clears the text.
- **Defaults** (`marley_workbench.rs`): `LayoutState` gains `zed_project_panel_indicator` and
  `zed_project_panel_width`, read from the raw defaults. `apply_defaults` sets them to
  `Some(true)` and `Some(320)` for the Marley layout and back for Zed's.
- **Width**: Phase 2 checks first whether the left dock's shared or stored size beat
  `default_width` in the review's tour. If it did, the Marley layout's first open of the left
  dock sets the project panel's size once, the way `inspector::narrow` does after its first
  frame. A drag afterwards is the user's.
- **File manifest:** `crates/marley_workbench/src/inspector.rs` (Marley crate);
  `crates/marley_workbench/src/marley_workbench.rs` (Marley crate);
  `crates/workspace/src/pane.rs` (Zed crate); `crates/project_panel/src/project_panel.rs` (Zed
  crate); `script/e2e/923-side-panels-as-the-board.sh` (Marley-owned).
- **Ledger rows** in `docs/marley/zed-touchpoints.md`: extend the `crates/workspace/src/pane.rs`
  row with `MarleyTabBarEnd` and its ask in `default_render_tab_bar_buttons`; extend the
  `crates/project_panel/src/project_panel.rs` row with `MarleyFilesFilter`, the field, and the
  filtered visible entries.

### Visual check plan
- REQ-001: `main.rs` open, its pane focused: `923-01-button` (unlit). After the click,
  `923-02-inspector` (lit). Marley Day: `923-09-day`.
- REQ-002: the Inspector open: `923-02-inspector` and `923-09-day` show the track, two equal
  segments, Changes filled.
- REQ-003: two files changed before launch: `923-02-inspector` reads "Changes 2". The scenario
  then appends a line to `src/board.rs` and settles: `923-03-count` reads "Changes 3".
- REQ-004: the lit button clicked: `923-04-off`, no Inspector, the button unlit.
- REQ-005, REQ-006: the rail's Files button: `923-05-files`. The shot's tree edge is measured
  against 320 px. The field heads the tree. "M" sits by `main.rs` and `README.md`.
- REQ-007: "board" typed in the field: `923-06-filtered` lists `src` › `board.rs` only. Escape:
  `923-07-cleared` shows the full tree with the folders as they were.
- REQ-008: `marley.layout` set to Zed's: `923-08-zed`, the tab bar's +, split and zoom alone.
  Files in Zed's right dock has no field.
- REQ-009: the review of the diff (comments and touchpoint rows).

### Risks
- The width may come from a stored or shared dock size, not `default_width`. Then a layout
  default alone changes nothing, and Phase 2 needs the one-time size set the Design names. The
  scenario keeps the copied profile's database so it sees the real case.
- Walking every file of a large worktree on each keystroke could stall the panel. The walk runs
  in the panel's existing background visible-entries task, and a new keystroke replaces the
  last task.
- Turning the setting on writes the settings file, which lands a moment later. The button's open
  must not wait for it, and #837's observer must not close the Inspector it just opened. Check
  this in the first run.
- The tab-bar hook runs inside the pane's render (F-claude-865). It reads settings and the
  workspace's weak handle only, never the pane.
- `resize_all_panels_in_dock: ["left"]` makes every left-dock panel share one size. Setting the
  project panel's size may resize another left-dock panel in a user's profile. Read the dock's
  panels in the run.

## Phase 2 — Code
- **Built:** `pane.rs` `MarleyTabBarButtons` asked in `default_render_tab_bar_buttons`;
  `project_panel.rs` `MarleyFilesFilter`, `marley_filter`/`marley_filter_text`, the keep set and
  its two checks in `update_visible_entries`, `marley_filter_field` with Escape; `inspector.rs`
  `init`, `tab_bar_button`, the project and its subscription, `follow_git`, `changes`, the
  segmented switcher; `marley_workbench.rs` the two project-panel defaults in `LayoutState` and
  `apply_defaults`, `toggle_inspector` made `pub(crate)`, the two globals set in `init`;
  `project_panel` added to `marley_workbench`'s dependencies.
- **Deviations:** `MarleyTabBarButtons` is a new hook beside #917's `MarleyTabBarEnd`, because
  the button shows on the focused pane only (D4) and #917's chips show on every pane. The
  first-open width: with `default_width` 320 the run opened Files at 320 px on the copied
  profile, so no stored size had to be cleared.
- **Review:** the Inspector reads its own project, never the workspace, in `render` (D3); the
  filter leaves `expanded_dir_ids` alone (D6); without the globals both Zed panels are Zed's.
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- **Scenario:** `script/e2e/923-side-panels-as-the-board.sh`, sway, 9 shots, each read.
  - `923-01-button`: "Inspector" before + on the focused editor pane, unlit. REQ-001.
  - `923-02-inspector`: the Inspector at the right, its button lit, the track "Changes 2 |
    Outline" with Changes filled. REQ-001, REQ-002, REQ-003.
  - `923-03-count`: after board.rs changed on disk, "Changes 3". REQ-003.
  - `923-04-off`: the Inspector gone, the button unlit. REQ-004.
  - `923-05-files`: Files 320 px wide (x 260 to 580), "Find a file", M on board.rs, main.rs and
    README.md. REQ-005, REQ-006.
  - `923-06-filtered`: "board": repo, src, board.rs only; `923-07-cleared`: the whole tree. REQ-007.
  - `923-08-zed`: Zed's layout: +, split, zoom; Files at the right with no field and no letters.
    REQ-008. `923-09-day`: Marley Day, the Inspector and Files.
- **Fixes in Test:** coordinates only (the Inspector pane takes the focus, so it is clicked before
  its button; Files opens from the palette).

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md`, the look plan's W3.1 row, ledger rows for `pane.rs` and
  `project_panel.rs`.
- Knowledge: nothing new.
- Brain: decision recorded.
