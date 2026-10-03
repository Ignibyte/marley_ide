---
pipeline_id: 53e5761f-5dd5-4949-ab42-78d6eb81fce5
ticket: docs/planning/tickets/open/TICKET-641-ssh-links-that-know-they-are-dead.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "SSH links that know they are dead"
type: feature
slice: prong 2, C3's remote terminals after #543 and D20's host collector after #610; herdr's link habits, given to Marley by rustal-harness's D164
references: [docs/planning/design-notes/herdr-and-hermes-2026-10-02.md, docs/planning/pipeline/completed/543-remote-terminals-survive-a-drop.spec.md, docs/planning/pipeline/completed/610-host-collector-over-ssh.spec.md, docs/marley/fleet-contract.md, docs/orca_architecture/04-remote-control-and-mobile.md]
---

## Title
Marley's SSH connections stop reading as live once their link is dead. Every ssh Marley starts
asks ssh's own keepalive to check the link, so a silent link ends within 20 s instead of holding
until TCP gives up (13 to 15 minutes on this box). A remote terminal whose ssh ends with ssh's
error status keeps its last screen, drawn dimmed with input off. Marley checks the host on a
backoff that doubles up to two minutes, and once the host answers it reattaches the same tmux
session with Zed's Rerun. The host collector takes the keepalive, so a silent host reads
unreachable and no longer holds the Fleet poll.

## Scope
### In
- **The keepalive** (`marley_remote`, pure): `-o ServerAliveInterval=5 -o ServerAliveCountMax=3`
  for every ssh Marley starts, from one function. A remote terminal's argv also gets
  `-o ConnectTimeout=10`, so an attempt against a host that accepts the connection and then says
  nothing ends too. The options go before the `--`.
- **A remote terminal that loses its link** (`marley_workbench::remote`, through Marley's task
  provider in `routing.rs`): when the task's ssh ends with status 255, ssh's own error status, the
  terminal turns *down*.
  - The view keeps the ended `Terminal`, so the last screen stays where it was.
  - An overlay dims the grid, catches the mouse, and carries one line. The line names the host,
    says why the link is down and when Marley checks next ("next check in 8 s"), counts the input
    not sent, and says Rerun tries at once. It counts down by the second.
  - Any other exit status is the remote command's own (the user's `exit`, tmux's end), and the
    terminal ends as #543 left it: not dimmed, not checked.
- **Input off.** While a terminal is down, `Terminal::input` sends nothing and counts each call
  it refuses: a key, a paste, an IME commit, or text a Marley feature would type, such as a block
  sent to an agent. Nothing is buffered and nothing is replayed after the reattach. Copy, the
  palette, Rerun and every other action still work. Before this ticket, keys typed into an ended
  task terminal reached a stopped event loop and vanished with no sign.
- **The check:** `ssh -T -o BatchMode=yes <keepalive> -o ConnectTimeout=10 [-p N] -- <dest>
  true`, through `process::output`, so there is no new spawn site and no password prompt. Its
  outcome is read by a pure function in `marley_remote`:
  - exit 0: the host answers;
  - exit 255 with one of ssh's link-failure messages (connect timed out, refused, no route,
    unreachable network, unresolved name, closed or reset connection, banner exchange, server not
    responding): the host is still down;
  - anything else (a refused login, a failed host key, a message Marley does not know): stop
    checking, put ssh's line on the overlay, and wait for the user's Rerun.
- **The backoff:** the first check runs 1 s after the drop. After each failed check the wait
  doubles (1, 2, 4 … s), measured from the end of the check, and never exceeds 120 s. The count
  carries across a reattach that drops again within a minute, and starts over only after a link
  has stayed up for 60 s.
- **The reattach:** when a check finds the host answering, Marley reruns the task in the same
  terminal through `TerminalPanel::spawn_task` with `reveal: Never`, skipping `move_to_center`,
  so no focus moves and no tab changes. The new `Terminal` replaces the ended one, the overlay goes
  with it, and tmux redraws the session. The user's own Rerun on a down terminal runs at once
  through Zed's path and drops the pending check.
