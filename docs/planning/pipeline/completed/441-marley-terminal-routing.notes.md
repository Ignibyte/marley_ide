# Terminal routing and keys in the Marley layout — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-441-marley-terminal-routing.md
- **Pipeline spec:** 441-marley-terminal-routing.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** Chad chose "Hide, route to center" for the bottom panel (2026-09-22).
- **Classification / tier:** feature, medium; `marley_workbench` plus one `zed.rs` line.
- **Recall (§18.3):** `PR-claude-unmodified-terminal-chords-yield-to-the-pty-001`;
  `PR-claude-new-chord-shadowed-by-hardcoded-key-001` (check each new chord against Zed's
  defaults in every context it can reach); `PR-claude-key-arm-above-the-keymap-must-gate-on-modifiers-001`.
- **Discovery:** the terminal sweep of 2026-09-22 listed every path that forces the panel
  (NewTerminal, OpenTerminal, tasks, vim, agent login, the toggles) and the two zero-touch
  overrides (the provider and capture-phase actions); the defaults sweep showed why
  `cx.bind_keys` at init cannot work.
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.

## Carried from #438's inspect (2026-09-22)
- Zed's Panel Layout presets misread the Marley layout: with `agent.dock` patched to the right,
  the title bar's menu shows "Custom", and choosing Classic there writes the four other panels
  to the left but not `agent.dock` (the patched default already says right), so back in the Zed
  layout every panel sits left; choosing Agentic writes `agent.dock: left`, which then overrides
  the Marley layout for good (`agent_settings.rs:92-111`, `:338-396`; `title_bar.rs:1289-1292`).
  Hide `UseClassicLayout` and `UseAgenticLayout` while the layout is `marley`, the way
  `title_bar.rs:190-203` hides them with AI off.

