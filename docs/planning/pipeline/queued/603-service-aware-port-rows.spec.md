---
pipeline_id: 9cf4965d-533d-4428-9d3c-3b822c9056d6
ticket: docs/planning/tickets/open/TICKET-603-service-aware-port-rows.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Port rows that know a service"
type: feature
slice: workbench shell (the rail's port rows), after #521
references: [docs/planning/pipeline/completed/521-ports-per-project.spec.md]
---

## Title
A port row names the systemd service that runs its listener, and Stop stops that service rather
than signalling its process, so a service with a restart policy stays down.

## Scope
### In
- **Finding the service.** For each listener, Marley reads `/proc/<pid>/cgroup`. A path ending in
  `<name>.service` under `system.slice` is a system service; one under the user's
  `user@<uid>.service` is a user service. Scopes (`.scope`, such as a desktop app's or Marley's own
  terminals') are not services and change nothing.
- **The row.** A service's name shows on the row's second line beside the URL, as
  `app-playwright.service`, and in its tooltip with whether it is a system or a
  user service.
- **Stop, for a user service:** `systemctl --user stop <unit>`; the row goes at the next look and
  stays gone.
- **Stop, for a system service:** `systemctl stop <unit>`, which asks the desktop's polkit agent
  for authorization. If that fails (refused, no agent, not allowed), a toast names the unit, says
  systemd would restart the process if Marley signalled it, and puts `sudo systemctl stop <unit>`
  on the clipboard with a Copy button.
- **Stop, for anything else:** SIGTERM as #521 does.
- **The Stop button's tooltip** says which of the three it will do.

### Out (explicitly deferred)
- Containers (Docker, Podman): their published ports belong to root's `docker-proxy`, which the
  rail does not list; rootless containers wait for a ticket of their own.
- Starting or restarting a service from the rail, and showing a unit's state or logs.
- Services of other users (the rail lists none).

## Reference (§20)
N/A — Marley-specific: port rows are Marley's own (#521), with no Warp or Zed counterpart. The
service is read the way systemd itself names a process's unit (`systemctl status <pid>` reads the
same cgroup path), and stopped with systemd's own command and its polkit authorization.

### Prior art
- **Behavior maps.** #521's spec (port rows, Stop by SIGTERM after a fresh look); Orca's port list
  (`docs/orca_architecture/05-terminal-and-workspace.md`) kills processes only.
- **Published material.** systemd's cgroup layout (`systemd.special(7)`: `system.slice`,
  `user@.service`, `app.slice`) and `systemctl stop`'s polkit action
  (`org.freedesktop.systemd1.manage-units`), which asks through the session's agent.
- **Code we already ship.** `marley_browser::ports` (`listeners_in`, `stop_in`,
  `crates/marley_browser/src/ports.rs`) reads `/proc` for listeners and signals them; the workbench
  wrapper `ports::stop` (`crates/marley_workbench/src/ports.rs:232`) runs it in the background.
  gate:22 (#541) allows process spawns only in listed adapter files, so the `systemctl` call goes in
  the workbench's `process.rs` (or another listed site), not in the rail.

## UI proof
The scenario `script/e2e/603-service-aware-port-rows.sh` starts a transient user service with
`systemd-run --user --unit=marley-e2e-603 --property=Restart=always python3 -m http.server <port>`
in the scratch project's folder, shoots the rail's row with the unit's name (`row.png`) and the
Stop tooltip (`tooltip.png`), clicks Stop and, after six seconds, shoots the rail without the row
(`stopped.png`) and records `systemctl --user is-active marley-e2e-603` (`state.txt`: inactive).
The system-service branch cannot be driven without root in the scenario: its toast is checked by
review, and by a scenario step that fakes `systemctl` first on the PATH to fail with polkit's
message (`refused.png`).

## Locked-In Decisions
- D1 — The cgroup path decides; a `.service` unit is a service, anything else is a process.
- D2 — Stop never signals a service's process; it stops the unit.
- D3 — A system service goes through `systemctl stop` and polkit; Marley never runs `sudo`.
- D4 — The `systemctl` spawn lives in a gate:22 listed file.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a listener's process belongs to a systemd service, its port row shall show the unit's name, and its tooltip whether it is a system or a user service. | Shot `row.png` |
| REQ-002 | WHEN the user clicks Stop on a user service's row, Marley shall run `systemctl --user stop <unit>`, and the row shall not come back while the unit stays stopped. | Shot `stopped.png`; `state.txt` |
| REQ-003 | WHEN the user clicks Stop on a system service's row, Marley shall run `systemctl stop <unit>`, and IF that fails, THEN show a toast naming the unit and offering `sudo systemctl stop <unit>` to copy. | Shot `refused.png`; review |
| REQ-004 | WHERE a listener belongs to no service, Stop shall send SIGTERM as before. | Review of the diff |
| REQ-005 | WHILE the pointer is on Stop, its tooltip shall say whether it stops a user service, a system service or a process. | Shot `tooltip.png` |

## Phase Plan
- **P1 Plan** — promote this pair, recall, the design in the notes.
- **P2 Code** — the cgroup read in `marley_browser::ports` (a `service` field on `Listener`), the
  row and tooltip in `rail.rs`, the stop paths through a listed spawn site; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/`, the guide page and the walkthrough
  (§21); the ledger; close, archive, commit.
