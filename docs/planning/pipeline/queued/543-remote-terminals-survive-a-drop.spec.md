---
pipeline_id: 4330ab79-0535-415d-b14c-ab45b74cd5d3
ticket: docs/planning/tickets/open/TICKET-543-remote-terminals-survive-a-drop.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Remote terminals that survive a dropped link"
type: feature
slice: prong 2, remote (report 04 §3.2 item 5, §3.4 rows 5 and 6); a stopgap before the harness's remote entry
references: [docs/planning/tickets/open/TICKET-519-claude-code-events-in-the-rail.md, docs/orca_architecture/04-remote-control-and-mobile.md, docs/marley_architecture/marley_remote.md, docs/planning/pipeline/completed/482-claude-code-notifications-chip.spec.md]
---

## Title
`marley: open remote terminal` opens a terminal on one of the SSH hosts saved in Zed's settings,
whose shell runs in a tmux session of Marley's own on the host. When the link drops, the shell and
any agent in it keep running, and Rerun on the terminal reattaches to the same session. Claude
Code's notifications from inside that session reach Marley through tmux's passthrough. This wires
`marley_remote`, which has had no caller since the fork.

## Scope
### In
- `crates/marley_remote/src/marley_remote.rs` (pure): `remote_terminal_command(target, session)`
  returns the argv `ssh -t [-p <port>] -- <[user@]host> tmux -L marley -f /dev/null new-session
  -A -s <session> -e MARLEY_REMOTE=1` followed by the server's options as tmux commands (`\;`
  between them, escaped for the remote shell): no status line, no prefix key, the mouse on,
  `allow-passthrough on`, and 24-bit colour for `xterm-256color`. A `SessionName` newtype makes
  `marley-` and eight lowercase hex digits and refuses any name outside `[a-z0-9-]`.
  `ssh_command` stays as it is.
- `crates/marley_workbench/src/remote.rs` (new, Marley-owned): the action `marley: open remote
  terminal` opens a picker of Zed's saved SSH hosts (`RemoteSettings::ssh_connections`,
  `crates/recent_projects/src/remote_connections.rs:30-35`), by nickname or host; each entry's
  host, user and port go through `parse_ssh_target`'s checks, and an entry that fails them is left
  out. Confirming schedules a Zed task through `Workspace::schedule_resolved_task`
  (`crates/workspace/src/tasks.rs:64`): command `ssh` with the argv's tail, labelled
  `<nickname or host> · <session>`, revealed in the center, with `show_rerun` on and
  `use_new_terminal` off. Zed's task terminal then shows the state on its tab (running, then a
  cross when ssh exits: `crates/terminal_view/src/terminal_view.rs:1542-1561`), and its Rerun
  (`RerunTask`, which reruns in the same terminal: `terminal_view.rs:1141-1148`) runs the same
  argv, which reattaches.
- The plugin's gate: `notify.sh` (line 5) and, once #519 has landed, `hooks/event.py` (whichever
  the plugin still has, since #538 removes `notify.sh`) also pass when `MARLEY_REMOTE` is `1`,
  which tmux sets for the session's shells. Nothing else in the hooks
  changes: inside tmux, Claude Code 2.1.283 wraps each `terminalSequence` it accepts in tmux's
  passthrough itself, and Marley's tmux server allows passthrough, so the OSC 777 reaches Marley's
  terminal. `plugin.json`'s version rises, and #519's chip offers the update.
- `docs/marley_architecture/marley_remote.md` says what the fork has and what it lacks (its
  "SHIPPED" sections describe the gpui-era app).

### Out (explicitly deferred)
- Reattaching after a Marley restart, and ending a host's sessions from Marley: the next slice,
  listing `tmux -L marley list-sessions` over ssh in the picker. A task terminal is not saved with
  the workspace (`TerminalView::serialize` returns nothing for a task, `terminal_view.rs:1941-1943`),
  so after a restart the session waits on the host until reattached by hand or ended by `exit`.
- Zed's remote-project terminals (`create_remote_shell`, `crates/project/src/terminals.rs:608`,
  whose `ssh -t <dest> "cd … && exec env … <shell> -l"` comes from `build_command_posix`,
  `crates/remote/src/transport/ssh.rs:1850-1966`): wrapping them is a Zed crate change for a
  later slice.
- Installing the plugin on a remote host: by hand in this slice (the marketplace folder copied,
  then `claude plugin marketplace add` and `claude plugin install` there).
- Typed targets not saved in settings, hosts from `~/.ssh/config`, a saved entry's own `args` and
  port forwards.
