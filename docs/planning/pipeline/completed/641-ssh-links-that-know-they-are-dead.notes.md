# SSH links that know they are dead — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-641-ssh-links-that-know-they-are-dead.md
- **Pipeline spec:** 641-ssh-links-that-know-they-are-dead.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-02)
- **Request:** herdr's remote-link habits (a health ping so a dead link never reads as online,
  reconnects backing off to two minutes, a host whose link is down drawn dimmed with input off,
  showing what was last known), from `docs/planning/design-notes/herdr-and-hermes-2026-10-02.md`
  Part 1 item 6. rustal-harness withdrew its TICKET-094 under D164, whose ground is Chad's "marley
  will handle this via ssh into a remote box" and "local or remote via ssh", so the habits belong
  to Marley's SSH connections: the remote terminals (#543) and the host collector (#610). Chad
  confirmed the batch on 2026-10-02: "Queue all three".
- **Classification / tier:** feature, prong 2 (C3's stopgap remote terminals, D20's collector).
  Rust in `marley_remote` and `marley_workbench`, plus one additive hunk in Zed's `terminal`
  crate that extends its existing ledger row. Queue.
- **What the discovery changed.**
  - Neither connection sets an `-o` keepalive. `ssh_command` builds `ssh [-p N] -- host`, and
    `remote_terminal_command` adds `-t` and tmux's words. No crate in `crates/` or `vendor/` sets
    `ServerAliveInterval`, Zed's own `transport/ssh.rs` included. So on a silent link, a hang ends
    only when TCP gives up. On this box that takes 795 s idle (`tcp_keepalive_time` 120 +
    9 x 75 s) or about 924 s with data waiting (`tcp_retries2` 15). On localhost with the far side
    stopped, the kernel keeps acknowledging, so it never ends.
  - The collector's hang is worse than #610 assumed. AD-claude-610 and #610's notes say "one slow
    host holds the loop for at most its connect timeout". That holds while connecting. Once a
    ControlMaster is open, a collection over a silent link waits on the master, and
    `ConnectTimeout` does not apply. `poll_while_shown` awaits the collection before the stores'
    calls and the read (`fleet.rs:353-401`). Staleness is measured against `read_ms`, which only
    a read sets (`fleet.rs:1273-1278`). So the whole panel freezes on its last reading, every
    agent as it was. That is the "dead link reads as online" case, and two options fix it, so it
    is in this slice.
  - Keys typed into an ended task terminal are lost today with no sign. `Terminal::input` still
    finds the PTY `Active` after the child exits and hands the bytes to an event loop that has
    stopped. `try_keystroke` still returns true. "Input off" therefore makes an existing silent
    loss visible; it does not take anything away.
  - The design note cites herdr's `src/client/endpoint/activation.rs:29-91`. That file is surface
    activation (it freezes input while switching machines). The link habits are in `health.rs`,
    `supervisor.rs`, `registry.rs:122-141` and `shell_runtime.rs:788-789`. The checkout's
    `connecting-machines.mdx` has 128 lines, and the text is at 68 and 90.
  - The collector's backoff and a host kept with its last reading are the next slice. The backoff
    alone is about 40 lines, but without the last reading it only slows recovery, and the two
    together touch `fleet.rs`'s drawing and the Hosts source.
