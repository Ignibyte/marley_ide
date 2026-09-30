# Port rows that know a service — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-603-service-aware-port-rows.md
- **Pipeline spec:** 603-service-aware-port-rows.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30, after Stop on the rail's `:3101 node` row kept coming back.
- **Evidence (2026-09-30, a client project on the dev box; unit names made generic):** `/proc/<pid>/cgroup` of the listener read
  `0::/system.slice/app-playwright.service`; the unit has `User=<the user>`,
  `Restart=always`, `RestartSec=5`; the journal shows "Scheduled restart job, restart counter is
  at 1" five seconds after each Stop. `app-octane.service` and `app-reverb.service`, stopped
  the same way, stayed down ("Deactivated successfully"): their restart policy does not cover a
  clean exit, and systemd then shows them inactive though no one stopped the unit.
- **Classification / tier:** feature; `marley_browser::ports`, `rail.rs`, the workbench's spawn
  site.
- **Recall (§18.3):** #521 (port rows; Stop after a fresh look); #541 (gate:22: spawns only in
  `.config/spawn-sites.txt` files: the PTY, Chromium's unit and relay, and `process.rs`).

### Visual check plan
- A transient user unit (`systemd-run --user`) with `Restart=always` stands in for the service;
  the runner's scratch profile keeps it apart from Chad's. Tear it down at the end
  (`systemctl --user stop marley-e2e-603`, `reset-failed`).
- The system branch: a fake `systemctl` first on the PATH that prints polkit's "Interactive
  authentication required." and exits 1, to shoot the toast.

### Risks
- The polkit prompt steals focus from Marley on a real desktop; acceptable, since the user asked.
- A service whose main process forks the listener (a child in the same cgroup) is still the
  service's: the cgroup path is the same.

### Promotion (Opus, 2026-09-30, back-to-back run)
- **Pre-flight:** no active pipeline, cargo idle, README marker present, branch level with its
  origin after #602.
- **Recall (§18.3):**
  - AD-claude-541: a spawn only in a listed file. The workbench's programs go through
    `process::output`, so `systemctl` adds no spawn call.
  - AD-claude-521: a listener belongs to the project whose folder holds its working directory.
    A `systemd-run --working-directory` inside the project places the scenario's service.
  - F-claude-521: the scan runs only while a rail shows. Unchanged here.
  - Brain (consultation e2b1ca9f): nothing on this seam; only unrelated follow-ups came back.
- **Seams re-verified:**
  - `marley_browser::ports` (`Listener`, `listeners_in`, `stop_in`).
  - The workbench's `ports::stop`, `attribute`, and `list`, which is MCP's `ports_list`.
  - `rail.rs`: `stop_port`, `render_port_row`, `port_snapshots`.
  - `marley_rail`: `PortSnapshot` and `PortRow`, one construction each.
  - `process::output`; `Toast::on_click`; `Tooltip::with_meta`.
  - The runner calls a scenario's `teardown` from its EXIT trap, and `setup` runs in its own
    shell before Marley starts, so a PATH set there reaches Marley.
- **Checked on the box:**
  - The shell's cgroup is an app scope under `app-graphical.slice`, so a desktop launch runs
    Marley in a scope.
  - Marley's relay runs inside its `marley-browser-*.service` user unit, left out by name as
    before.
  - A Marley started as a service would hold its terminals' dev servers in its own unit: hence
    D5.

### Design
- **Approach.**
  - *`marley_browser::ports` (pure parse, `*_in` IO):*
    - `Service { unit, user }`.
    - `service_of(cgroup_text)`: the unified line `0::<path>`; its last part names a
      `.service`; `user` when an earlier part is `user@….service`.
    - `Listener::service`, read from `<proc>/<pid>/cgroup` in `listeners_in`.
    - `own_service_in(proc_root)`, from `<proc>/self/cgroup`.
  - *The workbench's `ports.rs`:*
    - `attribute` drops the service of a listener in Marley's own unit (D5).
    - `stop` returns a `Stop`:
      - `Signalled(Stopped)` for a process;
      - `Unit { unit, user }` once `systemctl` stopped the unit;
      - `Refused { unit, user, reason }` when it failed.
    - It scans afresh: not listening is `Signalled(NotListening)`; a service is `stop_unit`,
      which runs `systemctl [--user] stop <unit>` through `process::output`; otherwise `stop_in`.
    - `hand_command(unit, user)` gives the command a refusal offers.
  - *`marley_rail`:* `PortService { unit, user }` on `PortSnapshot` and `PortRow`, copied by
    `rail_rows`.
  - *`rail.rs`:*
    - `port_snapshots` fills it and adds `user service <unit>` or `system service <unit>` to the
      row's tooltip.
    - `render_port_row`: the URL line takes the unit as its trailing state (` · <unit>`), as a
      command line takes its state. Stop's tooltip is `Tooltip::with_meta`, one of "Stop the
      Process" (SIGTERM to its pid), "Stop the User Service" (`systemctl --user stop <unit>`) or
      "Stop the System Service" (`systemctl stop <unit>`, which asks for authorization).
    - `stop_port` shows `Refused` as a toast with a Copy Command button (D6).
