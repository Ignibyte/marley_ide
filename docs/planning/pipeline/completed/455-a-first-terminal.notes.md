# A first terminal for a project opened fresh — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-455-a-first-terminal.md
- **Pipeline spec:** 455-a-first-terminal.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-455 (W6f), split from #451; autonomous per Chad's goal "lets continue
  working on the remaining tickets".
- **Classification / tier:** feature, medium; `marley_workbench`, plus one Zed touchpoint in
  `crates/workspace`.
- **Pre-flight:** no active pipeline, README marker present, cargo idle, every tool present.
- **Minted:** no queued pair; the doc pair is new, the BACKLOG row is gone, and the ticket
  records the signal chosen at promotion.
- **Recall (§18.3).**
  - `BF-claude-boot-restore-force-seeds-a-terminal-after-the-runtime-guard-dissolved-001`: the
    gpui-era boot restore gave every restored project with no terminal one, undoing the user's
    close. A restored project here keeps what it saved.
  - `PR-claude-restore-is-a-second-constructor-001`: the restore path builds the same
    workspace without the interactive path's seeding, so the rule reads the constructor's own
    lookup rather than guessing afterwards.
  - `L-claude-442-driving-a-window-restore-in-a-test-001`: Zed's restore path runs in a test.
    `apply_restored_multiworkspace_state` restores only window-level state, so a project's
    items need a real `new_local` over saved rows.
  - Brain: consultation `918ccfe4d2e443059e4bc2304271e080`, nothing on this seam.
- **Discovery.** One Explore sweep of the workspace lifecycle. Its facts are in the spec's
  Prior art. Three more matter here:
  - `open_paths` activates an existing workspace for the same roots and makes no new one;
  - `open_new` passes no paths, and empty roots never restore;
  - the harness's `Workspace::test_new` never reads the database, so its workspaces will report
    `None`.

### Design
- **`crates/workspace/src/workspace.rs` (Zed crate, the touchpoint).** Its row in
  `docs/marley/zed-touchpoints.md` comes first. Each hunk carries a `// Marley:` comment:
  - a field, `opened_from_saved_state: Option<bool>`, `None` in `Workspace::new`;
  - in `new_local`, `let opened_from_saved_state = serialized_workspace.is_some();` before the
    two branches, and `workspace.opened_from_saved_state = Some(opened_from_saved_state);`
    beside `centered_layout` in both closures that build the workspace;
  - a getter, `pub fn opened_from_saved_state(&self) -> Option<bool>`.
- **`crates/marley_workbench/src/routing.rs` (Marley crate).** The routing's `observe_new` hook
  calls `seed_first_terminal(workspace, window, cx)`. It returns unless the layout is Marley,
  the workspace says `Some(false)`, the center has no `TerminalView`, and there is a first
  visible worktree. Otherwise it calls `open_center_terminal(workspace, false,
  Some(root), window, cx)`.
- **File manifest:**
  - `crates/workspace/src/workspace.rs` (Zed crate);
  - `docs/marley/zed-touchpoints.md`;
  - `crates/marley_workbench/src/routing.rs` and `routing_tests.rs` (Marley crate).

  Docs at Complete: `CHANGELOG.md`, `docs/marley_architecture/marley_workbench.md`,
  `docs/marley/workbench-shell.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven (`routing_tests.rs`, real shells with the executor allowed to park): `Workspace::new_local` over a fresh folder in the Marley layout, into the test window: one center terminal, its working directory the root, focused |
| 002 | driven: after the first test's state is written (`flush_all_serialization`) and the workspace is gone, `new_local` over the same roots restores it: one terminal, the saved one; again with the terminal closed before the write: none |
| 003 | driven: the same fresh open in the Zed layout: none; `new_local` with no paths: none; `Workspace::test_new` in the Marley layout: none |
| 004 | driven: the three values read from the workspaces above |
| 005 | gate:16, and the review of the hunks |
| 006 | `script/gates.sh --diff` |

### Risks
- **The test harness for `new_local`.** It needs a test `AppState` and the workspace database
  in tests. The persistence tests (`crates/workspace/src/persistence.rs` around `:6123-6213`)
  are the template. If a real reopen cannot be driven from `marley_workbench`, the Code phase
  records it and falls back to the lookup's two values through a fresh and a restored
  `new_local` in whatever form works.
- **A real shell.** The seed goes through `create_terminal_shell`, so the tests allow parking,
  as `routing_tests.rs` already does.
- **Upstream churn.** `new_local` changes often upstream. The hunks are one line each, beside
  `centered_layout`, which moves with them.
- **Decisions for the brain at Complete:** D1 (the touchpoint as the signal) and D2 (only fresh
  projects).

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] the ledger row · [x] the touchpoint · [x] the seed ·
  [x] fmt · [x] clippy · [x] review.
- **Built.**
  - `docs/marley/zed-touchpoints.md`: the row for `crates/workspace/src/workspace.rs`, written
    before the first edit.
  - `crates/workspace/src/workspace.rs`, 18 added lines in six hunks, each with a `// Marley:`
    comment: the field, its `None` in `Workspace::new`, the recorded lookup in `new_local`, the
    assignment in both closures that build the workspace, and the getter. `rustfmt` reformatted
    only the new lines.
  - `crates/marley_workbench/src/routing.rs`: the routing's `observe_new` hook now passes its
    window to `seed_first_terminal`. That function opens one center terminal at the first
    visible root when the workspace has a window, a root, `Some(false)`, no center terminal,
    and the Marley layout.
