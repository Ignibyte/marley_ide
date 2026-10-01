# Restart a service from its port row, and its state and logs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-615-restart-a-service-from-its-port-row.md
- **Pipeline spec:** 615-restart-a-service-from-its-port-row.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-603: Stop stops a listener's service; a refusal is a toast with the reason and Copy Command.
  - A row's key is its port and pid, which change on a restart; the row must be kept across the gap.
  - More hover buttons would clip the row's text further (#618), so the actions go in a menu.
  - `unit_state` covers user units only; a system unit needs `systemctl show` without `--user`.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.

### Promotion (Opus, 2026-09-30)
- The pair moved to `active/`; the BACKLOG row removed; the ticket in-progress.
- **Recall, again:** as queued, plus #614's PR (an engine's own unit is never stopped from a
  row; restart follows the same filter). The brain (consultation
  5c4dd515cb0748758135cb6140933a1c): nothing on this seam.
- **Seams re-verified:**
  - `systemctl [--user] show -p Id,ActiveState,SubState u1 u2 …` prints one block a unit, in the
    order asked, separated by blank lines; `Id` is the unit's real name (`dbus.service` answers
    `dbus-broker.service`), so blocks are matched by order.
  - `ports::stop_unit`, `refusal_reason`, `hand_command`; the rail's `stop_port`, `refused_toast`,
    `render_port_row`, `stop_words`; `RowLine { text, state, color }` (a trailing state word,
    F-claude-603); the thread row's `right_click_menu` (`rail.rs` thread rows).
  - `agents::start_in_terminal(workspace, Some(dir), None, Some(send_payload(cmd)), window, cx)`
    as `launch.rs` runs a command in a new terminal.
  - #603's scenario starts its units with `systemd-run --user --unit=… --property=Restart=always
    --property=RestartSec=2 python3 -m http.server`.

### Design
- **Approach.**
  - *`marley_rail`*: `PortSnapshot.state` and `PortRow.state: Option<String>`, a word.
  - *`marley_workbench::ports`*: `Ports.units` (each service row's `(ActiveState, SubState)`,
    asked once a scan with one `systemctl show` for the user's units and one for the system's,
    only when service rows exist) and `Ports.restarting` (`Restarting { key, port, service,
    folder, since }`). After a Restart, while no listener holds the port in that project, the
    scan keeps a row for it (pid 0, the unit as its name), until the port listens again, the unit
    is `inactive`, or ten minutes pass. `unit_word(service)` gives the row's word: nothing when
    active and running, `restarting` (SubState `auto-restart`, or a kept row with no state),
    `starting`, `stopping`, `failed`, `stopped`. `restart_service(service)` runs `systemctl
    [--user] restart <unit>` and answers `Done` or `Refused { command, reason }`;
    `stop_service(service)` is `stop_unit` for a kept row, whose pid is no process; an engine's
    own unit is refused.
  - *The rail*: a port row gets a right-click menu (the thread row's pattern): Open in a Browser
    Tab, Copy URL, then Restart Service, Stop Service and Show Logs for a service, or Stop Process
    for any other. Restart records the kept row, then restarts; a refusal is a toast with the
    reason and Copy Command. Show Logs opens `journalctl [--user] -u <unit> -f` in a new terminal of
    the row's project through `start_in_terminal`. The unit line carries the word as its state,
    red for `failed`. A kept row's hover Stop stops its unit.
- **File manifest** (Marley crates only): `crates/marley_rail/src/marley_rail.rs`,
  `crates/marley_workbench/src/ports.rs`, `crates/marley_workbench/src/rail.rs`.
- **Visual check plan** (`script/e2e/615-restart-a-service-from-its-port-row.sh`, sway): a user
  unit of the scenario's own (`systemd-run --user`, `RestartSec=4`), serving from a script that
  serves while a flag file says so and else exits 1.

  | REQ | Set up and do | Shot |
  |---|---|---|
  | REQ-001 | Right-click the service's port row | `menu.png` |
  | REQ-002 | Restart Service | `restarting.png` (the kept row, `restarting`), `restarted.png` (the new pid in its tooltip) |
  | REQ-003 | Clear the flag, restart again | `failed.png` (the kept row, `failed` in red) |
  | REQ-004 | Show Logs | `logs.png` (a terminal running `journalctl --user -u … -f`) |

- **Risks and decisions:**
  - A unit that never listens again keeps its row ten minutes at most.
  - The state is one `systemctl show` per scan for all rows, not one per row (#603's note).

## Phase 2 — Code (2026-09-30)
- **Built,** as designed:
  - `marley_rail`: `PortSnapshot.state`, `PortRow.state`.
  - `ports.rs`: `UnitState`, `Restarting`, `Ports.units` and `Ports.restarting`; the scan asks
    `unit_states` (one `systemctl show -p ActiveState,SubState` per manager, blocks matched by
    order) for every service row and kept row, then `keep_restarting` (a pid-0 row while the port
    is quiet, until it listens, the unit is inactive, or ten minutes pass); `unit_word`;
    `ServiceAction`, `begin_restart`, `restart_service` and `stop_service`, each refusing an
    engine's own unit; `RESTART_KEPT` with `Duration::from_mins`.
  - The rail: `PortMenu` (Open in a Browser Tab, Copy URL, then Restart Service, Stop Service and
    Show Logs for a service, Stop Container, or Stop Process) on a `right_click_menu` around the
    row; `restart_port`, `stop_service_port`, `show_logs` (`journalctl [--user] -u <unit> -f`,
    the unit quoted, through `agents::start_in_terminal`); the unit line as a `RowLine` with the
    state word, red when `failed`; a kept row's hover Stop stops its unit; `port_row_buttons`.
- **Deviations:** `port_row_buttons` was taken out of `render_port_row` (clippy's
  `too_many_lines`); `show_logs` is an associated function (`unused_self`).
- **Review of the diff:**
  - REQ-001: the menu's service entries; non-service rows keep a Stop Process entry.
  - REQ-002: Restart keeps the row before `systemctl restart` runs, so the quiet port reads
    `starting` or `restarting`, then the new listener replaces it.
  - REQ-003: `failed` from the unit's `ActiveState`, on the kept row once the port is gone.
  - REQ-004: the journal in a terminal of the row's project.
  - The states cost one `systemctl show` per manager per scan, only with service rows.
- **Gate:** run 1 red on gate:2 (`semicolon_if_nothing_returned` on a closure rustfmt had made an
  expression), fixed; run 2 `just gate-diff` 17 PASS, 0 FAIL.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/615-restart-a-service-from-its-port-row.sh` under sway: a user unit
  of its own (`systemd-run --user`) running `serve.sh`, which waits 4 s and then serves the
  scratch project on a free port, and exits 1 at once when its flag file is gone. Positions
  measured from the first runs.
