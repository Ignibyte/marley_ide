# Send the editor's selection to a terminal agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-549-selection-to-the-agent.md
- **Pipeline spec:** 549-selection-to-the-agent.spec.md

## Phase 1 — Plan
- **Request:** the Warp second pass's finding 1 (2026-09-25), ranked first there, and Chad's
  answer of 2026-09-26 (a picker when several agents run); drafted in the spec batch of
  2026-09-26 that takes every remaining Orca and Warp finding.
- **Classification / tier:** feature, size S to M: the key and the send are the S, the picker
  and the working-directory rule the M. Marley code only; no Zed path changes (D7).
- **Recall (§18.3):**
  - AD-claude-496-picks-are-staged-and-sent-to-the-last-terminal-001: a pick's line is the
    reference and the tool the content; the send focuses the terminal and presses no Enter. The
    same shape here, with a different target rule.
  - F-claude-515-an-app-dispatch-inside-an-action-found-no-window-001: register the action on
    the workspace and dispatch through the window, never `cx.dispatch_action` from an app-level
    handler.
  - L-claude-450-driving-a-picker-and-a-keymap-in-a-marley-test-001: a modal takes focus in a
    deferred callback; `menu::Confirm` and `menu::SelectNext` reach it. The e2e runner's keys do
    the same from outside.
  - F-claude-496-ctrl-shift-c-in-a-tab-field-opened-the-collab-panel-001: a Zed `!Terminal`
    binding sits at a field's depth. No contest here: the key stays Zed's and the capture sits
    above every binding.
  - DL-cluster-c204c353bd4f (the bracketed-paste guard): `Terminal::paste` strips ESC, and the
    reference holds no control character.
  - Brain: no page on selection as context (searched 2026-09-26).
