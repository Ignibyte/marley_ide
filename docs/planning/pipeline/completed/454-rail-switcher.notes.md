# A switcher over recent terminals and threads — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-454-rail-switcher.md
- **Pipeline spec:** 454-rail-switcher.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint and split
  · [x] prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-454 (W6e), split from #442; autonomous per Chad's goal "lets continue
  working on the remaining tickets".
- **Classification / tier:** feature, medium; `marley_rail` and `marley_workbench`.
- **Pre-flight:** no active pipeline, README marker present, cargo idle, every tool present.
- **Split at promotion.** Next and Previous Project and Thread have no default key and share
  only the trait with the switcher: TICKET-459, queued after #458.
- **Recall (§18.3).**
  - `PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001`: pick inputs where
    the order with and without recency differ, and break each rule once in Test.
  - `PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001`: the rail's
    switcher does not select on hover, so a resting pointer cannot steal its selection.
  - `PR-claude-a-negative-assert-must-prove-the-machinery-ran-001`: the "not from a center
    terminal" check runs in the same test that first opens the switcher from the rail.
  - The draft criterion was REQ-007 of #442's queued spec (`git show
    906ab7be42:docs/planning/pipeline/queued/442-rail-polish.spec.md`, lines 32-33 and 87). Its
    Warp citation, `session-tabs-vs-sidebar.md`, describes clicks only.
  - Brain: consultation `7cdae1242c134e97862ac5016bc9d563`, nothing on this seam.
- **Discovery.** Two Explore sweeps.
  - Zed's thread switcher and the `Sidebar` hooks. Their facts are in the spec's Reference.
    Two more matter here:
    - `SidebarHandle` runs `toggle_thread_switcher`, `cycle_project` and `cycle_thread` in
      `window.defer`;
    - the overlay has no focus or dismissal of its own.
  - `ctrl-tab` in the Marley layout today:
    - from the rail, a center terminal or an editor it opens Zed's tab switcher, over the
      active pane's items;
    - in the Agent Panel it reaches `Rail::toggle_thread_switcher`, the trait's no-op, so it
      does nothing;
    - it never reaches the PTY.

### Design
- **`marley_rail`:**
  - `Selection` derives `Hash`, to key the rail's recency map.
  - `window_row(snapshot) -> Option<Selection>`: `focus.thread` when the displayed project
    lists it, else `focus.terminal` when it lists that; the walk's fold and filter do not
    apply.
  - `switcher_rows(snapshot, shown_at: impl Fn(&Selection) -> Option<u64>) -> Vec<Row>`. It
    returns every terminal and thread as a `Row::Terminal` or `Row::Thread`, unselected and
    unhighlighted, in the rail's order, then stably sorted by `Reverse(shown_at(row))`. Rows
    never shown keep the rail's order after the rest.
- **`marley_workbench::rail`:**
  - Fields: `shown_at: HashMap<Selection, u64>`, `shown_count: u64`, `window_row`, and the open
    switcher with its subscription and the focus to hand back.
  - `refresh`, while no switcher is open, compares `window_row` with the last one, and on a
    change to a row bumps it in `shown_at`. It prunes `shown_at` to the snapshot's rows.
  - `Sidebar::toggle_thread_switcher`:
    - with a switcher open, it steps it;
    - otherwise it builds the entries from `switcher_rows` (icons and project names from the
      snapshot) and returns below two;
    - it creates `RailSwitcher`, subscribes, calls `set_sidebar_overlay`, keeps
      `window.focused(cx)` and focuses the switcher.
  - On `Confirmed(selection)` it closes the switcher, then opens the row through `open_row`,
    which is Enter's handler moved out of `confirm`. On `Cancelled { restore_focus }` it
    closes the switcher and, after Escape, focuses what had focus.
- **`marley_workbench::switcher`** (new): `RailSwitcher`, `SwitcherEntry` and
  `SwitcherEvent`.
  - Its key context is `ThreadSwitcher`. It handles `ToggleThreadSwitcher` (step),
    `menu::Confirm`, `menu::Cancel` and `on_modifiers_changed`: the modifiers it opened with no
    longer all held confirms.
  - A click on an entry confirms that entry. Focus-out cancels without handing focus back. A
    flag keeps it from emitting twice.
  - The rows are a `ListItem` for a terminal (the agent's or the terminal icon, the title and
    the project name) and a `ThreadItem` for a thread. The selected row scrolls into view.
- **`keymap.json`:** the `MarleyRail && !Picker` block gains `ctrl-tab` and `ctrl-shift-tab` (with
  `select_last`) to `agents_sidebar::ToggleThreadSwitcher`.
- **File manifest.** Marley only:
  - `crates/marley_rail/src/marley_rail.rs`;
  - `crates/marley_workbench/src/switcher.rs` (new) and `marley_workbench.rs` (`pub mod
    switcher`);
  - `rail.rs`, `rail_tests.rs` and `keymap.json`.

  No Zed path. Docs at Complete: `CHANGELOG.md`, `docs/marley_architecture/marley_rail.md`,
  `marley_workbench.md` and `docs/marley/workbench-shell.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: from the rail, a window that showed A and then B opens with A selected, and with `select_last` on the last entry; one terminal only opens nothing; from the Agent Panel (threads harness) it opens with a thread among the entries |