- **Recall (§18.3):**
  - AD-claude-543-remote-terminals-in-marleys-tmux-001: Rerun runs the same argv in the same
    terminal and reattaches. Kept: the reattach is that rerun, started by Marley.
  - AD-claude-610-marley-reads-hosts-with-its-own-script-in-the-ssh-command-001: its bound on a
    slow host is wrong once a master is open (above). Complete records an AD that corrects it.
  - F-claude-543-a-rerun-task-lost-its-notifications-001 and
    PR-claude-543-follow-a-views-terminal-through-set-terminal-001: a rerun swaps the view's
    `Terminal`. The down state is keyed by the ended `Terminal`'s entity and removed when that
    entity is released, so a rerun or a closed tab clears it. Nothing subscribes to the view's
    first terminal only.
  - F-claude-623-a-reruns-new-terminal-was-not-watched-001: same class. The supervisor awaits the
    new terminal that `spawn_task` returns, not the view's old one.
  - F-claude-441-a-rerun-reopened-the-hidden-terminal-panel-001: `replace_terminal` reveals the
    pane that holds the terminal. An automatic reattach must not, so it uses `reveal: Never` and
    skips `move_to_center`.
  - F-claude-474-rerun-would-have-run-a-command-that-output-printed-001: a rerun must run only
    what Marley built. The reattach reruns the task's own `SpawnInTerminal`, and nothing on the
    screen feeds it.
  - BF-claude-ssh-command-leading-dash-host-is-option-smuggling-injection-001: the check is built
    through `ssh_command`, with its `--`, from the target `open` kept.
  - L-claude-543-a-zed-tasks-arguments-are-shell-text-001: the check is an argv to
    `process::output`, not a task, so no quoting layer applies; the terminal's new `-o` words have
    no shell meaning.
  - L-claude-543-tmux-on-this-box-for-a-scenario-001: a short `TMUX_TMPDIR`, the server killed at
    teardown. L-claude-584 and #610's `start_sshd`: the scenario's own sshd on 127.0.0.1.
  - PR-claude-nonblocking-readloop-needs-backoff-and-real-io-test-001: the waits use the
    executor's timer, and the proof is a real sshd stopped and resumed, not a stand-in.
  - L-claude-448 (`pgrep -f` matches itself): the scenario finds the sshd's processes by parent
    pid from its pid file, never by `-f`.
  - rustal-harness TICKET-094 (closed, withdrawn): RL-001 to RL-003 are this ticket's habits for
    its remote view, proved with `SIGSTOP` on the runtime.
  - Brain: `rusty-cli brain search` for ssh reconnects and backoff found nothing; the herdr research
    page (`research/agent-workspaces/herdr`) only repeats the design note. Promotion runs
    `brain_ask`.
- **Discovery:**
  - `crates/marley_remote/src/marley_remote.rs`: `ssh_command` 104-116 (`ssh [-p N] -- dest`, no
    `-o`); `remote_terminal_command` 266-288 (`-t` inserted at 1, tmux's words and options after
    the destination); `remote_status_from` 130-136; the crate depends on `serde` only.
  - `crates/marley_workbench/src/remote.rs`: the id base `marley-remote` 37, `is_remote` 49-53,
    `open` 100-140 (a random `SessionName` 110, `MARLEY_SSH` 114, the label 125,
    `allow_concurrent_runs: false` and `use_new_terminal: false` 129-130, scheduled at 138).
  - `crates/marley_workbench/src/routing.rs`: `RoutedTerminals::spawn` 101-139 (Marley's task
    provider in both layouts; `move_to_center` in the Marley layout), `run_task` 203-215, which
    awaits `Terminal::wait_for_completed_task` and returns the `ExitStatus`.
  - `crates/terminal/src/terminal.rs`: `TaskStatus` 1840-1849 keeps only `success`;
    `register_task_finished` 3657-3742; `wait_for_completed_task` 3645-3655 serves one waiter
    (a bounded(1) channel, 1075-1085); `write_to_pty` 2574-2592; `input` 2595-2605, which `paste`
    (2879-2891) and `marley_paste_bracketed` reach; `try_keystroke` 2850-2867. After the child
    exits, bytes go to a loop that has stopped
    (`vendor/alacritty_terminal/src/event_loop.rs:266-278`, `:344-355`).
  - `crates/terminal_view/src/terminal_view.rs`: `MarleyTerminalOverlay` 247-253, drawn in
    `render` 1778-1791 and placed over the grid at 1878; `key_down` 1697-1711; `commit_text`
    627-633; `paste` 1247-1266; `rerun_task` 988-996; `set_terminal` 1439-1448 (no event; the view
    observes its terminal at 1485).
  - `crates/terminal_view/src/terminal_panel.rs`: `spawn_task` 645-725 (public; with
    `allow_concurrent_runs` off it waits for the old task, which has already ended, 698-722);
    `terminals_for_task` by `full_label` 793-833; `replace_terminal` 1109-1208 (`Never` does
    nothing, 1207). `crates/task/src/task_template.rs:106-114`: `RevealStrategy`.
  - `crates/marley_workbench/src/block_filter.rs`: the only `MarleyTerminalOverlay` (45), its
    `overlay` 120-124, its occluding panel 463-473.
  - `crates/marley_workbench/src/rich_input.rs:521-549`: `send` types the prompt editor's text
    through `Terminal::input`.
  - `crates/marley_workbench/src/fleet_hosts.rs`: `CONNECT_TIMEOUT_S` 26, `CONTROL_PERSIST_S` 29,
    `collect` one host after another 102-121, `ssh_args` 177-210.
  - `crates/marley_workbench/src/fleet.rs`: `COLLECT_EVERY` 54, `poll_while_shown` 353-401;
    `fleet.rs:743-775` draws an unreachable host's chip with the reason as its tooltip.
  - `crates/marley_workbench/src/process.rs:20-39`: `output`.
  - Zed: `crates/remote/src/remote_client.rs:160-166` (5 s heartbeat, five misses, three
    reconnects); `crates/client/src/client.rs:87-88`, `703-735` (500 ms doubling to 30 s, with
    jitter).
  - herdr (Apache-2.0, read only): `src/client/endpoint/health.rs:3-4` and 39-55;
    `supervisor.rs:11-14`, 222-236, 361-369; `registry.rs:122-141`; `shell_runtime.rs:788-789`;
    `docs/next/website/src/content/docs/connecting-machines.mdx:68`, `:90`.
  - The box: OpenSSH 10.5p1 (`/usr/lib/ssh/sshd-session` per connection), tmux 3.7c.
    `sshd_config(5)`: `SetEnv` "override[s] the default environment", so the scenario's sshd can
    give its sessions their own `TMUX_TMPDIR` and `HOME`.
