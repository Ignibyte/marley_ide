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

- **Promotion, 2026-09-29:** every seam re-read. `pty_info.rs`: `ProcessIdGetter` (13,
  `tcgetpgrp` 37), `ProcessInfo` (69), `load` (177), `emit_title_changed_if_changed` (200, the
  comparison in its task). #538 landed: `notifications::notify` gates on `looking_at` and on
  `banner_allowed`'s cooldown per project, and `Attention` holds the unread marks the rail's dot
  reads and observes, so a long command's end sets the mark too. The rail's lines go through
  `row_card(.., lines: Vec<String>, ..)` (`rail.rs:5625`, six callers), all muted; the red exit
  needs a line with its own color. `terminal_snapshot` at `rail.rs:4751`, `filter_match` at
  5494. The scenario moves to headless sway and a private bus, since banners on the user's bus
  would show on Chad's desktop; `just regress` and #500's rerun leave with the 2026-09-29
  workflow. Brain (consultation cf4c7c94): nothing on this seam.

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
  and the flag; `render_terminal_row` adds the line, in `Color::Error` for a non-zero exit,
  through `row_card`'s lines becoming `Vec<impl Into<RowLine>>` (a `String` is a muted line, so
  the other five callers stay as they are);
  `build_snapshot` matches `filter` against the title and, failing that, the command's text.
- **`duration_label`** in `marley_terminal` (pure): seconds under a minute as `45 s`, then
  `4 m 12 s`, then `1 h 2 m`.
- **File manifest.** Zed: `crates/terminal/src/pty_info.rs`, `crates/terminal/src/terminal.rs`
  (the getter), `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`. Marley: `crates/marley_workbench/src/command_watch.rs` (new),
  `notifications.rs` (`notify` made `pub(crate)`, and `mark_unread` for #538's mark), `rail.rs`,
  `marley_workbench.rs`;
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
- **Built to the manifest.** `pty_info.rs`: `ProcessIdGetter::reads_password` (`tcgetattr` on
  the master, `ICANON` set and `ECHO` clear; Windows answers false), `ProcessInfo.reads_password`
  filled in `load` and part of `has_changed`. `terminal.rs`: `marley_foreground_reads_password`.
  `marley_terminal::duration_label`. The setting `marley.long_command_seconds` (30, 0 for never)
  in `settings_content`, `default.json`, `MarleySettings` and the Marley page's Terminal section.
  `marley_rail::CommandSnapshot` on the terminal's snapshot and row. `rail.rs`:
  `command_snapshot`, `command_line` and `RowLine` (a line's text, its color, and a `state` that
  stays whole while a long command is cut), the filter falling back to the command's text.
  `notifications.rs`: `notify` made `pub(crate)`, `mark_unread` added. `command_watch.rs` (new).
- **Deviations.** The watch's baseline is taken when the view is made, not at its first check,
  so a split of a terminal with blocks tells nothing of the ends before it. The agent test is
  `agent_in(terminal)` or the block's own command naming an agent (`agent_kind_of`), since the
  agent's block ends after the agent has left the foreground. The rail leaves out the line for
  an agent's block by the same test (found in Test).
- **Review.** Borrows reordered twice (the threshold and the entity id read before the global is
  taken mutably). The duration is rounded, not floored: the hooks' times make a `sleep 3` measure
  2.9 s. Re-entrancy: `check` reads the terminal inside `observe_in`/`subscribe_in` callbacks,
  never the view being updated.
- **Gate.** Run 1 red on two gates: `too_long_first_doc_paragraph` on `duration_label`'s doc, and
  gate:13, whose `// SAFETY:` must sit on the line directly above the `unsafe`; both fixed at the
  source. Run 2 red on clippy only: `too_many_lines` on `build_snapshot` (the filter's fallback
  moved into `terminal_match`), `missing_const_for_fn` on `RowLine::muted`, and
  `needless_pass_by_ref_mut` on `command_watch::init` (now `&App`). Run 3: `GATE GREEN [diff]`,
  16 passed, 0 failed, the receipt written.

## Phase 3 — Test
- **Scenario** `script/e2e/551-command-end-from-outside.sh` under `compositor sway`, on a private
  session bus whose notification server logs `app|summary|body`; the user's bus watched with
  `busctl monitor`. `marley.long_command_seconds` is 2 in the run's settings.
- **Run 2** passed every check but shot `551-04-password-row` cut the row at
  `sleep 1; read -s -p 'Password…`: the state was part of the one truncated label, so REQ-005's
  words did not show. Fixed: `RowLine.state` renders in its own `flex_none` label after a
  truncating one. Run 3 showed a doubled space before the `·` (the row's gap plus the text's);
  the gap went and the state leads with one space.
- **Run 4, every shot read** (rail crops scaled 3x):
  - `551-01-failed-row`: terminal 1's row reads `sleep 3; false · exit 1 · 3 s` in the error
    red with the unread dot; terminal 2 (in front) has no line. Log: `Marley|sleep 3; false|exit
    1 after 3 s`. REQ-001, REQ-004.
  - `551-02-running-row`: title `repo — sleep 7`, line `sleep 7 · running`, muted. REQ-003.
  - `551-03-done-row`: `sleep 7 · done · 7 s`, muted, the dot. Log: `done in 7 s`. REQ-004.
  - `551-04-password-row`: `sleep 1… · waiting for a password`, the state whole, the dot. Log:
    one banner `waiting for a password`. REQ-005.
  - `551-05-filter`: the filter holds `sleep`; only terminal 1's row is left, matched by its
    command (`sleep 3; false · exit 1 · 3 s`), terminal 2 hidden. The terminal shows the typed
    password hidden and its block ✓. REQ-006.
  - `551-06-setting`: the Marley page's Terminal section, `Long Command Seconds` at 2 with its
    reset arrow and its description; `default.json` sets 30. REQ-008.
  - Log checks: a short `true` and the focused terminal's own `sleep 3; false` posted nothing
    (REQ-002); the stand-in agent's 4 s block posted nothing (REQ-007); the user's bus saw no
    `Notify`.
- **Found in Test.** The same shot 06 showed the ended stand-in agent's row as `claude · done ·
  4 s`: REQ-007 says an agent's block changes no row. `command_snapshot` now leaves out a block
  whose command names an agent, the watcher's test.
- **Run 5** (after that fix and the gate's clippy fixes): every check passed again, and every shot
  was read again. Shots 01 to 05 are as run 4 describes; `551-06-setting`'s rail now shows
  terminal 1, whose last block was the stand-in `claude`, with no line under its title. REQ-007.
- **Not reachable:** a click on the banner (the notification server is the scenario's own; #478
  proved the click path, reused unchanged); a remote terminal's row (no PTY here, the flag reads
  false) is review.

## Phase 4 — Complete
- **Docs.** CHANGELOG Added; the plan's T7 row; `marley_workbench.md` (a section for
  `command_watch.rs` and the rail's command line), `marley_rail.md`
  (`TerminalSnapshot::command`), `terminal_blocks.md` (`duration_label`); the touchpoint rows for
  `pty_info.rs`, `terminal.rs`, the settings files and `default.json` describe what shipped.
- **Knowledge.** AD-claude-551-a-long-commands-end-told-from-outside-001,
  F-claude-551-a-rows-state-cut-with-its-command-001,
  L-claude-551-an-agents-block-outlives-the-agent-001,
  L-claude-551-the-safety-comment-sits-directly-above-the-unsafe-001. Brain: `brain_decide` on
  consultation cf4c7c94.
- **Closed** the ticket, archived the pair, committed.
