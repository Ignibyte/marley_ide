# Ask before a close or a quit ends a working agent, and hold the closed terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-550-ask-before-ending-a-working-agent.md
- **Pipeline spec:** 550-ask-before-ending-a-working-agent.spec.md

## Phase 1 — Plan
- **Request:** the Warp second pass's finding 2 and the Orca second pass's finding 1
  (2026-09-25), which ranks it first; Chad's answers of 2026-09-26: ask first, name the working
  agents, close idle ones without asking, and hold a closed working terminal for a configurable
  number of seconds ("allows stopping of accidental. so both"). Drafted in the spec batch of
  2026-09-26.
- **Classification / tier:** feature, size M: the questions S, the hold and the undo the rest.
  Two Zed paths (`pane.rs`, `workspace.rs`), each one call, plus the settings files.
- **Recall (§18.3):**
  - AD-claude-477-a-footer-hook-in-zeds-terminal-view-and-the-bar-in-marleys-crate-001: one hook
    global in the Zed crate, Marley's code behind it. The guard follows the pattern, in
    `crates/workspace`.
  - AD-claude-519-claude-codes-hook-events-ride-in-band-into-marley-fleet-001 and #519's D6: the
    seat's states and what sets them; `Waiting` is a permission or a question.
  - F-claude-515-an-app-dispatch-inside-an-action-found-no-window-001: actions on the workspace,
    dispatch through the window.
  - L-claude-478-omarchys-notifications-go-to-quickshell-001: not needed here (no banner), but
    the toast is in-app.
  - L-claude-494 (in #520's D1): the gpui entity id changes at every launch; the hold keys
    nothing by id across launches.
  - Brain: no page on quit guards (searched 2026-09-26).
- **Discovery:**
  - `crates/workspace/src/workspace.rs:720` `CloseIntent` (Quit, CloseWindow, ReplaceWindow);
    `:3643` `prepare_to_close` (`:3696` the call prompt, `:3751` the `save_all_internal` call);
    `:3880` `save_all_internal` (`:3944` the save-all prompt); `:11750`
    `prepare_windows_to_quit`; `:11794` `prepare_window_to_close`; `:3327`
    `reopen_closed_item`; `:730` `Toast`, `:747` `on_click`; `:4956` `add_item_to_active_pane`;
    `:6356` `active_pane`; `:5576` `activate_item`.
  - `crates/workspace/src/multi_workspace.rs:555` `close_window` (the platform close and the
    `CloseWindow` action both end here).
  - `crates/workspace/src/pane.rs:1662` `close_item_by_id`; `:1954` `close_items` (`:1990` the
    dirty filter, `:2030` `save_item`, `:2069` `remove_item`); `:2246` `save_item` (`:2390`,
    `:2530`); `:2096` `remove_item`; `:1347` `add_item`; `:336` `Event::RemovedItem`; `:405`
    `items`; `:507` `NavigationEntry` (weak).
  - `crates/workspace/src/item.rs:170` `Item`; `:216` `deactivated`; `:218` `on_removed`; `:279`
    `is_dirty`; `:294` `can_save`. No veto method.
  - `crates/zed/src/zed.rs:494` `on_window_should_close`; `:1750` `quit` (`:1755`
    `confirm_quit`, `:1773` the prompt, `:1793` `prepare_windows_to_quit`, `:1797` `cx.quit()`);
    `:351` `on_window_closed` on Linux; `:1749` `WAITING_QUIT_CONFIRMATION`.
  - `crates/gpui/src/app.rs:75` `SHUTDOWN_TIMEOUT`; `:1057` `shutdown`; `:2454` `on_app_quit`;
    `crates/gpui/src/window.rs:6448` `prompt` (`:6461` `unreachable!`); `:6651`
    `on_window_should_close`; `:5946` the binding loop.
  - `crates/ui_prompt/src/ui_prompt.rs:66` Enter confirms the active button (index 0), `:70`
    Escape picks `Cancel`.
  - `crates/terminal_view/src/terminal_view.rs:157` the view (a strong `Entity<Terminal>`);
    `:260` `new`; `:899` `terminal`; `:1854` `is_dirty`; no `on_removed` or `deactivated`.
  - `crates/terminal/src/terminal.rs:3582` `Drop` (`release_pty_resources` `:3255`; `:1503`
    `subscribe` owns the event-loop task); `crates/project/src/terminals.rs:27` weak handles.
  - `crates/marley_workbench/src/rail.rs:612` `close_terminal` (`:625` `close_item_by_id`);
    `:1717` `terminal_snapshot` (`:1735` the seat, `:1744` the quiet timer, `:1746`
    `seat_status`); `:99` `quiet_timers`; `:426` `note_output`.
  - `crates/marley_workbench/src/agent_events.rs:30` `seat`; `:136` `forget`, called on the
    view's release at `notifications.rs:50`.
  - `crates/marley_fleet/src/session.rs:13` `State` (Starting, Working, Idle, Waiting, Error,
    Done); `crates/marley_agent/src/claude_events.rs:172` `seat_status`;
    `crates/marley_agent/src/marley_agent.rs:131` `agent_status`, `:127` `WAITING_AFTER`.
  - Settings: `crates/settings_content/src/marley.rs:10`; `crates/marley_workbench/src/marley_workbench.rs:168`
    `MarleySettings` (`:183` `from_settings`); `crates/settings_ui/src/marley_page.rs:68` an
    item; `assets/settings/default.json:1665`.
  - `assets/keymaps/default-linux.json:694` `ctrl-shift-t` (`Pane`), `:1322` `ctrl-shift-w` in
    `Terminal` (`pane::CloseActiveItem`), `:25` `ctrl-shift-w` in `Workspace`
    (`workspace::CloseWindow`).
  - `script/e2e.sh:281` `quit_marley`; `:560` cleanup's SIGTERM; #540's scenario at
    `540-session-resume-after-restart.spec.md:105` quits with an Idle fake.
- **Decisions:** D1 to D9 in the spec.

### Design
- **The hook** (`crates/workspace`, beside `Workspace`): `pub struct MarleyCloseGuard(pub
  Arc<dyn Fn(MarleyClose, &mut Window, &mut App) -> Task<bool>>)` with `impl Global`, and
  `pub struct MarleyClose { pub items: Vec<Box<dyn ItemHandle>>, pub intent: Option<CloseIntent> }`
  (`None` for a tab close). `close_items` (`pane.rs:1954`): at the start of its async block,
  `if let Some(guard) = cx.try_global::<MarleyCloseGuard>() { if !guard.0(..).await { return Ok(()) } }`;
  `prepare_to_close` (`workspace.rs:3643`): the same, with the intent and
  `self.items(cx)`, before the `save_all_internal` call. Each hunk carries `// Marley:` and its
  ledger row.
- **`close_guard.rs`.** `init` sets the global and registers `UndoCloseTerminal` on the
  workspace. `working_agents(items, cx) -> Vec<Working { view, kind, project, state }>`: each
  `TerminalView` among the items whose `agent_in` is Some and whose state (the seat's, else the
  quiet timer's, read as the rail reads them) is working (D1). `guard(close, window, cx)`: with
  none, `Task::ready(true)`; with the question off, hold each and `true`; while `asking`,
  `false`; else `window.prompt(PromptLevel::Warning, message, Some(detail), &buttons)`, then
  Close or Quit holds each working view and answers `true`, Show activates the first (its
  window, workspace and item, as `show_sender` does in `notifications.rs:86`) and answers
  `false`, Cancel answers `false`.
- **The hold.** `HeldTerminals(Vec<Held { view: Entity<TerminalView>, pane: WeakEntity<Pane>,
  workspace: WeakEntity<Workspace>, closed_at: Instant, timer: Task<()> }>)`, a global. `hold`
  pushes and shows the toast (`Toast::new(NotificationId::unique::<UndoClose>(), "Closed Claude
  Code in marley_ide").on_click("Undo", ..)`); the timer, after `undo_close_seconds`, removes the
  entry (dropping the last strong handle: the view, then its `Terminal` and PTY) and dismisses
  the toast. With the setting at 0 nothing is held. `undo`: pop the newest, `pane.add_item(Box::new(view), true, true, None, ..)`
  on its pane if alive else the active pane, focus it, dismiss the toast; with none held,
  `cx.propagate()` so Zed's Reopen Closed Item runs.
- **The keymap.** `crates/marley_workbench/keymap.json`: `"context": "Pane"`,
  `"ctrl-shift-t": "marley::UndoCloseTerminal"`.
- **Settings.** `ask_before_ending_a_working_agent: Option<bool>` and
  `undo_close_seconds: Option<u64>` in `MarleySettingsContent`; `MarleySettings` reads them with
  `unwrap_or(true)` and `unwrap_or(60)`; `default.json`'s `marley` block; two items on the
  settings page (a bool renderer and the `u64` number field #547 uses).
- **File manifest.** Zed: `crates/workspace/src/pane.rs`, `crates/workspace/src/workspace.rs`
  (the hook type and the two calls), `crates/settings_content/src/marley.rs`,
  `crates/settings_ui/src/marley_page.rs`, `assets/settings/default.json`. Marley:
  `crates/marley_workbench/src/close_guard.rs` (new), `marley_workbench.rs` (the module, the
  action, the settings), `keymap.json`. Scripts: `script/e2e/550-ask-before-ending-a-working-agent.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: `crates/workspace/src/pane.rs` (new row:
  the guard call in `close_items`; at a merge, keep it before the dirty flow),
  `crates/workspace/src/workspace.rs` (the existing row gains the hook type and the call in
  `prepare_to_close`), and the settings rows updated for the two fields.

### E2E plan
The stand-in `claude` is #519's with two additions: a thread that prints a counter each second,
and its pid written to `$E2E_WORK/claude.pid`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `claude`, Return (working); the palette, `zed: quit`, Return | `550-01-quit-asks`: the dialog names `Claude Code · working` |
| REQ-001 | Escape | `550-02-still-running`: the terminal as it was |
| REQ-002 | `ctrl-shift-w` | `550-03-close-asks` |
| REQ-003 | Return (Close) | `550-04-held`: no tab, the toast; `expect kill -0 <pid>` |
| REQ-004 | `ctrl-shift-t` | `550-05-restored`: the terminal, the counter beyond its last shown value |
| REQ-005 | Return (Stop: idle); `ctrl-shift-w` | `550-06-idle-closes`: gone, no dialog, no toast |
| REQ-006 | `claude`, Return; `ctrl-shift-w`, Return; settle 10; `ctrl-shift-t` | `550-07-expired`: nothing restored; `expect` the pid gone |
| REQ-007 | the settings rewritten with the question off; `claude`, Return; `ctrl-shift-w` | `550-08-no-ask`: no dialog, the toast |
| REQ-008 | `ctrl-d` ends the stand-in; `quit_marley` | the run log: Marley exited within the wait |

Not reachable by a scenario: the window's close button and a compositor's close request (the
runner has no title bar and sends keys only); both reach `close_window` and the same
`prepare_to_close`, which the quit exercises. A logout is not a step; D3's argument is the
review's (no Marley work in `on_app_quit`, no wait outside the two paths).

### Risks
- A stall in other scenarios: any scenario that quits while a stand-in is working would wait on
  the dialog. On 2026-09-26 none does (the golden set's fakes are idle or plain shells); P1
  rechecks, and `quit_marley` could take an answer as a later runner change.
- A held view's `Terminal` keeps pumping events into its queue with no render to drain them;
  bounded by the hold, and restored views sync on their first frame.
- A drag between panes goes through `remove_item` and `add_item`, never `close_items`, so it
  never meets the guard and is never held.
- Upstream moves `close_items` or `prepare_to_close`: each hunk is one call; the ledger row
  says where it goes.
- `WAITING_QUIT_CONFIRMATION` guards Zed's own quit re-entrancy; the guard's `asking` flag is
  Marley's, and the two never open a prompt at once (Zed's runs first, before
  `prepare_windows_to_quit`).

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.

### Promotion (2026-09-26)
- **Recall at promotion:** AD-claude-452-the-rail-starts-zeds-own-rename-and-close-001 (the rail's
  Close is the pane's `close_item_by_id` with `SaveIntent::Close`, so it meets the guard in
  `close_items`, as the design says); AD-claude-494-browser-tabs-reattach-or-reopen-001 (494's
  quit and relaunch carry no agent); L-claude-547-marleys-path-is-the-login-shells-001 (the
  stand-in runs in a terminal, first on the PATH through the scenario's `.bashrc`). Brain
  consultation 3c055621c9a148c5a7082fff438d9cff: nothing on this seam.
- **Seams re-verified at `686a760b54`:** `Pane::close_items` at `pane.rs:1954`; `CloseIntent` at
  `workspace.rs:720` and `prepare_to_close` at `:3643`; the rail's `close_terminal` at
  `rail.rs:612`, its `close_item_by_id` at `:625`; `AgentEvents::seat` at `agent_events.rs:30`.
  One correction: Zed binds `ctrl-shift-t` to `pane::ReopenClosedItem` in the `Workspace`
  context (`default-linux.json:694`), not `Pane`; Marley's binding in `Pane`, a deeper context,
  is tried first, and propagates to Zed's when nothing is held. `ctrl-shift-w` in a terminal is
  `pane::CloseActiveItem` (`:1322`), the scenario's tab close.
- **The golden set and a quit:** the scenarios that quit (494, 502, 512) run no agent, so no
  question can stall their `quit_marley`.

## Phase 2 — Code
- **Built.**
  - Zed, `crates/workspace`: `MarleyClose { items, pane, intent }`, the `MarleyCloseGuard` global
    and `marley_close_guard` (yes when no guard is set) in `workspace.rs`; the guard asked at the
    head of `Pane::close_items`' task (the items and the pane; `false` returns as Zed's Cancel
    does), at the head of `prepare_windows_to_quit` (every window's items, once per quit), in
    `prepare_window_to_close` for a window's close (its workspaces' items), and in
    `prepare_to_close` for a replace. Each hunk carries `// Marley:`; rows written first.
  - `marley_workbench::close_guard` (new): `init` sets the guard, follows each terminal's
    `Wakeup` for the quiet timer (a `last_output` map, forgotten on release) and registers
    `UndoCloseTerminal` on every workspace (it propagates when nothing is held, so Zed's Reopen
    Closed Item keeps the key); `working_agents` and `working_status` (the seat's `Starting`,
    `Working` or `Waiting` for Claude Code, else the quiet timer's `Working`); the question per
    intent (one agent: "Close Claude Code in repo? It is working."; several, a quit, a window, a
    replace: a count and a line per agent), `Close`/`Quit`/`Close Window`, `Show`, `Cancel`, one
    at a time (`asking`); `hold` (a tab's close only: the view kept with its pane and workspace,
    a deadline task, a toast with Undo that dispatches the action); `undo` (the newest held view
    back in its pane, or the workspace's active pane); `show` (its window, project and tab).
  - Settings: `ask_before_ending_a_working_agent` (true) and `undo_close_seconds` (60) in
    `MarleySettingsContent`, `default.json`, `MarleySettings` and the Agents section; the
    `marley::UndoCloseTerminal` action; `ctrl-shift-t` in the `Workspace` context of Marley's keymap (it loads after Zed's, so it is tried first at the same depth, and the Workspace context is in the focus chain wherever the focus lands after the close, an empty pane or the rail).
- **Deviations, and why.**
  - Four hook sites, not the spec's two: `prepare_to_close` runs once per workspace, so a quit
    or a window's close with agents in two projects would have asked twice. The quit asks once
    in `prepare_windows_to_quit`, a window once in `prepare_window_to_close`, and
    `prepare_to_close` keeps only the replace (the rail's project removal runs it per workspace
    being removed, its "consent phase").
  - `MarleyClose` carries the pane, so an undo returns the view to the pane it left.
  - Each hook collects the items and hands them over, and the guard reads nothing but terminal
    views: the quit's hook reads every window before it updates the first, since a window's
    root may not be read while it is being updated.
  - The hold, with the question off, waits for the pane's update to end (a spawned task): the
    pane closing the tab is being updated when the guard runs.
- **Review of the diff.** Every return path of the guard answers; `asking` is cleared when the
  prompt resolves, whatever it answers; the hold's deadline drops the view (and its PTY) with
  the entry, and a quit ends held views with the app; no `on_app_quit` work was added.
- **Checks.** `cargo check --all-targets`, `cargo clippy --all-targets --all-features -D
  warnings` (after two findings: the pane passed by value, a missing `;`) and `cargo fmt` over
  `workspace`, `marley_workbench`, `settings_content` and `settings_ui`: clean.

## Phase 3 — Test
- **The scenario:** `script/e2e/550-ask-before-ending-a-working-agent.sh`, under `compositor sway`
  rather than the planned Hyprland, since Chad's own Marley is open and the runner refuses a
  Hyprland run beside it; keys only either way. The stand-in `claude` runs the plugin's real
  `event.py` at each Return (working, then idle), prints a tick a second and writes its pid; the
  profile holds a closed terminal for 8 seconds; Marley's MCP server's `terminal_list` tells
  whether the tab is there.
- **The run, every check passing:** Cancel keeps Marley running; the tab's close asks, Close
  removes the tab (`terminal_list` empty) and the stand-in still runs; Ctrl-Shift-T lists the
  terminal again with the stand-in alive; an idle agent's close ends it at once; a hold nobody
  claims ends the stand-in and Ctrl-Shift-T brings nothing back; with the question off the tab
  goes without a dialog and the stand-in is held; `quit_marley` with no working agent exits
  within its wait (REQ-008).
- **The shots, read:**
  - `550-01-quit-asks`: "Quit Marley? 1 agent is working:", the line `repo · Claude Code ·
    working`, and Quit, Show, Cancel (REQ-001).
  - `550-02-still-running`: after Escape, the terminal as it was, its ticks going on (REQ-001).
  - `550-03-close-asks`: "Close Claude Code in repo? It is working.", its line, and Close, Show,
    Cancel (REQ-002).
  - `550-04-held`: no tab, the toast "Closed Claude Code in repo" with Undo (REQ-003).
  - `550-05-restored`: the terminal back, its ticks at 19 against 14 when the close asked, the
    rail row `working · Refactor the parser` (REQ-004).
  - `550-06-idle-closes`: no tab, no dialog, no toast (REQ-005).
  - `550-07-expired`: nothing came back, no toast (REQ-006).
  - `550-08-no-ask`: no dialog, the toast (REQ-007).
- **Not reached:** the window's close button and a compositor's close request (keys only; both
  reach `prepare_window_to_close`, the path this hook shares with the rail's project removal),
  and a logout (D3 stands on the review: nothing in `on_app_quit`).
- **REQ-009:** the golden run's `515-01-marley-page` shows the Agents section's "Ask Before
  Ending a Working Agent" (on) and "Undo Close Seconds" (60); `default.json` sets true and 60.
- **Golden set:** 550 joins it; `just regress`: all 14 pass (550 in 93 s).
- **Gate:** `script/gates.sh --diff`: `GATE GREEN [diff]`, 16 passed.
- **Verdict:** Phase 3 PASS.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_workbench.md` (the
  guard); the ledger rows for `pane.rs` (new), `workspace.rs` and the three settings files
  describe what shipped.
- **Knowledge:** AD-claude-550-the-close-guard-asks-in-zeds-own-close-paths-and-holds-the-view-001.
- **Brain:** consultation 3c055621c9a148c5a7082fff438d9cff closed with a decision.
- **Ticket:** closed.
