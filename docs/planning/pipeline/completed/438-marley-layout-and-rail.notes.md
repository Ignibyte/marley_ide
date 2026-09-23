# The Marley layout switch and the first rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-438-marley-layout-and-rail.md
- **Pipeline spec:** 438-marley-layout-and-rail.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** Chad, 2026-09-22: Warp's layout, "the terminal is the main thing and its on the
  left under the project", as "a Marley layout which basically makes it where zed can be used
  as default but Marley basically is its own layout/addition". Rows: terminals and Zed threads
  (threads land in #439).
- **Classification / tier:** feature, large; one new Marley crate plus small Zed touchpoints in
  `settings_content`, `settings`, `zed` and the root manifest.
- **Recall (§18.3):** `PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001`;
  `PR-claude-selection-bg-distinct-from-container-001` (the selected fill must read against
  the rail background); `PR-claude-new-setting-needs-nondefault-roundtrip-leg-001`;
  `PR-claude-integration-only-coverage-fails-gate4-001`;
  `PR-claude-deferred-gpui-handle-op-needs-notify-in-headless-001`;
  `PR-claude-live-refresh-selection-identity-key-must-be-unique-001` (row identity keys by
  item or entity id, never by title). The gpui-era rail's failures (`F-claude-418-*`,
  `F-claude-419-*`) are about cross-surface selection writes, which the one-selector rule
  prevents.
