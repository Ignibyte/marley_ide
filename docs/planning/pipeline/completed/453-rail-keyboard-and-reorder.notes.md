# Keyboard navigation and project reorder in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-453-rail-keyboard-and-reorder.md
- **Pipeline spec:** 453-rail-keyboard-and-reorder.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint and split
  · [x] prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-453 (W6d), split from #442; autonomous per Chad's goal "lets continue
  working on the remaining tickets".
- **Classification / tier:** feature, medium; `marley_rail` and `marley_workbench`.
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Split at promotion.** The filter needs an editor, fuzzy matching and its own key: it is
  TICKET-457, queued next. The ticket file is renamed for the narrower scope.
- **Recall (§18.3).**
  - `PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001`: the cursor
    goes into the pure `Focus` and `selection` reads it, so the rail still has one highlight.
  - Zed's Threads Sidebar adds the `menu` context (`sidebar.rs:3287-3290`). Zed's defaults bind
    up, down, Enter, Home and End with no context, and left and right only in `menu`.
  - Brain: consultation `1e5870a8e0fa477d9a3bb2a6db42cff1`, nothing on this seam.

### Design
- **`marley_rail`:**
  - `Focus::cursor: Option<Selection>`.
  - `selection`: a cursor whose row is shown wins; otherwise the rule as before.
  - `step(snapshot, forward) -> Selection` over `rail_rows`, from `selection(snapshot)`, clamped
    at the ends; `None` when there are no rows. `first_row`, `last_row`.
  - `parent(snapshot, selection) -> Selection`: a terminal's or thread's project header; a
    project, or `None`, is its own parent.
- **`rail.rs`:**
  - A `cursor: Option<Selection>` field; `build_snapshot` puts it in `Focus::cursor` while the
    rail's focus handle contains the focus. A focus-out subscription on the rail's handle drops
    it and refreshes.
  - `.key_context("MarleyRail menu")` on the root, and `on_action` handlers:
    `select_next`/`select_previous`/`select_first`/`select_last` set the cursor and refresh;
    `select_parent` folds an open project under the cursor or moves the cursor to the parent;
    `select_child` unfolds a folded project; `confirm` opens the cursor's row through
    `activate_workspace`, `activate_terminal` or `open_thread`.
  - `render_project_row`: the row wrapped in `right_click_menu` with Move Project Up and Move
    Project Down (disabled on the first and last), calling
    `MultiWorkspace::move_project_group_up/down` with the group's key.
