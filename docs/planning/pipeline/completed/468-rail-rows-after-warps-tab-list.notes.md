# The rail's rows after Warp's tab list — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-468-rail-rows-after-warps-tab-list.md
- **Pipeline spec:** 468-rail-rows-after-warps-tab-list.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (Chad's goal, autonomous).
- **Request (Chad, 2026-09-23, verbatim):** "look in my Pictures folder on here there is a
  screen shot of today. The left pane lets make it look like the warp.dev one. It basically puts
  more padding around the project + terminal to make is a bit better. makes the icons bigger
  etc."
- **Classification / tier:** feature, medium; `marley_workbench` and `marley_rail` only.
- **Pre-flight:** #467 committed (`0a8598f5bd`); no other active pipeline; README marker
  present; cargo idle.
- **Recall (§18.3).**
  - `AD-claude-439-the-rail-does-not-claim-zeds-threads-list-001` and the workbench record: the
    rail draws threads with Zed's `ThreadItem` and terminals with `ListItem`; nothing binds it
    to either.
  - #418's beautifului notes: quiet rows, one lit selection. Kept.
  - `L-claude-438-prove-a-views-own-notify-with-a-selector-001`: debug selectors exist only in
    test builds, and bounds are what a driven test can read of a layout.
  - Brain: consultation `34b1d4ad1fce4340962e48a01fc182e0`, nothing on this seam.
- **Discovery.** The capture before the change (`rail-before`, scratchpad): a 12px title over an
  empty 10px line, a 14px icon, rows about 34px high, a full-width fill for the selection. Warp's
  screenshot measured at 3x: `docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`.

### Design
- **The row element** (`rail.rs`): `row_card(id, selected, icon, title, subtitle, cx) ->
  Stateful<Div>`, an `h_flex` with a `ROW_GROUP` hover group, `h_11`, `gap_2p5`, `pl_2 pr_1p5`,
  `rounded_md`, `border_1` in `colors.border` when selected and transparent otherwise, the
  selected fill `ghost_element_selected`, the hover fill `ghost_element_hover` on unselected
  rows, `cursor_pointer`. It holds `row_icon(icon, cx)`, a `size_7` `rounded_full` container
  filled `element_background`, and a `v_flex` of the title and, when present, the second line
  (`LabelSize::XSmall`, muted, truncated). The caller adds its trailing slot and its clicks.
- **Terminal rows:** the card with the terminal's or agent CLI's icon (`IconSize::Small`, muted),
  the trailing slot a relative box holding the close button (`visible_on_hover(ROW_GROUP)`) and,
  over it, the bell dot (`group_hover(ROW_GROUP, invisible)`). The click, the double-click
  rename and the right-click menu are unchanged. `ml_2` sets the rows under their project.
- **Thread rows:** the card with the agent's icon, `row_label` for the title with the filter's
  highlights, the second line `"{agent_name} · {status.label()}"`, the trailing slot the status
  mark. `thread_item` stays for the switcher.
- **The project header:** its own `h_flex` in the same card style at `h_8`, the chevron, the name
  in `LabelSize::Small`, muted unless selected, the attention dot and the `+` menu.
- **Dividers:** `render` emits `div().py_1p5().child(Divider::horizontal())` with the selector
  `marley-rail-divider-{index}` before each project header after the first row.
- **Padding:** the rows list `px_2 py_1p5 gap_0p5`; the filter row `px_3 py_2 gap_2`.
- **`marley_rail::ThreadStatus::label`** (`const fn`): idle, working, waiting, failed.
- **`agents::thread_agent_name(agent, project, cx)`**: "Zed Agent" for the native agent, else
  the agent server's display name, the registry's, or the id; `thread_agents` uses it.
  `ThreadEntry` gains `agent_name`.
- **File manifest.** Marley only: `crates/marley_workbench/src/rail.rs`, `rail_tests.rs`,
  `agents.rs`; `crates/marley_rail/src/marley_rail.rs`. No Zed path changes.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: a terminal at the project root (no second line), one in a subdirectory (a second line) and a thread row read one height from their selectors; each row's icon selector (`marley-rail-icon-…`) reads the icon size |
| 002 | driven: every row's bounds before and after the selection moves are equal; the live drive shows the card |
| 003 | driven: two projects draw `marley-rail-divider-1` and no `marley-rail-divider-0` |
| 004 | unit: `ThreadStatus::label` for each status; `thread_agent_name` for the native agent through the menu's list (driven); live drive for the line |
| 005 | the existing `marley_workbench` suite, unchanged |
| 006 | `script/gates.sh --diff` |

### Risks
- **Taller rows** fit fewer rows on screen. The list scrolls, and the heights follow the UI
  font size.
- **Hover targets:** the close button is laid out while hidden, as `ListItem`'s swap does, so a
  title truncates the same with or without the pointer.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] recall · [x] marker present · [x] `rail.rs` · [x] `agents.rs` · [x]
  `marley_rail.rs` · [x] `rail_tests.rs` · [x] fmt and clippy · [x] review.
