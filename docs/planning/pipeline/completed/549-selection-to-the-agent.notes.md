# Send the editor's selection to a terminal agent — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-549-selection-to-the-agent.md
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
- **Promotion (2026-09-26, at `cbe6141460`).** Every cited seam re-found by name (the line numbers
  moved): `marley_workbench.rs` `register_action_renderer` and `capture_action` (:318),
  `layout_preset` (:495), `marley_layout` (:513); `browser.rs` `send_pick` (:3748),
  `LastTerminal` (:5528), `track_terminals` (:5537), `reveal_terminal` (:5558), `pick_line`
  (:5786); `rich_input.rs` `Prompt` (:25), `Prompts` (:32), `open` (:57), `send` (:97), no
  `insert`; `agents.rs` `new_agent` (:233), `toggle_modal` (:244), `NewAgentPicker` (:251), its
  delegate (:357); `agent_bar::agent_in` (:129); `AgentEvents::seat` (:30); `mcp::terminals`
  (:363, #574 already calls it through `caller_terminal`); `agent_panel.rs` the handler (:644),
  `format_selection_for_terminal` (:764), `mention_path_for_terminal` (:803);
  `Editor::copy_file_location` (:9097). #508 has not landed, so the Waiting refusal is this
  ticket's. The plan's one real Claude Code run is dropped: no scenario starts the real `claude`
  (L-claude-547-marleys-path-is-the-login-shells-001); the stand-ins carry it. Brain consultation
  1dedb4018fc84987a24353b48b454d94: nothing on this seam.

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
- **Checklist (no TaskCreate in this harness):** REQ-001 · REQ-002 · REQ-003 · REQ-004 · REQ-005 ·
  REQ-006 · REQ-007 · golden set · gate.
- **Scenario:** `script/e2e/549-selection-to-the-agent.sh` (`compositor sway`, not the plan's
  Hyprland: the claude terminal's rail row is clicked). Stand-ins `claude` and `codex`, first on
  the scratch HOME's PATH, print and log each line they read; the `claude` one runs the plugin's
  `event.py` with a UserPromptSubmit and a PermissionRequest on `wait`. A scratch repository with
  `src/auth.txt` (ten lines) and `sub/`. The file opens through the palette's file finder (a
  terminal with the focus takes `ctrl-p`), lines 2 to 4 are selected with `shift-down` twice and
  `shift-end` (the plan's two `shift-down`s end at line 4's start, which the line rule leaves out).
- **Runs.** The first run's `ctrl->` did nothing: `press "CTRL SHIFT" period` does not match the
  binding `ctrl->`; ctrl and the `greater` key do (a diagnostic run through the palette action
  showed the send itself worked). Working through the rich input step showed the send typing
  into a rich input whose terminal sat behind another tab, so `send` brings the terminal to the
  front first (`rich_input::is_open`, `insert` no longer returns a bool): a Code change made in
  Test. The final run: every check passes.
- **Checks (all pass):** nothing in the stand-in's log before Return; `got: @src/auth.txt#L2-4`
  after it (REQ-001, REQ-003's relative case); `got: <repo>/src/auth.txt:2-4` in codex's
  (REQ-002, REQ-003's absolute case, REQ-004); one `got: @src` line in all, since the third send
  went into rich input (REQ-007); after `wait` and a refused send, a Return gives `got: ` empty:
  nothing was typed (REQ-005).
- **Shots, read:**
  - `549-00-selected`: `src/auth.txt` with lines 2 to 4 selected (the status bar: 3 lines, 44
    characters); the rail reads the stand-in's terminal as Claude Code.
  - `549-01-sent`: the claude terminal in front and focused, `@src/auth.txt#L2-4` at the
    stand-in's prompt, not yet read.
  - `549-02-unsent-until-enter`: after Return, `got: @src/auth.txt#L2-4`.
  - `549-03-picker`: "Send the selection to…" over the editor, rows `Codex · repo` (focused
    last, first) and `Claude Code · repo`.
  - `549-04-picked-outside-cwd`: the codex terminal in front, `/…/repo/src/auth.txt:2-4 ` at its
    prompt, absolute since its folder is `sub/`.
  - `549-07-rich-input`: the claude terminal brought forward, `@src/auth.txt#L2-4` in its rich
    input under the output.
  - `549-05-refused`: the claude row reads `waiting · Clean the build / Permission for Bash: rm
    -rf build`; the editor stays in front, and a toast says "Claude Code in repo waits on a
    permission or a question; nothing was sent."
  - `549-06-zed-fallback` (one run, not in the scenario now): both stand-ins ended, `ctrl->` from
    the editor opened Zed's Agent Panel on a new thread (REQ-006).
- **REQ-006 and why it left the scenario:** Zed's handler starts a thread of the Agent Panel's
  last agent, which in the run's copy of the user's profile is the user's own Claude agent: the
  run's log shows Zed's Claude agent creating a session (no prompt was sent, and nothing was left
  running). No scenario may start the real `claude` (L-claude-547-marleys-path-is-the-login-shells-001),
  so the step was checked once and removed; the capture's every no-agent path returns without
  stopping the action, as the review read.
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule
  and did not reload it".
- **Golden set:** 549 added; `just regress`: all 20 passed; the final run's log holds no Zed agent
  server start.
- **Gate:** `script/gates.sh --diff`: 16 passed, 0 failed, `GATE GREEN [diff]`, receipt written.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: send the editor's selection to an agent in a terminal);
  `docs/marley_architecture/marley_workbench.md` (the new section, "Sending the selection to an
  agent"). The plan has no row for the Warp findings; no Zed path.
- **Knowledge appended:** F-claude-549-a-rich-input-behind-another-tab-took-the-reference-001;
  L-claude-549-a-scenario-that-reaches-zeds-agent-panel-starts-the-profiles-last-agent-001;
  L-claude-549-wtype-sends-a-shifted-binding-by-its-symbol-001.
- **Brain:** `rusty-cli brain decide 1dedb4018fc84987a24353b48b454d94` →
  `decisions/marley-sends-the-editors-selection-to-a-terminal-agent-as-a-reference`.
- **Ticket:** closed; the pipeline archived to `completed/`.

## Phase 2 — Code
- **Built.**
  - `send_selection.rs` (new): `init` registers `marley::SendSelectionToAgent` and, through
    `register_action_renderer`, a capture of `agent::AddSelectionToThread` that acts only in the
    Marley layout and only when the send goes ahead, and otherwise lets the action on to Zed's
    handler; a `FocusOrder` global follows the terminals' focus-ins (bounded at 64).
    `send_selection` takes the active item as an `Editor` when it holds the focus, its newest
    selection as a file and lines (`selection_of`, `line_span`), and the agent targets: 0 is an
    error (the palette action's toast; the capture passes the key on), 1 is sent, several open
    `TargetPicker` (Zed's `Picker` in a `ModalView`, rows "agent · project · state").
    `reference` builds `@path#L<a>-<b>` for Claude Code and `path:<a>-<b> ` for the rest, the path
    relative to the agent's folder when the file is under it. `send` defers to after the
    workspace's update, then: rich input open on the terminal takes the text; an agent whose seat
    is Waiting gets nothing and a toast; otherwise the window activated, the terminal revealed
    (`browser::reveal_terminal`) and focused, one `Terminal::paste`, no Enter.
  - `rich_input::insert` (new): the text at the open editor's cursor, and the focus to it.
  - `mcp::terminals` and `browser::reveal_terminal` made `pub(crate)`; the action in `actions!`;
    the module and its `init` in the crate root.
- **Deviations.**
  - The target walk does not use `mcp::terminals`: that reads every workspace entity, and the
    action runs inside the current workspace's update, so `agent_targets` reads the current one
    through the `&Workspace` it is given and the window's others through their entities (the
    window's `MultiWorkspace` through `Workspace::multi_workspace`). `mcp::terminals` stays
    `pub(crate)` for nothing new, so it is put back private.
  - A row's state shows only for a terminal with a seat (Claude Code with Marley's plugin): the
    quiet timer's reading lives in the rail and the close guard, private to each.
  - The plan's single run of a real Claude Code is dropped (Phase 1's promotion note).
- **Review.** The capture's every early return lets the key on: no Marley layout, no focused
  editor, no file, no agent. Re-entrancy: the editor is updated (not the workspace) for its
  selection; the send and the picker's confirm defer past the updates they start in. The
  reference holds no control character (`Terminal::paste` strips ESC anyway). Provenance: the line
  rule follows the behavior of Zed's Copy File Location (1-based, a selection ending at a line's
  start leaves that line out) through the public selection and buffer calls, in `line_span`'s own
  shape; nothing from Warp.
- **Checks:** `cargo check`, `cargo fmt`, `cargo clippy -p marley_workbench --all-targets -- -D
  warnings` (one finding, `needless_pass_by_value` on `send`'s selection, fixed).
