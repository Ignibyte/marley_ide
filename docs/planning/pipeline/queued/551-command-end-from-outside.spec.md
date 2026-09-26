---
pipeline_id: 29244e1f-5ba2-43ee-8a00-589403be98ca
ticket: docs/planning/tickets/open/TICKET-551-command-end-from-outside.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A command's end, seen from outside its terminal"
type: feature
slice: prong 1 T7b with the rail (#478's notifications for plain commands); the Warp second pass's finding 3
references: [docs/planning/design-notes/warp-second-pass-2026-09-25.md, docs/planning/pipeline/completed/478-terminal-notifications.spec.md, docs/planning/pipeline/queued/538-notifications-with-content.spec.md, docs/planning/pipeline/queued/542-rail-attention-order.spec.md]
---

## Title
Marley watches each terminal's blocks. A block that ends after `marley.long_command_seconds`
(30, Warp's default; 0 for off) while its terminal is not the focused one of the active window
posts a desktop notification titled with the command, `done in 45 s` or `exit 1 after 4 m 12 s`,
through #478's notify-and-click path. The terminal's rail row gains a line with the running
command, or the last command's result with the exit in red, and the rail's filter matches the
command. A password prompt shows in the PTY's own flags: canonical mode with echo off while a
block runs means `sudo`, `ssh` or `read -s` is reading one, so the row says `waiting for a
password` and one notification goes out. Agent terminals are left out (Chad's default): their
session is one long block, and #538 words their banners.

## Scope
### In
- `crates/marley_workbench/src/command_watch.rs` (new): for each terminal view (the
  `observe_new` route `notifications.rs` uses), an observer of its `Terminal` entity that
  compares the blocks at each notify: a block newly Finished, whose `times()` give a duration
  of at least the threshold, whose terminal is not the focused terminal of the active window
  and whose foreground program is not a known agent, posts one notification through
  `notifications::notify` (made `pub(crate)`): the command as the title, `done in <duration>`
  or `exit <code> after <duration>` as the body.
- **The password check.** `crates/terminal/src/pty_info.rs`: `ProcessIdGetter::reads_password`
  reads the master's termios with `libc::tcgetattr` beside `tcgetpgrp` (Linux answers with the
  slave's flags: `ICANON` set and `ECHO` clear is a canonical read with echo off);
  `ProcessInfo` gains `reads_password: bool`, `load` fills it and the comparison in
  `emit_title_changed_if_changed` includes it, so a change emits `Event::TitleChanged`;
  `Terminal::foreground_reads_password` exposes it. In Marley: while the last block runs and the
  flag is set, the row reads `waiting for a password` and one notification
  `<command>: waiting for a password` goes out, once per block, agent terminals excluded.
- **The rail.** `TerminalSnapshot::command: Option<CommandSnapshot { text, running, exit_code,
  duration, password }>` in `marley_rail`, carried by `TerminalRow` and filled by `rail_rows`;
  `terminal_snapshot` fills it from the terminal's last block and the flag;
  `render_terminal_row` draws a line under the folder: `cargo build · running`,
  `cargo build · done · 45 s`, `cargo build · exit 101 · 4 m 12 s` (the exit in the error
  color), `sudo make install · waiting for a password`. The filter matches the command's text as
  it matches the title.
- The setting `marley.long_command_seconds` (u64, 30; 0 off) in `MarleySettingsContent`,
  `default.json`, `MarleySettings` and the settings page.
- `marley_terminal::duration_label(Duration) -> String` (pure): `45 s`, `4 m 12 s`, `1 h 2 m`,
  shared by the notification and the row.
- `script/e2e/551-command-end-from-outside.sh`.

### Out (explicitly deferred)
- A failure printed by a command that keeps running (a dev server): use 7 of
  `jev-system-one-2026-09-25.md`; it takes judgment.
- Agent terminals: #538 words their banners and their unread mark; #542 orders the rail.
- Blocks over ssh (#526): blocks stop at the first `ssh` today; the password check still sees
  ssh's own prompt, since it reads the local PTY.
- An unread mark and the rail's dot for a finished command: #538's mark, when it lands, is the
  one to set; here the row's red exit is the mark.
- Warp's stricter "only while Warp is not in front": Marley keeps #478's rule (the focused
  terminal of the active window is in front; every other terminal is not).
- A sound; a notification for the focused terminal; grouping bursts (#538's cooldown covers
  every Marley banner once it lands).

## Reference (§20)
- **Warp:** a notification when a command finishes after `long_running_threshold` (30 seconds,
  on by default) or waits for a password, only while Warp is not in front
  (docs.warp.dev/terminal/more-features/notifications/; `[notifications.preferences]` in
  docs.warp.dev/terminal/settings/all-settings/); the tab's error mark
  (docs.warp.dev/terminal/appearance/tabs-behavior/), the vertical tab titled with the last
  command (docs.warp.dev/terminal/windows/vertical-tabs/), the session palette's running and
  last commands (docs.warp.dev/terminal/sessions/session-navigation/). Marley matches the
  threshold, the two triggers, the row's command line and the filter; the rail is Marley's tab
  list (AD-claude-468).
- **Upstream Zed:** nothing notifies when a task or command ends; a finished task changes its
  tab icon (`crates/terminal_view/src/terminal_view.rs:1537`) and `Event::Bell` marks the tab
  (`:1177`). Marley keeps #478's notification path (AD-claude-478) and feeds it from blocks.
- **Warp's blocks map:** `docs/warp_architecture/subsystems/03-terminal-session-core.md` §6:
  a block carries its exit code and prompt context from the shell's hooks; Marley's blocks also
  carry their start and end times (`BlockTimes`).

### Prior art
- **Behavior maps and research.** The second pass's finding 3 (the termios claim, the
  agent-terminal exclusion, the 30 s default); #478's D3
  (`478-terminal-notifications.spec.md:70`: only when the terminal is not in front, the focused
  terminal in the active window); #538's scope (`538-notifications-with-content.spec.md:12`,
  `:36`: Claude Code's banners and the unread mark per terminal) and #542's
  (`542-rail-attention-order.spec.md:12`).
- **Published material.** Warp's pages above. termios(3): `ICANON` for canonical reads, `ECHO`
  for echo. On Linux `TCGETS` on a pty master answers the slave's settings, checked on the dev
  box on 2026-09-26 (kernel 7.2.5): a slave set to no echo reads back from the master as `ECHO`
  clear and `ICANON` set; a raw slave as both clear. The password readers (`sudo`, `ssh`,
  `read -s`, `passwd`) turn echo off and keep canonical mode; full-screen programs turn both off,
  which is what tells them apart.
- **The code we already ship.** `notifications.rs` carries a banner and its click: `:33` the
  `observe_new` on views, `:38` the terminal subscription, `:59` `notify`, `:66` the in-front
  gate, `:69` the per-terminal tag, `:76` `show_system_notification`, `:86` the click path.
  `Terminal::apply_shell_hook` (`crates/terminal/src/terminal.rs:1817`) stamps the times
  (`:1834`) and notifies (`:1835`) with no event, so an observer of the entity sees each block's
  end; `blocks` (`:1845`), `marley_anchored` (`:1852`) for `times`
  (`crates/marley_terminal/src/anchored.rs:170`, `BlockTimes` `:63`); `mcp.rs:477`
  `block_entry` computes a duration already. `ProcessIdGetter` holds the master fd
  (`crates/terminal/src/pty_info.rs:12`, `tcgetpgrp` at `:37`;
  `crates/terminal/src/alacritty.rs:66`); `ProcessInfo` (`:69`), `load` (`:177`) and
  `emit_title_changed_if_changed` (`:200`, the comparison `:208`, the event `:231`) refresh on
  each `Wakeup` (`terminal.rs:1777`, `:1782`); `libc` is a dependency of `crates/terminal`, and
  the vendored alacritty calls `tcgetattr` on the master at creation
  (`vendor/alacritty_terminal/src/tty/unix.rs:249`). The rail: `TerminalSnapshot`
  (`crates/marley_rail/src/marley_rail.rs:46`), `TerminalRow` (`:216`), `rail_rows` (`:505`);
  `terminal_snapshot` (`crates/marley_workbench/src/rail.rs:1717`, the agent at `:1729`), the
  filter on the title only (`:1852`; `filter_match` `:1919`), `render_terminal_row` (`:1319`),
  the `Wakeup` subscription (`:480`), the setting's reader `no_update_after_ms` (`:1788`). Does a
  crate we build own this seam? `notifications` and the rail own the display; the watch and the
  password read are new and small.

## UI proof
UI-AFFECTING: the rail row's line and the filter; the notifications are machine-checked.
`script/e2e/551-command-end-from-outside.sh` (Hyprland, keys only). Setup: a scratch
repository; a HOME whose `.bashrc` is the scenario's; the profile's settings with
`marley.long_command_seconds` at 2; `busctl --user monitor org.freedesktop.Notifications`
logging (L-claude-478); #519's stand-in `claude`. Steps: `ctrl-~` opens a second terminal;
`ctrl-tab` back to the first; `sleep 3; false`, Return, `ctrl-tab` at once; settle 5: the
first terminal's row reads `sleep 3; false · exit 1 · 3 s` in the error color
(`551-01-failed-row`), and the log holds one `Notify` with `sleep 3; false` and
`exit 1 after 3 s`. `ctrl-tab`, `true`, Return, `ctrl-tab`, settle 2: the `Notify` count
unchanged. `ctrl-tab`, `sleep 6`, Return, `ctrl-tab`, settle 2: `sleep 6 · running`
(`551-02-running-row`); settle 6: `sleep 6 · done · 6 s` (`551-03-done-row`) and a `Notify`
with `done in 6 s`. `ctrl-tab`, `read -s -p 'Password: ' x`, Return, `ctrl-tab`, settle 2:
`read -s -p 'Password: ' x · waiting for a password` (`551-04-password-row`) and one `Notify`
with `waiting for a password`; `ctrl-tab`, `secret`, Return. `sleep 3; false` in the focused
terminal, settle 5: no new `Notify`. The palette's focus of the sidebar filter
(`FocusSidebarFilter`), `sleep`: the first terminal's row shown and matched (`551-05-filter`);
Escape. `claude`, Return, Return (the stand-in's steps), `ctrl-d` after 3 s, `ctrl-tab`: no
`Notify` for that block.

## Locked-In Decisions
- D1: Blocks are the source, not the process tree: a block ends when the shell's `precmd` frame
  arrives with the exit code and the stamped end time, in any shell with Marley's hooks, local or
  (after #526) remote.
- D2: Warp's threshold, 30 s, as `marley.long_command_seconds`; 0 turns the end notification
  off; the row's line and the password check stay on.
- D3: Never for agent terminals (Chad's default): a block whose terminal's foreground program is
  a known agent, at its end or while it runs, posts nothing, and its row keeps the agent's lines
  (#519, #538).
- D4: #478's rule for "in front": the focused terminal of the active window; every other
  terminal, in the same window or not, gets the notification. Warp's rule is stricter (only while
  Warp is in the background); a cockpit with six terminals in one window needs the looser one.
- D5: One notification per block at its end, and one password notification per block when the
  flag first turns on while it runs. A block that ends while the flag is on (a wrong password,
  Ctrl-C) posts its end as any block.
- D6: The termios read sits with the foreground-process refresh, the same `Wakeup`-driven
  `load`, not a read per output chunk. `tcgetattr` on the master is one syscall and answers the
  slave's flags on Linux (verified 2026-09-26); where it does not, the flag stays false.
- D7: The row's line sits under the folder line, and only for a terminal with a block: the
  command cut to the row's width, then `running`, `done · <duration>`, `exit <code> · <duration>`
  with the exit in the error color, or `waiting for a password`.
- D8: The duration text comes from one pure function, `45 s`, `4 m 12 s`, `1 h 2 m`; the
  notification's body is `done in <duration>` or `exit <code> after <duration>`.
- D9: The filter matches the command's text as it matches the title, so `cargo` finds the
  terminal that is building.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a block ends after at least `marley.long_command_seconds` in a terminal that is not the focused terminal of the active window, the system shall post one desktop notification titled with the command whose body reads `done in <duration>` or `exit <code> after <duration>`. | The run log: `busctl` records one `Notify` with `sleep 3; false` and `exit 1 after 3 s`, and one with `done in 6 s` |
| REQ-002 | WHEN a block ends under the threshold, or in the focused terminal of the active window, the system shall post nothing. | The log: the `Notify` count unchanged after `true`, and after `sleep 3; false` in the focused terminal |
| REQ-003 | WHILE a block runs, the terminal's rail row shall show the command and `running`. | Shot `551-02-running-row` |
| REQ-004 | WHEN a block ends, the row shall show the command, `done` or `exit <code>` in the error color, and the duration. | Shots `551-01-failed-row`, `551-03-done-row` |
| REQ-005 | WHILE the running block's PTY reads with echo off in canonical mode, the row shall read `waiting for a password`, and the system shall post one notification for it when the terminal is not in front. | Shot `551-04-password-row`; the log: one `Notify` with `waiting for a password` |
| REQ-006 | WHEN the rail's filter holds text found in a terminal's running or last command, the system shall show and match that terminal's row. | Shot `551-05-filter` |
| REQ-007 | WHEN a block ends in a terminal whose foreground program is a known agent, the system shall post nothing and change no row. | The log: no `Notify` for the stand-in's block |
| REQ-008 | The setting shall appear on the Marley settings page, and `default.json` shall set it to 30. | The 515 scenario's page shot; review of `default.json` |
| REQ-009 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion re-verify the
  seams and rerun the termios probe on the current kernel.
- **P2 Code:** the ledger rows first (`crates/terminal/src/pty_info.rs` new,
  `crates/terminal/src/terminal.rs` updated, the settings files); `duration_label`; the flag;
  `command_watch.rs`; the rail's model, row and filter; the setting. fmt and clippy clean; a
  review of the diff (§14: the `tcgetattr` call carries its `// SAFETY:`; no notification for
  agents; one per block).
- **P3 Test:** write and run the scenario, read every shot and the `busctl` log; `just regress`;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_rail.md`, `marley_workbench.md`
  and `terminal_blocks.md`; the touchpoints rows checked; the plan's slice status; the ledger
  capture; close the ticket, archive, commit.
