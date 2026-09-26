---
pipeline_id: edef95de-eb7a-44af-99a9-320b4e0aa008
ticket: docs/planning/tickets/open/TICKET-549-selection-to-the-agent.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Send the editor's selection to a terminal agent"
type: feature
slice: prong 2 with prong 1 (the editor feeds the CLI agent in a terminal); the Warp second pass's finding 1
references: [docs/planning/design-notes/warp-second-pass-2026-09-25.md, docs/planning/pipeline/queued/508-approvals-inbox.spec.md, docs/planning/pipeline/queued/520-terminal-identity.spec.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md]
---

## Title
`marley: send selection to agent` puts the editor's selection into a CLI agent's prompt in a
Marley terminal as a typed reference: `@src/auth.rs#L12-40` for Claude Code, the form its
JetBrains plugin inserts, relative to the agent's working directory; `src/auth.rs:12-40 ` for
Codex, Gemini CLI and OpenCode, the form Zed pastes into its own terminal threads. One paste, no
Enter, the terminal focused so the question is typed after it. In the Marley layout `ctrl->`,
Zed's Add to Agent Thread, takes this route while a CLI agent runs in a terminal of the window:
one agent takes the reference, several open a picker (Chad, 2026-09-26), none leaves the key to
Zed's Agent Panel.

## Scope
### In
- `crates/marley_workbench/src/send_selection.rs` (new): the action, the reference text (a pure
  function of the path, the agent's working directory, the line range and the agent kind), the
  target rule, the picker and the send.
- `marley::SendSelectionToAgent` in `actions!(marley, [...])`, registered on the workspace and in
  the palette; and a capture of `agent::AddSelectionToThread` at the workspace's root in the
  Marley layout, as `layout_preset` captures Zed's layout actions (`marley_workbench.rs:269`,
  `:427`), which runs the send when the focus is in a file's editor and a terminal agent runs,
  and propagates otherwise.
- The target: the terminals of the window's workspaces, center panes and the terminal dock,
  whose foreground program is a known agent (`agent_bar::agent_in`); with several, a picker
  (Zed's `Picker` in a `ModalView`, the shape of `agents.rs`'s New Agent picker) lists each with
  its project, agent and state, newest focus first.
- The send: `Terminal::paste` with the window activated and the terminal revealed and focused,
  as a pick's Send does (`browser.rs:3645`); into rich input when it is open on that terminal,
  through a new `rich_input::insert`.
- The refusal: while the target's seat is Waiting (a permission prompt or an AskUserQuestion,
  #519), nothing is typed and a toast says so.
- `script/e2e/549-selection-to-the-agent.sh`.

