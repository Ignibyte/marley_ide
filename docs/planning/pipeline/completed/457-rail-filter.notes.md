# A filter for the rail — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-457-rail-filter.md
- **Pipeline spec:** 457-rail-filter.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-457 (W6h), split from #453 at its promotion; autonomous per Chad's goal
  "lets continue working on the remaining tickets".
- **Classification / tier:** feature, medium; `marley_rail` and `marley_workbench`.
- **Pre-flight:** no active pipeline, README marker present, cargo idle, every tool present.
- **Minted:** no queued pair; the doc pair is new, the BACKLOG row is gone, and the ticket is
  corrected: Zed's filter is a substring match, not fuzzy, and the behavior maps describe no
  Warp "Search tabs".
- **Recall (§18.3).**
  - `PR-claude-live-refresh-selection-identity-key-must-be-unique-001`: the rows keep their ids
    and keys; the filter changes which rows show, never how they are keyed.
  - `PR-claude-new-chord-shadowed-by-hardcoded-key-001`,
    `L-claude-450-driving-a-picker-and-a-keymap-in-a-marley-test-001` and
    `L-claude-453-a-key-context-test-must-press-a-key-only-that-context-binds-001`: bind Zed's
    default keymap and the Marley keymap in the test, press the real keys, and break the
    binding once to see the test fail.
  - The gpui-era Search Tabs (`completed/search-tabs.*`) filtered before grouping, which kept
    the order stable, and a later row type missed its filter guard
    (`PR-claude-new-render-arm-mirror-sibling-guards-001`): one visibility function here.
  - Brain: consultation `32955477393d4641b9fd07bb567446cb`, nothing on this seam.
- **Discovery.** Two Explore sweeps:
  - Zed's Threads Sidebar filter (`crates/sidebar/src/sidebar.rs`). Its rules, keys and
    matcher are in the spec's Reference. Three more facts matter here:
    - `fuzzy` and `fuzzy_nucleo` both depend on gpui;
    - no default keymap has a `ThreadsSidebar > Editor` context: up, down, Enter and Escape
      reach the sidebar because the single-line editor propagates;
    - the query is not saved and survives focus loss.
  - The `ctrl-f` shadow sweep. The rail's path is `Workspace > MarleyRail menu`, not
    `MarleyRail` alone: the `MultiWorkspace` root carries the workspace's context. No binding
    of `ctrl-f` or `cmd-f` matches there. The rail's key context carries no `os`, so the
    binding uses `secondary-f`. The Marley keymap loads last among the defaults, before the
    user's.

### Design
- **`marley_rail`:**
  - `RailSnapshot::filtering: bool`, and `matched: Option<Vec<usize>>` on `ProjectSnapshot`,
    `TerminalSnapshot` and `ThreadSnapshot`: the byte offsets the filter matched in the name or
    title, `None` when it did not match or no filter is set.
  - `highlight: Vec<usize>` on `ProjectRow`, `TerminalRow` and `ThreadRow`, empty when nothing
    is highlighted.
  - One private walk over the shown rows, in display order, with the fold or the filter
    applied. `shown` and `rail_rows` both map it, so the cursor functions and the rows cannot
    disagree.
  - `first_match(snapshot) -> Selection`: the first shown row whose own text matched, else
    `Selection::None`.
  - A header's attention: a row under it that the walk does not show needs the user. Without a
    filter that is the fold's rule as before.
- **`marley_workbench::rail`:**
  - `filter_editor: Entity<Editor>` (`Editor::single_line`, placeholder "Filter…"), with a
    subscription to its `EditorEvent::BufferEdited`: refresh, then, when it has text, move the
    cursor to `first_match`.
  - `build_snapshot` takes the filter's text. When the text is not empty it sets `filtering`
    and fills each `matched` with `fuzzy_match_positions(text, string)`, for the string the
    row draws.
  - `render_filter`, a row under the header: `IconName::MagnifyingGlass`, the editor, then
    `KeyBinding::for_action_in(&FocusSidebarFilter, &focus_handle, cx)` while it is empty, or
    an `IconButton` with the tooltip "Clear Filter" while it is not.
  - Rows: `HighlightedLabel` for a header's name and a terminal's title when `highlight` is not
    empty; `ThreadItem::highlight_positions` for a thread. The chevron is left out while
    filtering. "No matches" (selector `marley-rail-no-matches`) draws when the filter leaves no
    row.
  - Handlers on the root: `FocusSidebarFilter` focuses the editor; `menu::Cancel` per REQ-006,
    else `cx.propagate()`. `select_parent` and `select_child` do not fold while filtering;
    climbing still works.
- **`keymap.json`:** a block for `MarleyRail && !Picker`, with `use_key_equivalents` as Zed's
  sidebar blocks have it, binding `secondary-f` to `agents_sidebar::FocusSidebarFilter`.
- **`Cargo.toml`:** `editor` moves from the dev-dependencies to the dependencies;
  `zed_actions` is added.