- **File manifest** (all Marley crates; no Zed path):
  - `crates/marley_browser/src/ports.rs`;
  - `crates/marley_workbench/src/ports.rs`;
  - `crates/marley_rail/src/marley_rail.rs`;
  - `crates/marley_workbench/src/rail.rs`;
  - `script/e2e/603-service-aware-port-rows.sh` (Test).
- **Visual check plan** (sway):

| REQ | Scenario step | Shot |
|---|---|---|
| 001 | Two transient user units with `Restart=always`, in the project's folder | `row.png`: each row's second line ends ` · marley-e2e-603….service`; `row-tooltip.png`: `user service …` |
| 005 | Hover the first row's Stop | `tooltip.png`: "Stop the User Service", `systemctl --user stop …` |
| 002 | Click Stop; wait eight seconds (past `RestartSec`) | `stopped.png`: its row gone; `state.txt`: `inactive` |
| 003 | Click Stop on the second; a fake `systemctl` refuses it | `refused.png`: the toast names the unit and the reason; Copy Command, then `clipboard.txt` holds `systemctl --user stop …` |
| 004, 006 | Not reachable: a plain process's SIGTERM is #521's path unchanged, and Marley's own unit needs Marley started as a service | Review |

### Risks (added at promotion)
- Polkit's dialog takes the focus while `systemctl stop` waits for it. The call runs in the
  background, so Marley stays live.
- A `.service` holding many programs (a desktop session's manager, say) would all stop. Only a
  leaf `.service` that holds the listener counts, and Stop is a button the user presses on that
  row.

## Phase 2 — Code (2026-09-30)
- **Built:**
  - `marley_browser::ports`: `Service`, `service_of`, `own_service_in`, and `Listener::service`,
    read from each listener's `cgroup` file.
  - The workbench's `ports.rs`:
    - `attribute` clears the service of a listener in Marley's own unit;
    - `Stop`, and `stop` with a fresh scan;
    - `stop_unit` through `process::output("systemctl", …)`;
    - `refusal_reason` and `hand_command`.
  - `marley_rail`: `PortService` on `PortSnapshot` and `PortRow`.
  - `rail.rs`:
    - `port_snapshots` adds the service line to the row's tooltip;
    - `render_port_row` shows the unit as the URL line's trailing state and gives Stop a
      `with_meta` tooltip from `stop_words`;
    - `stop_port` maps `Stop` to its toast, and `refused_toast` carries the Copy Command button.
- **Deviations:**
  - A refusal of a user service gets the same toast as a system one, with `systemctl --user stop`
    as its command, so the Test can shoot the toast with a user unit.
  - `refusal_reason` keeps systemd's own words after `Failed to stop <unit>: ` on the first line,
    since the toast already names the unit and the second line only points at the logs.
- **Review of the diff:**
  - REQ-001: the row's line and tooltip. REQ-005: the three tooltip texts.
  - REQ-002 and REQ-003: `systemctl [--user] stop <unit>`, no `sudo`; the unit comes from a fresh
    scan, not the row.
  - REQ-004: no service goes to `stop_in` unchanged.
  - REQ-006: Marley's own unit is cleared in `attribute` for the row and filtered in `stop` for
    the action.
  - The spawn is in `process.rs`, and gate:22 passes with its pin unchanged.
  - Nothing is written to the clipboard until Copy Command is pressed.
  - No entity is touched in the background task; the toast is shown through the window's
    workspace, as before.
- **Gate:**
  - First run: RED, gate:2 clippy. `cargo fmt` had turned the Copy Command closure into a block
    without a semicolon (`semicolon_if_nothing_returned`); the semicolon was added.
  - Second run: `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/603-service-aware-port-rows.sh` (sway).
  - `setup` writes a fake `systemctl` into `$E2E_WORK/bin` and puts it first on the PATH, which
    reaches Marley. The fake refuses `stop` for `marley-e2e-603-refused` with polkit's message
    and hands every other call to `/usr/bin/systemctl`.
  - `setup` also starts `marley-e2e-603` (38603) and `marley-e2e-603-refused` (38604) with
    `systemd-run --user`, `Restart=always` and `RestartSec=2`, working in the project's folder.
  - `teardown` stops and resets both units; none was left after either run.
- **Run 1: a red.**
  - `row.png` showed each port row's second line as only ` · marley-e2e-603.service`: the unit,
    laid out as the line's fixed trailing part, had squeezed the URL out entirely (confirmed on a
    3x crop).
  - Fix at the source: a service's unit gets a line of its own under the URL, so the row is two
    lines tall.
  - Also changed: the toast wrapped the command in backticks, which a toast shows as they are;
    it now reads "run: <command>".
  - The gate ran again: 17 PASS, GATE GREEN.
  - The run's clipboard check failed only because Copy Command's position was a guess; it was
    measured from `refused.png` (1199, 933).
