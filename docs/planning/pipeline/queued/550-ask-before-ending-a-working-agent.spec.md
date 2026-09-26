---
pipeline_id: 1a5be75c-9381-485f-9d44-22730dfd2119
ticket: docs/planning/tickets/open/TICKET-550-ask-before-ending-a-working-agent.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Ask before a close or a quit ends a working agent, and hold the closed terminal"
type: feature
slice: prong 2 (the agents Marley hosts survive Marley's own gestures); the Warp second pass's finding 2 with the Orca second pass's finding 1
references: [docs/planning/design-notes/warp-second-pass-2026-09-25.md, docs/planning/design-notes/orca-second-pass-2026-09-25.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/queued/540-session-resume-after-restart.spec.md]
---

## Title
A close or a quit that would end a working agent asks first and names the agents; an idle agent
closes without asking; a working terminal closed anyway is held, PTY and all, for
`marley.undo_close_seconds` (60), and Undo or `ctrl-shift-t` brings it back with its agent still
running (Chad, 2026-09-26: "so both"). One guard hook in Zed's pane and workspace close paths
serves the rail's Close, the tab's close, `ctrl-shift-w`, the window's close and the quit; a
logout, a shutdown or a signal is never held up.

## Scope
### In
- `crates/marley_workbench/src/close_guard.rs` (new): the guard, the question, the hold list,
  the undo and the two settings.
- **Which terminals count.** A terminal whose foreground program is a known agent and which is
  working: its seat (#519) in `Working`, `Starting` or `Waiting`; with no seat, the quiet
  timer's `Working` (output within the last 2 s). `Idle`, `Error`, `Done` and the quiet timer's
  `Waiting` close without asking.
- **The question.** For a tab close (every path through `Pane::close_items`, the rail's Close
  included): `Close Claude Code in marley_ide? It is working.` with `Close`, `Show`, `Cancel`.
  For the window's close and the quit (`Workspace::prepare_to_close`): `Quit Marley? 2 agents
  are working:` (or `Close this window?`) and one line per agent,
  `marley_ide · Claude Code · working`, with `Quit` (or `Close window`), `Show`, `Cancel`. Show
  cancels and brings the first named terminal to the front. Nothing asks when no listed terminal
  is working, when `marley.ask_before_ending_a_working_agent` is false, or while the guard
  already has a question open (a second request is answered as cancelled).
- **The hold.** When a working terminal closes (after Close, or with the question off), the
  guard keeps its `Entity<TerminalView>` in a hold list before the pane removes it, with the
  pane and the workspace it came from and a timer of `marley.undo_close_seconds`, and shows a
  toast `Closed Claude Code in marley_ide` with Undo. Undo, or `marley::UndoCloseTerminal`
  (`ctrl-shift-t` in `Pane`, ahead of Zed's Reopen Closed Item, which propagates when nothing
  is held), puts the newest held view back in its pane (or the active pane) and focuses it. At
  the deadline the entry drops and the PTY ends as it would have. A quit ends held terminals
  too; the quit's question does not count them.
- **The Zed hunks** (§14, additive, each with its ledger row): `crates/workspace/src/pane.rs`
  `close_items` calls a `MarleyCloseGuard` global, when set, with the items about to close and
  awaits its answer before Zed's own dirty-item flow; `crates/workspace/src/workspace.rs`
  `prepare_to_close` calls it with the intent and every item of the workspace before
  `save_all_internal`. The hook's type lives in `crates/workspace` beside the calls, as
  `MarleyTerminalFooter` lives in `terminal_view`.
- Settings: `marley.ask_before_ending_a_working_agent` (bool, true) and
  `marley.undo_close_seconds` (u64, 60; 0 turns the hold off) in `MarleySettingsContent`,
  `default.json`, `MarleySettings` and the Marley settings page's Agents section.
- `script/e2e/550-ask-before-ending-a-working-agent.sh`.

### Out (explicitly deferred)
- A plain command still running (a dev server, a build): Chad's answer names agents, and Warp
  asks for any running process. The guard's list is one function, so a later slice adds running
  blocks behind a setting.
- Zed's `confirm_quit` and its "Are you sure you want to quit?": the two questions stack when
  both are on.
- A "Don't ask again" button: the setting is the switch (gpui's prompt has no checkbox).
- Rail rows for held terminals; a countdown on the toast.
- Session resume (#540) for what a quit does end; remote terminals whose shell survives on the
  host (#543).
- Orca's probe of live child processes (`probePtyRunningWork`): Marley reads the agent's state,
  not the process tree.

## Reference (§20)
- **Warp:** the quit warning while a session runs a process, with quit, show the running
  sessions, cancel and don't ask again (docs.warp.dev/terminal/more-features/quit-warning/); the
  close-session confirmation (`should_confirm_close_session`, changelog 2025.01.08); a closed
  tab reopened within `[general.undo_close] grace_period = 60`
  (docs.warp.dev/terminal/windows/tabs/, Tab Restoration;
  docs.warp.dev/terminal/settings/all-settings/); since 2026.07.03 a logout, shutdown or update
  is never blocked (changelog). Marley asks for working agents rather than any process (Chad's
  scope), keeps Warp's 60 s as the hold's default, and holds only a close made in Marley.
- **Orca:** the tab guard asks only for a close the user made, never a bulk or lifecycle close;
  the quit prompt names what would die (`orca-second-pass-2026-09-25.md`, finding 1:
  `running-terminal-close-guard.ts`, `window-close-running-work.ts`). Marley names each agent
  with its project and state.
- **Upstream Zed:** `Workspace::prepare_to_close` is the one path for a quit, a window close and
  a project replace (`crates/workspace/src/workspace.rs:720` `CloseIntent`, `:3643`), and
  `Pane::close_items` for a tab (`crates/workspace/src/pane.rs:1954`); their prompts
  (`save_all_internal` `:3880`, `save_item` `:2246`) stay, and Marley's question comes before
  them. `ctrl-shift-t` is `pane::ReopenClosedItem` (`assets/keymaps/default-linux.json:694`),
  which reopens by path and cannot restore a terminal; Marley's undo takes the key only while
  something is held.

### Prior art
- **Behavior maps and research.** The two second passes above; #519's states; #540's D4
  (`540-session-resume-after-restart.spec.md:123`: a Marley quit closes the terminal under
  Claude Code, the case to resume) and its scenario, which quits with an Idle seat
  (`SessionStart` only) and so meets no question. `docs/warp_architecture/` has no page on the
  quit warning.
- **Published material.** Warp's pages above. gpui's own contract: `App::on_app_quit` "It is not
  possible to cancel the quit event at this point" (`crates/gpui/src/app.rs:2454`) with
  `SHUTDOWN_TIMEOUT` of 200 ms (`:75`), so the question cannot live there;
  `Window::on_window_should_close` may veto (`crates/gpui/src/window.rs:6651`), and Zed's
  registration always vetoes and closes asynchronously through `close_window`
  (`crates/zed/src/zed.rs:494`).
- **The code we already ship.** No Item hook vetoes a close: `Item::is_dirty`
  (`crates/workspace/src/item.rs:279`) and `can_save` (`:294`) drive `save_item`, and a dirty
  terminal (`TerminalView::is_dirty`, `crates/terminal_view/src/terminal_view.rs:1854`: a
  running task or a bell) falls through to `Ok(true)` with no prompt (`pane.rs:2390`, `:2530`)
  since it cannot save. Zed's `quit` (`zed.rs:1750`) prompts only with `confirm_quit` (`:1773`)
  and then runs `prepare_windows_to_quit` (`workspace.rs:11750`), `prepare_window_to_close`
  (`:11794`) and `prepare_to_close`; the last window's close on Linux quits through
  `on_window_closed` (`zed.rs:351`). The only strong owner of a terminal view is the pane's
  items (`pane.rs:405`); `Project::terminals` keeps weak handles
  (`crates/project/src/terminals.rs:27`); `Drop for Terminal` releases the PTY
  (`crates/terminal/src/terminal.rs:3582`, `release_pty_resources` `:3255`: SIGTERM, then
  SIGKILL after 100 ms). `Pane::add_item` (`pane.rs:1347`) re-adds an item, the path a drag
  between panes uses, so a held view goes back whole; Reopen Closed Item's history holds weak
  handles and paths (`pane.rs:507`), never a terminal. The rail's Close is `close_item_by_id`
  (`rail.rs:612`, `:625`), so it meets the guard in `close_items`. The seat: `AgentEvents::seat`
  (`agent_events.rs:30`), `claude_events::seat_status` (`claude_events.rs:172`), the quiet
  timer `agent_status` (`marley_agent.rs:131`, `WAITING_AFTER` `:127`); a seat is forgotten only
  when the view is released (`notifications.rs:50`), so a held view keeps its seat. Prompts:
  `Window::prompt` (`gpui/src/window.rs:6448`); a re-entrant prompt is `unreachable!` (`:6461`);
  on Linux the in-app prompt takes Enter as the first button and Escape as `Cancel`
  (`crates/ui_prompt/src/ui_prompt.rs:66`, `:70`). Toasts: `Toast::new(..).on_click("Undo", ..)`
  (`workspace.rs:730`, `:747`). Keys: gpui tries the bindings of a keystroke in order until one
  is not propagated (`gpui/src/window.rs:5946`), so the Marley binding can hand `ctrl-shift-t`
  back. The e2e runner's `quit_marley` runs the palette's `zed: quit` and waits for the exit
  (`script/e2e.sh:281`); its cleanup sends SIGTERM (`:560`), which meets no guard. Does a crate
  we build own this seam? Zed's workspace owns the close paths and gains the hook; the question,
  the hold and the undo are Marley's.

## UI proof
UI-AFFECTING: a dialog, a toast, a restored terminal.
`script/e2e/550-ask-before-ending-a-working-agent.sh` (Hyprland, keys only). Setup: a scratch
repository; a HOME whose `.bashrc` is the scenario's; the profile's settings with
`marley.undo_close_seconds` at 8; #519's stand-in `claude`, which runs the plugin's `event.py`
with the next step's payloads at each Return, and also prints a counter every second and
writes its pid to a file. Steps: `claude`, Return (UserPromptSubmit and PreToolUse: working);
the palette's `zed: quit` by hand (`quit_marley` waits for an exit that must not come): the
dialog naming `Claude Code · working` (`550-01-quit-asks`); Escape: Marley still runs
(`550-02-still-running`). `ctrl-shift-w` in the terminal: the close dialog
(`550-03-close-asks`); Return (Close): the tab gone, the toast with Undo (`550-04-held`);
`ctrl-shift-t` within the hold: the terminal back, its counter past the numbers it showed
before the close (`550-05-restored`). Return to the stand-in (Stop: idle); `ctrl-shift-w`:
closed at once, no dialog, no toast (`550-06-idle-closes`). A second `claude`, Return (working);
`ctrl-shift-w`, Return; settle 10 s; `ctrl-shift-t`: nothing comes back (`550-07-expired`),
and `expect` that the stand-in's pid is gone. A step rewrites the profile's settings with
`marley.ask_before_ending_a_working_agent: false` (Zed reloads the file); a third `claude`,
Return; `ctrl-shift-w`: no dialog, the toast (`550-08-no-ask`). `quit_marley` at the end, with
no working agent, exits as before.

## Locked-In Decisions
- D1: "Working" is the seat's word: `Working`, `Starting` or `Waiting` (a turn in flight, a
  permission or a question pending) asks; `Idle`, `Error` and `Done` do not. Without a seat the
  quiet timer's `Working` asks and its `Waiting` (2 s of quiet, or a bell) does not, so an
  agent thinking silently for longer closes unasked until the plugin's events reach Marley: the
  plugin is the fix, not a longer timer.
- D2: One hook in Zed's two close paths, called before Zed's own prompts. `Pane::close_items`
  and `Workspace::prepare_to_close` are the only paths a tab close, the rail's Close,
  `ctrl-shift-w`, the window's close and the quit share; `on_app_quit` cannot cancel, and
  `on_window_should_close` is Zed's, always vetoing and closing asynchronously. The hook returns
  `Task<bool>`; `false` cancels the close as a Cancel in Zed's own prompt does.
- D3: Asked only for a close made in Marley. A SIGTERM, a compositor's kill of the process, a
  logout or a shutdown reach no prompt: nothing in Marley waits on a dialog outside the two
  paths, and Marley adds no `on_app_quit` work.
- D4: Enter confirms and Escape cancels: the buttons are `Close` (or `Quit`), `Show`, `Cancel`,
  the order Zed's own quit prompt uses. Show cancels and activates the first named terminal.
- D5: One question at a time. The guard keeps an `asking` flag; a close requested while a
  question is open is answered `false` (gpui's re-entrant prompt is `unreachable!`).
- D6: The hold keeps the view, not only the `Terminal`: the pane's items list is the only strong
  owner, the guard runs before the removal and takes a handle, and `Pane::add_item` puts the
  same view back with its scrollback, its seat and its bell, as a drag between panes does. Only
  a working terminal is held (Chad: "a closed working terminal"); an idle agent's or a shell's
  tab closes as today.
- D7: The hold's timer runs on the foreground executor; at the deadline the entry drops and
  `Drop for Terminal` ends the PTY as a close does today. A quit drops every held entry with the
  app.
- D8: `ctrl-shift-t` is Marley's only while something is held: the binding sits in `Pane` beside
  Zed's, loads later so it is tried first, and propagates when the hold list is empty, so Reopen
  Closed Item keeps its key.
- D9: Two settings, in the schema, `default.json` and the settings page: the question can be
  turned off, and the hold's seconds set (0 off). Warp's `grace_period` default of 60 is kept.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user quits or closes the window while an agent terminal is working, the system shall show a dialog naming each working agent with its project and state, and shall not quit until the user confirms. | Shots `550-01-quit-asks`, `550-02-still-running` |
| REQ-002 | WHEN the user closes a working agent's terminal (the tab's close, `ctrl-shift-w`, or the rail's Close), the system shall ask first, naming the agent. | Shot `550-03-close-asks` |
| REQ-003 | WHEN the user confirms the close of a working terminal, the system shall remove the tab, show a toast with Undo, and keep the terminal's process running for `marley.undo_close_seconds`. | Shot `550-04-held`; the log: the stand-in's pid alive after the close |
| REQ-004 | WHEN the user presses `ctrl-shift-t` or clicks Undo within the hold, the system shall put the terminal back with its scrollback and its agent still running. | Shot `550-05-restored`: the counter continued |
| REQ-005 | WHEN the user closes an idle agent's terminal or a plain shell, the system shall close it at once with no dialog and no hold. | Shot `550-06-idle-closes` |
| REQ-006 | WHEN the hold passes with no undo, the system shall end the terminal's process, and `ctrl-shift-t` shall restore nothing. | Shot `550-07-expired`; `expect` the pid gone |
| REQ-007 | WHILE `marley.ask_before_ending_a_working_agent` is false, the system shall ask nothing and still hold a closed working terminal. | Shot `550-08-no-ask` |
| REQ-008 | WHEN no agent terminal is working, a quit shall proceed as before. | `quit_marley` at the end of the run exits within its wait |
| REQ-009 | The two settings shall appear on the Marley settings page, and `default.json` shall set them to true and 60. | The 515 scenario's page shot; review of `default.json` |
| REQ-010 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion re-verify the
  two Zed seams, and check the golden set's scenarios for a quit with a working stand-in (none on
  2026-09-26: #540's fake is idle), since a question would stall `quit_marley`.
- **P2 Code:** the ledger rows first (`crates/workspace/src/pane.rs` new,
  `crates/workspace/src/workspace.rs` updated, the settings files); the hook type and its two
  calls; `close_guard.rs`; the settings; the keymap line; fmt and clippy clean; a review of the
  diff (every return path of the hook, the re-entrancy flag, no `on_app_quit`).
- **P3 Test:** write and run the scenario and read every shot; `just regress`;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_workbench.md`; the touchpoints
  rows checked against what shipped; the plan's slice status; the ledger capture; close the
  ticket, archive, commit.
