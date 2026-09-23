---
pipeline_id: 17b8633d-11d2-4b4b-944e-714d09fc12da
ticket: docs/planning/tickets/open/TICKET-441-marley-terminal-routing.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Terminal routing and keys in the Marley layout
type: feature
slice: workbench shell W5
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/queued/438-marley-layout-and-rail.spec.md]
---

## Title
In the Marley layout nothing opens the bottom Terminal Panel: tasks, New Terminal, Open in
Terminal and the terminal keys all land in center terminals, and a project opened with no
terminal gets one at its root. In the Zed layout every one of those behaves exactly as
upstream.

## Scope
### In
- A Marley `TerminalProvider` installed on each workspace after the Terminal Panel installs
  its own (`workspace::Event::PanelAdded`). In the Marley layout it sets `reveal_target` to
  `Center` and hands the task to `TerminalPanel::spawn_task`; in the Zed layout it hands the
  task over untouched.
- `workspace::NewTerminal` and `workspace::OpenTerminal` caught in the capture phase (through
  `Workspace::register_action_renderer`) in the Marley layout and turned into a center
  terminal at the same working directory; in the Zed layout they propagate.
- The Marley keymap: a JSON file in the crate, parsed with `KeymapFile::load`, tagged
  `KeybindSource::Default`, bound from one line at the end of `load_default_keymap` in
  `crates/zed/src/zed.rs` (a touchpoint). First bindings: `` ctrl-` `` to
  `marley_workbench::ToggleTerminal`, `ctrl-~` to `marley_workbench::NewCenterTerminal`, and a
  New Agent chord; each Marley action does the Zed action it shadows when the layout is Zed.
- A project whose workspace has no center terminal when it is first shown in the Marley
  layout gets one at its root.

### Out (explicitly deferred)
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
  (`crates/gpui/src/window.rs:6309-6330`); `KeymapFile::load`
  (`crates/settings/src/keymap_file.rs:258`) and `KeybindSource`; `load_default_keymap`
  (`zed.rs:2344-2376`).

## UI proof
UI-AFFECTING.
- **Driven tests:** a task spawned in each layout lands in the center or the dock; New
  Terminal and Open in Terminal in the Marley layout add center terminals at the right cwd and
  leave the panel closed; each Marley action falls back in the Zed layout; a project shown
  with no terminal gets one; the Marley bindings resolve after a keymap reload.
- **Live drive:** in the Marley layout run a task from the task modal, press `` ctrl-` `` and
  `ctrl-~`, and use "Open in Terminal" on a folder; screenshot each, confirming the bottom
  dock never opens.

## Locked-In Decisions
- D1 — Routing reads the layout on every call; nothing is installed or removed on a layout
  switch.
- D2 — The Terminal Panel stays loaded (tasks and agent logins need it).
- D3 — The Marley keymap loads through one line in `load_default_keymap`, below the user's
  keymap and above Zed's defaults at equal depth.
- D4 — Terminal-context bindings on unmodified keys are not added
  (`PR-claude-unmodified-terminal-chords-yield-to-the-pty-001`).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the layout is `marley`, a task spawned through the workspace shall open in a center terminal | driven test |
| REQ-002 | WHILE the layout is `zed`, a task shall follow its own `reveal_target` as upstream | driven test |
| REQ-003 | WHILE the layout is `marley`, `workspace::NewTerminal` and `workspace::OpenTerminal` shall open a center terminal (at the requested directory for `OpenTerminal`) and the Terminal Panel shall stay closed | driven tests |
| REQ-004 | WHILE the layout is `zed`, those actions shall reach the Terminal Panel as upstream | driven test |
| REQ-005 | The Marley bindings shall be active after every keymap reload and shall lose to a user binding on the same keys | driven test through the zed keymap-reload path |
| REQ-006 | WHEN `marley_workbench::ToggleTerminal` runs in the Zed layout, it shall do what `terminal_panel::Toggle` does | driven test |
| REQ-007 | WHEN a project is first shown in the Marley layout with no center terminal, one shall open at its root | driven test |
| REQ-008 | The keymap hook shall be one line with a `Marley:` comment and a ledger row, and the diff gate shall be green | review, gate:16, `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — the design: the provider wrapper, the capture handlers, the keymap asset and
  its loader, the first-show hook, the reload test path.
- **P2 Code** — provider, capture, keymap, the zed.rs line, the auto terminal; the review of
  the diff checks capture-phase side effects on other panes, provider install ordering, double
  terminals on first show and key collisions with Zed's defaults.
- **P3 Test** — tests, `script/gates.sh --diff`, the live drive.
- **P4 Complete** — CHANGELOG, crate note, ledger, close, archive, commit.