### Out (explicitly deferred)
- Hunks from the diff view or Code Review (Warp's second source), and the terminal's own text
  selection to a terminal agent (Zed's menu item keeps sending it to the Agent Panel); #555
  sends blocks.
- A "Send to Agent" item in the editor's right-click menu
  (`crates/editor/src/mouse_context_menu.rs:294`, a Zed touch): the key and the palette first.
- The pull side (`editor_selection` and `editor_diagnostics` on Marley's MCP server, plan D9)
  and Claude Code's IDE protocol (the lock file, the WebSocket).
- Each CLI's own mention syntax beyond Claude Code's: Codex, Gemini CLI and OpenCode get Zed's
  `path:a-b ` form until each is checked in a Marley terminal.
- Several selections (multi-cursor): the newest selection only, as Zed's Copy File Location
  takes it.

## Reference (§20)
- **Warp:** selection as context. With a CLI agent running in a tab, `Ctrl+Shift+L` puts the
  editor's selection into the agent's input as the relative path, the line numbers and the text,
  not submitted (docs.warp.dev/agents/local-agents/agent-context/selection-as-context/; "Attach
  code as context" for every CLI in docs.warp.dev/agents/cli-agents/overview/). Marley matches
  the gesture and the unsent reference; it keeps `ctrl->` since `Ctrl+Shift+L` is Zed's Select
  All Matches on Linux, and sends a reference rather than the text since Claude Code reads the
  file itself from an `@` mention.
- **Upstream Zed:** `agent::AddSelectionToThread` (`crates/zed_actions/src/lib.rs:600`; the
  handler at `crates/agent_ui/src/agent_panel.rs:644`) goes into an Agent Panel thread as a
  mention, or into the panel's own terminal thread as `path:2-3 ` relative to the terminal's
  working directory, pasted with no Enter (`format_selection_for_terminal` `:764`,
  `mention_path_for_terminal` `:803`, the paste `:739`; the test at `:9334` asserts
  `file.rs:2-3 `). Marley keeps that behavior for the panel and takes the terminal form for the
  other CLIs; only the target changes.
- **Claude Code**, published docs: the JetBrains plugin's file reference shortcut inserts
  `@File#L1-99` (code.claude.com/docs/en/jetbrains, "File reference shortcuts"), the form Marley
  types for Claude Code.

### Prior art
- **Behavior maps and research.** The Warp second pass, finding 1: its "What Marley would do"
  and its hard parts (the target when several agents run, paths relative to the agent's
  directory, a paste into a permission prompt answers it). `docs/warp_architecture/` has no page
  on selection as context; its `crates/local_control.md:92` notes an `input.insert` verb that
  injects into the input editor, the same act from outside. #508's D6
  (`508-approvals-inbox.spec.md:122`): nothing is pasted into a terminal whose agent waits on a
  permission. #510's worktree agent runs in its worktree, not the editor's project root, which is
  why the path is relative to the agent's own directory. #520 gives terminals a stable id later;
  this ticket needs none.
- **Published material.** Claude Code's JetBrains page for the `@File#L1-99` form and Warp's
  selection-as-context page. Claude Code's `@` mention reads the file, so the reference is
  enough; the other CLIs' mention syntaxes are unchecked (Out).
- **The code we already ship.** Zed's handler builds the same reference (`agent_panel.rs:764`,
  `:803`; `acp_thread::mention::line_range_suffix` at `crates/acp_thread/src/mention.rs:851`),
  but its selection types are `pub(crate)` (`crates/agent_ui/src/completion_provider.rs:45`), so
  Marley reads the selection itself as `Editor::copy_file_location` does
  (`crates/editor/src/editor.rs:9097`: `selections.newest`, the buffer range, 1-based lines, the
  end line dropped at column 0) with `File::path` and `Project::absolute_path`. The pick's Send
  (`browser.rs:3645`; `pick_line` `:5658`; `reveal_terminal` `:5430`; `Terminal::paste`
  `crates/terminal/src/terminal.rs:2593`, bracketed when the program asked) is the send path;
  `LastTerminal` (`browser.rs:5399`) is any terminal, so the target rule is new.
  `agent_bar::agent_in` (`agent_bar.rs:129`) and `Terminal::working_directory`
  (`terminal.rs:3070`) give the agent and its directory; `AgentEvents::seat`
  (`agent_events.rs:30`) with `State::Waiting` (`crates/marley_fleet/src/session.rs:13`), set by
  PermissionRequest and AskUserQuestion (`claude_events.rs:303`, `:353`), gives the refusal.
  Zed's `Picker` (`crates/picker/src/picker.rs:164`) behind `Workspace::toggle_modal` as
  `agents.rs:244` uses it (`NewAgentPicker` `:251`, its delegate `:357`) gives the picker.
  `layout_preset` (`marley_workbench.rs:427`) captures a Zed action at the workspace root in the
  Marley layout, and gpui tries the bindings of a keystroke in turn until one is not propagated
  (`crates/gpui/src/window.rs:5946`), so the capture leaves Zed's handler its turn.
  `Editor::insert` (`crates/editor/src/input.rs:941`) puts text into rich input's editor
  (`rich_input.rs:57` `open`; the `Prompts` global `:31`). `mcp.rs:350` `terminals` walks every
  workspace's center panes and dock. Does a crate we build own this seam? `agent_ui` owns the
  panel's half and keeps it; the terminal-agent half is Marley's, from the pieces above.

## UI proof
UI-AFFECTING: the reference at the agent's prompt, the picker, a toast.
`script/e2e/549-selection-to-the-agent.sh` (Hyprland, keys only). Setup: a scratch repository
with `src/auth.txt` (ten numbered lines) and `sub/`; a HOME whose `.bashrc` is the scenario's;
a stand-in `claude` and a stand-in `codex` first on the PATH, Python scripts that print each line
they read as `got: <line>` (the rail names a Python script by its file), the `claude` one also
running the plugin's `event.py` with a PermissionRequest payload when it reads `wait`. Steps:
`claude` in the first terminal; `ctrl-p`, `auth`, Return opens the file; `ctrl-home`, `down`,
`shift-down` twice selects lines 2 to 4; `ctrl->`: the reference at the stand-in's input,
echoed, the terminal focused (`549-01-sent`); Return: `got: @src/auth.txt#L2-4`
(`549-02-unsent-until-enter`). `ctrl-~` opens a second terminal, `cd sub && codex`; the same
selection and `ctrl->`: the picker with two rows (`549-03-picker`); `down`, Return:
`/…/repo/src/auth.txt:2-4 ` at the codex stand-in's input, absolute since the file is outside
`sub/` (`549-04-picked-outside-cwd`). In the claude terminal `ctrl-g` opens rich input; from the
editor `ctrl->`, the claude row, Return: the reference in the rich input's editor
(`549-07-rich-input`); Escape. `wait` and Return to the claude stand-in; from the editor `ctrl->`,
the claude row, Return: nothing typed and the toast (`549-05-refused`). Both stand-ins ended
with `ctrl-d`; `ctrl->` from the editor: Zed's Agent Panel with the selection
(`549-06-zed-fallback`). Machine checks: the run log holds `got: @src/auth.txt#L2-4` once, and
`mcp_agent terminal-read codex` holds `/src/auth.txt:2-4`.

## Locked-In Decisions
- D1: A reference, not the text. Claude Code reads a file from an `@` mention itself, which keeps
  the paste short and the file current; Warp sends the text because its agent has no file to read.
  With no selection (a caret), the reference is the file alone: `@src/auth.rs`.
- D2: Claude Code gets `@<path>#L<a>-<b>` (its JetBrains plugin's form); every other agent gets
  `<path>:<a>-<b> ` with a trailing space, the form Zed pastes into its terminal threads, until
  each CLI's syntax is checked. One line gives `#L<a>` and `:<a> `.
- D3: The path is relative to the agent's working directory (`Terminal::working_directory`, the
  foreground process's) when the file lies under it, and absolute otherwise: a worktree agent
  (#510) runs in its worktree, not the editor's project root.
- D4: The target rule (Chad, 2026-09-26): one agent terminal in the window takes the reference;
  several open a picker; none leaves `ctrl->` to Zed's `AddSelectionToThread`, and the palette's
  action shows a toast. Open question 2 (Zed's key in the Marley layout) got no answer and keeps
  its default, yes: the capture runs in the Marley layout only, like the layout actions.
- D5: The send is a pick's send: `Terminal::paste` (bracketed when the program asked, as Claude
  Code does), the window activated, the terminal revealed and focused, no Enter. While rich input
  is open on the target the reference is inserted at its cursor and the send ends there.
- D6: Nothing is typed into a terminal whose seat is Waiting (#508's D6): a paste would answer the
  permission prompt or the question. An agent without a seat (no plugin events, or not Claude
  Code) gets the paste; the quiet timer cannot tell a prompt from an idle agent.
- D7: No Zed hunk: the capture sits at the workspace root, the Marley keymap is untouched
  (`ctrl->` stays Zed's binding), and the picker and the send live in `marley_workbench`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses `ctrl->` in a file's editor while one CLI agent runs in a terminal of the window, the system shall paste `@<path>#L<a>-<b>` (Claude Code) at that agent's prompt without Enter and focus that terminal. | Shot `549-01-sent`; shot `549-02-unsent-until-enter` and the log: `got:` appears only after the scenario's own Return |
| REQ-002 | WHEN two or more agent terminals are in the window, the system shall open a picker listing each with its project, agent and state, and send to the one confirmed. | Shots `549-03-picker`, `549-04-picked-outside-cwd` |
| REQ-003 | WHEN the file lies outside the agent's working directory, the system shall send the absolute path; WHEN inside, the path relative to that directory. | Shot `549-04-picked-outside-cwd` (absolute); shot `549-01-sent` (relative) |
| REQ-004 | WHEN the target is not Claude Code, the system shall send `<path>:<a>-<b> `. | Shot `549-04-picked-outside-cwd`; `mcp_agent terminal-read codex` |
| REQ-005 | WHILE the target's seat waits on a permission or a question, the system shall paste nothing and show a toast naming the terminal. | Shot `549-05-refused` |
| REQ-006 | WHEN no CLI agent runs in a terminal of the window, `ctrl->` shall reach Zed's Add to Agent Thread as before. | Shot `549-06-zed-fallback` |
| REQ-007 | WHILE rich input is open on the target, the system shall insert the reference into it instead of the terminal. | Shot `549-07-rich-input` |
| REQ-008 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion re-verify the
  seams (the working tree had uncommitted edits in `marley_workbench` on 2026-09-26) and check
  whether #508 has landed its refusal in `send_pick`, to share one check.
- **P2 Code:** `send_selection.rs`, the action, the capture, the picker, `rich_input::insert`,
  the toast; fmt and clippy clean; a review of the diff against each REQ (§18.1: the capture
  propagates in every case it does not handle).
- **P3 Test:** write and run the scenario and read every shot; `just regress`;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_workbench.md`; the plan's slice
  status; the ledger capture; close the ticket, archive, commit.