- **The collector** (`fleet_hosts::ssh_args`): the same keepalive. The ControlMaster connection
  it opens carries it too, so a silent host ends the collection in flight within 20 s and reads
  unreachable with ssh's reason. Until now an open master on a silent link held the whole poll
  loop, every store's reads included, until TCP gave up.
- `script/e2e/641-ssh-links-that-know-they-are-dead.sh`.

### Out (explicitly deferred)
- **The next slice, Fleet hosts that keep their last reading:** a host whose collection fails
  keeps its last snapshot, drawn dimmed with its age, and is retried on the same backoff (from
  5 s up to two minutes) instead of on every 5 s poll, so a dead host stops costing each poll its
  5 s connect timeout. It touches `fleet.rs`'s drawing and the Hosts source, which this ticket
  leaves alone.
- A bound on a collection whose link is alive but whose script hangs (a hung mount under `df`):
  the keepalive cannot see it, and it is not a link fault.
- A banner or a rail mark when a remote terminal that is not in front loses its link.
- Zed's own remote-project terminals (`create_remote_shell`) and Zed's `remote_server` link,
  which keeps its own heartbeat (`crates/remote/src/remote_client.rs:160-166`).
- Reattaching after a Marley restart, still #543's next slice.
- Sharing one check between several terminals on the same host.
- Retrying a host that refuses the login: fail2ban and sshd's own limits count every failed
  login, so a retry every two minutes can lock the user out.

## Reference (§20)
herdr (Apache-2.0, may be read like Orca; nothing is copied, since Marley's version is ssh's own
keepalive plus a separate check). Its docs: "When a connection is lost, the last workspace and
agent state remains visible but dimmed … Input and navigation into those cached panes stay
disabled until a fresh connection and matching screen arrive" (`connecting-machines.mdx:68`), and
"Repeated failures increase the delay up to two minutes; brief successful connections do not
reset it. A connection must remain healthy for a minute before the next interruption gets a fast
retry" (`:90`). Its code: a 5 s heartbeat with a 10 s timeout (`src/client/endpoint/health.rs:3-4`);
500 ms doubling to 120 s, 60 s to count as stable (`src/client/endpoint/supervisor.rs:11-14`,
`:222-236`, `:361-369`). Marley differs in one place: herdr drops input while offline
(`src/client/shell_runtime.rs:788-789`, `if !active_endpoint_online { continue; }`), and Marley
counts it on the screen.

Upstream Zed: the task terminal, its tab states and its Rerun in the same terminal are kept as
#543 used them; `RevealStrategy::Never` (`crates/task/src/task_template.rs:106-114`) is Zed's own
"do not alter focus". Warp: N/A; `docs/warp_architecture/` names a reconnecting transport trait
(`crates/remote_server.md:29`) and no behavior for a dead link.

### Prior art
- **Behavior maps:** Orca's SSH relay keeps a 5 s keepalive with a 20 s timeout and reconnects
  at 1, 2, 5, 5, 10, 10, 10, 30 and 30 s; its phone client waits 0.5 s doubling to 60 s, then
  every 90 s (`docs/orca_architecture/04-remote-control-and-mobile.md:252`, `:395`, `:404`). Zed's
  remote link: 5 s heartbeat, five misses, three reconnects
  (`docs/zed_architecture/subsystems/06-project-fs-search.md:438-439`).
- **Published material:** `ssh_config(5)`: `ServerAliveInterval` and `ServerAliveCountMax` are
  sent through the encrypted channel and "valuable when the client or server depend on knowing
  when a connection has become unresponsive"; `ConnectTimeout` covers the connection and "the
  initial SSH protocol handshake and key exchange"; the first value obtained wins, so Marley's
  `-o` options win over the user's config. `ssh(1)`: "ssh exits with the exit status of the
  remote command or with 255 if an error occurred". `sshd_config(5)`'s `SetEnv`, for the scenario.
  rustal-harness's withdrawn TICKET-094: its RL-001 makes a link silent with `SIGSTOP` on the far
  side, the method this scenario uses.