| 002 | driven: two further toggles step down and wrap; a `select_last` toggle steps up |
| 003 | driven: `simulate_modifiers_change(control)`, the toggle, then `none`: the selected terminal is the active item with focus in it and the overlay is gone; Enter and a click do the same; a thread entry opens in its Agent Panel |
| 004 | driven: Escape closes it with focus back where it was; a focus move closes it; the order afterwards is unchanged; opened without a modifier, a modifier change does not confirm |
| 005 | unit: `window_row`; `switcher_rows` order, the unshown rows in the rail's order, no headers, fold and filter ignored. Driven: after showing rows in turn, the entries follow |
| 006 | driven (Linux): Zed's keymap and the Marley keymap bound; `ctrl-tab` with `ctrl` held from the rail opens it and releasing confirms; from a focused center terminal `ctrl-tab` leaves the rail's switcher closed |
| 007 | `script/gates.sh --diff` |

### Risks
- **Focus.** Modifier events reach only the focused path, and the overlay does not focus its
  view, so the rail focuses the switcher itself right after installing it, as Zed's sidebar
  does. The release tests would fail without it.
- **The switcher's own focus change.** Opening moves focus off the Agent Panel, which would
  change the window's row; the rail notes nothing while a switcher is open.
- **The deferred hook.** Tests dispatch, then `run_until_parked`, before reading the switcher.
- **Double events.** A confirm moves focus, which would also cancel; the switcher emits once,
  and the rail drops its subscription on the first event.
- **Decisions for the brain at Complete:** D1 (Zed's split of `ctrl-tab`, the center left to
  Chad) and D3 (the rail's own recency).

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] `marley_rail` order · [x] the rail's recency and hook ·
  [x] the switcher view · [x] `keymap.json` · [x] fmt · [x] clippy · [x] review.
