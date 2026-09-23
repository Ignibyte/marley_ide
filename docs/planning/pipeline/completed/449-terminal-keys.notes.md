# Terminal keys in the Marley layout — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-449-terminal-keys.md
- **Pipeline spec:** 449-terminal-keys.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint the pair ·
  [x] prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-449, the W5b slice split from #441; autonomous per Chad's goal "lets
  continue working on the remaining tickets".
- **Classification / tier:** feature, small; `marley_workbench` only.
- **Pre-flight:** no active pipeline, README marker present, gate and tools present. Another
  project's release build (`harness-cli`) held cargo during planning; no cargo was needed.
- **Recall (§18.3).**
  - `PR-claude-unmodified-terminal-chords-yield-to-the-pty-001`: all three keys are modified,
    and no Terminal-context binding of Zed's shadows them.
  - `PR-claude-new-chord-shadowed-by-hardcoded-key-001`: no new chord here; it binds
    TICKET-450.
  - `PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001` (#441): the
    listeners run inside the workspace's update, so the dock check reads the dock and calls
    into no panel (`first_enabled_panel_idx` would call each panel's `enabled`).
  - Brain: consultation `451c33940a794c809999a31fcc092cae`, nothing on this seam (it listed
    other projects' due follow-ups).
- **Minted, not promoted.** #449 had no queued spec. The pair comes from the templates; the
  ticket is renamed for its scope (`TICKET-449-terminal-keys.md`), and BACKLOG and #441's
  archived spec point at the new name.
- **Split at promotion.** The ticket planned the Marley keymap of workbench-shell D7 for these
  keys. Zed's defaults already bind them to actions a capture listener can catch, as #441
  catches New Terminal, and catching the actions also routes the palette, the menus and a
  user's own bindings, which a remap would miss (AD-441). Only the New Agent chord has no Zed
  action behind it, so the keymap, its `zed.rs` line and the chord moved to TICKET-450 (W5c),
  queued first.

### Design
- **Where.** `crates/marley_workbench/src/routing.rs`, beside #441's listeners; its tests in
  `routing_tests.rs`, sharing the #441 harness.
- **Listeners.** The action renderer gains three `capture_action` listeners:
  - `terminal_panel::Toggle` and `terminal_panel::ToggleFocus`: in the Marley layout,
    `cx.stop_propagation()` and `toggle_terminal`;
  - `workspace::ToggleBottomDock`: in the Marley layout and while
    `bottom_dock_would_show_terminal_panel`, the same; otherwise it returns with propagation
    on, and Zed's `toggle_dock` runs.
- **`toggle_terminal(workspace, window, cx)`.**
  - A center terminal has focus when the workspace's active item is a `TerminalView` whose
    focus handle contains the focus. Then the target is the most recently used center item
    that is not a terminal: the latest entry, across the center panes'
    `activation_history()`, whose item is not a `TerminalView`. It is activated and focused
    with `Workspace::activate_item`; with no such item, nothing changes.
  - Otherwise `recent_active_item_by_type::<TerminalView>()` is activated and focused; with
    none, `open_center_terminal` from #441, where New Terminal would start
    (`default_working_directory`).
- **`bottom_dock_would_show_terminal_panel(workspace, cx)`**, a read of the dock only: closed,
  with the Terminal Panel's index (`panel_index_for_type`) equal to `active_panel_index`, or,
  with no active panel, equal to 0.
- **File manifest.** All Marley-owned; no Zed touchpoint:
  - `crates/marley_workbench/src/routing.rs`, `routing_tests.rs`;
  - docs at Complete: `CHANGELOG.md`, `docs/marley_architecture/marley_workbench.md`,
    `docs/marley/workbench-shell.md` (D2's toggle line, D7, the slices).
- **Dependencies.** `settings` (a dev-dependency already) gives the tests
  `KeymapFile::load_asset_allow_partial_failure` and `DEFAULT_KEYMAP_PATH`;
  `workspace::item::test::TestItem` (test-support) stands in for an editor.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: Marley layout, no terminal, a test item focused; Toggle opens one center terminal and focuses it; the panel's dock stays closed |
| 001 | driven: with a terminal and a test item, focus on the item; Toggle focuses the existing terminal and opens no second one |
| 002 | driven: from the focused terminal, Toggle focuses the test item; from a terminal in a workspace with no other item, nothing changes |
| 003 | driven: ToggleFocus does what Toggle does, both ways |
| 004 | driven: with the bottom dock closed and the Terminal Panel its only panel, ToggleBottomDock focuses a center terminal and leaves the dock closed; with the dock opened by `focus_panel`, it closes the dock |
| 005 | driven: Zed's default keymap bound (`load_asset_allow_partial_failure`); `` ctrl-` `` opens and focuses a center terminal, `` ctrl-` `` again returns to the item, `ctrl-~` adds a center terminal, `ctrl-j` toggles; the panel stays empty and closed |
| 006 | driven: Zed layout; Toggle opens and focuses the Terminal Panel; ToggleBottomDock opens the bottom dock |
| 007 | `script/gates.sh --diff` |

Tests dispatch from the focused item or the focused center pane, below the workspace's root
(#441). The keystroke test binds only the default bindings whose actions the test binary
links; the three it presses are among them.

### Risks
- **Other bottom-dock panels.** With a debugger panel active in the bottom dock, `ctrl-j`
  goes to Zed and shows it. That is intended: only a dock that would show the Terminal Panel
  is caught.
- **No active panel.** A fresh project's bottom dock has none, and `toggle_dock` then shows
  the first enabled panel. Docks sort panels by activation priority (`dock.rs:784-795`): the
  Terminal Panel's is 2 and the debugger's 7 (`terminal_panel.rs:1811`,
  `debugger_panel.rs:1580`), so in Zed's own bottom dock the Terminal Panel is first and the
  check catches it. A panel with a lower priority placed before it would take the toggle to
  Zed, since the check calls no panel's `enabled`.
- **A panel opened anyway.** When Vim's `:!` or an agent login has opened the Terminal Panel,
  `` ctrl-` `` toggles center terminals, not the panel; `ctrl-j` closes the open dock.

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] `routing.rs` · [x] `routing_tests.rs` · [x] review ·
  [x] clippy at deny level · [x] coverage.
- **Built** in `crates/marley_workbench/src/routing.rs`:
  - three more capture listeners in the action renderer: `toggle::<Toggle>`,
    `toggle::<ToggleFocus>` (one generic handler) and `toggle_bottom_dock`;
  - `toggle_terminal`, `last_item_besides_terminals` and
    `bottom_dock_would_show_terminal_panel`, as designed;
  - the module doc names the toggles.
- **Tests** in `routing_tests.rs`, eight more, with helpers `dispatch` (from whatever has
  focus), `press` (keystrokes), `add_item` (a `TestItem` in the center), `focused` and
  `center_views`. Beyond the plan:
  - a test that the toggle returns to the terminal used last, not the newest;
  - a test with a `TestPanel` active in the bottom dock, where `ctrl-j` goes to Zed and shows
    it. It proves D3's other half.
- **Deviations from the design.** None in the code. The keystroke test is Linux-only
  (`#[cfg(target_os = "linux")]`): `DEFAULT_KEYMAP_PATH` is the platform's keymap, and
  macOS binds `cmd-j`, Windows `` ctrl-shift-` ``.
- **Review against the criteria.**
  - Re-entrancy: the listeners run inside the workspace's update. `toggle_terminal` reads
    the panes and the terminal view and updates a pane through `Workspace::activate_item`, as
    Zed's own workspace methods do. The dock check reads the `Dock` entity and calls into no
    panel. Every path is driven through real dispatch, where a panic would show
    (PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001).
  - Keys that must still reach Zed: `ctrl-j` over an open dock, or over a dock whose active
    panel is another, goes to Zed (tested); in the Zed layout all three go to Zed (tested).
  - Errors: a terminal that cannot open reaches #441's prompt.
  - Provenance: no Warp source. `last_item_besides_terminals` is written from the public
    contracts (`Pane::activation_history`, `Pane::items`); no GPL function body is carried
    over. No Zed crate changed.
- **Negative checks.**
  - With the three listeners removed, six Marley-layout tests fail. The two upstream-behavior
    tests (the Zed layout; another bottom panel) still pass, as they should.
  - With the dock check forced to always catch, both bottom-dock tests fail.
- **Checks.** `cargo clippy -p marley_workbench --all-targets --all-features -- -D warnings`
  clean. `cargo nextest run -p marley_workbench -E 'test(/routing/)'`: 15 passed. Coverage
  `routing.rs` 204 of 204 lines, 31 of 31 functions.

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 · [x] REQ-002 · [x] REQ-003 · [x] REQ-004 ·
  [x] REQ-005 · [x] REQ-006 · [x] the gate · [x] the live drive (not run; why below).
- **Tests** (`crates/marley_workbench/src/routing_tests.rs`, the #441 harness):

  | REQ | Test |
  |---|---|
  | 001 | `toggle_in_the_marley_layout_opens_a_center_terminal_when_there_is_none` |
  | 001 | `toggle_goes_back_to_the_terminal_used_last`: of two terminals, the one used last |
  | 001, 002, 003 | `toggle_and_toggle_focus_switch_between_the_code_and_its_terminal`: both actions, both ways, one terminal reused |
  | 002 | `toggle_from_a_terminal_with_nothing_else_open_changes_nothing` |
  | 004 | `the_bottom_dock_toggle_in_the_marley_layout_toggles_center_terminals`: both ways, then a dock opened by hand closes as upstream |
  | 004 | `the_bottom_dock_toggle_shows_another_active_bottom_panel_as_upstream`: a `TestPanel` active in the bottom dock |
  | 005 | `zeds_default_keys_reach_the_center_terminals` (Linux): Zed's default keymap bound; `` ctrl-` `` twice, `ctrl-~`, `ctrl-j` |
  | 006 | `in_the_zed_layout_the_toggles_open_the_terminal_panel` |
  | 007 | `script/gates.sh --diff` |
- **Runs.** `cargo nextest run -p marley_workbench`: 58 passed. The fifteen routing tests
  five times in a row: 15 passed each time.
- **Gate.** `script/gates.sh --diff`: 19 passed, 0 failed, `GATE GREEN [diff]`. Coverage
  1494 of 1494 lines in `marley_workbench` (`routing.rs` 204 of 204).
- **Live drive: not run.** It needs keys. At 00:14 the desk looked as it did for #441: the
  idle monitor is off (stay-awake), so nothing shows Chad away, and workspace 3, which the
  headless output borrows, holds his fullscreen Teams window. The keystroke test drives Zed's
  own default bindings in its place; the drive is owed with W3's, W4's and W5's to the next
  headless session while he is away.
- **Pre-existing failures:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket
  · [x] archive · [x] commit.
- **Documented.** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_workbench.md` (the
  toggles under Routing, the tests, the known limits); `docs/marley/workbench-shell.md` (W5b
  shipped, the W5 row, D2's toggle line, D7, the touchpoint row now W5c). No Zed path changed;
  gate:16 passed.
- **Knowledge appended.**
  - `AD-claude-449-terminal-keys-catch-zeds-actions-and-the-keymap-waits-for-new-keys-001`
  - `L-claude-449-driving-keys-and-docks-in-a-gpui-test-001`
  - No `F-` block: Code and Test found no bug. The two compile errors met while writing the
    tests (a boxed action is not an `Action`; a glob gives an underscore import's methods, not
    its name) went into the lesson.
- **Brain.** Consultation `451c33940a794c809999a31fcc092cae` closed with `brain decide`:
  `decisions/marleys-terminal-keys-catch-zeds-actions-the-marley-keymap-waits-for-keys-with-no-zed-action`,
  follow-up by 2026-10-07.
- **Closed and archived.** TICKET-449 in `tickets/closed/`; this pair in `pipeline/completed/`;
  TICKET-450's source link follows it.