- **Built.**
  - `rail.rs`: `row_frame`, the frame every row shares, with the border kept on every row, clear
    unless selected; `row_card` (`h_11`, a `size_7` round icon, the title over the optional
    second line); `raised`, the icon circle's fill and the selected border; `rail_terminal_icon`,
    a `>_` in the buffer font for a shell and the agent's icon for an agent CLI;
    `thread_subtitle`; `thread_status_mark`; `ROW_GROUP` for the hover swap. The project header,
    terminal and thread rows are drawn with them; `render` puts a divider in each project's
    wrapper after the first row; the list and the filter row are padded. `ListItem` left the
    rail; `thread_item` now serves the switcher only.
  - `agents.rs`: `thread_agent_name`, which `thread_agents` uses for every name.
  - `marley_rail.rs`: `ThreadStatus::label`.
- **Deviations.**
  - **The shell's icon is a `>_` in the buffer font**, not Zed's `IconName::Terminal`. The first
    capture showed Zed's icon as a filled box with the prompt cut out, and all three of Zed's
    terminal icons (`terminal`, `terminal_alt`, `tool_terminal`) draw the prompt in a box; Warp's
    is a bare `>_`. The switcher keeps Zed's icon.
  - **`raised`, the text color at 10%**, for the circle and the selected border. With the plan's
    `element_background` and `border` the circle matched the pane (46,52,62 on 47,52,62) and the
    border matched the selected fill (70,75,87 on 69,74,86), both invisible in One Dark. Warp
    draws both a step lighter than what they sit on (pane 37,41,44, circle 56,60,64; card
    72,76,80, border 87,93,95). The text color at 10% gives that step over any fill, lighter in a
    dark theme and darker in a light one.
  - **The divider lives in the project's wrapper**, from `rail_rows`' position, rather than as
    its own element pushed in a loop: the loop added `continue` and `if let` fall-throughs no
    test reaches, which gate:4's line floor would count as missed
    (L-claude-438-the-coverage-floor-counts-lines-per-function-001). The project's selector
    stays on the header alone.
  - **`thread_agent_name` keeps the registry lookup on one line**, as `thread_agents` had it, for
    the same floor.
- **Review.**
  - Every selector the tests use stays on the element it was on; the terminal's and the thread's
    wrappers take the indent (`pl_2`), so a click at a wrapper's center lands on its card.
  - The close button is laid out while hidden, as `ListItem`'s swap lays it out, so titles
    truncate the same with the pointer over the row or not.
  - The disclosure is an `IconButton`, which stops the click's propagation, so it folds without
    activating the project, as inside `ListItem`.
  - Re-entrancy: nothing new reads an entity during render beyond the theme.
  - Provenance: the layout was measured from a screenshot of Warp; no Warp code was read. The
    hover swap is gpui's `group`, `visible_on_hover` and `group_hover`; no body was copied from
    Zed's `ListItem` or `ThreadItem`.
  - Upstream: no Zed path changed.

## Phase 3 — Test (2026-09-23)
- **Checklist:** [x] REQ-001 · [x] REQ-002 · [x] REQ-003 · [x] REQ-004 · [x] REQ-005 · [x] negative
  checks · [x] live drive · [x] gate.