- **File manifest.** Marley only: `crates/marley_rail/src/marley_rail.rs` (and its tests),
  `crates/marley_workbench/src/rail.rs`, `rail_tests.rs`. Docs at Complete: `CHANGELOG.md`,
  `docs/marley_architecture/marley_rail.md`, `docs/marley_architecture/marley_workbench.md`,
  `docs/marley/workbench-shell.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | unit: `step` forward and back over projects, terminals and threads, stopping at the ends; `first_row`, `last_row`; a cursor on a hidden row falls back. Driven: `menu::SelectNext`/`SelectPrevious`/`SelectFirst`/`SelectLast` with the rail focused move `selected` |
| 002 | driven: `menu::Confirm` on a terminal row makes it the active item with focus in the terminal |
| 003 | unit: `parent`. Driven: `SelectParent` on an open header folds it, `SelectChild` unfolds it, `SelectParent` on a terminal row moves to its header |
| 004 | driven: focus moved into a terminal: `selected` is the window's row again |
| 005 | driven: right-click a header: Move Project Down moves it, Up on the first is disabled |
| 006 | driven (Linux): Zed's default keymap bound; `down`, `up`, `enter` from the focused rail |
| 007 | `script/gates.sh --diff` |

### Risks
- **The `menu` context's other bindings.** Any binding Zed puts in `menu` reaches the rail; the
  rail handles only the actions above, and the rest fall through, as in Zed's sidebar.
- **Focus-out timing.** The cursor is dropped on focus-out; a click on a row moves focus as the
  row's handler runs, so the highlight lands on the clicked row either way.

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] recall · [x] the cursor in `marley_rail` · [x] the keys
  and handlers in `rail.rs` · [x] the header's reorder menu · [x] the manifest · [x] fmt ·
  [x] clippy · [x] review.
- **Built.**
  - `marley_rail`: `Focus::cursor`; `selection` prefers a shown cursor; `shown`, `step`,
    `first_row`, `last_row`, `parent`.
  - `rail.rs`: a `cursor` field; a focus-out subscription that drops it; `refresh` copies it
    into the snapshot while the rail holds focus. The key context `MarleyRail menu` with
    handlers for `menu::SelectNext`, `SelectPrevious`, `SelectFirst`, `SelectLast`,
    `SelectParent`, `SelectChild` and `Confirm`; `move_project`; the header's right-click menu
    with Move Project Up and Down, disabled at the ends.
  - `Cargo.toml`: `menu` moves from the dev-dependencies to the dependencies.
- **Deviations.**
  - The plan said the `menu` key context brings every key. Zed binds up, down, Enter, Home and
    End with no context (`default-linux.json:3-22`); only left and right need `menu`
    (`:51-56`). The spec and the recall are corrected. A negative check: with the key context
    `MarleyRail` alone, the keystroke test's up, down and Enter still passed, so the test now
    presses left and right too, and fails without `menu`.
  - `refresh` asks `contains_focused`, not `is_focused`, to match the focus-out subscription,
    which fires only when focus leaves the rail and everything in it. #457's filter will sit
    inside the rail.
- **Review.**
  - clippy: two `option_if_let_else` in `confirm` (each lookup now clones what it needs, then
    `map_or`), and a `semicolon_if_nothing_returned` in a test helper.
  - Coverage found `confirm`'s project, thread and nothing-highlighted arms untested. Three
    driven tests: `enter_shows_the_highlighted_project`, `enter_opens_the_highlighted_thread`,
    `enter_with_no_row_highlighted_does_nothing`.
  - REQ-004's check in `enter_opens_the_highlighted_terminal` could not fail: the cursor's row
    and the window's row were the same. `the_keyboards_row_goes_when_focus_leaves_the_rail`
    leaves from a different row; with the focus-out clear removed it fails
    (`left: Terminal(…)`, `right: Project(0)`).
  - Found while writing the nothing-highlighted test: when the displayed project loses its last
    folder, its row stays until another change rebuilds the rail. Zed's
    `handle_project_group_key_change` returns early on an empty key without a notify, and
    `WorktreeRemoved` emits no `workspace::Event`. Out of scope: TICKET-458, queued before #448.
  - Re-entrancy: the focus-out listener refreshes outside any `MultiWorkspace` update (the Enter
    tests drive activate, the focus move and the refresh). The reorder handler updates the rail
    through a weak handle from the context menu's update, as #452's row menu does.
  - Provenance: no Warp source. Zed's sidebar was read for its key model and the `menu` actions;
    the cursor functions are written from the contract. No Zed crate is touched.
- **So far:** `cargo clippy -p marley_workbench -p marley_rail --all-targets --all-features --
  -D warnings` clean; `cargo llvm-cov nextest -p marley_workbench -p marley_rail` 114 of 114
  passed, 100% of lines and functions, before the focus-out test was added.

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-006 tests · [x] the crate suites ·
  [x] the live drive (not run; why below) · [x] the gate.
- **Tests:** four unit tests in `crates/marley_rail/src/marley_rail.rs`, nine driven tests in
  `crates/marley_workbench/src/rail_tests.rs`.

  | REQ | Tests |
  |---|---|
  | 001 | `the_keyboards_row_is_selected_while_it_is_shown`, `step_walks_the_shown_rows_and_stays_at_the_ends`, `step_starts_at_an_end_when_nothing_is_selected`; driven `the_arrow_keys_walk_the_rails_rows` |
  | 002 | `enter_opens_the_highlighted_terminal`, `enter_shows_the_highlighted_project`, `threads::enter_opens_the_highlighted_thread`, `enter_with_no_row_highlighted_does_nothing` |
  | 003 | `a_rows_parent_is_its_projects_header`; driven `left_and_right_fold_unfold_and_climb` |
  | 004 | `the_keyboards_row_goes_when_focus_leaves_the_rail` |
  | 005 | `a_project_headers_menu_moves_it_up_and_down` |
  | 006 | `zeds_default_keys_walk_and_open_the_rail` (Linux): Zed's default keymap bound, `ctrl-alt-;` from a focused terminal reaches the rail (asserted unfocused before), then the arrows, left, right and Enter |
  | 007 | the gate |
- **Negative checks:** the key context without `menu` fails `zeds_default_keys_walk_and_open_the_rail`;
  the focus-out handler without its clear fails `the_keyboards_row_goes_when_focus_leaves_the_rail`
  (Phase 2).
- **Run:** `cargo nextest run -p marley_workbench -p marley_rail`: 115 tests run, 115 passed.
- **Gate:** `script/gates.sh --diff`, run again after the keystroke test gained `ctrl-alt-;`: 19
  passed, 0 failed, `GATE GREEN [diff]`. gate:3 ran 388 tests over the scope; gate:4 ran 115
  with 100% of lines and functions in `marley_workbench` and `marley_rail`. The receipt matches
  the tree (`3687d5ff…`).
- **Live drive: not run.** Chad is at the desk: his windows are on ws3, which the headless
  output borrows (L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001), and
  the session is not idle. The drive needs keys and clicks. It is owed to the next headless
  capture, with those of W3 to W6.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket ·
  [x] archive · [x] commit.
- **Documented:** `CHANGELOG.md` (Added: the rail from the keyboard, and project reorder);
  `docs/marley_architecture/marley_rail.md` (the cursor, `step`, `first_row`, `last_row`,
  `parent`, twenty tests); `docs/marley_architecture/marley_workbench.md` (keys and reorder,
  the test counts, the known limits and the owed drive); `docs/marley/workbench-shell.md` (W6d
  shipped, W6h added to the table). No Zed path changed, so no `zed-touchpoints.md` row.
- **Knowledge:** `AD-claude-453-the-rails-keys-are-zeds-list-actions-001`,
  `L-claude-453-a-key-context-test-must-press-a-key-only-that-context-binds-001`,
  `F-claude-453-the-rail-missed-a-projects-last-folder-going-001` (open: TICKET-458). Brain:
  consultation `1e5870a8e0fa477d9a3bb2a6db42cff1` closed as
  `decisions/the-rails-keys-are-zeds-list-actions`, follow-up by 2026-10-07.
- **Tickets:** #453 closed; #457 and #458 point at the completed pair; no BACKLOG row is stale.
