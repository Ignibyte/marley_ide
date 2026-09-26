# A command's end, seen from outside its terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-551-command-end-from-outside.md
- **Pipeline spec:** 551-command-end-from-outside.spec.md

## Phase 1 — Plan
- **Request:** the Warp second pass's finding 3 (2026-09-25) and its third recommendation, with
  Chad's answer to open question 4 left at its default on 2026-09-26: Warp's 30 seconds, plain
  commands only, never agent terminals. Drafted in the spec batch of 2026-09-26.
- **Classification / tier:** feature, size S with one Zed touchpoint for the password check:
  the watch, the row and the filter are Marley code; the flag is a hunk in `pty_info.rs` and a
  getter in `terminal.rs`.
- **Recall (§18.3):**
  - AD-claude-478-the-terminal-reads-the-notification-escapes-other-terminals-read-001: the
    banner path, the in-front rule, a click shows the terminal; rejected a notification for the
    focused terminal. This ticket reuses the path and the rule.
  - L-claude-478-omarchys-notifications-go-to-quickshell-001: `busctl --user monitor` shows each
    `Notify`; the scenario's proof for the banners.
  - AD-claude-464-blocks-are-anchored-ranges-read-from-the-grid-001 and
    AD-claude-491 (the stamped times): a block's start and end come from `stamp` at the hook.
  - F-claude-546-block-reads-answered-from-the-alternate-screen-001: read the state the change
    acts on; the watch reads `blocks()` and `times()`, never the grid.
  - AD-claude-468-the-rail-draws-its-own-rows-after-warps-tab-list-001: the rail is the tab list;
    a row's lines are where Warp's tab metadata goes.
  - Brain: no page on long-command notifications (searched 2026-09-26).
- **Discovery:**
  - `crates/marley_workbench/src/notifications.rs:24` `Senders`; `:30` `init`; `:33`
    `observe_new` on `TerminalView`; `:38` `subscribe_in(&terminal, ..)`; `:59` `notify`
    (private); `:66` the gate `window.is_window_active() && view.focus_handle(cx).contains_focused(window, cx)`;
    `:69` the tag `marley-terminal-<id>`; `:76` `show_system_notification`; `:86` `show_sender`.
  - `crates/terminal/src/terminal.rs:1777` the `Wakeup` arm (`:1782` `emit_title_changed_if_changed`);
    `:1817` `apply_shell_hook` (`:1834` `stamp`, `:1835` `cx.notify()`); `:1845` `blocks`;
    `:1852` `marley_anchored`; `:729` `Event` (no block event).
  - `crates/terminal/src/pty_info.rs:12` `ProcessIdGetter { handle, fallback_pid }` (private
    fields); `:33` `pid` with `:37` `tcgetpgrp(self.handle)`; `:69` `ProcessInfo { name, cwd,
    argv }`; `:177` `load`; `:200` `emit_title_changed_if_changed` (`:208` the comparison of
    cwd and name, `:231` `Event::TitleChanged`); `crates/terminal/src/alacritty.rs:66` the
    getter built from the PTY's master fd.
  - `crates/marley_terminal/src/anchored.rs:38` `AnchoredBlock` (command, state, exit code,
    prompt); `:63` `BlockTimes { started, finished }`; `:170` `times`.
  - `crates/marley_rail/src/marley_rail.rs:46` `TerminalSnapshot` (id, title, subtitle, bell,
    agent, activity, matched); `:216` `TerminalRow`; `:505` `rail_rows` (terminal rows at
    `:524`); `:643` `has_attention`.
  - `crates/marley_workbench/src/rail.rs:1717` `terminal_snapshot` (`:1729` the agent kind,
    `:1749` title and subtitle); `:1805` `build_snapshot`; `:1852` `filter_match(filter, &terminal.title)`;
    `:1919` `filter_match`; `:1319` `render_terminal_row` (`:1371` the lines); `:1982`
    `row_card`; `:473` to `:491` the per-view subscriptions (`Wakeup` to `note_output`, the rest
    to `refresh`); `:1788` `no_update_after_ms`.
  - `crates/marley_workbench/src/mcp.rs:477` `block_entry` (a duration in milliseconds).
  - `vendor/alacritty_terminal/src/tty/unix.rs:249` `tcgetattr(&master)` at creation.
  - Settings: `crates/settings_content/src/marley.rs:10` (`:28` a `u64` field);
    `crates/marley_workbench/src/marley_workbench.rs:168`; `crates/settings_ui/src/marley_page.rs:68`;
    `assets/settings/default.json:1665`.
  - The probe (2026-09-26, kernel 7.2.5): `tcgetattr` on the master of a pty pair shows the
    slave's `ECHO` and `ICANON` bits as the slave sets them.
- **Decisions:** D1 to D9 in the spec.

