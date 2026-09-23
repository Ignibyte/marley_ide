# Next and Previous Project and Thread in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-459-rail-cycle-actions.md
- **Pipeline spec:** 459-rail-cycle-actions.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-459, split from #454; autonomous per Chad's goal "lets continue working on
  the remaining tickets".
- **Classification / tier:** feature, small; `marley_rail` and `marley_workbench`.
- **Pre-flight:** no active pipeline, README marker present, cargo idle, hooks wired.
- **Recall (§18.3).**
  - #454's notes: `SidebarHandle` runs `toggle_thread_switcher`, `cycle_project` and
    `cycle_thread` in `window.defer`, so the hooks may update the `MultiWorkspace`.
  - #438's notes: while the rail is registered the actions do nothing (the trait defaults).
  - `workbench-shell.md` D8 points the hooks at this ticket.
  - Brain: consultation `e8109636bf5548f6a865b4b61238550a`, nothing on this seam.
- **Discovery:** read `multi_workspace.rs` (the forwarding, `SidebarHandle`, `activate`, which
  ends in `focus_active_workspace`), Zed's `cycle_project_impl` and `cycle_thread_impl` for
  behavior, and the rail's `selection`, `walk`, `step`, `open_row` and `fold`.

### Design
- **`marley_rail`** (pure, beside `step`):
  - `cycle_project(snapshot, forward) -> Selection`: the shown headers, in order. The current
    one is the selected row's project (`parent` of `selection`). The next or previous header
    wraps; with nothing selected, the first going forward and the last going back;
    `Selection::None` when no header shows.
  - `cycle_row(snapshot, forward) -> Selection`: the shown rows (`walk`), headers included, so
    a selected header has a place. From the selected row's position it looks one step at a time
    in the direction given, wrapping, for the first terminal or thread row. A lone such row
    reaches itself. With nothing selected the search starts before the first row going forward
    and after the last going back. `Selection::None` when none shows.
- **`rail.rs`**: `cycle_project` and `cycle_thread` in `impl Sidebar for Rail` call
  `open_row(marley_rail::cycle_*(&self.snapshot.rail, forward), …)` and log a failure, as
  `confirm` does. They run deferred (`SidebarHandle`), outside the `MultiWorkspace`'s update, so
  `activate` is safe. Activation moves focus out of the rail (`focus_active_workspace`, or the
  terminal's or thread's focus), which clears the keyboard's row, so the next action starts
  from the row the window now shows.