- **Decisions:** D1 to D8 in the spec.

### Design
- **`marley_remote`** (pure):
  - `keepalive_options() -> Vec<String>` gives `-o ServerAliveInterval=5 -o
    ServerAliveCountMax=3`. The constant `CONNECT_TIMEOUT_S = 10` is for terminals and checks.
  - `remote_terminal_command` puts both after `-t`.
  - `link_check_command(&SshTarget) -> Vec<String>` gives `ssh -T -o BatchMode=yes <keepalive>
    -o ConnectTimeout=10 [-p N] -- dest true`, through `ssh_command`.
  - `LinkCheck::{Answers, Down(reason), Stopped(reason)}`, and `read_link_check(code:
    Option<i32>, stderr: &str) -> LinkCheck`. The reason is the last non-empty stderr line.
    `Down` takes 255 with a line that contains any of: `connect to host`, `timed out`,
    `Could not resolve hostname`, `Connection closed`, `Connection reset`, `not responding`,
    `banner exchange`, `kex_exchange_identification`, `Network is unreachable`,
    `No route to host`, `Broken pipe`.
  - `check_delay(failures: u32) -> Duration` is 1 s x 2^(failures - 1), capped at 120 s.
    `STABLE_LINK` is 60 s.
- **`marley_workbench::remote`:**
  - `open` keeps each remote task's label and `SshTarget` by `TaskId` in a global `RemoteLinks`.
    The same global holds the backoff by `TaskId` (`failures`, `up_since`), and the down
    terminals by the ended `Terminal`'s entity id. Each down terminal carries its phase
    (`Waiting { until }`, `Checking`, `Stopped`), the reason, and the check's `Task`.
  - `supervise(task, terminal, status, panel, cx)`: on 255, call `marley_hold_input` on the
    `Terminal`, reset `failures` if the link had been up `STABLE_LINK`, and start the loop. The
    loop waits `check_delay`, waking each second to notify the terminal so the view redraws its
    countdown, then runs the check through `process::output` (with `MARLEY_SSH` honored, as `open`
    does).
    - `Answers`: `panel.spawn_task(&SpawnInTerminal { reveal: RevealStrategy::Never, ..task })`.
      Record `up_since`, await the new terminal's completion, and supervise again.
    - `Down`: count the failure and loop.
    - `Stopped`: set the phase and end the loop.
    - Any other status clears the task's backoff.
  - `cx.observe_release` on the ended `Terminal` drops its entry and the check. A user's Rerun
    swaps in a new terminal and releases it; a closed tab releases it too.
  - `link_overlay(context, window, cx)`: for a down terminal,
    `div().absolute().inset_0().occlude()` over the grid, the editor background at 60 % opacity,
    and one line at the top, built from the phase and the refused count:
    - "Link to e2e lost: Timeout, server 127.0.0.1 not responding · next check in 8 s · Rerun to
      try now · 22 not sent";
    - "Checking e2e…";
    - "Not reconnecting to e2e: Permission denied (publickey). Rerun to log in."
- **`routing.rs`:** `run_task` returns the terminal with the status. `RoutedTerminals::spawn`
  hands a task whose id starts with `marley-remote` to `remote::supervise` when it ends, and
  still returns the status to its caller.
- **`block_filter.rs`:** `overlay` asks `remote::link_overlay` first. A down terminal shows that
  and no filter. The global stays single.