### Design
- **`command_watch.rs`.** `init`: `cx.observe_new::<TerminalView>` (as `notifications.rs:33`),
  and for each view `cx.observe(&terminal, ..)` plus a subscription to `Event::TitleChanged`;
  a `Watched { last_finished: Option<usize>, password_told: Option<usize> }` per view in a
  global keyed by the view's entity id (rich input's pattern), released with the view. At each
  notify: the last block; if Finished and its index is newer than `last_finished`, read
  `times(index)`; a duration at or over the threshold, a terminal not in front (the gate
  `notify` uses) and no agent in the foreground (`agent_in`) post `notify(view, Some(command),
  body, ..)`. At each `TitleChanged`: if the last block runs, `foreground_reads_password()` is
  true and `password_told != Some(index)`, post `<command>: waiting for a password` and remember
  the index. The rail's refresh (any event but `Wakeup`) redraws the row.
- **The flag.** `pty_info.rs`: `ProcessIdGetter::reads_password(&self) -> bool`:
  `libc::tcgetattr(self.handle, &mut termios)` (its `// SAFETY:`: the handle is the PTY's open
  master fd for the getter's life), `ICANON` set and `ECHO` clear; `ProcessInfo.reads_password`
  filled in `load`; the comparison at `:208` includes it. `Terminal::foreground_reads_password()`
  reads `info.current`. Remote terminals have no PTY here and answer false.
- **The rail.** `CommandSnapshot { text: String, running: bool, exit_code: Option<i32>,
  duration: Option<String>, password: bool }` on `TerminalSnapshot` and `TerminalRow`;
  `terminal_snapshot` fills it from the last block (the text cut to 60 characters), its `times`
  and the flag; `render_terminal_row` adds the line with `exit <code>` in `Color::Error`;
  `build_snapshot` matches `filter` against the title and, failing that, the command's text.
- **`duration_label`** in `marley_terminal` (pure): seconds under a minute as `45 s`, then
  `4 m 12 s`, then `1 h 2 m`.
- **File manifest.** Zed: `crates/terminal/src/pty_info.rs`, `crates/terminal/src/terminal.rs`
  (the getter), `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`. Marley: `crates/marley_workbench/src/command_watch.rs` (new),
  `notifications.rs` (`notify` made `pub(crate)`), `rail.rs`, `marley_workbench.rs`;
  `crates/marley_rail/src/marley_rail.rs`; `crates/marley_terminal/src/marley_terminal.rs` (or
  a small `duration.rs`). Scripts: `script/e2e/551-command-end-from-outside.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: `crates/terminal/src/pty_info.rs` (new
  row: the termios read and the field; at a merge, keep the read beside `tcgetpgrp` and the
  field in the comparison), `crates/terminal/src/terminal.rs` (the row gains the getter), the
  settings rows updated.

### E2E plan
The stand-in `claude` is #519's. `busctl --user monitor` runs from setup, as #519's scenario
runs it; the `Notify` lines are counted with `grep -c`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, 004 | terminal 2 opened; in terminal 1 `sleep 3; false`, Return, `ctrl-tab`; settle 5 | `551-01-failed-row`; the log: one `Notify` with `sleep 3; false`, `exit 1 after 3 s` |
| REQ-002 | `true` in terminal 1, `ctrl-tab`, settle 2 | the log: the count unchanged |
| REQ-003 | `sleep 6`, Return, `ctrl-tab`, settle 2 | `551-02-running-row` |
| REQ-001, 004 | settle 6 | `551-03-done-row`; the log: `done in 6 s` |
| REQ-005 | `read -s -p 'Password: ' x`, Return, `ctrl-tab`, settle 2 | `551-04-password-row`; the log: one `Notify` with `waiting for a password` |
| REQ-002 | `secret`, Return; `sleep 3; false` in the focused terminal; settle 5 | the log: the count unchanged |
| REQ-006 | the sidebar filter focused, `sleep` typed | `551-05-filter`: terminal 1's row matched |
| REQ-007 | `claude`, Return, Return, 3 s, `ctrl-d`, `ctrl-tab`, settle 3 | the log: no `Notify` naming `claude` |

Not reachable by a scenario: a click on the banner (Quickshell's banner is not in the run's
window); #478 proved the click path and this ticket reuses it unchanged. The rail's row for a
remote terminal (no PTY flag) is review.

### Risks
- `tcgetattr` on a master whose slave is closed fails; the read treats an error as false.
- A `Wakeup` refresh already running skips the next (`pty_info.rs:201`); the flag can lag one
  refresh, which the next output corrects.
- The rail's row grows a line, so scenarios that click rail rows by coordinates shift
  (L-claude-498); only #500's clicks the rail, on the first project's row, above any terminal
  row's lines. Test reruns it.
- A command's text with a newline or an escape in the row: cut and sanitized as the rail's
  titles are.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