- **Run 2: every shot read.**
  - `row.png` and `row-tooltip.png` (REQ-001): each service row shows its URL, then its unit, both
    clipped at the default rail width while the hover buttons keep their room. The row's tooltip
    ends "user service marley-e2e-603.service".
  - `tooltip.png` (REQ-005): Stop's tooltip is "Stop the User Service" over
    `systemctl --user stop marley-e2e-603.service`.
  - `stopped.png` and `state.txt` (REQ-002): after Stop and eight seconds (past the two-second
    restart delay), the :38603 row is gone and the unit is `inactive`; the check passed.
  - `refused.png`, `clipboard.txt` and `refused-state.txt` (REQ-003's toast): the toast reads
    "Could not stop the user service marley-e2e-603-refused.service: Interactive authentication
    required. Marley leaves its process alone, since systemd would start it again. To stop it
    yourself, run: systemctl --user stop marley-e2e-603-refused.service", with Copy Command.
    After Copy Command, the clipboard holds that command (the check passed), and the unit is
    still `active`: nothing signalled its process.
- **Not reachable by a scenario:**
  - REQ-003's system branch (`systemctl stop` through polkit, the toast's `sudo systemctl stop`)
    needs a system service, which needs root. The same toast code runs with `hand_command`'s
    other arm; covered by the review.
  - REQ-004 (SIGTERM for a plain process) is #521's path, unchanged behind the `None` arm.
  - REQ-006 (Marley's own unit) needs Marley started as a service.
  - The Stop tooltips for a system service and a process: `stop_words`, reviewed.
- **Focus:** its own headless sway; Hyprland had 0 Marley windows before and after.
- **Pre-existing, not in scope:** a port row's end buttons keep their width while hidden, so the
  row's lines are clipped at the default rail width, a layout #521 set.

## Phase 4 — Complete (2026-09-30)
- **Documented:**
  - `CHANGELOG.md`.
  - `docs/marley_architecture/marley_browser.md` (`Listener::service`, `service_of`,
    `own_service_in`).
  - `docs/marley_architecture/marley_workbench.md` (`stop`, `Stop`, the row's line and
    tooltips, the toast).
  - `docs/marley_architecture/marley_rail.md` (`PortService`).
  - `docs/marley/workbench-shell.md` (the slice line), `docs/marley/guide.md` (port rows) and
    `docs/marley/walkthrough.md` (an optional check in 2.8).
  - The guide page's `port-rows` article, rebuilt from the scratch parts.
  - No Zed path touched.
- **Knowledge:**
  - F-claude-603-a-long-trailing-state-squeezed-the-rows-text-out-001;
  - L-claude-603-stop-a-service-not-its-process-001;
  - L-claude-603-a-scenario-fakes-a-program-for-marley-through-setups-path-001;
  - AD-claude-603-stop-stops-a-listeners-service-001.
  - Brain: consultation e2b1ca9f closed as
    `decisions/marleys-port-rows-stop-a-servers-systemd-service-not-its-process`.