- **Zed, `crates/terminal/src/terminal.rs`:** two fields, `marley_input_held` and
  `marley_inputs_refused`; `pub fn marley_hold_input(&mut self)`; and
  `pub fn marley_inputs_refused(&self) -> Option<usize>`, which is `Some` while held. `input`
  returns early while held, counting the call, before it sets `keyboard_input_sent`, notes the
  input for blocks, or scrolls. The hunk carries `// Marley: a remote terminal whose link is down
  sends nothing and counts what it refused (#641)`. It extends the existing row
  (`docs/marley/zed-touchpoints.md:80`).
- **Conditional, Zed, `crates/terminal_view/src/terminal_view.rs`:** Code checks whether a refused
  key redraws the view: `key_down` pauses the cursor blink. If it does not, `key_down` notifies
  while the terminal holds input, and the existing row (`zed-touchpoints.md:85`) grows by that
  line. The per-second tick bounds the lag at one second either way.
- **`fleet_hosts.rs`:** `ssh_args` adds `marley_remote::keepalive_options()` before the control
  options.
- **File manifest:**
  - Marley: `crates/marley_remote/src/marley_remote.rs`;
    `crates/marley_workbench/src/remote.rs`, `routing.rs`, `block_filter.rs` and
    `fleet_hosts.rs`; `script/e2e/641-ssh-links-that-know-they-are-dead.sh`.
  - Zed: `crates/terminal/src/terminal.rs` (row 80), and `crates/terminal_view/src/terminal_view.rs`
    (row 85) only if the redraw needs it. No new row and no new dependency. The check goes through
    `process::output`, so `.config/spawn-sites.txt` does not change.
- **At Complete:** CHANGELOG; `docs/marley_architecture/marley_remote.md` (the keepalive, the
  check, `LinkCheck`) and `marley_workbench.md` (the Remote terminals section); the fleet
  contract's collector line; an AD for the link checks that corrects AD-610's bound.