- **Deviations.** None from the design. The test plan's fallback was not needed:
  `Workspace::new_local` runs in `marley_workbench` tests with `AppState::test` and the
  harness's test database. `flush_all_serialization` and a second `new_local` over the same
  roots do a real restore.
- **Review.**
  - clippy: an unused import, `clone_on_ref_ptr`, and three `needless_pass_by_ref_mut` in test
    helpers and tests; `#[gpui::test]` takes `&TestAppContext`, as the crate's other tests do.
    `cargo clippy -p workspace --all-targets --all-features -- -D warnings` is clean, as gate:2
    will run it.
  - The first tests opened every folder in a new window, so only one of the two closures that
    set the value ran. `a_folder_opened_into_a_window_starts_with_a_terminal_too` opens one into
    an existing window.
  - Re-entrancy: the seed runs inside the observer's update of the new workspace, the context
    the routing's action handlers already call `open_center_terminal` from.
  - Provenance: the touchpoint is new lines only; no Zed code is copied into a Marley crate.
- **So far:**
  - clippy clean on both crates;
  - `cargo nextest run -p workspace`: 274 of 274 passed;
  - `cargo llvm-cov nextest -p marley_workbench`: 119 of 119 passed (before the window test),
    with 100% of lines and functions.

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-005 tests · [x] negative checks ·
  [x] the crate suites · [x] the live drive (not run; why below) · [x] the gate.
- **Tests:** five driven tests in `crates/marley_workbench/src/routing_tests.rs`, all through
  Zed's real `Workspace::new_local` except the last.

  | REQ | Tests |
  |---|---|
  | 001 | `a_folder_opened_fresh_starts_with_a_terminal_at_its_root` (the shell's working directory is the root, focused), `a_folder_opened_into_a_window_starts_with_a_terminal_too` |
  | 002 | `a_folder_opened_from_saved_state_gets_no_terminal_of_its_own`: a reopen brings its saved terminal back alone, and a project saved with none, from the Zed layout, reopens in the Marley layout with none |
  | 003 | `nothing_else_gets_a_first_terminal` (the Zed layout, a window with no folder), `a_workspace_made_another_way_gets_no_first_terminal` (`test_new`) |
  | 004 | the values read in all five: `Some(false)`, `Some(true)`, `None` |
  | 005 | gate:16, and the review of the six hunks |
  | 006 | the gate |

  One test differs from the plan. Its case "without a saved terminal" saves a project opened
  in the Zed layout, rather than closing a real shell before the write, which would ask about
  the running process.
- **Negative checks**, each file restored and checked by checksum:
  - seeding whatever the saved state: the restore test fails on the project saved empty
    (`left: 1`). Its first half cannot fail: a restore whose saved center has panes swaps that
    center in and drops a seed (L-claude-455-driving-a-real-open-and-restore-through-new-local-001);
  - seeding in the Zed layout too: `nothing_else_gets_a_first_terminal` fails (`left: 1`);
  - either `new_local` closure left without the assignment: its own test fails
    (`left: None`, `right: Some(false)`), the new window's and the requesting window's alike.
- **Run:** `cargo nextest run -p marley_workbench -p workspace`: 394 tests run, 394 passed.
- **Gate.** The first start found rust-analyzer's own `cargo check`, started by the restored
  files, and stood down. The run after it: 19 passed, 0 failed, `GATE GREEN [diff]`, with
  gate:16 on the new row. gate:3 ran 696 tests; gate:4 ran 120, with 100% of lines and
  functions in `marley_workbench`. The receipt matches the tree (`6c7d3230…`).
- **Live drive: not run.** Chad is at the desk: his windows are on ws3, which the headless
  output borrows (L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001), and
  the session is not idle. The drive needs a relaunch, so it is owed with W3 to W6's.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket ·
  [x] archive · [x] commit.
- **Documented:**
  - `CHANGELOG.md`, under Added: a first terminal for a new project;
  - `docs/marley_architecture/marley_workbench.md`: the first terminal under Routing, twenty
    routing tests, the known limit and the owed drive;
  - `docs/marley/workbench-shell.md`: W6f shipped;
  - `docs/marley/zed-touchpoints.md`: the `crates/workspace/src/workspace.rs` row, written in
    Code, which describes what shipped.
- **Knowledge:** `AD-claude-455-a-first-terminal-reads-zeds-own-lookup-001`,
  `L-claude-455-driving-a-real-open-and-restore-through-new-local-001`. No `F-…` block: no bug
  was found.
- **Brain:** consultation `918ccfe4d2e443059e4bc2304271e080` closed as
  `decisions/a-first-terminal-reads-zeds-own-lookup-of-saved-state`, with a follow-up by
  2026-10-07.
- **Ticket:** #455 is closed, and no BACKLOG row is stale.