## Phase 1 — Plan (promoted 2026-09-22, after #440)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] promote, split and
  re-verify · [x] prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Split at promotion.** The queued W5 held four slices, so #441 keeps the routing:
  - the Marley keymap, with its `zed.rs` touchpoint and keymap-reload tests, is TICKET-449,
    queued before W6;
  - the first-show terminal (at risk of doubling a restored workspace's terminals) and Zed's
    Panel Layout presets (the title bar re-shows them on every settings change) move to W6's
    notes.
- **Recall.**
  - `PR-claude-input-handler-overlay-arms-must-stop-propagation-001`, in spirit: a capture
    handler that consumes an action must call `cx.stop_propagation()`, or the panel's bubble
    handler runs as well (D4).
  - `L-claude-439-focus-lands-in-a-dock-panel-only-once-its-dock-is-open-001`: "the panel
    stays closed" is checked on the dock, not on focus.
  - Brain: consultation `1fe35545e8fc46958c969d14bf7ae040`, nothing on this seam.
- **Re-verified:** see the spec's re-swept Prior art. No crate the routing uses changed since
  the queue.

### Design
- **New module `crates/marley_workbench/src/routing.rs`** (a component of its own, tests in
  `routing_tests.rs` through `#[path]`, as `rail.rs` does). `routing::init` runs from the
  crate's `init`:
  - `cx.observe_new::<Workspace>` registers, per workspace:
    - `register_action_renderer`, whose callback adds `capture_action` listeners for
      `workspace::NewTerminal` and `workspace::OpenTerminal`;
    - `subscribe_self` for `workspace::Event::PanelAdded`: when the added panel downcasts to
      `TerminalPanel`, `set_terminal_provider(RoutedTerminals(panel))`.
- **`RoutedTerminals`**, a `workspace::TerminalProvider`. In the Marley layout (read on every
  call, D1) it sets the task's `reveal_target` to `Center`. It then calls
  `panel.spawn_task(&task, window, cx)`. An async helper awaits the new terminal and its
  `wait_for_completed_task`, with each `?` on an executed line
  (PR-claude-bind-a-multi-line-closure-before-its-question-mark-001). A failed spawn is
  `Some(Err)`, and a terminal gone before it completes is `None`, as Zed's provider answers.
- **Capture listeners.** In the Zed layout they return and the action propagates. In the
  Marley layout they call `stop_propagation` and open a center terminal through
  `TerminalPanel::add_center_terminal`: `create_local_terminal` when `local`, else
  `create_terminal_shell` with New Terminal's `default_working_directory` or Open in
  Terminal's directory. Errors reach `detach_and_prompt_err`.
- **Layout check.** `marley_layout(cx)` in `marley_workbench.rs`, shared with the rail.
- **Manifest.** `marley_workbench` gains `task` (`SpawnInTerminal`, `RevealTarget`) if its
  existing dependencies do not re-export them.
- **File manifest.** All Marley-owned; no Zed touchpoint:
  - `crates/marley_workbench/src/routing.rs` and `routing_tests.rs` (new);
  - `marley_workbench.rs` (the module, `init`, `marley_layout`);
  - `Cargo.toml`;
  - docs: `docs/marley_architecture/marley_workbench.md`, `docs/marley/workbench-shell.md`,
    `CHANGELOG.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven (real shell, `allow_parking`): in the Marley layout, `Workspace::spawn_in_terminal` with an `echo` task opens a center terminal and returns its successful exit; the panel's pane stays empty |
| 001, failure | driven: a task running `__nonexistent_program__` returns `Some(Err)` and opens nothing |
| 002 | driven: in the Zed layout, a dock task opens in the Terminal Panel and the center gains nothing |
| 003 | driven: in the Marley layout, `workspace::NewTerminal`, dispatched from inside the workspace, opens a center terminal; the panel's dock stays closed and empty. With `local: true`, the same through `create_local_terminal` |
| 004 | driven: in the Marley layout, `workspace::OpenTerminal` for a temporary directory opens a center terminal whose shell starts there |
| 005 | driven: in the Zed layout, both actions open in the Terminal Panel |
| 006 | driven: after a switch from `zed` to `marley`, the next `NewTerminal` routes to the center |
| 007 | `script/gates.sh --diff` |

A driven test must dispatch from inside the workspace, with its center pane focused, so the
workspace's root is on the dispatch path. With nothing focused, the dispatch starts at the
window's root and misses the workspace's capture listeners. The live drive needs input (spec,
UI proof).

### Risks
- **Real shells.** The tests spawn real shells, as Zed's panel tests do; they take longer and
  need `allow_parking`. Coverage runs them too.
- **Capture scope.** The listeners sit on the workspace's root, so every `NewTerminal` and
  `OpenTerminal` in the window (panels, context menus, the palette) routes to the center in
  the Marley layout. That is intended: nothing should open the bottom panel.
- **Provider order.** If Zed ever installs its provider after `PanelAdded`, the Marley
  provider would lose. The REQ-001 test catches that: a Marley-layout task would reach the
  dock.

## Phase 2 — Code (2026-09-22)
- **Checklist** (no `TaskCreate`): [x] `routing.rs` · [x] `marley_workbench.rs` · [x] `Cargo.toml`
  · [x] `routing_tests.rs` · [x] review · [x] clippy at deny level · [x] coverage.
- **Built.**
  - `crates/marley_workbench/src/routing.rs`: `init` (the capture listeners through
    `register_action_renderer`, the provider on `PanelAdded`), `RoutedTerminals`, `run_task`,
    `new_terminal`, `open_terminal`, `open_center_terminal`.
  - `marley_workbench.rs`: `pub mod routing`, `routing::init` from `init`, `marley_layout`.
    `Cargo.toml`: `task`.
  - `routing_tests.rs`: seven driven tests on real shells. `init_zed_sidebar` in the layout
    tests is now `pub(crate)`.
- **Deviations from the design.**
  - The provider defers the spawn to the window's next turn (`window.spawn`, then
    `update_in`). The workspace calls a provider while it is being updated, and `spawn_task`
    reads the workspace. The first draft called the panel at once, and the REQ-001 test,
    run against it, panics: "cannot read workspace::Workspace while it is already being
    updated". The review caught it before any test ran; the F block is written at Complete.
  - `routing` is a `pub mod` with a `pub fn init`. One level deep, clippy counts `pub(super)`
    as `pub(crate)` (`redundant_pub_crate`), and `unreachable_pub` rejects a bare `pub` in a
    private module; AD-447 settles that by structure.
  - The tests load the panel through `TerminalPanel::load`, as `crates/zed` does, not
    `TerminalPanel::new` as planned. Zed's provider is then in place first, so the install
    order (D3 and the provider-order risk) is under test.
  - REQ-001's failure case is a missing shell, not a missing program. `spawn_task` wraps the
    command in the task's shell, so a missing program comes back as the shell's exit status
    127 (`Some(Ok)`); only a missing shell fails the spawn.
  - The layout-switch test needs `init_zed_sidebar`: the switch back to the Zed layout builds
    Zed's sidebar, which reads the agent stores.
- **Review against the criteria.**
  - Re-entrancy: the provider bug above. The capture listeners get `&mut Workspace` and call
    `add_center_terminal`, which takes it directly and spawns before touching the project.
  - Capture scope: the listeners sit on the workspace's root, so in the Marley layout every
    New Terminal or Open in Terminal dispatched inside the workspace goes to the center: a
    pane's `+` menu, the project panel's Open in Terminal, the palette, and the Terminal
    Panel's own `+` if the panel is opened by hand. Intended (spec, Risks).
  - Errors: a center terminal that cannot open shows a prompt with the error as its detail;
    Zed's panel path only logs. A failed task spawn is `Some(Err)` to the caller.
  - Provenance: no Warp source. The Zed calls are API use (`spawn_task`,
    `add_center_terminal` with the project's two shell constructors,
    `wait_for_completed_task`); no GPL function body is carried over. No Zed crate changed,
    so there is no ledger row.
- **Negative checks.** With the provider's install replaced by a drop, the REQ-001 and REQ-006
  tests fail. With the capture listeners removed, the REQ-003, REQ-004 and REQ-006 tests fail.
- **Checks.** `cargo clippy -p marley_workbench --all-targets --all-features -- -D warnings`
  clean. Coverage: `routing.rs` 89 of 89 lines and 13 of 13 functions. The two missed regions
  are the `?` exits for a window closed before the spawn and a terminal dropped before its
  completion is read; no test can close either at that instant.

## Phase 3 — Test (2026-09-22)
- **Checklist** (no `TaskCreate`): [x] REQ-001 · [x] REQ-001, failure · [x] REQ-002 · [x] REQ-003
  · [x] REQ-004 · [x] REQ-005 · [x] REQ-006 · [x] the gate · [x] the live drive (not run; why
  below).
- **Tests** (`crates/marley_workbench/src/routing_tests.rs`, real shells, `allow_parking`; the
  panel is loaded with `TerminalPanel::load` and added, as `crates/zed` does):

  | REQ | Test |
  |---|---|
  | 001 | `a_task_in_the_marley_layout_runs_in_a_center_terminal`: an `echo` task through `Workspace::spawn_in_terminal` succeeds in one center terminal; the panel has none and its dock stays closed |
  | 001 | `a_task_that_cannot_start_reports_its_error_and_opens_nothing`: a task whose shell is missing returns `Some(Err)` and opens no terminal |
  | 002 | `a_task_in_the_zed_layout_opens_where_its_reveal_target_says`: a dock task lands in the panel, a center task in the center |
  | 003 | `new_terminal_in_the_marley_layout_opens_in_the_center`: New Terminal, then with `local: true`, gives two center terminals; the panel stays closed and empty |
  | 004 | `open_terminal_in_the_marley_layout_starts_a_center_shell_in_its_directory`: the shell's working directory is the requested temporary directory |
  | 005 | `in_the_zed_layout_both_actions_open_in_the_terminal_panel`: both actions land in the panel, which opens |
  | 006 | `a_layout_switch_routes_the_next_terminal_and_task`: Zed, then Marley, then Zed again; each action and task follows the layout of the moment |
  | 007 | `script/gates.sh --diff` |
- **Runs.** `cargo nextest run -p marley_workbench`: 50 passed. The seven routing tests five
  times in a row: 7 passed each time.
- **Gate.** `script/gates.sh --diff`: 19 passed, 0 failed, `GATE GREEN [diff]`. Coverage
  1379 of 1379 lines in `marley_workbench` (`routing.rs` 89 of 89).
- **Live drive: not run.** It needs input: keys in the task modal, a right-click for Open in
  Terminal. At 23:48 nothing showed Chad away from the desk: the idle monitor is off
  (stay-awake), so its `lastEventAt` says nothing about input. Workspace 3, which the headless
  output borrows, holds his fullscreen Teams window. The driven tests are the proof; the drive
  is owed, with W3's and W4's, to the next headless session while he is away.
- **Pre-existing failures:** none.

## Phase 4 — Complete (2026-09-22)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket
  · [x] archive · [x] commit.
- **A finding while documenting, fixed before the commit.** The CHANGELOG was to say Zed's rerun
  rules hold in the center, and checking it showed they also hold in the panel. `spawn_task`
  reruns a task in its last terminal wherever it is (`terminals_for_task` searches the panel's
  panes and the center's) and `replace_terminal` reveals that pane. So in the Marley layout, a
  task that last ran in the Zed layout reran in the hidden panel and opened it. Fixed in
  `routing.rs`: the provider keeps a weak handle to its workspace, and in the Marley layout it
  first moves the task's terminals from the panel to the active pane (`move_to_center`, through
  `workspace::move_item`). Tests extended: REQ-001's reruns its task and still has one center
  terminal; REQ-006's reruns, in the Marley layout, a task that last ran in the panel, and the
  terminal comes out to the center. With the move disabled, the switch test fails (2 center
  terminals, not 3). Clippy at deny level clean; `script/gates.sh --diff` rerun: 19 passed, 0
  failed, `GATE GREEN [diff]`, `routing.rs` 142 of 142 lines. The spec's scope and the design
  doc's D2 say so.
- **Documented.** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_workbench.md` (the
  Routing section, the tests, the known limits); `docs/marley/workbench-shell.md` (W5 shipped,
  W5b named, D2's rerun line). No Zed path changed; `Cargo.lock`'s row already covers the new
  `task` dependency, and gate:16 passed.
- **Knowledge appended.**
  - `F-claude-441-a-task-provider-read-the-workspace-inside-its-update-001`
  - `F-claude-441-a-rerun-reopened-the-hidden-terminal-panel-001`
  - `PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001`
  - `L-claude-441-driving-tasks-and-terminal-actions-in-a-gpui-test-001`
  - `AD-claude-441-the-marley-layout-routes-terminals-without-touching-zed-001`
- **Brain.** Consultation `1fe35545e8fc46958c969d14bf7ae040` closed with `brain decide`:
  `decisions/marley-layout-terminals-route-to-the-center-through-a-task-provider-and-capture-listeners-with-no-zed-hunk`,
  follow-up by 2026-10-06.
- **Closed and archived.** TICKET-441 in `tickets/closed/`; this pair in `pipeline/completed/`.
  TICKET-449's source link follows it.