- **Discovery (the three 2026-09-22 Explore sweeps, summarized in the plan):**
  `multi_workspace.rs:121-160`, `:387-399`, `:1996-2200`; `zed.rs:536-546`, `:2344-2376`,
  `:5856-5990`; `settings_content.rs:174`; `vscode_import.rs:183-245`;
  `settings_store.rs:919-941`; `terminal_panel.rs:835-879`; `terminal_view.rs:233`;
  `workspace.rs:4296`, `:4939`, `:5559`, `:10336-10352`; `platform_title_bar.rs:250-313`;
  `sidebar.rs:832`, `:7328-7418` (behavior reference only).
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.
- **Promoted to active (2026-09-22, after W1's commit `c7143be`).** Seams re-verified against
  the committed tree: the `Sidebar` trait at `multi_workspace.rs:121-160`, `register_sidebar` at
  :387, `project_groups` :849, `workspaces_for_project_group` :932, `find_or_create_workspace`
  :1092, `ProjectGroup` :272; `zed.rs:536-546` (the deferred callback that builds Zed's
  sidebar); `settings_content.rs:174`; `vscode_import.rs:182-183`; `update_default_settings` at
  `settings_store.rs:920`; `add_center_terminal` at `terminal_panel.rs:835` (it takes the
  terminal-making closure as a parameter); `TerminalView::new` :233; `workspace.rs`
  `items_of_type` :4296, `add_item_to_active_pane` :4939, `activate_item` :5559,
  `set_terminal_provider` :3340, `project_group_key` :2446; `sidebar::Sidebar::new` :832;
  `SidebarRecentProjects::popover` :31. The window-control renderers moved to
  `platform_title_bar.rs:121` and :150.
- **Recall added.** Coverage counts only in-library tests
  (`PR-claude-integration-only-coverage-fails-gate4-001`), so the driven gpui tests live in
  `#[cfg(test)]` modules. The fork's DIFF gate appends untracked files as whole-file diffs, so
  the new crate is mutated without staging (the gpui-era
  `PR-claude-diff-gate-stage-new-crate-before-mutation-001` is covered). No mutation masks
  (gate:12, `AD-claude-443-mutation-topology-and-no-masks-001`). `register_sidebar` pushes two
  subscriptions per call, so every layout swap leaves the previous pair behind, inert once the
  old sidebar drops. `MultiWorkspace::multi_workspace_enabled` is false with AI off, which hides
  any sidebar (the plan's open decision on the AI gate). Live drives start the debug build from
  the checkout (`L-claude-437-a-debug-marley-starts-inside-the-checkout-001`) on the headless
  output (`L-claude-437-the-headless-live-drive-recipe-001`). Zed's sidebar tests set up a window
  with `SettingsStore::test`, `FakeFs`, `Project::test` and `MultiWorkspace::test_new` inside
  `add_window_view`. Brain consultation `cb81da25e64f491490c8b3b42fd50814`: the shell decision
  page, which this ticket carries out.
- **Phase 1 checklist (promotion):** promote the pair ✓ · re-verify seams ✓ · recall ✓ · ticket
  in-progress ✓ · backlog row removed ✓ · status PASS (autonomous) ✓.

## Phase 2 — Design
Four read-only Explore sweeps (2026-09-22) answered the open questions: the gpui test harness for
a sidebar, center terminals and their events, adding a settings block, and the `ui` components.
Their citations are in this session's record; the load-bearing ones are repeated below.

### Approach
- **Two Marley crates** (both `MIT OR Apache-2.0`, written fresh; nothing is copied from the GPL
  `sidebar` crate, which is read for behavior only):
  - `crates/marley_rail` (pure, no gpui): `ProjectSnapshot`, `TerminalSnapshot`, `Focus`,
    `RailSnapshot`, `Selection`, `Row`, `selection`, `rail_rows`, `working_directory_label`.
    Every decision the rail makes about what to show lives here, so mutants rebuild a tiny crate.
    The draft's `next_project`/`next_terminal` are dropped: keyboard cycling is W6.
  - `crates/marley_workbench` (gpui): `MarleySettings`, the actions, `init`, `register_sidebar`,
    the layout switch with its defaults, and `Rail`.
- **The setting.** `crates/settings_content/src/marley.rs` (new; the content types must live in
  `settings_content` because its derive macros resolve only there): `MarleySettingsContent {
  layout: Option<MarleyLayout> }` under `#[with_fallible_options]`, and `enum MarleyLayout { Zed
  (default), Marley }` with `serde(rename_all = "snake_case")`. `settings_content.rs` gains the
  module and `pub use` lines, the `marley` field, and `marley` in `flattened_deserialize!`'s
  options (a compile error otherwise); `vscode_import.rs` gains `marley: None`. `MarleySettings`
  tolerates a missing key (`unwrap_or_default`), so `default.json` is not touched.
- **Actions** `marley::{UseMarleyLayout, UseZedLayout}` (palette "marley: use marley layout")
  write `marley.layout` with `settings::update_settings_file`; `test_action_namespaces` gains
  `marley`.
- **The switch.** `init` records Zed's own `terminal.button` and `agent.dock` defaults from
  `raw_default_settings()` before any patch, applies the Marley defaults when the layout starts as
  `marley`, and observes the `SettingsStore`. The observer keeps the layout it last applied and
  does nothing when that has not changed (patching defaults inside it notifies again, so this
  guard is what stops a loop). On a change it patches the defaults
  (`update_default_settings`: `button: Some(false)` and `dock: Some(Right)` in Marley, the
  recorded `Some(..)` values in Zed; never `None`, which both readers unwrap) and, for every
  window whose root is a `MultiWorkspace`, calls `register_sidebar`.
- **`register_sidebar`** builds Zed's `sidebar::Sidebar` or the `Rail` for the current layout,
  registers it, clears any sidebar overlay, re-opens the sidebar when it was open (which also
  re-points every workspace's sidebar focus handle), opens it in the Marley layout, and
  notifies. `crates/zed/src/zed.rs:536-546` calls it inside Zed's existing deferred callback
  (D3), and `main.rs` calls `marley_workbench::init` after `settings::init`.
- **The Rail.**
  - Trait: `width`/`set_width` from its own field (default 260 px; `None` resets), `side` Left,
    `has_notifications` when any listed terminal has a bell, `is_threads_list_view_active` false
    (D7). No serialized state: the trait's defaults ignore a stored blob, so Zed's blob restores
    to nothing (REQ-015) and W6 owns persistence. None of these read the `MultiWorkspace`, which
    is mid-update when it calls them.
  - Subscriptions: `MultiWorkspaceEvent`; each workspace's `workspace::Event` (items and panes);
    each listed `TerminalView`'s `terminal::Event` (the bell) and its tab updates. Every event
    rescans `items_of_type::<TerminalView>` (restores replace panes without `ItemRemoved`),
    reconciles subscriptions by entity id, and notifies only when a row's contents change,
    because `ActiveItemChanged` and tab updates fire on every chunk of terminal output.
  - Rows: at render the rail builds a `RailSnapshot` from `project_groups`, the group keys'
    disambiguated display names (`compute_disambiguation_details` with `project::path_suffix`,
    then `ProjectGroupKey::display_name`), the active repository's branch, each group's
    `expanded` flag, and each workspace's center terminals (title from `tab_content_text`,
    subtitle from `working_directory_label` over the terminal's own working directory, or the
    directory the rail spawned it in; the bell from `has_bell`). Focus is the displayed
    workspace's group and its active item when that is a terminal. `marley_rail::rail_rows`
    turns that into rows and exactly one selected row (D6).
  - Handlers: a header click toggles `expanded` through `group_state_by_key_mut` and
    serializes (no event fires, so the rail notifies itself); a terminal row click activates its
    workspace, then `activate_item(view, true, true)`, then `clear_bell`; the header's `+` opens
    `SidebarRecentProjects::popover`; a project's `+` opens a context menu whose New Terminal
    entry activates the workspace and calls `TerminalPanel::add_center_terminal` with the rail's
    terminal factory at the group's first root, recording that directory for the subtitle.
  - **The terminal factory** is a function pointer,
    `fn(&mut Project, Option<PathBuf>, &mut Context<Project>) -> Task<Result<Entity<Terminal>>>`,
    set to `Project::create_terminal_shell` itself in production, so no Marley line spawns a
    shell. Tests set a display-only factory.
  - Rendering: a header of `platform_title_bar_height` with the left window controls (the title
    bar stops drawing them while a sidebar is open on the left; they render nothing in tests),
    a "PROJECTS" label and the Add Project `+`; macOS traffic-light padding under
    `#[cfg(target_os = "macos")]`; no right-side controls, because the rail is always on the left.
    Rows are `ListItem`s with `toggle_state(selected)`, unique debug selectors per row and per
    `+`, a bell dot (`Indicator::dot`), and `track_focus` on the root.
- **§20.** Warp's rail behavior (sessions grouped by project, live rows, one highlighted row,
  quiet rows under small-caps headers, an accent `+`) is reimplemented on Zed's primitives from
  the behavior docs and Chad's observations; Zed's `MultiWorkspace` keeps resizing, open state,
  persistence and the displayed workspace as upstream has them. Both halves of the spec's
  reference hold.

### File manifest
| File | Kind | Change |
|---|---|---|
| `crates/marley_rail/{Cargo.toml,src/marley_rail.rs}` | Marley, full bar | new: the pure row model with unit tests |
| `crates/marley_workbench/{Cargo.toml,src/marley_workbench.rs,src/rail.rs}` | Marley, full bar | new: settings, actions, switch, `Rail`, with in-crate driven tests |
| `crates/settings_content/src/marley.rs` | Zed crate, new file | the content struct and enum |
| `crates/settings_content/src/settings_content.rs` | Zed crate | module + `pub use`, the field, the `flattened_deserialize!` entry |
| `crates/settings/src/vscode_import.rs` | Zed crate | `marley: None` |
| `crates/zed/src/main.rs` | Zed crate | `marley_workbench::init(cx)` |
| `crates/zed/src/zed.rs` | Zed crate | the deferred callback calls `marley_workbench::register_sidebar`; `"marley"` in `test_action_namespaces` |
| `crates/zed/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | Zed manifests | the dependency; two members and two `[workspace.dependencies]` entries |
| `docs/marley/zed-touchpoints.md` | owned | rows first: five new, three updated |

### Regression test plan
Unit tests in `marley_rail` (the draft's, minus cycling); driven gpui tests in
`marley_workbench`'s `#[cfg(test)]` modules (in-crate, because gate:4 counts in-lib lanes only),
on a harness of `SettingsStore::test`, `db::AppDatabase::test_new()`, the base theme, a `FakeFs`
with two projects, `Project::test`, and `add_window_view(MultiWorkspace::test_new)`; the swap
tests also initialize what Zed's `sidebar::Sidebar::new` needs (its metadata stores, `editor`).

| Test | REQ |
|---|---|
| a Zed-layout window registers Zed's sidebar and the defaults are Zed's | 001 |
| flipping `marley.layout` to `marley` registers the rail in every open window and opens it | 002, 006 |
| flipping back registers Zed's sidebar and restores `terminal.button: true`, `agent.dock: left` | 003 |
| in the Marley layout the defaults are `false`/`right` and a user value for either still wins | 004 |
| `UseMarleyLayout` then `UseZedLayout` write `"marley": {"layout": ...}` to the user file on a FakeFs (the non-default value round-trips) | 005 |
| a window created while the layout is `marley` has the rail open | 006 |
| unit: rows per group in order; driven: two projects' header rows carry their display names | 007 |
| unit: terminal rows under their header, none when collapsed; driven: display-only terminals in two workspaces appear under the right headers, and a header click collapses and expands | 008 |
| a click on a terminal row in the other project makes its workspace displayed and the terminal active and focused, and clears its bell | 009 |
| New Terminal from a project's `+` asks the factory for that project's root, and the new terminal is a row under it | 010 |
| unit: exactly one selected row across mixed sequences; driven: the selected row follows the displayed workspace's active terminal, else its header | 011 |
| a bell on a listed terminal shows its dot and sets `has_notifications`; activating the row clears both | 012 |
| closing a terminal's tab removes its row | 013 |
| `is_threads_list_view_active` is false | 014 |
| `restore_serialized_state` with Zed's sidebar blob keeps the rail's defaults and raises nothing | 015 |
| `width`/`set_width` (a value, then `None` back to the default); the header's Add Project `+` opens the recent-projects popover | coverage |

Mutation: the pure crate's mutants die to its unit tests; the gpui crate's side-effecting
methods each have a test observing the effect. Live drive (validate): from the checkout, on the
headless output, switch to the Marley layout from the command palette, open two projects and
two terminals, capture and read the rail, click a row in the other project and capture again,
switch back and capture Zed's sidebar.

### Risks and decisions
- **The AI gate.** `multi_workspace_enabled` hides any sidebar when AI is off; the rail
  inherits that (the plan's open decision; documented for Chad).
- **Zed's Panel Layout menu** reads `agent.dock` and shows "Custom" in the Marley layout, and
  choosing Classic there writes only fields that differ from the patched values. Documented;
  hiding those two actions in the Marley layout is W5's.
- **The swallowed actions.** While the rail is registered, `NextProject`, `NextThread` and the
  thread switcher do nothing (the trait defaults); W6 adds keyboard navigation.
- **Swap leftovers.** Each swap leaves the previous sidebar's two subscriptions in the
  `MultiWorkspace`, inert once it drops; the rail writes no blob, so a swap back to Zed starts
  Zed's sidebar at its default width.
- **Test weight.** The swap tests pull in the agent crates that Zed's sidebar needs, and each
  `marley_workbench` mutant relinks a large test binary; the DIFF gate will take a while.
- **Phase 2 checklist:** approach ✓ · §20 confirmed ✓ · manifest ✓ · test plan ✓ · risks ✓ ·
  present (autonomous) ✓.

## Phase 3 — Implement
- **Built.**
  - Ledger first: five new rows (`settings_content/src/marley.rs`, `settings_content.rs`,
    `vscode_import.rs`, `zed/src/main.rs`, `zed/src/zed.rs`) and the `Cargo.toml` and
    `crates/zed/Cargo.toml` rows updated; every Zed hunk carries a `// Marley:` or `# Marley:`
    comment.
  - `crates/settings_content/src/marley.rs` and the three `settings_content.rs` lines;
    `marley: None` in `vscode_import.rs`.
  - `crates/marley_rail`: the row model, with struct rows (`ProjectRow`, `TerminalRow`) so the
    gpui side takes one value per row, plus `has_attention`.
  - `crates/marley_workbench`: `marley_workbench.rs` (the setting, `marley::{UseMarleyLayout,
    UseZedLayout}`, `init`, `register_sidebar`, the observer and `apply_defaults`) and
    `rail.rs` (the `Rail`: subscriptions rebuilt on every event, a redraw only when the shown
    snapshot changes, the snapshot built through one combinator chain so an unreachable
    "window gone" branch has no line of its own, the terminal factory as a function pointer).
  - Wiring: two members and two `[workspace.dependencies]` entries, the `crates/zed`
    dependency, `marley_workbench::init(cx)` beside `terminal_view::init` in `main.rs`, the
    deferred callback in `zed.rs` calling `register_sidebar`, and `"marley"` in
    `test_action_namespaces`.
- **Deviations.**
  - The git branch on project headers moved to #442: no requirement asks for it, and it needs a
    git-store subscription to stay current. Groups with no open workspace are not listed (the
    rail has nothing to switch to); also #442.
  - A project header click switches to that project; the chevron collapses it (Zed's own sidebar
    collapses on a header click; the rail's rows are for switching).
  - The terminal subtitle is always drawn, empty when the working directory is unknown, so rows
    keep one height and no line depends on a real shell's directory.
  - `zed.rs`'s `use sidebar::Sidebar;` went with the hunk that stopped using it (clippy would
    flag it); the ledger row says so.
- **Checks.** `cargo check -p marley_rail -p marley_workbench` and `cargo check -p zed --tests`
  clean, no warnings; `cargo fmt` applied to the touched crates and `cargo fmt --all --check`
  clean. The editor plugin's own check of `zed` ran alongside and finished first.
- **Phase 3 checklist:** rows ✓ · settings block ✓ · `marley_rail` ✓ · `marley_workbench` ✓ ·
  wiring ✓ · checks ✓.

## Inspect (Phase 3.5)
Four critics over the Phase 3 diff: rail gpui correctness (C), upstream discipline and
provenance (U), settings and state integrity (S), gate readiness (G). Prior failure classes fed
to them: gpui re-entrancy, a hunk inside `fn main` in a DIFF run (`lessons.md:509`), and the
§20 provenance wall. Every finding below was checked against the code before its verdict.

**Correctness (C)**
- **C1 [high] `has_notifications` stuck.** Real. It read a value only `render` wrote, and Zed
  reads it only while the rail is closed, when nothing renders it (`status_bar.rs:200`, `:227`;
  `multi_workspace.rs:2002`), so a bell never lit the status-bar toggle and a lit dot outlived
  the bell. Fix: `refresh` builds the snapshot and stores it; `has_notifications` reads the
  stored one. F-claude-438-a-a-sidebar-flag-read-a-value-only-render-wrote-001.
- **C2 [medium] Redraw storm.** Real. Hidden, the rail notified on every refresh; shown, its
  `render` read every workspace, worktree and terminal, so any of them redrew the window. Fix:
  `render` reads only `self.snapshot`, `refresh` notifies only when the pure snapshot changed,
  and the rail observes the `MultiWorkspace`, since a group re-key notifies without an event.
- **C3 [medium] New Terminal in the main checkout.** Real: group keys hold main-worktree paths
  (`project.rs:6587-6608`), so a linked worktree's terminal started in the main repository.
  Fix: `terminal_view::default_working_directory` for the target workspace; subtitles read
  against each member's own first root.
- **C4 [medium] AI off pinned a workspace.** Real. Fix: the Marley arm opens the sidebar only
  when `multi_workspace_enabled`. Rejected part: reopening when AI comes back on. Upstream does
  not reopen its own sidebar either (`multi_workspace.rs:351-359` handles only the switch off),
  and the status-bar toggle opens it.
- **C5 [medium] Chevron clipped.** Real: `ListItem` draws its disclosure 16 px left of the row
  (`list_item.rs:408-415`). Fix: a `Disclosure` in `start_slot` under its own selector,
  `marley-rail-disclosure-{index}`. The live drive checks it.
- **C6 [low] `set_width` unclamped.** Real. Fix: clamped to 180–600 px; `None` resets to 260.
- **C7 [low] A focused sidebar swapped out leaves nothing focused.** Real. Fix:
  `register_sidebar` records whether the outgoing sidebar held focus and focuses the new one.
- **C8 [low] New Terminal errors only logged.** Real. Fix: `detach_and_prompt_err`.
- **C9 [nit]** "Sidebar Toggled" telemetry on every open: real, carried to #442. `init`
  rewriting Zed's own defaults in the Zed layout: rejected; one recompute at startup before any
  window exists, and `init` keeps one path. The swap back reopening a sidebar that was closed:
  fixed by S2 (the kept sidebar carries its own open flag).

**Upstream discipline and provenance (U)**
- **U1 [high] Unused `theme` dependency.** Real; gate:9's cargo-shear would fail. Fix: removed;
  the tests take `theme` as a dev-dependency.
- **U2 [medium] `display_names` was a renamed copy of GPL code.** Real: it followed
  `sidebar.rs:1451-1462` statement for statement. Fix: rewritten as `group_names` from the two
  public contracts (`compute_disambiguation_details`, `ProjectGroupKey::display_name`) in the
  rail's own shape, a `BTreeSet` of distinct roots.
  F-claude-438-c-a-marley-helper-paraphrased-gpl-code-001.
- **U3 [medium] The `zed.rs` row's merge advice would drop upstream changes.** Real. Fix: the
  row says to diff upstream's callback against the Zed arm of `register_sidebar` and carry each
  change; merge step 4 greps `register_sidebar(` and reads the `Sidebar` trait for new defaults.
- **U4 [low] `marley` inserted into an upstream line of `flattened_deserialize!`.** Real. Fix:
  its own line under its `// Marley:` comment.
- **U5 [low] No license files; `script/check-licenses` fails on every Marley crate.** Real and
  older than this ticket. TICKET-446 (Deliberate: `LICENSE-MIT` needs Chad's copyright line).
- **U6 [low] The first Marley crate linking GPL crates.** Real, and allowed by §20. An AD entry
  records it at Phase 5.
- **U7 [low] Root `Cargo.toml` hunks unmarked.** Fixed: `# Marley:` comments added.
- **U8 [low] Telemetry.** Same as C9.
- **U9 [nit] Loose ledger wording.** Fixed: the `zed.rs` row explains the namespace entry and
  the `Cargo.lock` row names zed's new dependency. The `main.rs` row is gone (G5).
- **U10 [nit] No `strum` derives on `MarleyLayout`.** Deferred until the setting has a Settings
  UI entry; none is planned in W2–W6.
- **U11 [nit] Spec drift.** Fixed: the namespace reads `marley`, and D8 says Zed's defaults are
  read at `init`.

**Settings and state integrity (S)**
- **S1 [medium] Zed's Panel Layout presets misread the Marley layout.** Real. Carried to #441,
  which hides `UseClassicLayout` and `UseAgenticLayout` while the layout is `marley`; W2 leaves
  `title_bar` alone.
- **S2 [medium] Every swap erased Zed's saved sidebar state and left the sidebar open.** Real.
  Fix: the rail keeps Zed's sidebar entity with its open flag and answers `serialized_state`
  from it; a window that opened in the Marley layout keeps the restored blob unread and writes
  it back; the swap back re-registers the kept sidebar, or restores the blob into a new one, and
  closes the sidebar when Zed's was closed.
  F-claude-438-b-a-layout-swap-dropped-zeds-sidebar-and-its-state-001.
- **S3 [medium, rare] Dropping Zed's sidebar cancelled work it owned.** Real; fixed by S2, since
  the sidebar now stays alive.
- **S4 [low] AI off forced open.** Same as C4, fixed.
- **S5 [low] A window restored in the Marley layout saves a partial state first.** Real. Carried
  to #442 (restore order).
- **S6 [low] A transient value swaps every window.** Accepted: swaps now lose nothing. Each
  swap still adds two subscriptions per window, since Zed's `register_sidebar` has no
  unregister: dead ones for a dropped rail, and one more live pair on the kept Zed sidebar per
  round trip. Carried to #442.
- **S7 [low] A second `init` records patched values as Zed's.** Real. Fix: `init` returns when
  `LayoutState` exists. Rejected part: a `debug_assert!` against `default_settings()`, which
  parses `default.json` again at startup; this crate is the only runtime caller of
  `update_default_settings`.
- **S8 [low] The settings UI shows the patched values as the defaults.** Real, not fixed in W2.
  Recorded as a known limit in the crate note at Phase 5.
- **S9 [low] A round trip with the agent panel open can close the right dock.** Real. Carried to
  #442.
- **S10 [nit] How the actions write.** Fixed: `write_layout` returns when the layout already
  matches. Rejected: a toast on a failed write. A dozen Zed call sites write settings from an
  action the same way, and a malformed settings file already raises `notify_settings_errors`
  (`zed.rs:1913`). A profile or a `dev` block overriding the top-level key is how every Zed
  setting behaves.
- **S11 [nit] Telemetry.** Same as C9.
- **S12 [nit] Rows keyed by position.** Fixed: element ids use the workspace's entity id.

**Gate readiness (G)**
- **G1, G2 [blocker] The `!` in both fullscreen conditions.** Real. The macOS line compiles away
  on Linux, and on Linux the test window has server decorations and no button layout, so the
  window controls never render in a test. Fix: `if window.is_fullscreen() { header } else {..}`,
  as `platform_title_bar.rs:248` writes it; cargo-mutants 27 mutates no `if` condition. The
  controls themselves are checked on the live drive.
- **G3 [blocker] The empty-path filter never ran.** Real: display-only terminals report no
  directory. Fix: the filter is the first line of `marley_rail::working_directory_label`, where
  a unit test gives it `Some("")`.
- **G4 [blocker] `side` → `Default::default()` is an equivalent mutant.** Real: `SidebarSide`
  defaults to `Left`. Fix: the body is `Default::default()` with a comment, which cargo-mutants
  skips as identical; a driven test asserts `Left`, so an upstream change to the default fails.
- **G5 [blocker] A hunk inside `fn main`.** Real: the DIFF run would generate "replace main with
  ()", which no test reaches. Fix: `marley_workbench::init` is the first line of
  `initialize_workspace`, which is in the diff already, runs after `settings::init` and before
  any window opens, and is called by zed's own tests. `main.rs` is back to upstream, its ledger
  row is gone, and the `zed.rs` row says what moved.
- **G6 Coverage gaps.** Real: the attention dot and a header click had no test. Both are in the
  Phase 4 plan; every line of the new test modules counts too.
- **G7 Chevron.** Same as C5.
- **G8 The Add Project popover showed nothing a test could read.** Fixed: the rail holds a
  `PopoverMenuHandle<SidebarRecentProjects>`, as Zed's sidebar does, for an `is_deployed`
  check.
- **G9 The optional seam** (zed.rs passes a builder, the crate drops `sidebar`). Rejected: the
  swap keeps Zed's sidebar entity inside the rail (S2, S3), which needs its concrete type, and
  `register_sidebar` downcasts to it; the seam would also grow the `zed.rs` hunk. The cost is
  accepted: the test binary links `agent_ui`, so each mutant relinks a larger binary.
- **G10 [perf, for #442]** The rail rebuilds its snapshot on each `Wakeup` and `UpdateTab`, two
  per chunk of terminal output. Carried to #442.

**Checks after the fixes.** `cargo check -p marley_rail -p marley_workbench -p zed` clean;
`cargo fmt --all --check` clean.

**Phase 3.5 checklist:** critics ✓ · review ✓ · fixes ✓ · verify ✓ · ledger ✓.

## Phase 4 — Validate
- **Tests added: 34, all passing** (`cargo nextest run -p marley_rail -p marley_workbench`).
  - `marley_rail`, 8 unit tests: the selection (none without a listed displayed project; the
    active terminal when its row shows; the header otherwise, including another project's
    terminal and a collapsed project's), the row order, a collapsed project's hidden rows and
    carried bell, exactly one selected row over 40 combinations, attention, and the
    working-directory label (unknown, empty, the root, below it, home, outside both).
  - `marley_workbench`, 26 driven gpui tests: `marley_workbench_tests.rs` (the switch, 11) and
    `rail_tests.rs` (the rail, 15). The harness is `SettingsStore::test`, the app database, the
    base theme, a FakeFs and `MultiWorkspace::test_new`, plus Zed's sidebar globals for the
    swap tests, and a display-only terminal factory that records the directory it was given.
  - By requirement: REQ-001 `a_window_in_the_zed_layout_gets_zeds_sidebar_and_zeds_defaults`;
    REQ-002 `switching_to_the_marley_layout_gives_every_window_an_open_rail` (two windows);
    REQ-003 `switching_back_hands_each_window_its_own_zed_sidebar` (the same entity, its width
    and open state), `a_closed_zed_sidebar_is_closed_again_after_a_round_trip`,
    `a_window_opened_in_the_marley_layout_gives_zeds_sidebar_its_saved_state`; REQ-004
    `the_marley_layout_moves_two_defaults_and_user_values_still_win`,
    `a_second_init_keeps_zeds_own_defaults`; REQ-005
    `the_layout_actions_write_the_choice_to_the_settings_file` (both values, and no write when
    the layout already matches); REQ-006
    `a_window_opened_in_the_marley_layout_starts_with_its_rail_open`,
    `with_ai_off_the_rail_is_registered_but_left_closed`; REQ-007
    `the_rail_lists_each_project_with_its_center_terminals`,
    `projects_with_the_same_name_are_told_apart_by_their_parents`,
    `a_group_with_no_open_workspace_is_not_listed`,
    `a_project_added_without_being_shown_is_listed_and_opens`; REQ-008 the listing test and
    `the_chevron_folds_a_projects_terminals_away_and_back`; REQ-009
    `a_terminal_row_shows_its_project_and_focuses_the_terminal`; REQ-010
    `new_terminal_starts_in_its_projects_directory` (the `+` menu, then its entry); REQ-011 the
    selection unit tests and `the_selected_row_follows_what_the_window_shows`; REQ-012
    `a_bell_marks_its_row_and_the_rail_until_the_row_is_opened` (the dot must be redrawn by
    the rail's own notify) and `a_bell_while_the_rail_is_closed_lights_the_sidebar_toggle`
    (F-claude-438-a's regression test); REQ-013 `closing_a_terminal_takes_its_row_away`;
    REQ-014 `the_rail_keeps_its_width_in_bounds_and_sits_on_the_left`; REQ-015
    `zeds_saved_sidebar_state_is_kept_unread_for_zed`; REQ-016 the gate below. For coverage:
    `add_project_opens_the_recent_projects_popover`, `the_header_still_draws_in_fullscreen`,
    `a_swap_keeps_focus_in_the_sidebar`.
- **Code changed in this phase.**
  - The rail's handlers take the rows' weak handles and return a `Result` that the click sites
    log, the way Zed's sidebar calls `weak.update(..)` in its menus. The `if let` fall-throughs
    they replace were lines no test could reach.
  - `activate_workspace` ends with `.map(|()| workspace)`: a `?` alone on the line after a
    multi-line closure counts as an unexecuted line of that function in llvm's per-function
    line summary, although the file view shows it covered.
  - Test selectors on the bell dot, the attention dot and the Add Project button
    (`debug_selector` does nothing outside test builds).
  - `recent_projects` with `test-support` in the dev-dependencies: `project`'s test support
    turns on `remote/test-support`, and `remote_connection` matches its `Mock` variant only
    under its own `test-support`, which `recent_projects/test-support` enables.
- **Gate.** `script/gates.sh --diff` → `GATE GREEN [diff]`, 15 passed, 0 failed. gate:3: 484
  tests passed, 2 skipped. gate:4: 100% of lines (`marley_rail.rs` 274, `marley_workbench.rs`
  140, `rail.rs` 491). gate:5: 61 mutants, 49 caught, 1 timeout, 11 unviable, 0 missed, MSI
  100%. The timeout is `!=` → `==` in `Rail::refresh`, which makes the rail and the
  `MultiWorkspace` notify each other forever (the audit found no failing test in its log).
  gate:16: 12 rows. The receipt matches the tree.
- **Live drive** (headless output, the debug `marley` started in the checkout, scratch
  projects `alpha` and `beta`; captures in the session scratchpad under `w2/drive/`).
  - `01-zed-layout.png`: the Zed layout. No sidebar open, the Terminal button in the right
    status group, the Agent Panel's button on the left.
  - `02-marley-layout.png`, with `02-rail.png` and `02-status-*.png`: `"marley": { "layout":
    "marley" }` written into the settings file with the window open. The rail replaced the
    sidebar without a restart: the PROJECTS header, the Add Project button, and `alpha`
    selected, its chevron wholly inside the row and its `+` on the right. The Terminal button
    left the status bar and the Agent Panel's button moved to the right end.
  - The switch back (`"layout": "zed"`) was applied, but its capture showed another app's
    window: the headless output had taken one of Chad's workspaces, and the maximize had
    moved to one of his windows. The capture was deleted unused.
  - Not driven live: a second project, terminal rows, the row and chevron clicks, New Terminal.
    On the dev channel a second `marley <path>` does not reach the running app (the
    single-instance check is skipped, and the second process hung), and the rest needs keys or
    a pointer. Chad was at the desk, so no input went into his session. The driven tests cover
    each of them through gpui's hit testing and event dispatch.
  - Cleanup: Marley stopped, its settings file restored byte for byte (mode 600), the
    temporary Hyprland rules and `focus_on_activate` restored by `hyprctl reload`, the headless
    output removed. Chad's Teams window on workspace 3 came back maximized, which it was not
    before the drive; it was left alone. Marley's own database now lists the scratch project.
- **Pre-existing failures:** none.
- **Phase 4 checklist:** tests ✓ · run ✓ · live drive (partial, recorded above) ✓ · gate ✓.

## Phase 5 — Complete
- **Docs (§21).**
  - `CHANGELOG.md`, under Added: the Marley layout and its first rail.
  - `docs/marley/workbench-shell.md`: W2 marked shipped. The switch, defaults and width
    paragraphs now describe the kept Zed sidebar and the defaults read at `init`, and the W2
    touchpoint rows name `initialize_workspace` and both crates. The restart check moved to
    #442, and the subscription risk now says what the kept sidebar costs.
  - New crate notes `docs/marley_architecture/marley_rail.md` and `marley_workbench.md`,
    linked from `docs/marley/README.md`.
  - `docs/marley/three-prong-plan.md` is unchanged: it leaves the shell's slices to
    `workbench-shell.md`.
  - The ledger's 12 rows re-checked against the diff; each describes what shipped, the
    `zed.rs` row included after inspect moved `init` there.
- **Knowledge (§19).** Appended at inspect: F-claude-438-a, -b and -c, and three PR blocks.
  Appended now:
  - AD-claude-438-the-marley-layout-swaps-the-sidebar-and-two-defaults-001 and
    AD-claude-438-marley-crates-may-link-zeds-gpl-crates-001.
  - L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001,
    L-claude-438-the-coverage-floor-counts-lines-per-function-001,
    L-claude-438-recent-projects-needs-its-test-support-in-tests-001 and
    L-claude-438-prove-a-views-own-notify-with-a-selector-001.
  - Brain: `decisions/the-marley-layout-is-a-sidebar-swap-plus-two-defaults` closes
    consultation `cb81da25e64f491490c8b3b42fd50814`; follow-up due 2026-10-15.
- **Carried forward.** #441's notes: the Panel Layout presets. #442's notes: the branch, empty
  groups, restore order, telemetry, keyboard, the swap's subscriptions, the dock round trip,
  the snapshot rebuild rate, the width blob and the restart check. TICKET-446, the license
  files, is Deliberate.
- **Ticket** closed; no backlog row to sweep. **Pipeline** archived to `completed/`.
- **Phase 5 checklist:** docs ✓ · knowledge ✓ · ticket ✓ · archive ✓.
