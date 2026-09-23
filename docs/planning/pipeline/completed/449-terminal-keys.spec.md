---
pipeline_id: de024bd4-e92d-4579-bb1f-33b1dcd04068
ticket: docs/planning/tickets/open/TICKET-449-terminal-keys.md
status: Phase 4 — Complete PASS
title: Terminal keys in the Marley layout
type: feature
slice: workbench shell W5b
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/441-marley-terminal-routing.spec.md, docs/planning/tickets/open/TICKET-450-new-agent-from-the-keyboard.md]
---

## Title
In the Marley layout the terminal keys work on center terminals: `` ctrl-` `` switches between
the code and the project's terminal, `ctrl-~` opens a new one, and `ctrl-j` stops showing the
bottom Terminal Panel. #441 left these keys on the panel. In the Zed layout they do what
upstream does.

## Scope
### In
- Capture-phase listeners on the workspace's root, beside #441's, for
  `terminal_panel::Toggle` (`` ctrl-` ``), `terminal_panel::ToggleFocus`, and
  `workspace::ToggleBottomDock` (`ctrl-j`, `cmd-j` on macOS). In the Marley layout they stop
  propagation and run one Marley toggle:
  - from anywhere but a focused center terminal, focus the workspace's most recently used
    center terminal, or open one where New Terminal would when it has none;
  - from a focused center terminal, go back to the most recently used center item that is not
    a terminal, and do nothing when there is none.
- `ToggleBottomDock` is caught only while the bottom dock is closed and would show the
  Terminal Panel; otherwise it goes to Zed.
- `ctrl-~` is Zed's `workspace::NewTerminal`, which #441 already routes; this slice proves the
  key end to end.

### Out (explicitly deferred)
- The Marley keymap (workbench-shell D7) and the New Agent chord: TICKET-450. Catching Zed's
  actions needs no keymap; only a binding with no Zed action behind it does.
- `ToggleLeftDock` and `ToggleRightDock`, for a Terminal Panel a user has moved to a side dock.
- The panel when something opens it anyway (Vim's `:!`, an agent login): `` ctrl-` `` still
  toggles center terminals then, and `ctrl-j` closes the open dock as upstream.
- Hiding the panel's actions from the command palette.

## Reference (§20)
- **Warp:** the terminal is the main surface and sits beside the work, never in a drawer
  (`docs/warp_architecture/subsystems/03-terminal-session-core.md`); a key that reaches for
  the terminal lands on a session in the main area.
- **Upstream Zed:** `terminal_panel::Toggle` and `ToggleFocus` show the terminal and focus it,
  and from the terminal hand focus back to the code (`crates/terminal_view/src/terminal_panel.rs:60-70`,
  `Workspace::toggle_panel_focus`); `workspace::ToggleBottomDock` toggles the bottom dock
  (`crates/workspace/src/workspace.rs:4518-4582`). Marley keeps that meaning and moves the
  terminal to the center. The Zed layout keeps every upstream route.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (docks,
  panes, the palette) and `09-vim-keymap-contexts.md` (key contexts and precedence); the Warp
  terminal-session map above.
- **Published material:** Zed's key-binding docs (`docs/src/key-bindings.md:169-172`: a lower
  context wins, and at one level the later binding, so user bindings beat the defaults);
  VS Code's Toggle Terminal (`` ctrl+` ``), which also shows and focuses the terminal.