- **The code we already ship:**
  - No crate in `crates/` or `vendor/` sets `ServerAliveInterval` or `ServerAliveCountMax`. Zed's
    `crates/remote/src/transport/ssh.rs` sets `ConnectTimeout` (1795) and runs a ControlMaster
    (188-208), with no keepalive.
  - Zed's collab client backs off 500 ms doubling to 30 s with jitter
    (`crates/client/src/client.rs:87-88`, `703-735`); the pattern holds, the numbers are herdr's.
  - Marley's task provider already awaits each task's `ExitStatus`
    (`crates/marley_workbench/src/routing.rs:203-215`), the only waiter
    `Terminal::wait_for_completed_task` serves (`crates/terminal/src/terminal.rs:3645-3655`).
  - `TerminalPanel::spawn_task` is public (`crates/terminal_view/src/terminal_panel.rs:645`), and
    `replace_terminal` moves focus only for `Always` and `NoFocus`; `Never` does nothing
    (1109-1208, the arm at 1207).
  - `MarleyTerminalOverlay` (`crates/terminal_view/src/terminal_view.rs:247-253`, drawn at
    1778-1791 and 1878) already draws over the grid; `block_filter.rs:45` sets it, and its panel
    occludes (463-473).
  - `fleet_providers.rs:488` bounds a call with the executor's timer; `process::output`
    (`process.rs:20-39`) is the workbench's spawn for the check.
  - No dependency in `Cargo.lock` owns this: no `russh`, `ssh2`, `backoff` or `backon`; Marley
    drives the system `ssh`, as #543 and #610 do.

## UI proof
`script/e2e/641-ssh-links-that-know-they-are-dead.sh` (`compositor sway`). An sshd of the
scenario's own on 127.0.0.1 (#610's `start_sshd`) stands in for a host, with `SetEnv` giving its
sessions a short `TMUX_TMPDIR` and a HOME of the run's own, so its tmux server is the run's
(L-claude-543). `MARLEY_SSH` names a wrapper that logs each call's start time, argv, end time and
status, then runs the real ssh with the scenario's key. The settings save the sshd as `e2e` in
`ssh_connections` and list it as `lab` in `marley.fleet.hosts`; the Fleet panel stays open
throughout. "Silent" is `SIGSTOP` on the sshd and every process under it; "back" is `SIGCONT`.
Shots:
- `641-01-connected`: the remote terminal on `e2e`, a counter ticking; `lab` with its resource line.
- `641-02-silent`: 25 s after the stop, the terminal dimmed over its last ticks with its line,
  and `lab` marked unreachable.
- `641-03-host-reason`: the pointer on `lab`'s chip, ssh's reason in the tooltip.
- `641-04-input-off`: after typing `echo typed-while-down` and Enter, the line reads 22 not sent.
- `641-05-reattached-behind`: a local terminal opened in front, then `SIGCONT`; the local
  terminal still in front after the next check.
- `641-06-reattached`: the remote tab brought back: undimmed, the same session, the counter well
  past the stop, no `typed-while-down`.
- `641-07-refused`: the key taken out of `authorized_keys` and the session's sshd killed: dimmed,
  the line says Marley stopped, with `Permission denied (publickey)`.
- `641-08-rerun-now`: the key put back and `terminal: rerun task` run: reattached at once.
- `641-09-ended`: `exit` in the remote shell: the tab ended, not dimmed, no line.

## Locked-In Decisions
- D1 — ssh's own keepalive is the health ping: `ServerAliveInterval=5`, `ServerAliveCountMax=3`,
  close to herdr's 5 s and 10 s, Orca's 5 s and 20 s and Zed's 5 s and five misses. A silent link
  ends ssh within 20 s. Marley sends no ping of its own over the link.
- D2 — Only 255 means the link. Every other status is the remote command's, and a terminal the
  user ended stays ended.
- D3 — The check is its own `ssh … true` in `BatchMode`, not a rerun. The dimmed terminal keeps
  the last screen until the host answers, and no password prompt opens behind a scrim.
