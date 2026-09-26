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