- **File manifest.** Marley only: `crates/marley_rail/src/marley_rail.rs`,
  `crates/marley_workbench/src/rail.rs`, `rail_tests.rs`, `keymap.json` and `Cargo.toml`. No
  Zed path. Docs at Complete: `CHANGELOG.md`, `docs/marley_architecture/marley_rail.md`,
  `docs/marley_architecture/marley_workbench.md`, `docs/marley/workbench-shell.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | unit: a name match lists every row of its project; a child's match lists the header and that row; a project with no match is left out; no filter changes nothing. Driven: typing into the filter narrows `listing` by a project's name, a terminal's title and a thread's title |
| 002 | unit: filtering ignores the fold; each row carries its highlight; a shown header's dot covers its hidden rows. Driven: a folded project's matching terminal shows; no `marley-rail-disclosure-*` is drawn; left and right fold nothing; a title with a multibyte character before the match draws |
| 003 | driven: `marley-rail-no-matches` draws when nothing matches |
| 004 | unit: `first_match`; `step` and `selection` over filtered rows. Driven: typing moves `selected` to the first match; `menu::SelectNext` from the focused field moves it; `menu::Confirm` opens it |
| 005 | driven (Linux): Zed's default keymap and the Marley keymap bound; `ctrl-f` from the rows focuses the field, then `down` and `enter` from the field; `ctrl-f` in the add-project picker leaves the picker focused |
| 006 | driven: `escape` in the field clears it and keeps focus, and again moves focus to the rows; `escape` on the rows with text clears it; with no text, a `menu::Cancel` handler above the rail still runs |
| 007 | driven: clicking `marley-rail-filter-clear` empties the field, and every row returns |
| 008 | `script/gates.sh --diff` |

### Risks
- **Highlight positions.** `HighlightedLabel` debug-panics on a position off a char boundary,
  which a test shows as a panic. The positions are computed on the exact string the row draws,
  and a test with a multibyte character before the match guards it.
- **Falling through the editor.** Up, down and Escape in the field rely on the single-line
  editor propagating and gpui trying the next binding. The driven keystroke test proves it with
  Zed's keymap.
- **The binding's reach.** Deferred popovers dispatch through the rail, so `ctrl-f` in a row's
  context menu also moves focus to the filter and closes the menu. `!Picker` keeps it out of the
  add-project picker.
- **Rebuild cost.** The field's text is read and matched on every rebuild; the matcher is a
  linear scan of each string, as in Zed's sidebar.
- **Decisions for the brain at Complete:** D1 (Zed's matcher), D3 (Zed's action behind the
  Marley keymap) and D4 (the focus key keeps landing on the rows).

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] `marley_rail` rows · [x] `rail.rs` field, matching,
  handlers and rows · [x] `keymap.json` · [x] `Cargo.toml` · [x] fmt · [x] clippy · [x] review.
- **Built.**
  - `marley_rail`: `RailSnapshot::filtering`; `matched` on the three snapshots and `highlight`
    on the three rows; one private walk (`walk`, with `row_shows`) that `selection`, `shown`,
    `first_match` and `rail_rows` all read; `first_match`; a header's attention now covers the
    rows the walk hides (`hidden_rows_need_the_user`), which without a filter is the fold's rule
    as before. `selection` now picks the focused thread, the active terminal or the header in
    that order, each only when the walk shows it under the displayed project.
  - `rail.rs`: `filter_editor` (single line, "Filter…") and its `BufferEdited` subscription
    (`filter_edited`: refresh, then the cursor to `first_match`); `build_snapshot` takes the
    filter's text and fills `matched` through `filter_match`, which calls
    `fuzzy_match_positions` on the string each row draws; `render_filter` under the header
    (icon, field, then the key hint or the clear button); `row_label` (`HighlightedLabel` when
    something matched); `ThreadItem::highlight_positions`; "No matches"; the chevron left out
    while filtering; the `FocusSidebarFilter` and `menu::Cancel` handlers; `fold` does nothing
    while filtering.
  - `keymap.json`: the `MarleyRail && !Picker` block. Its header comment now says the keymap
    also carries Zed's actions in Marley's contexts.
  - `Cargo.toml`: `editor` moves to the dependencies; `zed_actions` is added.
- **Deviations.**
  - A `ProjectRow::foldable` flag was dropped: it tripped `struct_excessive_bools`, and it only
    restated `filtering`, which the render reads from the snapshot to leave the chevron out.
  - The `listing` test helper filed each terminal under its project's index, which assumed
    every project shows. It now files a terminal under the header before it.
- **Review.**
  - clippy: `struct_excessive_bools` (the flag above), `missing_const_for_fn` (`row_shows`),
    `too_long_first_doc_paragraph` (`rail_rows`), `needless_pass_by_ref_mut` (`clear_filter`
    takes `&self`), `clone_on_ref_ptr` in a test.
  - The first run of the new driven tests passed 8 of 9; the failure was the `listing` helper
    above, not the rail.
  - Re-entrancy: the editor's event reaches the rail after the editor's update ends, so
    `refresh` reads the editor safely; `clear_filter` updates the editor from inside the rail's
    handler, which is allowed.
  - Provenance: Zed's matcher is called through `agent_ui`, which the crate already depends on,
    not copied; the walk is written from the rules in the spec. No Zed crate is touched.
- **So far:** clippy clean on both crates; `cargo llvm-cov nextest -p marley_workbench -p
  marley_rail`: 132 of 132 passed, 100% of lines and functions.

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-007 tests · [x] negative checks ·
  [x] the crate suites · [x] the live drive (not run; why below) · [x] the gate.
- **Tests:** seven unit tests in `crates/marley_rail/src/marley_rail.rs` and ten driven tests
  in `crates/marley_workbench/src/rail_tests.rs`.

  | REQ | Tests |
  |---|---|
  | 001 | `a_filter_shows_every_row_of_a_project_whose_name_matched`, `a_filter_shows_a_project_for_the_rows_under_it_that_matched`, `a_filter_that_matches_nothing_shows_and_selects_nothing`; the twenty earlier unit tests pass unchanged on the new walk; driven `typing_in_the_filter_narrows_the_rail` (a title in capitals, a project's name), `threads::the_filter_matches_thread_titles` |
  | 002 | `filtered_rows_carry_the_characters_that_matched`, `a_shown_header_carries_the_attention_of_the_rows_the_filter_hides`; driven `typing_in_the_filter_narrows_the_rail` (a folded project, the chevron), `left_and_right_do_not_fold_while_filtering`, `the_filter_highlights_what_it_matched` (a multibyte title, drawn) |
  | 003 | `a_filter_that_matches_nothing_says_so` |
  | 004 | `first_match_is_the_first_shown_row_that_matched`, `the_selection_and_the_keyboard_keep_to_the_filtered_rows`; driven `the_keyboard_starts_on_the_first_match_and_opens_it` |
  | 005 | `ctrl_f_reaches_the_filter_and_zeds_keys_work_in_it`, `ctrl_f_in_the_add_project_picker_stays_there` (Linux, Zed's keymap and the Marley keymap) |
  | 006 | `escape_clears_the_filter_then_leaves_it` (with a stand-in `menu::Cancel` handler on the workspace), and `escape` in the keystroke test |
  | 007 | `the_clear_button_empties_the_filter` |
  | 008 | the gate |
- **Negative checks**, each file restored and checked by checksum:
  - the key rebound to `secondary-g`: `ctrl_f_reaches_the_filter_and_zeds_keys_work_in_it`
    fails ("ctrl-f focuses the filter");
  - `!Picker` dropped: `ctrl_f_in_the_add_project_picker_stays_there` fails, since `ctrl-f` takes
    focus out of the picker;
  - the fold guard dropped: `left_and_right_do_not_fold_while_filtering` first **passed**. It
    pressed left, then right, and the second key undid the first's fold. It now checks each key
    on its own, and fails without the guard (`left: ("alpha", [])`);
  - the first-match move dropped: the first version of
    `the_keyboard_starts_on_the_first_match_and_opens_it` could not have failed, since its first
    match was also the window's row. It now types a query that hides the displayed project,
    and fails without the move (`left: None`).
  - These two, with #453's two, make
    `PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001`.
- **Run:** `cargo nextest run -p marley_workbench -p marley_rail`: 132 tests run, 132 passed.
- **Gate.** The first `script/gates.sh --diff` was red at gate:18: `typos` read the test query
  `"BUI"` as a misspelling. The query is now `"BUILD"`, which still tests the case. The second
  run: 19 passed, 0 failed, `GATE GREEN [diff]`. gate:3 ran 405 tests; gate:4 ran 132, with 100%
  of lines and functions in `marley_workbench` and `marley_rail`. The receipt matches the tree
  (`71ace4a4…`).
- **Live drive: not run.** Chad is at the desk: his windows are on ws3, which the headless
  output borrows (L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001), and
  the session is not idle. The drive needs keys. It is owed with W3 to W6's, and it is the only
  check on how the highlights look: the tests check the positions and that a multibyte title
  draws, not the styling.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket ·
  [x] archive · [x] commit.
- **Documented:**
  - `CHANGELOG.md`, under Added: a filter for the rail;
  - `docs/marley_architecture/marley_rail.md`: the filter's fields, the one walk,
    `first_match`, the hidden-rows attention, twenty-seven tests;
  - `docs/marley_architecture/marley_workbench.md`: the filter, the test counts, the known
    limits and the owed drive;
  - `docs/marley/workbench-shell.md`: W6h shipped.

  No Zed path changed, so there is no `zed-touchpoints.md` row.
- **Knowledge:**
  - `AD-claude-457-the-rails-filter-is-zeds-sidebar-filter-001`;
  - `L-claude-457-a-single-line-editor-hands-zeds-list-keys-to-its-container-001`;
  - `PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001`.

  No `F-…` block: Code and Test found no bug in the product, only two tests that could not
  fail (the rule above).
- **Brain:** consultation `32955477393d4641b9fd07bb567446cb` closed as
  `decisions/the-rails-filter-is-zeds-sidebar-filter`, with a follow-up by 2026-10-07.
- **Ticket:** #457 is closed, and no BACKLOG row is stale.
