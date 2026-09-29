# The e2e runner's cleanup after its headless compositor exits — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-588-e2e-cleanup-after-the-compositor-exits.md
- **Pipeline spec:** 588-e2e-cleanup-after-the-compositor-exits.spec.md

## Phase 1 — Plan
- **Request:** TICKET-588, from #560's golden run (2026-09-28): 508's headless sway exited between
  two steps; the cleanup stopped on `SEAT_POINTER_PID: unbound variable`, so Marley's log was never
  copied.
- **Classification:** chore, the e2e harness; shell only, no Rust.
- **Recall (§18.3):** L-claude-448's `pgrep -f` trap (a pattern matches the command that holds it;
  the runner matches sway by `-c $SWAY_DIR/config`, which only sway's own command line and now its
  wrapper's hold). #560's notes, Phase 3 (the exit and the unbound variable). Brain: consultation 26e04b8099d0411b95fed61b7d828778, nothing on
  this seam.
- **Design.** In `sway_start`, `setsid -f sh -c 'sway --verbose -c "$1" …; echo "$?" > "$2"'` with
  the config and `$SWAY_DIR/exit`. `sway_stop` first reads `$SWAY_DIR/exit`: present means sway
  exited on its own; it records `COMPOSITOR_EXIT` for the cleanup. Marley's pid: `launch_marley`
  sets `MARLEY_PID` from `window` once it answers (the sway case); `sway_stop` uses
  `sway_marley_pid`, else `MARLEY_PID` when `/proc/<pid>/cmdline` holds `$E2E_PROFILE`.
  `${SEAT_POINTER_PID:-}`. The cleanup, after copying the log, prints the line and `exit 1` when
  `COMPOSITOR_EXIT` is set. The status line names a signal for 128+N.
- **Manifest.** `script/e2e.sh`; `script/e2e/588-cleanup-after-the-compositor-exits.sh`.
- **Risks.** `--verbose` makes sway's log longer; it stays in the shots folder, outside the repo.

## Phase 2 — Code
- **Built** (`script/e2e.sh`): `MARLEY_PID` and `COMPOSITOR_EXIT`; sway starts through a wrapper
  script in the run's sway folder (`sway --verbose -c <config>`, then its status into `exit`), a
  quoted heredoc rather than `sh -c '…'`, which shellcheck's SC2016 flags; `launch_marley` keeps the
  pid its window gave; `sway_stop` reads the status before it asks sway to exit, stops the Marley by
  that pid when sway's tree cannot name it and the process still runs on the run's profile, reads
  `${SEAT_POINTER_PID:-}`, and removes a dead sway's IPC and Wayland sockets; `stop_browser_units`
  stops the user units of the run's Browser tabs before the profile goes; the cleanup names the
  compositor's exit (`exit_status` gives the signal) and exits 1.
- **Deviations.** Two additions the runs showed were needed: the dead sway's sockets (a killed sway
  left `sway-ipc.*.sock`, `wayland-2` and its lock in the runtime folder) and the browser units
  (two units from this session's earlier runs still ran on removed profiles and wrote them back).
- **Gate.** GATE GREEN [diff] (no `.rs`), twice: after the runner and after `shot.sh`.

## Phase 3 — Test
- **The 588 check** (`script/e2e/588-cleanup-after-the-compositor-exits.sh`, sway): after a first
  shot it kills its sway with SIGKILL. The runner printed `sway: gone before the run's end; its
  sockets removed, the run's Marley, pointer and keyboard stopped` and `sway: the compositor exited
  during the run with status 137 (SIGKILL), so the run failed; see …sway.log`, and exited 1
  (REQ-001). Marley's log was copied beside the shot, and the profile and work folders were gone
  (REQ-002). After it no process named the run's profile, no `seat-pointer`, no `wtype -s`, no sway
  socket but Hyprland's `wayland-1` (REQ-003). sway's verbose log ends with the wrapper's
  `Killed`, the line an unasked exit now leaves.
- **Regress** (`just regress script/e2e/588-cleanup-after-the-compositor-exits.sh`): `FAIL
  588-cleanup-after-the-compositor-exits 15 s: sway: the compositor exited during the run with
  status 137 (SIGKILL), so the run failed; …` (REQ-004).
- **A normal sway run**, `563-terminal-shortcut-note.sh`: exit 0, `sway: stopped, with the run's
  Marley, pointer and keyboard` (REQ-005).
- **A red in the check itself.** The first 588 script had no `setup` and no `open_path`, so the
  run's Marley restored the copied profile's last session: the user's own project and an Agent
  Panel thread, whose agent Zed started over ACP (its log: `session/query resume=…`; no prompt was
  sent, and no agent process outlived the run). The shot and the run's folders were deleted. Two
  fixes: the scenario opens a scratch repository, and the runner refuses a scenario whose setup
  names no folder (`no folder to open: …`), checked with a scratch scenario; `shot.sh`, the one
  scenario that opened nothing by design, opens a scratch repository unless `OPEN` names one, and
  `just shot` was run and its shot read (the scratch repository, no agent in its log). Every other
  scenario already calls `open_path`.
- **Housekeeping found on the way.** Six scratch profiles (368 MB of the runtime folder's tmpfs)
  from runs whose cleanup had stopped, two browser units still running on two of them, and two dead
  sways' sockets: removed after checking no process used them. The real Marley's browser unit was
  left alone.

## Phase 4 — Complete
- Ledger: F-claude-588-a-scenario-that-opened-nothing-restored-the-users-session-001,
  PR-claude-588-every-scenario-names-what-marley-opens-001,
  L-claude-588-a-detached-programs-exit-and-a-coprocs-pid-001.
- Brain: decision recorded on consultation 26e04b8099d0411b95fed61b7d828778.