### Visual check plan
- **Setup:** `compositor sway`. #610's `start_sshd`, with `SetEnv TMUX_TMPDIR=<mktemp -d under
  /tmp> HOME=$E2E_WORK/remote-home` added to its config. `MARLEY_SSH` points at a wrapper that
  runs the real ssh with the scenario's key and known-hosts file. It does not `exec`: it appends
  the start time, the argv, the end time and the status to `ssh.log`, then exits with ssh's
  status. The settings save `ssh_connections`
  `[{"host": "127.0.0.1", "port": <port>, "nickname": "e2e"}]` and `marley.fleet.hosts`
  `[{"ssh": "127.0.0.1:<port>", "name": "lab"}]`. A plain `.bashrc` sits in the remote HOME.
- **Stop and resume:** "stop" sends `SIGSTOP` to the sshd's pid and every descendant, found by
  `pgrep -P` from its pid file. "Resume" sends `SIGCONT` to the same list. The tmux server has
  daemonized out of that tree, so the session keeps running.
- **Teardown:** `SIGCONT` and kill the sshd, then kill the run's tmux server and remove its folder.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-011 | Palette `marley: open remote terminal`, Enter on `e2e`; start a counter (`n=0; while :; do n=$((n+1)); echo "tick $n"; sleep 1; done &`); open the Fleet panel | `641-01-connected`; `ssh.log`: the terminal's argv carries the three options before `--`, and the collector's carries the two |
| REQ-002, REQ-012 | Stop the sshd; settle 25 s | `641-02-silent`: the terminal dimmed over its last ticks, its line naming `e2e` and the next check; `lab` unreachable |
| REQ-012 | Pointer on `lab`'s chip (coordinates read at Test, as #610 did) | `641-03-host-reason`: ssh's reason in the tooltip |
| REQ-003 | Pointer away; type `echo typed-while-down`, Enter | `641-04-input-off`: the line counts 22 not sent |
| REQ-004, REQ-006 | `workspace: new terminal` (a local one in front); after about 40 s of checks, resume the sshd; settle to the next check | `641-05-reattached-behind`: the local terminal still in front; `ssh.log`: the checks' gaps (end to next start) 1, 2, 4 … s |
| REQ-007 | `pane: activate previous item` | `641-06-reattached`: the same session undimmed, ticks well past the stop, no `typed-while-down`; `tmux capture-pane -p` in the log holds none either |
| REQ-008 | Empty `authorized_keys`; kill the session's `sshd-session` (a drop that closes); settle past the first check | `641-07-refused`: dimmed, "Not reconnecting … Permission denied (publickey)"; `ssh.log`: one check, none after it in the next 20 s |
| REQ-009 | Restore the key; palette `terminal: rerun task` | `641-08-rerun-now`: reattached; `ssh.log`: a terminal argv within 2 s of the palette |
| REQ-010 | `kill %1; exit` in the remote shell | `641-09-ended`: the tab ended, not dimmed, no line; `ssh.log`: no check after it |
| REQ-005 | Review of the diff: `up_since`, `STABLE_LINK` and the reset | |

What no scenario reaches: a real network that drops packets, which the hand check at the end
covers on a host Chad names (suspend the laptop or cut the Wi-Fi, then watch the dim, the checks
and the reattach). A two-minute wait at the cap is reviewed, not watched.

### Risks
- **Reading ssh's words.** The check's outcome rests on ssh's stderr lines, which have been stable
  for years but are not an interface. An unknown line stops the checks, so a change in ssh
  degrades to today's manual Rerun, never to a loop.
- **Lockouts.** Retrying a refused login would trip fail2ban or sshd's `MaxAuthTries` on hosts
  that count failures, so a refused login stops the checks (D7).
- **The window before ssh notices.** Keys typed in the up to 20 s before the keepalive ends ssh go
  to ssh and are lost with it. Marley cannot tell a silent link from a quiet one sooner without a
  ping of its own over the link, which D1 rejects.
- **The prompt editor.** `rich_input::send` types its text through `Terminal::input` and may
  clear the editor after. Code checks it keeps the text when the terminal holds input; if not,
  `send` skips a held terminal.
- **The user's ssh config.** Marley's `-o` options win over a host's own `ServerAliveInterval` in
  `~/.ssh/config`, for Marley's connections only.
- **A first connect that fails.** A host that is down when the terminal opens now dims and is
  checked, instead of ending with ssh's error as in #543. A refused login on first connect stops at
  once.
- **Exit 255 from a signal.** ssh also exits 255 when the remote command dies by a signal. Marley
  then checks and reattaches, which finds the tmux session if the host still has it.
- **The deferred spawn.** With `allow_concurrent_runs` off, `spawn_task` waits for the old task,
  which has already ended. Promotion confirms it replaces at once.
- **Several terminals on one host.** Each checks on its own; on a host with many terminals that is
  one login attempt per terminal per wait.
- **An old master.** A ControlMaster an older Marley started without the keepalive lives until
  `ControlPersist`'s 60 s of idleness, and the new options apply from the next master.

### Promotion (2026-10-04)
- Promoted into `active/`; the BACKLOG row removed; the ticket in-progress. Pre-flight green, no
  other active pipeline, cargo idle, `/mnt/fast` 251G free.
- **Seams re-verified** at f63b6f05cc: `marley_remote.rs` `ssh_command` `:104`,
  `remote_status_from` `:130`, `remote_terminal_command` `:266`; `remote.rs` `ID_BASE` `:37`,
  `is_remote` `:49`, `MARLEY_SSH` `:114`, `use_new_terminal: false` and `allow_concurrent_runs:
  false` `:129-130`; `routing.rs` `RoutedTerminals::spawn` (`move_to_center` in the Marley layout)
  and `run_task`, which awaits `Terminal::wait_for_completed_task`; `fleet_hosts.rs`
  `CONNECT_TIMEOUT_S` `:26`, `CONTROL_PERSIST_S` `:29`, `ssh_args` `:177`; `terminal.rs`
  `write_to_pty` `:2575`, `input` `:2595`, `try_keystroke` `:2850`, `paste` `:2879`,
  `wait_for_completed_task` `:3645`; `terminal_view.rs` `MarleyTerminalOverlay` `:250`, read at
  `:1780`; `block_filter.rs:45` sets the overlay.
- **The two checks the plan left to promotion:**
  - `spawn_task` with `allow_concurrent_runs` off pops the newest terminal under the task's
    label and defers the replace until `wait_for_terminals_tasks` over the *rest* of them, none
    for a remote task's unique label, so the replace runs on the next turn, at once
    (`terminal_panel.rs:645-725`). `terminals_for_task` searches the panel's panes and the
    workspace's center panes (`:793-833`), so a remote terminal moved to the center in the
    Marley layout is found, and `replace_terminal`'s `RevealStrategy::Never` arm does nothing
    (`:1207`), so no focus moves.
  - Whether a refused key redraws the view stays a Code-phase check, as the design says (the
    per-second tick bounds any lag to a second).
- **Brain:** `brain ask` (consultation `4cbcfe958f144ad199b44004d34bf2b7`) returned due follow-ups
  on other work only. Nothing new.
- **The scenario's sshd:** #610's `start_sshd` (`script/e2e/610-host-collector-over-ssh.sh:76`)
  is the base; 641's adds `SetEnv` to its config.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20 and the templates.
- [x] Read the brief, the design note with Chad's answers, rustal-harness TICKET-094 and D164.
- [x] Read herdr's link code and docs (Apache-2.0, read only, nothing copied).
- [x] Discovery: #543's and #610's specs and notes, `marley_remote`, `remote.rs`, `fleet_hosts.rs`,
  `fleet.rs`'s poll loop, `routing.rs`; an Explore pass over Zed's `terminal`, `terminal_view`
  and `remote` crates and the touchpoint rows.
- [x] Recall: the knowledge ledgers and the completed pipelines (codes above); the brain searched.
- [x] Prior art: the behavior maps (Orca, Zed, Warp), the ssh manuals, Zed's crates and
  `Cargo.lock`.
- [x] Scope cut: the collector's keepalive in, its backoff and last reading as the next slice.
- [x] Reference (§20), UI proof, decisions, EARS criteria and the phase plan in the spec.
- [x] Ticket doc in `tickets/open/`, the spec and these notes in `pipeline/queued/`. BACKLOG.md
  and `active/` untouched. No build run.

## Phase 2 — Code (2026-10-04)
- **Built:**
  - Ledger first: the `crates/terminal/src/terminal.rs` row (`zed-touchpoints.md:80`) gained the
    held input and a merge note (keep `input`'s held check first).
  - `marley_remote` (pure): `SERVER_ALIVE_INTERVAL_S` 5, `SERVER_ALIVE_COUNT_MAX` 3,
    `CONNECT_TIMEOUT_S` 10, `MAX_CHECK_DELAY` 120 s, `STABLE_LINK` 60 s; `keepalive_options()`;
    `remote_terminal_command` now `ssh -t <keepalive> -o ConnectTimeout=10 [-p N] -- dest tmux …`;
    `link_check_command` (`ssh -T -o BatchMode=yes <keepalive> -o ConnectTimeout=10 … -- dest
    true`); `LinkCheck { Answers, Down, Stopped }` and `read_link_check` (0 answers; 255 with a
    link failure in ssh's last line is down; anything else stops, the reason ssh's last line);
    `check_delay(failures)` (1 s doubling, capped). Both argv builders append after `ssh` rather
    than splicing (`unused_results` on `splice`), and `ssh_command` and its tests are unchanged.
  - Zed's `terminal.rs`: `marley_input_held`, `marley_inputs_refused`, `marley_hold_input()`,
    `marley_inputs_refused() -> Option<usize>`, and `input`'s early return that counts while held.
  - `remote.rs`: a `Links` global (each task's `Host` from `open`, its `Backoff { failures,
    up_since }`, and the down terminals by entity, each `Down { task, host, phase, reason,
    _checks, _redraws }`); `is_remote_task`; `starting` (the provider's run: `up_since` now, the
    task's down entries dropped); `supervise` (255: reset the failures after a stable minute,
    hold the input, spawn the checks and the per-second redraws, drop the entry on the terminal's
    release; anything else: the backoff goes); `check_until_up` (wait, `Checking`, the check
    through `process::output` with `MARLEY_SSH` honoured, then answers → `reattach`, down →
    `failures + 1` and wait again with ssh's reason, stopped → the phase says why);
    `reattach` (`up_since` now, `panel.spawn_task` with the provider's own unprepared task and
    `reveal: Never`, and a detached follow that drops the old entry and supervises the new
    terminal); `link_overlay` (an occluding, 60 %-dimmed cover with one line).
  - `routing.rs`: `starting` before a remote run; `run_task` hands a remote task's end to
    `supervise`. `block_filter.rs`: the overlay hook asks `link_overlay` first.
    `fleet_hosts.rs`: `ssh_args` takes the keepalive. `rich_input.rs`: `send` leaves a held
    terminal's prompt in the editor.
- **Deviations from the design:**
  - The reattach respawns the task the provider received, not the ended terminal's
    `spawned_task`: that one is already wrapped for the shell (`prepare_task_for_spawn`), and
    `spawn_task` would wrap it again. `RoutedTerminals::spawn`'s `task` is the unprepared one.
  - The follow of a reattached terminal is a detached task of its own: the rerun replaces the old
    terminal, whose release drops its `Down` entry and with it the checks that started the rerun,
    so the checks could not follow it themselves.
  - A ticker per down terminal (`_redraws`) draws the line each second whatever the checks do. The
    first full run's `641-04` read "1 not sent" for 22 keys: the line redraws only with the view,
    a key redraws it only when it pauses the cursor's blink, and during a check (its whole 10 s
    connect timeout against a stopped sshd) nothing else ticked. With the ticker the count reads
    22. The checks now wait on one timer.
  - The checking phase's line keeps the host and the reason (`Link to e2e lost: … · checking now
    · Rerun to try now`), not `Checking e2e…`: a check takes its whole timeout against a silent
    host, so `641-02` landed in it.
  - The stopped line trims ssh's own closing period (`publickey).. Rerun` in the first run).
  - No `terminal_view.rs` change: `key_down` pauses the cursor's blink on every key, which redraws.
  - `supervise` takes `&Entity<Terminal>` and `&Window` (`needless_pass_by_value`,
    `needless_pass_by_ref_mut`), and the `Down` entry is built after its tasks are spawned, so
    nothing writes into the underscore fields later.
- **Review of the diff** against REQ-001 to REQ-012: every ssh Marley starts takes the keepalive
  (the terminal, the check, the collector; REQ-001, REQ-011). Only 255 turns a terminal down
  (REQ-010); the reattach runs only on `Answers` (REQ-006, REQ-008), with `reveal: Never` and no
  `move_to_center`; a user's Rerun goes through the provider, whose `starting` drops the pending
  checks before the run (REQ-009); `failures` is reset only by a drop after 60 s up (REQ-005);
  `check_delay` caps at 120 s (REQ-004). Input: every key, paste and Marley sender goes through
  `Terminal::input`, held and counted (REQ-003); rich input keeps its text. Re-entrancy: the
  overlay reads the terminal entity, not the view being drawn; the global is touched in short
  updates; `observe_release` removes by id. Provenance: herdr's numbers and its docs only, as
  the spec cites; no Zed function body in a Marley crate (the provider's task is Zed's type,
  respawned through Zed's public `spawn_task`). A fault the scenario found: the redraw (above),
  an `F-…` block at Complete.
- **Checks:** `cargo clippy -p marley_remote` and `-p marley_workbench --all-targets` green after
  four rounds (doc paragraphs, `splice`'s result, similar names, an unneeded `gpui::` path, a
  missing `ResultExt`, then the five above); `cargo doc` of both crates with `-D warnings` green
  (L-640); `just build` green after one fix: an inserted method took the `#[cfg(test-support)]`
  of the method it went in front of, so the build (no test-support) lost `marley_hold_input` and
  gained a public `keyboard_input_sent`; the attribute is back on its own method.