- Marley's blocks and scrollback inside tmux: tmux draws the screen, so Marley's terminal holds the
  screen and tmux holds the history, scrolled with the wheel. #526 brings blocks to a plain ssh.
- GNU screen and zellij on the host.
- rustal-harness's remote entry (`rh remote serve` as an OpenSSH `ForceCommand`), the long-term
  answer, which replaces the tmux wrapper once the harness is embedded.

## Reference (§20)
Warp: N/A. Its SSH wrapper and "Warpify" bring blocks to a remote shell
(docs.warp.dev/terminal/warpify/subshells/, the once-over's item 3) and say nothing of surviving a
dropped link. Upstream Zed's remote development keeps its `remote_server` link with a 5-second
heartbeat and three reconnect attempts (`crates/remote/src/remote_client.rs:160-166`), but each of
its terminals is an ssh process of its own that ends with the link (`create_remote_shell`). Zed's
task terminals, with their state on the tab and their Rerun, are what Marley reuses for the
reconnect. The behavior matched is Orca's SSH worktrees (report 04 §2.10): PTYs live in a daemon on
the remote host, so "a sleeping laptop leaves agents running and they reattach". Marley gets the
same outcome from tmux, found on any host, in place of Orca's Node daemon, and later from
rustal-harness, whose remote view marks a lost connection stale and reconnects "to the original
workspace, pane and process" (`/srv/stacks/rustal-harness/docs/REMOTE.md`).

### Prior art
- **Behavior maps.** Report 04 §2.10 (Orca's SSH worktrees and their reconnect), §3.2 item 5 (the
  tmux argv, the `TERM_PROGRAM` gate, the passthrough), §3.4 rows 5 and 6, §4 (no Node relay on
  remote hosts); docs/marley_architecture/detached-sessions.md (the gpui-era note: "A detached
  remote session is just `ssh dev -t 'tmux attach -t agents'`").
- **Published material.** tmux 3.7c's manual, installed on the box: `allow-passthrough [on | off |
  all]` lets a program "bypass tmux using a terminal escape sequence (\ePtmux;...\e\\)", and with
  `on` only while the pane is visible; tmux forwards OSC 9;4 progress, OSC 7 and titles, and no OSC 9
  or 777 notification of its own. Claude Code 2.1.283's installed bundle, read and not run: a hook's
  `terminalSequence` may hold "Only notification/title OSCs (0, 1, 2, 9, 99, 777) and BEL";
  "anything else is dropped"; and the sanitizer rebuilds each accepted OSC and wraps it as
  `ESC P tmux; … ESC \` when `TMUX` is set (the screen form when `STY` is). #519's spec found the
  same. So report 04's plan for the hook to wrap its own sequence is not needed, and would not
  work: a DCS in `terminalSequence` is dropped.
- **Code we already ship.** `crates/marley_remote/src/marley_remote.rs` (`parse_ssh_target` 43-71
  with its leading-dash guard, `ssh_command` 104-116 with its `--`, `RemoteHost` and
  `remote_palette_actions` 156-192); no crate in the tree depends on it (the CHANGELOG's port entry,
  2026-09-18: "Not yet wired into the app"). Zed's saved hosts, `SshConnection`
  (`crates/settings_content/src/settings_content.rs:1397-1418`: host, username, port, args,
  nickname). Zed's tasks: `TaskTemplate` (`crates/task/src/task_template.rs:24`, `resolve_task`
  at 162), `SpawnInTerminal`'s `use_new_terminal`, `reveal_target` and `show_rerun`
  (`crates/task/src/task.rs:61-77`), the inventory's `last_scheduled_task` that Rerun reads
  (`crates/tasks_ui/src/tasks_ui.rs:106-117`). `crates/terminal/src/terminal.rs:715-725`: Zed puts
  `TERM_PROGRAM=zed` in a remote project's terminal, but a plain ssh does not carry it, and inside
  tmux the variable reads `tmux`.

## UI proof
UI-AFFECTING (a new action and picker, the remote terminal, its tab and its reattach).
`script/e2e/543-remote-terminals-survive-a-drop.sh` (keys only, Hyprland's hidden workspace). A
fake `ssh` first on the PATH acts out a host on this box: it logs its argv and pid, drops the
options and runs the remote command with a scratch `TMUX_TMPDIR` and `HOME`, so the run's tmux
server is its own and Chad's is never touched. The profile copy's settings save two hosts,
`e2e-host` and one whose host starts with `-o`. Marley runs on a private D-Bus bus with a fake
notification server, as #538's scenario does. Inside the session, a fake `claude` runs the plugin's
real `notify.sh` and, as Claude Code 2.1.283 does under tmux, wraps the answer's
`terminalSequence` in tmux's passthrough before writing it to its terminal. Shots:
`543-01-picker` (the picker lists `e2e-host` only), `543-02-connected` (the task terminal on the
tmux shell, a counter printing a tick a second), `543-03-dropped` (the fake ssh killed by its
logged pid: the tab's cross, the output stopped), `543-04-reattached` (after `terminal: rerun
task`, the same shell with the counter past where it stopped), `543-05-notified` (the hook run
inside the session while another terminal holds focus: the remote terminal's row marked, and the
banner in the bus's log). Test adds a live check with a real Claude Code inside `tmux -L marley`,
by L-claude-482's pty method, that its own wrap reaches the outer side.

## Locked-In Decisions
- D1 — tmux on the host, not a daemon of Marley's: tmux is on every host Chad reaches, and the
  embedded harness replaces it later. `-L marley` keeps Marley's sessions on a server of their own,
  apart from Chad's own tmux there, and `-f /dev/null` keeps his `~/.tmux.conf` out of it.
- D2 — The server's options make tmux invisible: no status line; no prefix key, so Ctrl+B reaches
  Claude Code, which binds it to "run in background" and under tmux otherwise asks for it twice;
  the mouse on for tmux's history; passthrough on; 24-bit colour.
- D3 — The reconnect is Zed's task Rerun, and the terminal is Zed's task terminal: its tab already
  shows running and ended, it is never restored as a local shell, and Rerun runs the same argv in
  the same terminal.
- D4 — One session name per terminal, `marley-` and eight hex digits, fixed for the terminal's life,
  so Rerun reattaches that session and no other.
- D5 — The remote command is fixed words and the session name, checked to `[a-z0-9-]`: nothing a
  user typed reaches the remote shell, and the host still goes through `parse_ssh_target` and the
  `--`, as `marley_remote` has always required.
- D6 — The wrap is Claude Code's. It wraps a hook's accepted sequence for tmux when `TMUX` is set,
  so the hooks answer inside tmux as they do elsewhere, and only their gate changes. A hook writing
  to `/dev/tty` itself, which AD-claude-482 rejected, stays rejected.
- D7 — `MARLEY_REMOTE=1`, set by `new-session -e`, marks a Marley terminal where `TERM_PROGRAM`
  reads `tmux`; a plain tmux or another terminal stays silent as today.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user runs `marley: open remote terminal`, the system shall list the SSH hosts saved in Zed's settings whose host, user and port pass `parse_ssh_target`'s checks. | Shot `543-01-picker` |
| REQ-002 | WHEN the user picks a host, the system shall open a terminal running `ssh -t … -- <dest> tmux -L marley -f /dev/null new-session -A -s <session> …` with a session name unique to that terminal. | The fake ssh's argv log; shot `543-02-connected` |
| REQ-003 | WHEN a remote terminal's ssh process exits, the system shall keep the terminal open and mark its tab as ended. | Shot `543-03-dropped` |
| REQ-004 | WHEN the user reruns an ended remote terminal, the system shall run the same argv in the same terminal, which shall show the session's shell with the output it made meanwhile. | Shot `543-04-reattached`; the argv log's second entry equals the first |
| REQ-005 | WHEN Claude Code's notification hook runs in a shell of Marley's tmux session, its OSC 777 shall reach Marley through tmux's passthrough, and the system shall mark the terminal and show the banner. | Shot `543-05-notified`; the bus's log; Test's live check |
| REQ-006 | WHEN the hook runs where neither `TERM_PROGRAM` is `zed` nor `MARLEY_REMOTE` is `1`, it shall answer nothing. | A scenario part running the hook with `TERM_PROGRAM=tmux` and no `MARLEY_REMOTE`: its output is empty |
| REQ-007 | The remote command shall hold only fixed words and a session name of `[a-z0-9-]`. | The argv log; a review of the builder |

## Phase Plan
- **P1 Plan** — this spec, and the design and the test plan in the notes; `brain_ask` at
  promotion.
- **P2 Code** — the builder in `marley_remote`; `remote.rs` and its action in `marley_workbench`
  (with `marley_remote` as its dependency); the plugin's gate and version; fmt and clippy clean; a
  review of the diff.
- **P3 Test** — write and run the scenario and read every shot; the live check; one real host by
  hand (tmux 3.3 or later there); `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_remote.md` and
  `marley_workbench.md`, the three-prong plan's prong 2 section, ledger capture, close the ticket,
  archive, commit.