- **Tests** (`rail_tests.rs` unless named):
  - `a_terminal_row_is_a_fixed_height_with_a_round_icon` (REQ-001): `h_11` and a square
    `size_7` icon, read in the test window's rems (14px there, so 38.5px and 24.5px).
  - `agents::rows_with_and_without_a_second_line_share_one_height_and_icon` (REQ-001).
  - `threads::a_thread_row_is_drawn_as_a_terminal_row_is` (REQ-001, REQ-004): one height, one
    icon size and one icon column; the stub agent's name falls back to its id.
  - `moving_the_selection_moves_no_row` (REQ-002): every row's, icon's and chevron's bounds are
    unchanged after a click selects another terminal and after `SelectFirst` selects a header.
  - `a_line_runs_between_projects_and_none_above_the_first` (REQ-003), including a filter that
    leaves only the second project.
  - `a_thread_row_marks_each_status_at_its_end` and
    `a_thread_rows_second_line_names_its_agent_and_what_it_does` (REQ-004); in `marley_rail`,
    `each_thread_status_has_its_word`.
  - `threads::agents_take_their_name_and_icon_from_the_registry`, extended: the row's second line
    names the registry's agent ("Zeta Code"), and the switcher draws the thread with the
    registry's icon.
  - REQ-005: every existing test passed unchanged; the crates' suites, 176 passed.
- **Negative checks,** each restored by checksum:
  - `h_11` removed: the fixed-height test fails (`26.5px` against `38.5px`), and so does the
    second-line test (`26.5px` against `33px`).
  - The border drawn only on the selected row: `moving_the_selection_moves_no_row` fails.
  - A divider above every project, in the loop form and again in the final wrapper form:
    `a_line_runs_between_projects_and_none_above_the_first` fails.
- **Gate.** The first run was red on gate:4, one line: `thread_item`'s `AgentIcon::Svg` arm,
  which the rail's own thread rows used to reach and which only the switcher reaches now. The
  registry test above now opens the switcher over the registry's thread. The second run:
  `GATE GREEN [diff]`, 20 passed, the Marley lines at 100%, the receipt matching the tree.
- **Live drive.** The debug `marley` on a copy of Chad's profile on hidden workspace 9, shot by
  its toplevel with no input sent (L-claude-467-capture-one-window-by-its-toplevel-001); for the
  last two shots two more terminals were added to the copy's saved session (`crates/` and the
  home directory), so unselected rows show.
  - Before: a 12px title over an empty line, a 14px icon, a full-width fill for the selection.
  - First look: the card and the circle drew, but Zed's terminal icon read as a filled box, the
    circle matched the pane and the border matched the fill (the deviations in Phase 2).
  - Final (`rail-after-4`, on the committed code): the selected `marley_ide — bash` card with a
    light border; `crates — bash` over `crates` and `cpeppers — bash` over `~`, unselected, each
    with a `>_` in a circle a step lighter than the pane; the project name muted with its
    chevron and `+`; the taller filter row.
  - Not seen live: the divider, which needs a second open project (a restored window reopens
    only its active project, and the rail lists no group without an open workspace), and the
    thread rows, which this profile has none of. Both are covered by the driven tests above.

## Phase 4 — Complete (2026-09-23)
- **Docs (§21).** `CHANGELOG.md` (Changed); `docs/marley_architecture/marley_workbench.md` (the
  rows' look, rename and close, the switcher's rows, thread rows, the filter's highlights);
  `docs/marley_architecture/marley_rail.md` (`ThreadStatus::label`);
  `docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md` and its entry in the
  observed README. No Zed path changed, so no ledger row.
- **Ledger (§19).** `AD-claude-468-the-rail-draws-its-own-rows-after-warps-tab-list-001`,
  `L-claude-468-sample-the-capture-before-trusting-a-theme-token-001`. No F block: nothing
  shipped broken; the one test failure in Code was the test's own assumption about which row
  starts selected.
- **Brain.** Consultation `34b1d4ad1fce4340962e48a01fc182e0` closed by
  `decisions/the-rail-draws-its-own-rows-after-warps-vertical-tab-list`, follow-up 2026-10-07.
- **Ticket** closed; the pipeline archived; one commit.