- **Code we already ship.** gpui and #441's routing own this seam; no keymap change is needed.
  - Zed's defaults bind all three in the Workspace context on every platform:
    `default-linux.json:657` (`ctrl-~`), `:667` (`` ctrl-` ``), `:680` (`ctrl-j`); macOS
    `:716`, `:722`, `:735` (`cmd-j`); Windows `:654` (`` ctrl-shift-` ``), `:660`, `:673`. No
    Terminal-context binding shadows them.
  - The capture phase through `Workspace::register_action_renderer` and `capture_action`, as in
    `marley_workbench::routing` (#441); dispatch stops once a listener stops propagation
    (`crates/gpui/src/window.rs:6309-6330`).
  - `Workspace::recent_active_item_by_type` (`workspace.rs:2959`) and `Workspace::activate_item`
    (`:5559`); `Pane::activation_history` (`pane.rs:827`) for the last non-terminal item.
  - `Dock::is_open` (`dock.rs:513`), `active_panel_index` (`:574`) and `panel_index_for_type`
    (`:527`). `toggle_dock` shows the dock's active panel, or with none active its first
    enabled one (`workspace.rs:4545-4553`, `dock.rs:552`).
  - `KeymapFile::load_asset_allow_partial_failure` (`crates/settings/src/keymap_file.rs:204`)
    loads Zed's real default keymap in a test, keeping the bindings whose actions are linked.

## UI proof
UI-AFFECTING.
- **Driven tests** (the #441 harness: real shells, the panel loaded through
  `TerminalPanel::load`): in the Marley layout, Toggle, ToggleFocus and a closed bottom dock's
  toggle open or focus a center terminal and return to the code, with the Terminal Panel
  closed; with Zed's default keymap bound, `` ctrl-` ``, `ctrl-~` and `ctrl-j` do the same from
  simulated keystrokes; in the Zed layout the actions open the panel and the dock as upstream.
- **Live drive:** in the Marley layout press `` ctrl-` `` from an editor and from the terminal,
  then `ctrl-~` and `ctrl-j`; screenshot each, confirming the bottom dock never opens. It needs
  keys, so it runs only while Chad is away from the desk; otherwise the Test phase records why.

## Locked-In Decisions
- D1 — Catch the actions, not the keys. Zed's bindings, a user's own bindings to these
  actions, the palette and the menus all route, with no keymap asset and no Zed touchpoint
  (AD-claude-441-the-marley-layout-routes-terminals-without-touching-zed-001). D7's keymap
  waits for a binding with no Zed action behind it: TICKET-450.
- D2 — One toggle for all three actions: from outside a center terminal, the most recently
  used center terminal, or a new one; from a focused center terminal, the most recently used
  center item that is not a terminal.
- D3 — `ToggleBottomDock` is caught only while the dock is closed and would show the Terminal
  Panel: its active panel, or with none active, the Terminal Panel as the dock's first panel.
  The check reads the dock and calls into no panel, since the listener runs inside the
  workspace's update
  (PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001).
- D4 — The layout is read at each call, as in #441.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the layout is `marley` and no center terminal has focus, `terminal_panel::Toggle` shall focus the workspace's most recently used center terminal, or open a center terminal where New Terminal would when there is none, and the Terminal Panel shall stay closed | driven tests |
| REQ-002 | WHILE the layout is `marley` and a center terminal has focus, `terminal_panel::Toggle` shall focus the most recently used center item that is not a terminal, and shall change nothing when there is none | driven tests |
| REQ-003 | WHILE the layout is `marley`, `terminal_panel::ToggleFocus` shall do what `terminal_panel::Toggle` does | driven test |
| REQ-004 | WHILE the layout is `marley` and the bottom dock is closed and would show the Terminal Panel, `workspace::ToggleBottomDock` shall do what `terminal_panel::Toggle` does, and the dock shall stay closed; WHILE the dock is open, it shall close the dock as upstream | driven tests |
| REQ-005 | WHILE the layout is `marley`, Zed's default `` ctrl-` ``, `ctrl-~` and `ctrl-j` shall reach these routes from the keyboard, with the Terminal Panel closed | driven test with Zed's default keymap bound and simulated keystrokes |
| REQ-006 | WHILE the layout is `zed`, the three actions shall do what upstream does: Toggle and ToggleFocus open and focus the Terminal Panel, and ToggleBottomDock opens the bottom dock | driven tests |
| REQ-007 | The diff gate shall be green, with no path outside the Marley-owned set changed | `script/gates.sh --diff`, gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the three listeners and the toggle in `marley_workbench::routing`; fmt and
  clippy clean; a review of the diff for re-entrancy and for keys that should still reach Zed.
- **P3 Test** — write and run the tests; the live drive; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