- **The scenario, run in this phase** (written before the gate, as #642's and #640's): run 1
  passed its checks but showed the remote shell with the system's default prompt in the user's
  own home folder (tmux starts a login shell, which reads `.bash_profile`, and sshd starts it in
  the passwd home), and the two faults above; nothing reached the user's files (no tmux server
  left, no history line). Run 2 failed its collector check: `grep | head -1 | grep -q` under the
  runner's `pipefail` fails once the collector has run twice (L-582's class); `grep -m1` now.
  Run 3: every check green, every shot as the criteria say (read in Phase 3).
- **Gate:** `just gate-diff`: `GATE GREEN [diff]`, 17 passed, the receipt written, on the first
  run (the docs gate checked beforehand with `cargo doc -D warnings`, L-640).

## Phase 3 — Test (2026-10-04)
- **Build and run:** `just build` on the gated tree, then `just e2e
  script/e2e/641-ssh-links-that-know-they-are-dead.sh` (`compositor sway`), exit 0, every check
  passed: the terminal's argv has `-t -o ServerAliveInterval=5 -o ServerAliveCountMax=3 -o
  ConnectTimeout=10 -p N --` (REQ-001); the collector's has the keepalive (REQ-011); the checks
  reattached once the host answered; the host's session never got the refused line (REQ-007); no
  check after the refusal (REQ-008); none after the user's `exit` (REQ-010). The checks' log:
  each against the stopped sshd ran its full 10 s, then the next started 2.0 s, 4.0 s and 8.0 s
  after the end of the one before; the fourth, after the resume, answered (0) in 0.1 s
  (REQ-004); after the refusal one check, 255 in 0.1 s, and none in the next 20 s. Focus report:
  "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and did not reload
  it". Every shot is Marley's window; none deleted. The run before it, in the Code phase, showed
  the same, its `641-04` in the waiting phase.
- **The shots, read:**
  - `641-01-connected` (REQ-001, REQ-011): the remote tab `e2e · marley-…` with `remote$` and the
    counter at tick 16; the Fleet panel's `lab` with its resource line and its processes.
  - `641-02-silent` (REQ-002, REQ-012): 25 s after the stop: the grid dimmed over ticks 1 to 17,
    the line "Link to e2e lost · checking now · Rerun to try now" (the first check under way), the
    tab's icon and the rail row `exit 255`; `lab` marked `unreachable`.
  - `641-03-host-reason` (REQ-012): the pointer on `lab`'s chip: "Connection timed out during banner
    exchange".
  - `641-04-input-off` (REQ-003): after typing `echo typed-while-down` and Enter: the screen
    unchanged and the line ending "22 not sent" (the Code-phase run's showed the waiting phase:
    "Link to e2e lost: Connection timed out during banner exchange · next check in 2 s · Rerun to
    try now · 22 not sent").
  - `641-05-reattached-behind` (REQ-006): after the resume and the reattach: the local `repo —
    bash` still the active tab; the rail's `e2e` row `running`; `lab` unreachable until its next
    poll.
  - `641-06-reattached` (REQ-007): `pane: activate previous item`: the remote tab undimmed, the
    same `remote$` session, ticks 48 to 92, no `typed-while-down`.
  - `641-07-refused` (REQ-008): the key removed and the session's connection killed: dimmed, "Not
    reconnecting to e2e: cpeppers@127.0.0.1: Permission denied (publickey). Rerun to log in.",
    ssh's own "Connection to 127.0.0.1 closed by remote host." at the bottom.
  - `641-08-rerun-now` (REQ-009): the key restored, `terminal: rerun task`: reattached at once,
    ticks 145 to 189, undimmed.
  - `641-09-ended` (REQ-010): `kill %1; exit`: the tab ended with ✓ and `[exited]`, the rail row
    `done`, no dim and no line.
  - REQ-005 (the reset after 60 s up) and the 120 s cap: the review (`supervise`, `check_delay`);
    the refusal step waits 62 s after the reattach, and its first check came 1 s after the drop
    (one 0.1 s check right after the kill), which shows the reset.
- **Not reached:** a real network that drops packets (SIGSTOP stands in), a wait at the 120 s cap,
  and a host-key change; the hand check on a real host stays for Chad (sleep the laptop or cut the
  Wi-Fi with a remote terminal open).
- **Pre-existing — not in scope:** none.

## Phase 4 — Complete (2026-10-04)
- **Documented (§21):** `CHANGELOG.md` Added ("SSH links that know they are dead");
  `docs/marley_architecture/marley_remote.md` (the keepalive, the check, `read_link_check`, the
  backoff) and `marley_workbench.md` (the Remote terminals section: `Links`, `supervise`, the
  checks, the reattach, the overlay); `docs/marley/three-prong-plan.md` (the C3 row's stopgap and
  the #610 line); `docs/marley/fleet-contract.md` (the collector's keepalive);
  `docs/marley/guide.md` (a Remote terminals section after Blocks over ssh);
  `docs/marley/walkthrough.md` (stop 4.16). The `terminal.rs` row in `zed-touchpoints.md`
  describes what shipped (the held input and its accessors); `terminal_view.rs` was not touched.
- **Knowledge (§19):** `F-claude-641-a-down-terminals-count-did-not-redraw-during-a-check-001`
  with `PR-claude-641-what-a-view-draws-from-another-entity-redraws-on-its-own-001`;
  `F-claude-641-an-inserted-method-took-the-next-methods-cfg-001` with
  `PR-claude-641-insert-an-item-after-an-item-never-between-it-and-its-attributes-001`;
  `AD-claude-641-ssh-links-know-they-are-dead-by-keepalive-and-a-checked-reattach-001` (it
  corrects AD-610's bound); `L-claude-641-a-tasks-spawned-task-is-already-wrapped-for-the-shell-001`,
  `L-claude-641-tmux-starts-a-login-shell-which-reads-bash-profile-001`,
  `L-claude-641-a-reader-that-quits-early-fails-a-pipe-under-pipefail-001`.
- **Brain:** `brain decide` on consultation `4cbcfe958f144ad199b44004d34bf2b7` (follow-up
  2026-10-25).
- **Closed:** the ticket to `tickets/closed/`, its link at `completed/`; no BACKLOG row left; the
  pair archived.
