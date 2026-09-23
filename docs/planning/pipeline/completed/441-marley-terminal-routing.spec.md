---
pipeline_id: 17b8633d-11d2-4b4b-944e-714d09fc12da
ticket: docs/planning/tickets/open/TICKET-441-marley-terminal-routing.md
status: Phase 4 — Complete PASS
title: Terminal routing in the Marley layout
type: feature
slice: workbench shell W5
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/438-marley-layout-and-rail.spec.md, docs/planning/tickets/open/TICKET-449-marley-keymap.md]
---

## Title
In the Marley layout nothing opens the bottom Terminal Panel: tasks, New Terminal and Open in
Terminal land in center terminals. In the Zed layout each behaves exactly as upstream.
Narrowed at promotion (2026-09-22): the Marley keymap is TICKET-449; the first-show terminal
and the Panel Layout presets move to W6 (#442).

## Scope
### In
- A Marley `TerminalProvider`, installed on each workspace when its Terminal Panel is added
  (`workspace::Event::PanelAdded`), after the panel has installed its own. In the Marley
  layout it sets the task's `reveal_target` to `Center`; in the Zed layout it leaves the task
  alone. Either way it runs the task through `TerminalPanel::spawn_task` and resolves the exit
  status the trait promises (`Terminal::wait_for_completed_task`). In the Marley layout it
  first moves the task's terminals out of the panel, so a rerun replaces them in the center
  (added at Complete; see the notes).
- `workspace::NewTerminal` and `workspace::OpenTerminal` caught in the capture phase
  (`Workspace::register_action_renderer`, `capture_action`) in the Marley layout. Each opens
  a center terminal: New Terminal in the workspace's default working directory, Open in
  Terminal in the directory it names. Both honour `local`, send errors to a prompt, and stop
  propagation, so the panel never sees the action. In the Zed layout both propagate untouched.

### Out (explicitly deferred)
- The Marley keymap (`` ctrl-` ``, `ctrl-~`, a New Agent chord): TICKET-449.
- The first-show terminal and Zed's Panel Layout presets: W6 (#442), carried in its notes.
- Vim's `:!` (hard-codes the dock, `crates/vim/src/command.rs:2464-2486`) and external agent
  login terminals (they call `TerminalPanel::spawn_task` directly); both stay in the panel.
- Debugger terminals (they live in the debug panel).
- Hiding the Terminal Panel's own toggle actions from the command palette.

## Reference (§20)
- **Warp:** the terminal is the main surface and commands run in terminal sessions, never in a
  secondary drawer (`docs/warp_architecture/subsystems/03-terminal-session-core.md`; the
  observed layout in `docs/planning/design-notes/session-tabs-vs-sidebar.md`).
- **Upstream Zed:** the task pipeline's `RevealTarget::Center` semantics and the
  `TerminalProvider` seam (`crates/workspace/src/tasks.rs:101-233`,
  `crates/terminal_view/src/terminal_panel.rs:632-733`), kept as Zed does them; the Zed layout
  keeps every default route.

### Prior art
- **Behavior maps:** the Warp terminal-session map above;
  `docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md` (the task sink and reveal
  model).
- **Published material:** Zed's task docs (`docs/src/tasks.md`: `reveal_target`), the keymap
  docs (`docs/src/key-bindings.md`: precedence, `null` unbinds).
- **Code we already ship:** `Workspace::set_terminal_provider` (`workspace.rs:3340`) and the
  `TerminalProvider` trait (`:328-335`); `TerminalPanel::spawn_task`; `RevealTarget`;
  `Workspace::register_action_renderer` (`workspace.rs:8492-8498`) with gpui's capture phase
  (`crates/gpui/src/window.rs:6309-6330`).
- **Re-swept at promotion (2026-09-22).** None of these crates changed since the queue.
  - The panel installs its provider in `TerminalPanel::load` (`terminal_panel.rs:245-249`),
    before its caller adds it. A panel built with `TerminalPanel::new` (tests) installs none,
    so the Marley provider is the one under test either way.
  - `spawn_task` sends `RevealTarget::Center` through `add_center_terminal`
    (`terminal_panel.rs:715-730`). Zed's provider, a private struct, awaits the new
    terminal's `wait_for_completed_task`.
  - The panel's `new_terminal` opens in the center only when the focused center item is a
    terminal (`:736-775`). `open_terminal` always uses the panel (`:609-630`).
  - `div.capture_action` calls its listener in the capture phase, and dispatch stops once
    one stops propagation (`window.rs:6309-6330`).
  - `Workspace::spawn_in_terminal` (`tasks.rs:222`) is the public way into the provider.
  - Zed's own tests drive real shells (`allow_parking`, an `echo` task,
    `terminal_panel.rs:2874-2906`).

## UI proof
UI-AFFECTING.
- **Driven tests** on real shells, as Zed's panel tests run them: a task in each layout lands in
  the center or in the dock. New Terminal and Open in Terminal in the Marley layout add center
  terminals, the second in the named directory, and leave the panel closed. In the Zed layout
  both reach the panel. A layout switch changes the routing on the next call.
- **Live drive:** in the Marley layout run a task from the task modal and use "Open in
  Terminal" on a folder; screenshot each, confirming the bottom dock never opens. It needs
  input, so it runs only while Chad is away from the desk; otherwise the Test phase records
  why.

## Locked-In Decisions
- D1 — Routing reads the layout on every call; nothing is installed or removed on a layout
  switch.
- D2 — The Terminal Panel stays loaded (tasks and agent logins need it).
- D3 — The provider goes in on `PanelAdded`, which comes after the panel's own `load` has
  installed Zed's provider, and it runs every task through the panel, so only the reveal
  target differs between the layouts.
- D4 — A capture handler stops propagation only when it handles the action (the Marley
  layout); otherwise it returns with propagation on
  (`PR-claude-input-handler-overlay-arms-must-stop-propagation-001`, in spirit).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the layout is `marley`, a task spawned through the workspace shall open in a center terminal, and the Terminal Panel shall gain no terminal | driven test with a real shell |
| REQ-002 | WHILE the layout is `zed`, a task shall follow its own `reveal_target` as upstream: a dock task opens in the Terminal Panel | driven test |
| REQ-003 | WHILE the layout is `marley`, `workspace::NewTerminal` shall open a center terminal, and the Terminal Panel shall stay closed with no new terminal | driven test |
| REQ-004 | WHILE the layout is `marley`, `workspace::OpenTerminal` shall open a center terminal whose shell starts in the requested directory, and the Terminal Panel shall stay closed | driven test |
| REQ-005 | WHILE the layout is `zed`, `workspace::NewTerminal` and `workspace::OpenTerminal` shall open in the Terminal Panel as upstream | driven test |
| REQ-006 | WHEN the layout changes, the next task or terminal action shall follow the new layout with nothing reinstalled | driven test across a switch |
| REQ-007 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — the design: the provider, the capture handlers, the driven tests.
- **P2 Code** — the provider and the capture handlers in a `routing` module of
  `marley_workbench`, installed from `init`. The review of the diff checks for capture-phase
  side effects on other panes and for the provider install order.
- **P3 Test** — tests, `script/gates.sh --diff`, the live drive.
- **P4 Complete** — CHANGELOG, crate note, ledger, close, archive, commit.