- **File manifest.** Marley only: `crates/marley_rail/src/marley_rail.rs` (and its tests),
  `crates/marley_workbench/src/rail.rs`, `rail_tests.rs`. Docs at Complete: `CHANGELOG.md`,
  `docs/marley_architecture/marley_rail.md` and `marley_workbench.md`,
  `docs/marley/workbench-shell.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | unit: `cycle_project` forward and back from each project, wrapping; driven: Next and Previous Project change the displayed workspace, and wrap |
| 002 | unit: `cycle_row` across two projects, skipping headers, from a header, wrapping both ways; driven: Next and Previous Thread focus the right terminal, across projects and wrapping |
| 003 | unit: a folded project's rows and the filter's hidden rows are skipped, and a project the filter hides is not reached; driven: with alpha folded, Next Thread skips its terminals |
| 004 | unit: nothing selected goes first or last; no rows gives `Selection::None`; a lone row reaches itself |
| 005 | driven: with the rail closed, Next Project and Next Thread still move |
| 006 | `script/gates.sh --diff` |

### Risks
- **The palette's focus.** The palette gives focus back before it dispatches, so the rail's
  selection is the row the user left. A driven test dispatches from a focused center terminal,
  as that path does.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] recall · [x] marker present · [x] `marley_rail.rs` · [x] `rail.rs` · [x]
  fmt and clippy · [x] review.
- **Built.**
  - `marley_rail::cycle_project` and `cycle_row`, over one private `cycle`: the shown rows,
    reversed going back, turned so the search starts just past `from`, then the first row the
    kind takes. A lone row reaches itself, and a `from` that is not shown starts the search at
    an end.
  - `Rail::cycle_project` and `cycle_thread` in `impl Sidebar`, each one `open_row` call.
- **Deviations:** none from the design. Clippy's `too_long_first_doc_paragraph` split the two
  public doc comments.
- **Review.**
  - Re-entrancy: the hooks run inside the rail's update, from `SidebarHandle`'s `window.defer`,
    and `open_row` updates the `MultiWorkspace`, a workspace and a terminal view, never the
    rail. That is the path a header or row click already takes.
  - Activation moves focus: `MultiWorkspace::activate` ends in `focus_active_workspace`, and a
    terminal or thread takes focus. The keyboard's row clears with the rail's focus, so no
    action can stay on a stale cursor.
  - Errors: a row closed since the last rebuild is logged, as `confirm` logs it.
  - Provenance: Zed's `cycle_project_impl` and `cycle_thread_impl` were read for their
    behavior. The rail's walk is its own, over `shown`.
  - Upstream: no Zed path changed.

## Phase 3 — Test (2026-09-23)
- **Checklist:** [x] REQ-001 to REQ-005 tests · [x] negative checks · [x] suites · [x] live
  drive (skipped, reason below) · [x] gate.
- **Unit tests** (`marley_rail`):
  - `next_and_previous_project_go_round_the_shown_headers` (REQ-001);
  - `next_and_previous_thread_go_round_the_terminals_and_threads` (REQ-002);
  - `cycling_passes_over_what_the_fold_and_the_filter_hide` (REQ-003);
  - `cycling_from_nothing_starts_at_an_end_and_finds_nothing_in_an_empty_rail` (REQ-004).
- **Driven tests** (`rail_tests.rs`; `open_rail_over_three` lists gamma, beta, alpha):
  - `next_and_previous_project_go_round_the_rails_projects` (REQ-001);
  - `in_the_rail_next_project_goes_from_the_keyboards_row` (D1);
  - `next_and_previous_thread_go_round_the_rows_and_focus_each` (REQ-002);
  - `next_thread_passes_over_a_folded_projects_rows` (REQ-003);
  - `the_actions_work_with_the_rail_closed` (REQ-005).
- **Negative checks**, each restored by sha256 checksum:
  - N1, the hooks do nothing: all five driven tests fail;
  - N2, a row starts from its own place, not its project's header: the project unit test
    fails;
  - N3, starting from the displayed project instead of the keyboard's row: two unit tests and
    the D1 driven test fail;
  - N4, no wrap: eight tests fail;
  - N5, headers not passed over: five tests fail;
  - N6, hidden rows walked: the fold-and-filter unit test and the fold driven test fail;
  - N7, Previous from nothing starts at the first row: the from-nothing unit test fails.
- **Live drive: not run.** The palette needs keys, and Chad was at the desk: `IdleHint=no`,
  and his windows were on ws3. Owed to the next headless capture.
- **Suites:** `cargo nextest run -p marley_rail -p marley_workbench`: 167 passed.
- **Gate:** `script/gates.sh --diff`: GATE GREEN [diff], all 18 steps passing; coverage at 100%
  of lines (3770) and functions (523). The receipt matches the tree.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist:** [x] document · [x] knowledge · [x] close the ticket · [x] archive · [x] commit.
- **Docs:** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_rail.md` (the two
  functions, the test count); `docs/marley_architecture/marley_workbench.md` (the hooks, the
  known limit replaced by vim's bindings, the live drive owed);
  `docs/marley/workbench-shell.md` (D8, the W6 paragraph and row). No Zed path changed.
- **Knowledge:** `AD-claude-459-the-rails-cycle-actions-go-round-its-shown-rows-from-the-highlight-001`.
  No `F-` block (no bug found) and no lesson beyond the decision.
- **Brain:** consultation `e8109636bf5548f6a865b4b61238550a` closed with a decision, follow-up
  by 2026-10-07.
- **Ticket:** #459 closed; no BACKLOG row left.
