# Remote terminals that survive a dropped link — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-543-remote-terminals-survive-a-drop.md
- **Pipeline spec:** 543-remote-terminals-survive-a-drop.spec.md

## Phase 1 — Plan
- **Request:** from the Orca survey Chad asked for on 2026-09-25 (report 04 §3.2 item 5, §3.4 rows
  5 and 6): `marley_remote`'s ssh command wraps the remote shell in `tmux -L marley new-session -A
  -s <id>` so a dropped link leaves the shell and its agent running and the next connect
  reattaches; the plugin's `notify.sh` gate, which checks `TERM_PROGRAM`, and tmux's passthrough
  for OSC 777. The embedded rustal-harness is the long-term answer.
- **Classification / tier:** feature, prong 2 (remote). Rust in `marley_remote` and
  `marley_workbench` (Marley crates) and the plugin's hook gates; no Zed crate. Queue.
- **What the discovery changed.**
  - Report 04 §2.10 describes `marley_remote` as adding "connect:" palette actions and a ⇄ or ✗
    badge. That was the gpui-era app. In the fork nothing depends on the crate (only the
    workspace's member list names it), and the CHANGELOG's port entry of 2026-09-18 says the five
    ported crates were "Not yet wired into the app". Wrapping `ssh_command` alone would change
    nothing Chad can see, so the slice opens the terminal too.
  - Report 04 §3.2 item 5 says the hook must wrap its OSC 777 in tmux's passthrough inside `$TMUX`.
    Claude Code 2.1.283 already does: its `terminalSequence` sanitizer rebuilds each accepted OSC
    and wraps it as `ESC P tmux; … ESC \` when `TMUX` is set, and screen's form when `STY` is. A
    hook that wrapped its own sequence would be refused by the same sanitizer, whose allowlist
    takes only OSC 0, 1, 2, 9, 99 and 777 and BEL. So the plugin changes only its gate, and
    Marley's tmux server turns passthrough on. #519's spec read the same bundle and says so.
- **Recall (§18.3):**
  - AD-claude-482-claude-code-sends-marleys-notifications-through-a-plugin-001: the plugin answers
    with `terminalSequence`, only where `TERM_PROGRAM` is `zed`; rejected then, hooks writing to
    `/dev/tty` themselves. That stands (D6).
  - L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001: test a hook's
    sequence with an interactive Claude Code in a Python pty, the method of Test's live check.
  - L-claude-485-log-the-bytes-a-shell-writes-inside-marley-with-script-001: `script -q -f` logs a
    program's own bytes, if the live check needs to see the wrapped sequence.
  - L-claude-448-running-zeds-dylint-library-on-the-fork-001 (cited in #502's notes): `pgrep -f`
    matches more than it means to. The fake ssh is killed by the pid it logged; `pkill ssh` would
    end Chad's own ssh sessions on this box.
  - The dev-box skill describes Chad's own tmux-wrapped SSH sessions to the Mac and the agent fleet;
    `-L marley` keeps Marley's sessions apart from those.
  - Brain: not consulted at drafting (read-only overnight drafting); the promotion runs
    `brain_ask`.
- **Discovery:**
  - `crates/marley_remote/src/marley_remote.rs`: `SshTarget` (28-35), `parse_ssh_target` (43-71,
    the leading-dash rejection at 57-63), `ssh_command` (104-116: `ssh [-p N] -- [user@]host`),
    `RemoteStatus` and its glyph (120-145), `RemoteHost` and `remote_palette_actions` (156-192).
    `crates/marley_remote/Cargo.toml` depends on `serde` only.
  - `docs/marley_architecture/marley_remote.md` (2026-07-12, "current to M15"): #84 to #87 marked
    SHIPPED in the gpui-era `RootView`; `docs/marley_architecture/detached-sessions.md:49`.
  - `crates/recent_projects/src/remote_connections.rs:30-35`: `RemoteSettings { ssh_connections,
    wsl_connections, read_ssh_config }`; `crates/settings_content/src/settings_content.rs:1397-1418`:
    `SshConnection`.
  - Zed's tasks: `crates/task/src/task.rs` (`SpawnInTerminal` at 42, `use_new_terminal` 61,
    `allow_concurrent_runs` 63, `reveal_target` 67, `hide` 69, `show_rerun` 77);
    `crates/task/src/task_template.rs` (`TaskTemplate` 24, `shell` 68, `resolve_task` 162);
    `crates/workspace/src/tasks.rs:64-140` (`schedule_resolved_task`, which records the task in the
    inventory and spawns it through the terminal provider); `crates/tasks_ui/src/tasks_ui.rs:106-117`
    (Rerun reads `last_scheduled_task`); `crates/terminal_view/src/terminal_view.rs` (`RerunTask`
    94-98 in the `terminal` namespace, `rerun_task` 686-693, `rerun_button` 1121,
    `terminal_rerun_override` 1141-1148 with `use_new_terminal: Some(false)`, the tab's icon by
    task status 1542-1561, `serialize` skipping tasks 1941-1943).
  - Zed's remote terminals: `create_remote_shell` (`crates/project/src/terminals.rs:608-642`),
    which inserts Zed's env and asks `build_command` for an interactive command titled
    `<host> — Terminal`; `build_command_posix` (`crates/remote/src/transport/ssh.rs:1850-1966`):
    `ssh <options> -o LogLevel=ERROR -t <dest> "cd <dir> && exec env <vars> <shell> -l"`;
    `crates/remote/src/remote_client.rs:160-166`: 5 missed heartbeats, a 5-second interval, 3
    reconnect attempts.
  - The plugin: `notify.sh` (the gate at line 5, the answer at 14), `hooks.json`,
    `crates/marley_workbench/src/claude_plugin.rs` (the files compiled in, written at install;
    `plugin.json` at `1.1.0`). #519 adds `hooks/event.py` with the same `TERM_PROGRAM` gate, moves
    the version to 1.2.0 and gives the agent bar's chip an "Update Marley's plugin" state.
  - Claude Code 2.1.283's bundle (`~/.local/share/claude/versions/2.1.283`), read with `grep`: the
    hook answer's sanitizer maps each accepted OSC through a wrapper that returns
    `\x1BPtmux;` + the sequence with every ESC doubled + `\x1B\\` when its multiplexer check sees
    `TMUX` (screen's form for `STY`); the writer then hands it to the interactive UI. The same
    bundle binds Ctrl+B to `task:background` and tells a tmux user to press it twice.
  - The box: `tmux` 3.7c and `openssh` 10.5 installed. tmux's manual: `allow-passthrough` is a
    pane option (`on` passes only while the pane is visible); OSC 9;4 progress (`Spb`), the working
    directory notification (`Swd`) and titles are forwarded, and no OSC 9 or 777 notification is.
  - #526 (drafted the same night) wraps a plain `ssh` typed in a Marley shell with Marley's shell
    integration and passes one with a remote command through as typed, so this ticket's argv is
    untouched when the task's shell runs it.
  - rustal-harness: `docs/REMOTE.md` (the remote native view over an OpenSSH `ForceCommand`, no
    network listener; "Runtime connection loss makes the view stale. `Ctrl-b r` reconnects to the
    original workspace, pane and process as an observer"); `docs/TERMINALS.md` (tmux terminals
    outlive the runtime and its clients).
- **Decisions:** D1 to D7 in the spec.

### Design (at promotion)
Re-verified against the tree on 2026-09-29; this section wins where the drafted design differs.
- **The plugin since #519 and #538.** `notify.sh` is gone; `hooks/event.py`'s `main` gates on
  `TERM_PROGRAM == "zed"` (line 137) and answers #519's `marley-event` frames, which #538 turns into
  banners and the unread mark. The gate also passes on `MARLEY_REMOTE == "1"`; `plugin.json` goes
  from 1.5.0 to 1.6.0, and #547's chip offers the update.
- **Frames from a remote terminal.** `agent_events::on_frame` drops a frame unless
  `agent_bar::agent_in` finds Claude Code in the terminal's foreground, and in a remote terminal
  the foreground is `ssh`. A Marley remote terminal (its task's id starts with `marley-remote`,
  the id base Marley resolves it with) counts as Claude Code's for frames. The rail's
  `terminal_snapshot` takes such a terminal with a live seat as Claude Code's, so its row shows the
  seat's status, and `note_claude_code` then leaves its seat alone: only a `SessionEnd` from the
  host ends it. A remote Claude Code that dies without one leaves its last state on the row, which
  `no update in N m` marks.
- **The ssh program.** `MARLEY_SSH`, when set, names the program the task runs in place of `ssh`,
  as `MARLEY_CLAUDE` does for #547: a task's PATH may come from the login shell, and a scenario's
  fake must never lose to the real ssh.
- **The task.** `marley_workbench` already depends on `recent_projects` (`RemoteSettings`),
  `task` and `rand`. `routing.rs`'s `RoutedTerminals::spawn` puts a task in the center in the
  Marley layout and reruns it in its last terminal, so Zed's Rerun (`terminal: rerun task`,
  `zed_actions::Rerun` with the task's id and `use_new_terminal: false`) reattaches in place.
  The task is scheduled with `schedule_resolved_task` so the inventory keeps it for Rerun.
- **Checks outside the scenario.** The live check with a real Claude Code and the real-host drop
  are left for the end with #587's and #535's hand checks: scenarios never start the real `claude`
  and the drop needs a host Chad names.
- **Compositor.** The scenario runs under `compositor sway` like the others since #519.
- **Brain.** Consultation ad5ba3ce6cbe4f4e98072882b57a9cbe: nothing on this seam.

### Design
- **The argv.** `remote_terminal_command(&SshTarget, &SessionName) -> Vec<String>`: `ssh`, `-t`,
  `-p N` when a port is given, `--`, the destination, then the remote command's words: `tmux -L
  marley -f /dev/null new-session -A -s <name> -e MARLEY_REMOTE=1` and, each after a `\;` word,
  `set-option -g status off`, `set-option -g prefix None`, `set-option -g mouse on`,
  `set-option -g allow-passthrough on` and `set-option -as terminal-features ,xterm-256color:RGB`.
  ssh joins the words after the destination with spaces and the remote shell parses them, so `\;`
  reaches tmux as `;`, and every other word is plain text with no shell meaning. `SessionName` has
  a private field; `SessionName::new()` makes eight random hex digits and `SessionName::parse`
  accepts only `[a-z0-9-]`. Whether `set-option` after `new-session -A` also applies on a reattach
  is checked at Code on tmux 3.7c (it should: the commands run on the server either way).
- **The action.** `remote.rs` registers `marley: open remote terminal` on every workspace, as
  `agents.rs` registers `NewAgent`; its picker, a `PickerDelegate` like the New Agent picker, lists
  `RemoteSettings::get_global(cx).ssh_connections()`, each mapped to `SshTarget` through
  `parse_ssh_target` on `[user@]host[:port]` and filtered. Confirm builds a `TaskTemplate` (label,
  `command: "ssh"`, `args`, `reveal_target: Center`, `show_rerun: true`, `use_new_terminal: false`,
  `allow_concurrent_runs: false`), resolves it with an empty `TaskContext`, and schedules it with
  `schedule_resolved_task`. Where the Marley layout puts a center terminal is read at Code
  (`crates/marley_workbench/src/routing.rs`).
- **The hooks.** The gate in `notify.sh` becomes `TERM_PROGRAM=zed` or `MARLEY_REMOTE=1`, and
  `event.py`'s the same once #519 has landed. The answer stays a plain `terminalSequence`.
- **File manifest.** `crates/marley_remote/src/marley_remote.rs` (Marley crate; the random source
  for the name taken from a crate `Cargo.lock` already has, checked at Code);
  `crates/marley_workbench/src/remote.rs` (new) and `marley_workbench.rs` (`mod remote`,
  `remote::init`), `crates/marley_workbench/Cargo.toml` (`marley_remote`); the plugin's `notify.sh`,
  `event.py` if present, and `plugin.json`; `docs/marley_architecture/marley_remote.md`;
  `script/e2e/543-remote-terminals-survive-a-drop.sh` and its fakes (Test). No Zed path, so no
  touchpoint row.
- **Ledger rows.** None up front. At Complete: a lesson that Claude Code wraps a hook's
  `terminalSequence` for tmux itself (2.1.283) and that tmux forwards it only with
  `allow-passthrough`; an architecture decision for the tmux wrapper and its retirement by the
  harness.

### E2E plan
Shared fixtures: the fake `ssh` (logs `argv` and `$$` to `$E2E_WORK/ssh.log`, skips `-t`, `-p N`
and `--`, drops the destination, `exec`s `sh -c` on the remaining words with `TMUX_TMPDIR` and
`HOME` pointed into `$E2E_WORK`); the profile copy's settings with `ssh_connections`
`[{"host": "e2e-host", "nickname": "e2e"}, {"host": "-oProxyCommand=touch pwned"}]`; a `.bashrc` in
the scratch HOME that gives a plain prompt; a private session bus with the fake notification server
(#538's helper); a fake `claude` that runs the plugin's real `notify.sh finished` with
`CLAUDE_PROJECT_DIR` set, takes `terminalSequence` from its answer, and writes it wrapped for tmux,
as Claude Code does when `TMUX` is set.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | Palette: `marley: open remote terminal` | `543-01-picker`: `e2e` listed, the `-o` entry absent; no `pwned` file anywhere in `$E2E_WORK` |
| REQ-002 | Enter on `e2e`; in the shell, start a counter in the background (`n=0; while :; do n=$((n+1)); echo "tick $n"; sleep 1; done &`), so the shell still takes commands | `543-02-connected`; the argv log's first line, with `-s marley-<8 hex>` and `MARLEY_REMOTE=1` |
| REQ-003 | `kill -HUP` the pid the fake logged; settle | `543-03-dropped`: the tab's cross, no new ticks |
| REQ-004 | Ten seconds later, `terminal: rerun task` from the palette with the remote terminal focused | `543-04-reattached`: ticks about ten past where they stopped; the argv log's second line equals the first |
| REQ-005 | In the session's shell, `sleep 3; claude` (the fake); within the three seconds, open a second terminal so focus moves there | `543-05-notified`: the remote terminal's row marked; the bus's log: summary `Claude Code`, body `<folder> finished` |
| REQ-006 | In a plain local terminal: `env -u MARLEY_REMOTE TERM_PROGRAM=tmux sh <plugin>/hooks/notify.sh finished` | Its output is empty |
| REQ-007 | The argv log | Every word after the destination is one of the fixed words or the session name |

Not reachable by a scenario: a real host over a real network, and a real Claude Code turn there.
Test runs the live check (a real `claude --plugin-dir <plugin>` in a Python pty inside `tmux -L
marley-check` with passthrough on, L-claude-482's method: the outer side's bytes hold the OSC 777),
then one real host by hand, on a host Chad names (tmux 3.3 or later for `allow-passthrough`): drop
the link by suspending the laptop or cutting its network, rerun, and note what came back.

### Risks
- **Leftover sessions.** Closing a remote terminal's tab detaches; the session keeps running on the
  host until its shell exits. With no reattach after a restart in this slice (Out), sessions can
  pile up on a host. The next slice lists them in the picker with reattach and end.
- **tmux versions.** `allow-passthrough` came with tmux 3.3 and `new-session -e` with 3.2; an older
  tmux refuses the command, and the terminal shows its error. The picker cannot know a host's tmux
  before connecting; the error in the terminal is the message.
- **Claude Code's wrap is undocumented.** It sits in the bundle, not in the hooks reference; a
  later Claude Code could drop it, and notifications from inside tmux would stop. Test's live check
  records the version it proved, and the lesson names it.
- **Hidden panes.** With `allow-passthrough on`, a sequence sent while no client is attached (the
  link down) is dropped: a notification during a drop is lost, and the reattached terminal shows
  the agent's state on screen.
- **The plugin update.** An installed plugin keeps its old hooks until updated; the version rises,
  and #519's chip offers the update. If this ticket lands before #519, the chip's update state does
  not exist yet, and Test reinstalls the plugin by hand.
- **The Marley layout's routing.** A task terminal revealed in the center should land where the
  Marley layout puts terminals; `routing.rs` is read at Code.

## Phase 2 — Code
- **Built.** `marley_remote`: `saved_target`, `SessionName` (`from_bits`, `parse`, `as_str`) and
  `remote_terminal_command` with the server's options. `marley_workbench`: `remote.rs` (the action
  `marley::OpenRemoteTerminal`, the saved-host picker, the task with the id base `marley-remote`,
  `MARLEY_SSH`, `is_remote`); `agent_events::on_frame` lets a remote terminal's frames in; the
  rail's `remote_claude` makes such a terminal with a seat Claude Code's; the manifests
  (`marley_remote` in the workspace's dependencies, its ledger row grown). The plugin: `event.py`'s
  gate also passes on `MARLEY_REMOTE=1`; 1.5.0 to 1.6.0 in `plugin.json` and `marketplace.json`.
- **Deviations.** The frame gate, the rail's kind and `note_claude_code`'s end list were not in the
  drafted design (they came with #519 after it). The live check with a real Claude Code and the
  real-host drop wait for the end with #587's and #535's hand checks.
- **Review.** The host and user pass `parse_ssh_target` and sit after `--`; every word is quoted for
  the task's shell; the remote command's words are fixed, the session name or `\;`. A project not
  on this machine is refused, since its task would run ssh on that project's host. No entity is
  read or updated inside its own update.

## Phase 3 — Test
- **Scenario.** `script/e2e/543-remote-terminals-survive-a-drop.sh`, under sway, on a private
  session bus, with a stand-in ssh (`MARLEY_SSH`) that runs the remote command here with a short
  `TMUX_TMPDIR` and a HOME of its own, and a stand-in Claude Code on the "host" that wraps the
  plugin's real answers for tmux.
- **Fixes the runs found.**
  1. The first runs logged `;` where the builder wrote `\;`: a Zed task's arguments reach its shell
     unquoted (`build_no_quote`). The workbench now quotes each word for the system shell and the
     builder keeps the exec-correct `\;` (L-claude-543-a-zed-tasks-arguments-are-shell-text-001).
  2. No frame reached `on_frame` after the Rerun, though tmux forwarded it (checked on a scratch
     server with `script -f`, and before the drop in the scenario). `TerminalView::set_terminal`
     gives the view a new terminal on Rerun, and `notifications::init` had subscribed to the first
     one only. `notifications.rs` now watches the view's new terminal
     (F-claude-543-a-rerun-task-lost-its-notifications-001, PR-claude-543-…-001). `close_guard.rs`
     and `command_watch.rs` keep the same gap for a rerun task; not in scope.
  3. The banner names the folder the host's Claude Code runs in (`remote-home`), not the local
     project; the check was set to that.
- **Shots (run 11, all read).**
  - `543-01-picker`: `e2e` with `e2e-host` at its end; `smuggled` (host `-oProxyCommand=…`) absent.
    REQ-001.
  - `543-02-connected`: tab `▶ e2e · marley-192a94f1`, `remote$` prompt with no tmux status line,
    ticks 1 to 5. The log: one argv, `-t -- e2e-host tmux -L marley -f /dev/null new-session -A -s
    marley-192a94f1 -e MARLEY_REMOTE=1 \; set-option …`, every word fixed or the name. REQ-002,
    REQ-007.
  - `543-03-dropped`: the tab's red cross, `[lost tty]` and "Task `e2e · marley-192a94f1` finished
    with exit code: 1"; the terminal stays open. REQ-003.
  - `543-04-reattached`: after `terminal: rerun task`, the same tab and session, ticks at 21, past
    the 5 on screen at the drop. The log's second argv equals the first. REQ-004.
  - `543-05-notified`: a second local terminal in front; the remote terminal's row reads Claude
    Code, `idle · Tidy the imports`, "Tidied the imports on the host.", with the unread dot, listed
    first by #542's order. The private bus: `Marley|remote-home: Claude finished|Tidied the imports
    on the host.` REQ-005.
  - Setup: the hook answers `{}` with `TERM_PROGRAM=tmux` and no `MARLEY_REMOTE`, and a sequence
    with it. REQ-006. Nothing with `STRING "Marley"` reached the user's bus.
- **Gate.** `just gate-diff` after the fixes: gate:14 red on a module doc's link to the private
  `is_remote`, unlinked; then GATE GREEN [diff].
- **Not reachable here.** A real host over a real network and a real Claude Code there: Chad's hand
  check at the end, on a host he names (tmux 3.3 or later).

## Phase 4 — Complete
- Ledger: F-claude-543-a-rerun-task-lost-its-notifications-001,
  PR-claude-543-follow-a-views-terminal-through-set-terminal-001,
  L-claude-543-a-zed-tasks-arguments-are-shell-text-001, L-claude-543-tmux-on-this-box-for-a-scenario-001,
  AD-claude-543-remote-terminals-in-marleys-tmux-001.
- Brain: decision recorded on consultation ad5ba3ce6cbe4f4e98072882b57a9cbe.