- **Discovery:**
  - `crates/marley_workbench/src/marley_workbench.rs:66` the `actions!` list; `:269` the
    `observe_new` on `Workspace` with `register_action_renderer` and `capture_action`; `:427`
    `layout_preset`; `:445` `marley_layout`.
  - `crates/marley_workbench/src/browser.rs:3645` `send_pick` (`:3654` reads `LastTerminal`,
    `:3676` the deferred activate, reveal and focus, `:3685` the paste); `:5399` `LastTerminal`;
    `:5409` `track_terminals` (any terminal, on focus-in, the rich input's focus included);
    `:5430` `reveal_terminal`; `:5658` `pick_line`; `:3779` a toast through
    `workspace.show_toast`.
  - `crates/marley_workbench/src/rich_input.rs:25` `Prompt`, `:31` `Prompts` (a global keyed by
    the view's entity id, released at `:88`), `:57` `open`, `:97` `send` (`:110` paste, then
    `\r`), `:140` `element`. No insert exists.
  - `crates/marley_workbench/src/agents.rs:233` `new_agent`, `:244` `toggle_modal`, `:251`
    `NewAgentPicker`, `:357` its `PickerDelegate`.
  - `crates/marley_workbench/src/agent_bar.rs:129` `agent_in`;
    `crates/marley_agent/src/marley_agent.rs:32` `AgentKind` (Claude, Codex, Gemini, OpenCode),
    `:76` `agent_kind_of`.
  - `crates/marley_workbench/src/agent_events.rs:30` `seat`;
    `crates/marley_agent/src/claude_events.rs:303` PermissionRequest and `:353` AskUserQuestion
    both `wait`, so `State::Waiting` covers both.
  - `crates/marley_workbench/src/mcp.rs:350` `terminals` (center panes and the dock of every
    workspace); `crates/workspace/src/workspace.rs:4313` `items_of_type` (center panes only).
  - `crates/editor/src/editor.rs:9097` `copy_file_location`;
    `crates/editor/src/selections_collection.rs:309` `newest`; `crates/editor/src/input.rs:941`
    `Editor::insert`.
  - `crates/agent_ui/src/agent_panel.rs:644` the handler; `:764` `format_selection_for_terminal`;
    `:803` `mention_path_for_terminal`; `:739` the paste; `:9334` the test.
  - `crates/terminal/src/terminal.rs:2593` `paste`; `:3070` `working_directory`; `:3082`
    `foreground_process_command_name`.
  - `assets/keymaps/default-linux.json:143` (`Editor && mode == full`), `:258` (`AcpThread`),
    `:1340` (`Terminal`): `ctrl->`.
  - `crates/gpui/src/window.rs:5946`: the bindings of a keystroke are tried in order until one
    is not propagated.
- **Decisions:** D1 to D7 in the spec.

### Design
- **`send_selection.rs`.** `pub fn init(cx)`: `SendSelectionToAgent` registered on the
  workspace, and `capture_action` for `AddSelectionToThread` in the Marley layout: when the
  focused item is an `Editor` over a file and `agent_targets` is not empty, `send(..)` and
  `cx.stop_propagation()`; otherwise return, so Zed's handler runs.
- **`reference(kind, path, cwd, lines) -> String`** (pure): the path relative to the agent's
  directory by `Path::strip_prefix`, else absolute; Claude gets `@{path}#L{a}-{b}` (`#L{a}` on one
  line), the rest `{path}:{a}-{b} ` (`{path}:{a} `). The lines as `copy_file_location` counts
  them: 1-based, the end line dropped when the selection ends at column 0.
- **`agent_targets(window, cx) -> Vec<Target>`**: every terminal of the window's workspaces
  (`mcp::terminals`' walk, made `pub(crate)`) whose `agent_in` is Some, with the kind, the
  workspace's project name, the seat's status (or the quiet timer's) and the title; ordered by
  last focus. The rail's `shown_at` is private, so a `LastFocused` map beside `LastTerminal`,
  filled by `track_terminals`' focus-ins, gives the order; the Code phase may merge the two.
- **The picker.** `TargetPicker` (`ModalView` over `Picker<TargetDelegate>`), rows like
  `Claude Code · marley_ide · working`; `confirm` sends and dismisses; Escape dismisses.
- **`send(target, text, window, cx)`**: if `rich_input::insert(view, text, window, cx)` returns
  true, done; else refuse when the seat is Waiting (a toast: "Claude Code in marley_ide is
  waiting on a permission; nothing sent"); else the pick's route: activate the window,
  `reveal_terminal`, focus, `terminal.paste(&text)`.
- **`rich_input::insert(view, text, window, cx) -> bool`**: true when the view's prompt is open;
  inserts through `Editor::insert` and focuses the editor.
- **File manifest.** Marley: `crates/marley_workbench/src/send_selection.rs` (new),
  `marley_workbench.rs` (the module, the action, `init`), `rich_input.rs` (`insert`),
  `browser.rs` (`reveal_terminal` made `pub(crate)`), `mcp.rs` (`terminals` made `pub(crate)`).
  Scripts: `script/e2e/549-selection-to-the-agent.sh`. No Zed path.
- **Ledger rows.** None.

### E2E plan

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | the `claude` stand-in in terminal 1; the file opened, lines 2 to 4 selected; `ctrl->` | `549-01-sent`: `@src/auth.txt#L2-4` at the input, terminal 1 focused |
| REQ-001 | Return in terminal 1 | `549-02-unsent-until-enter`: `got: @src/auth.txt#L2-4`; the log has no `got:` before it |
| REQ-002 | `ctrl-~`, `cd sub && codex`; the selection again; `ctrl->` | `549-03-picker`: two rows |
| REQ-002, 003, 004 | `down`, Return | `549-04-picked-outside-cwd`: `/…/repo/src/auth.txt:2-4 ` in the codex terminal |
| REQ-007 | terminal 1 focused, `ctrl-g`; the editor, `ctrl->`, the claude row, Return | `549-07-rich-input`: the reference in the rich input's editor |
| REQ-005 | Escape closes rich input; `wait` and Return to the stand-in (PermissionRequest); the editor, `ctrl->`, the claude row, Return | `549-05-refused`: nothing at the input, the toast |
| REQ-006 | both stand-ins ended (`ctrl-d`); the editor, `ctrl->` | `549-06-zed-fallback`: the Agent Panel with the selection |
| REQ-001, 004 | machine checks | `holds` the log for `got: @src/auth.txt#L2-4`; `mcp_agent terminal-read codex` holds `/src/auth.txt:2-4` |

Not reachable by a scenario: a real Claude Code's handling of the pasted `@` mention (a model's
replies cannot be held steady for shots). Test runs one real Claude Code once in a pty
(L-claude-482's method) to see the reference accepted at its input, and records it.

### Risks
- The capture must never swallow `ctrl->` when it does not send: a terminal selection, an
  AcpThread, an editor with no file. Each path propagates; the review reads every early return.
- Without the plugin's events a paste at a permission prompt answers it (D6). The plugin is the
  fix; the toast that #547's chip offers for an old plugin is the pointer.
- The rows' state words follow the rail's; the scenario reads the agent's name only.
- A worktree agent (#510) has not landed; the `sub/` case stands in for it.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