- **Built.**
  - `marley_rail`: `Selection: Hash`; `Focus::terminal_focused`; `window_row`; `SwitcherRow`
    (a terminal or a thread, never a header) with `selection`; `switcher_rows`.
  - `rail.rs`:
    - `shown_at`, `shown_count` and `window_row`, noted in `refresh` by `note_window_row`;
    - `OpenSwitcher`, and `Sidebar::toggle_thread_switcher`, which steps an open switcher or
      opens one on two rows or more, sets the overlay and focuses it;
    - `switcher_ended`;
    - `open_row` (Enter's handler, now shared) and `switcher_entries`;
    - two shared helpers, `thread_item` (the rail's thread rows and the switcher's) and
      `terminal_icon`.
  - `rail_switcher.rs` (the rail's `switcher` module): `RailSwitcher`, `SwitcherEntry`,
    `SwitcherEvent`. Its key context is `ThreadSwitcher`. It steps on `ToggleThreadSwitcher`,
    confirms on Enter, a click or the release of the modifiers it opened with, and cancels on
    Escape or focus-out, emitting once.
  - `keymap.json`: `ctrl-tab` and `ctrl-shift-tab` join `secondary-f` in the
    `MarleyRail && !Picker` block.
- **Deviations.**
  - **Recency follows focus, not display.** The first thread test failed: opening a thread
    shows its project, whose active terminal was the window's row for one rebuild before the
    Agent Panel took focus. That noted a terminal the user never went to. `window_row` now
    needs the terminal to hold focus (`Focus::terminal_focused`), as a panel thread already
    did. The switcher's own focus is then no row, so the planned "note nothing while it is
    open" guard guards nothing. Removed from `refresh`, all ten switcher tests still passed.
    It is gone, and the spec says so.
  - The switcher is the rail's child module rather than a crate-level `switcher`. It shares the
    rail's row helpers, and `rail` is a private module.
  - `switcher_rows` returns `SwitcherRow`, not `Row`: a header can never be an entry, so no
    render or test needs an arm for one.
- **Review.**
  - clippy: `too_long_first_doc_paragraph` (`window_row`), `redundant_clone` in a test,
    `needless_pass_by_ref_mut` (`open_row` takes `&self`), `similar_names` in a test.
  - Coverage found two paths no input reaches. The first was the "no switcher" arm of
    `switcher_ended`: the switcher emits once, so it now takes the switcher unconditionally.
    The second was the switcher's own copy of the terminal icon closure, which is now
    `terminal_icon`, shared with the rail's rows and covered by their agent tests.
  - Re-entrancy: the hook runs deferred, and the switcher's events arrive after its own
    update, so neither the overlay nor `open_row` updates the `MultiWorkspace` inside its
    update. The layout swap already clears the overlay (#451), so a switcher cannot outlive
    the rail.
  - Provenance: the view is written from the behavior in the spec; only Zed's `ThreadSwitcher`
    context name is reused, on purpose. No Zed crate is touched.
- **So far:** clippy clean on both crates; `cargo llvm-cov nextest -p marley_workbench -p
  marley_rail`: 144 of 144 passed, 100% of lines and functions.

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-006 tests · [x] negative checks ·
  [x] the crate suites · [x] the live drive (not run; why below) · [x] the gate.
- **Tests:** two unit tests in `crates/marley_rail/src/marley_rail.rs` and ten driven tests in
  `crates/marley_workbench/src/rail_tests.rs`.

  | REQ | Tests |
  |---|---|
  | 001 | `the_switcher_opens_on_the_row_shown_before_the_current_one` (and `select_last`), `the_switcher_needs_two_rows`, `threads::the_switcher_opens_a_thread_and_opening_it_notes_nothing` (from the Agent Panel) |
  | 002 | `toggling_again_steps_through_the_switcher`, `two_presses_before_the_switcher_opens_step_it_once` |
  | 003 | `letting_go_of_ctrl_opens_the_selection`, `enter_or_a_click_opens_an_entry`, the thread test's confirm |
  | 004 | `escape_or_focus_elsewhere_closes_the_switcher_and_opens_nothing`, the thread test's reopen |
  | 005 | `the_window_row_is_the_focused_panels_thread_else_the_focused_terminal`, `the_switcher_lists_the_recently_shown_first_then_the_rails_order`; driven `the_switcher_lists_every_row_whatever_the_fold_and_the_filter` |
  | 006 | `ctrl_tab_opens_the_switcher_from_the_rail_but_not_from_a_center_terminal` (Linux, Zed's keymap and the Marley keymap) |
  | 007 | the gate |
- **Negative checks**, each file restored and checked by checksum:
  - the `ctrl-tab` binding removed: the keystroke test fails (`left: None`);
  - the view's key context renamed from `ThreadSwitcher`: `ctrl-tab` inside the switcher no
    longer steps it (`left: Some(1)`, `right: Some(2)`);
  - recency by display, not focus: the thread test fails with the transient order of the
    first version;
  - the switcher not focused on open: the release and Escape tests fail, the switcher staying
    open.
  - (Phase 2) the while-open guard removed: all ten tests passed, so it was taken out.
- **Run:** `cargo nextest run -p marley_workbench -p marley_rail`: 144 tests run, 144 passed.
- **Gate:** `script/gates.sh --diff`: 19 passed, 0 failed, `GATE GREEN [diff]`. gate:3 ran 417
  tests; gate:4 ran 144, with 100% of lines and functions in `marley_workbench` and
  `marley_rail`. The receipt matches the tree (`59bab9ca…`).
- **Live drive: not run.** Chad is at the desk: his windows are on ws3, which the headless
  output borrows (L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001), and
  the session is not idle. The drive needs held keys, so it is owed with W3 to W6's. Only the
  drive can show how the overlay looks.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket ·
  [x] archive · [x] commit.
- **Documented:**
  - `CHANGELOG.md`, under Added: the switcher;
  - `docs/marley_architecture/marley_rail.md`: `window_row`, `switcher_rows`,
    `Focus::terminal_focused`, twenty-nine tests;
  - `docs/marley_architecture/marley_workbench.md`: the switcher, the test counts, the known
    limits and the owed drive;
  - `docs/marley/workbench-shell.md`: W6e shipped, D8's hooks pointed at #454 and #459.

  No Zed path changed, so there is no `zed-touchpoints.md` row.
- **Knowledge:**
  - `AD-claude-454-the-rails-switcher-is-zeds-thread-switcher-over-the-rails-rows-001`;
  - `F-claude-454-recency-noted-a-terminal-the-user-never-went-to-001`;
  - `L-claude-454-driving-a-hold-and-release-switcher-in-a-gpui-test-001`.
- **Brain:** consultation `7cdae1242c134e97862ac5016bc9d563` closed as
  `decisions/the-rails-switcher-is-zeds-thread-switcher-over-the-rails-rows`, with a follow-up
  by 2026-10-07.
- **Ticket:** #454 is closed, and #459 points at the completed pair. No BACKLOG row is stale.
