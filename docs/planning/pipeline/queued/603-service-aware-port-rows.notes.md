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
