---
pipeline_id: 64650c16-2433-4d56-8896-01228db245de
ticket: docs/planning/tickets/closed/TICKET-459-rail-cycle-actions.md
status: Phase 4 — Complete PASS
title: Next and Previous Project and Thread in the rail
type: feature
slice: workbench shell (W6, the trait hooks in D8)
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/454-rail-switcher.spec.md, docs/planning/pipeline/completed/453-rail-keyboard-and-reorder.spec.md]
---

## Title
Zed's `NextProject`, `PreviousProject`, `NextThread` and `PreviousThread` reach the sidebar's
`cycle_project` and `cycle_thread` hooks, and the rail leaves both at the trait's no-op
defaults, so in the Marley layout the four palette entries do nothing. The rail now walks its
own projects and rows with them, and opens what it reaches as a click does.

## Scope
### In
- **The model** (`marley_rail`). Two pure functions over the rows the rail shows, read from the
  selected row:
  - `cycle_project`: the project header after or before the selected row's project, wrapping
    at the ends;
  - `cycle_row`: the terminal or thread row after or before the selected row, skipping project
    headers, wrapping at the ends.

  With no row selected, both go to the first going forward and the last going back.
- **The hooks** (`marley_workbench::rail`). `Sidebar::cycle_project` and
  `Sidebar::cycle_thread` open the row the model names through `open_row`, the handler a click
  and Enter use.

### Out (explicitly deferred)
- Vim's `ThreadsSidebar` bindings (`] p`, `[ p` and the rest) in the rail's key context. They
  are a parity item of their own.
- Default keys for the four actions: Zed binds none.
- Unfolding the target project, which Zed's sidebar does (D2).
- A project group with no open workspace: the rail does not list one.

## Reference (§20)
- **Upstream Zed.**
  - `MultiWorkspace` forwards the four actions to its sidebar, open or closed
    (`crates/workspace/src/multi_workspace.rs:2105-2140`).
  - `SidebarHandle` runs the hooks in `window.defer`, outside the `MultiWorkspace`'s update
    (`multi_workspace.rs:234-250`).
  - Zed's sidebar walks its shown project headers, and its shown thread and terminal entries,
    with wrap. With no current entry it goes to the first. It activates the target with focus,
    and for a project it expands the target group first
    (`crates/sidebar/src/sidebar.rs:7142-7267`).

  Marley keeps the wrap, the shown rows only, terminals and threads alike, and activation with
  focus. It differs where the rail's own conventions already decide (D1 to D3).
- **Warp:** N/A. These are Zed's actions over Zed's projects and threads.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md:231-235`
  lists the actions; nothing on their order.
- **Published material:** Zed's docs do not describe the actions (`docs/src` has no match).
- **Code we already ship.**
  - The rail's `step`, `first_row` and `last_row` walk the same shown rows for up, down,
    Home and End.
  - `selection` names the one highlighted row.
  - `open_row` opens a row as a click does.

## UI proof
UI-AFFECTING.
- **Driven tests:** the four actions dispatched in a window with two projects and terminals in
  both. Each shows the right project or focuses the right terminal, wraps at both ends, and
  skips a folded project's rows. The same holds with the rail closed.
- **Live drive:** the palette's Next Project and Next Thread in a running Marley; screenshot. It
  needs keys, so it runs only while Chad is away from the desk; otherwise the Test phase records
  why.

## Locked-In Decisions
- D1 — The walk starts from the rail's selected row, the one it highlights: the keyboard's row
  while the rail holds focus, else the row the window shows. So each action moves the highlight
  by one, as up and down do. Zed starts from the panel's current thread even while an editor has
  focus; the rail highlights that thread only while the panel holds focus, and the actions
  follow the highlight.
- D2 — A project reached by Next or Previous Project keeps its fold. Zed's sidebar expands the
  target group so that its threads show. In the rail a folded project shows in the center just
  the same, and the fold is the user's choice.
- D3 — With nothing selected, Next goes to the first row and Previous to the last, as the
  rail's up and down keys do. Zed goes to the first either way.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Next Project or Previous Project is dispatched in the Marley layout, the window shall show the project after or before the selected row's project in the rail's order, wrapping at the ends | driven tests; unit tests |
| REQ-002 | WHEN Next Thread or Previous Thread is dispatched in the Marley layout, the rail shall open the terminal or thread row after or before the selected row, as a click on it does, skipping project headers and wrapping at the ends | driven tests; unit tests |
| REQ-003 | The actions shall reach only the rows the rail shows: a folded project's rows, and the rows the filter hides, are skipped | unit tests; a driven test for the fold |
| REQ-004 | WHILE nothing is selected, Next shall go to the first row and Previous to the last; WHILE the rail shows no row of the kind, the action shall do nothing | unit tests |
| REQ-005 | WHILE the rail is closed, the actions shall behave as they do while it is open | driven test |
| REQ-006 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — `cycle_project` and `cycle_row` in `marley_rail`, the two hooks in `rail.rs`;
  fmt and clippy clean; a review of the diff.
- **P3 Test** — the unit and driven tests, with negative checks; the live drive;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