- **Shots and files, each read** (the last run):
  - `menu.png` (REQ-001): the port row's menu, with Open in a Browser Tab, Copy URL, then Restart
    Service, Stop Service and Show Logs; no tooltip over it.
  - `restarting.png` (REQ-002): two seconds after Restart Service, the kept row ":<port>
    marley-e…" with its unit line ending `starting` (the unit is up and its port not yet
    listening; with no state yet it would read `restarting`).
  - `restarted.png` (REQ-002): ":<port> python3" again, its tooltip with the new pid.
  - `failed.png` (REQ-003): the flag removed and the unit restarted: the kept row's unit line in
    red, ending `failed`; `state.txt`: `failed`.
  - `logs.png` (REQ-004): Show Logs opened "repo — journalctl --user -u …" in `repo`, following the
    unit's journal (its Started, Stopping and "Failed with result 'exit-code'" lines).
- **Reds found and fixed:**
  - Run 1: the row's tooltip lay over the menu it had just opened. The tooltip is now built in
    the menu's trigger only while the menu is closed (`menu_open`), as Zed's dock buttons do.
  - Run 3: Show Logs opened its terminal in Marley's own working folder; it now opens in the
    project's first folder, as a terminal from its `+` does.
  - Run 2 stopped early on the scenario's own `systemctl is-active`, which ends non-zero for a
    failed unit; the step tolerates it.
  - `just gate-diff` after the fixes: 17 PASS, 0 FAIL.
- **Focus:** sway stopped with the run's Marley each time; Hyprland had no Marley windows
  before or after; the unit was stopped and reset in `teardown`.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; `workbench-shell.md`; `marley_workbench.md` (a Restart, state and
  logs bullet in the Ports section); the guide, the guide page and the walkthrough (an optional
  check in 2.8).
- **Knowledge:** F-claude-615-a-rows-tooltip-lay-over-the-menu-it-opened-001,
  F-claude-615-show-logs-opened-in-marleys-own-folder-001,
  PR-claude-615-a-menus-trigger-owns-its-tooltip-001.
- **Brain:** consultation 5c4dd515cb0748758135cb6140933a1c closed with `brain decide`.