- D4 — The backoff starts at 1 s, doubles, stops growing at 120 s, and starts over only after a
  link has stayed up 60 s (herdr's numbers, except a 1 s start, since each check is a full login).
  No jitter: the checks are one user's terminals, not a crowd of clients on one server.
- D5 — Input off, nothing replayed. Text typed against the last screen may not fit the screen
  tmux redraws, and a `y` meant for an old prompt could answer a new one. The line says how many
  inputs were not sent, so nothing is lost silently. The gate sits in `Terminal::input`, which
  keys, pastes, IME text and Marley's own senders all reach.
- D6 — The reattach is still Zed's rerun in the same terminal (#543's D3), started by Marley
  with `reveal: Never` and without `move_to_center`.
- D7 — Only messages ssh prints for a link failure lead to another check. A refused login, a
  failed host key and anything unknown stop the checks and show ssh's words; the user's Rerun
  logs in by hand.
- D8 — The collector takes the keepalive only. Its backoff, and a host kept with its last reading,
  are the next slice.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The system shall start every remote terminal's ssh with `-o ServerAliveInterval=5 -o ServerAliveCountMax=3 -o ConnectTimeout=10` before the `--`. | The wrapper's argv log; review |
| REQ-002 | WHEN a remote terminal's link goes silent, the system shall within 25 s draw the terminal dimmed over its last screen, with a line naming the host and when it is checked next. | Shot `641-02-silent` |
| REQ-003 | WHILE a remote terminal is dimmed, the system shall send nothing typed, pasted or composed to it, and its line shall count each input not sent. | Shot `641-04-input-off` |
| REQ-004 | WHILE a dimmed terminal's host does not answer, the system shall check it 1 s after the drop and then wait twice as long after each failed check, never more than 120 s. | The wrapper's log of the checks; review of the cap |
| REQ-005 | WHERE a reattached link ends with 255 within 60 s, the system shall continue the backoff where it stood, and after 60 s up shall start again from 1 s. | Review of the diff |
| REQ-006 | WHEN a check finds the host answering, the system shall rerun the terminal's task in the same terminal without moving focus or changing the active tab. | Shot `641-05-reattached-behind`; the log |
| REQ-007 | WHEN a dimmed terminal is reattached, it shall show the same tmux session undimmed, with none of the input refused while it was dimmed. | Shot `641-06-reattached`; `capture-pane` in the log |
| REQ-008 | IF a check fails for a reason other than a link failure, THEN the system shall stop checking and show ssh's reason on the line. | Shot `641-07-refused`; no later check in the log |
| REQ-009 | WHEN the user reruns a dimmed terminal, the system shall rerun it at once and drop the pending check. | Shot `641-08-rerun-now`; the log |
| REQ-010 | WHEN a remote terminal's ssh ends with a status other than 255, the system shall neither dim the terminal nor check its host. | Shot `641-09-ended`; the log |
| REQ-011 | The system shall run the host collector's ssh with `-o ServerAliveInterval=5 -o ServerAliveCountMax=3`. | The wrapper's argv log; review |
| REQ-012 | WHEN a listed host's link goes silent, the Fleet panel shall within 25 s mark it unreachable, with ssh's reason in its tooltip. | Shots `641-02-silent`, `641-03-host-reason` |

## Phase Plan
- **P1 Plan** — this spec and the design in the notes; at promotion, `brain_ask`, and confirm
  that `spawn_task` replaces an ended task's terminal at once with `allow_concurrent_runs` off,
  and that a refused key redraws the view.
- **P2 Code** — the options, the check and its reading, and the backoff in `marley_remote`; the
  supervisor and the overlay in `remote.rs`; the provider's hand-off in `routing.rs`; the overlay
  chained in `block_filter.rs`; the input hold in `terminal.rs` with its ledger row; the
  collector's options; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_remote.md` and
  `marley_workbench.md`, the fleet contract's collector line, ledger capture (with an AD that
  corrects AD-claude-610's bound), close the ticket, archive, commit.
