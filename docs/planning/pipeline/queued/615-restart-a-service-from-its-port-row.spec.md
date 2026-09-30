---
pipeline_id: 0d263290-71b0-45e7-bdee-11700f9199c7
ticket: docs/planning/tickets/open/TICKET-615-restart-a-service-from-its-port-row.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Restart a service from its port row, and its state and logs"
type: feature
slice: workbench shell, the rail's ports; after #603
references: [docs/planning/pipeline/completed/603-service-aware-port-rows.spec.md]
---

## Title
A port row served by a systemd unit gets a right-click menu with Restart Service, Stop Service and
Show Logs, and the row says the unit's state when it is not simply running.

## Scope
### In
- **The menu:** a port row gains a right-click menu (the thread row's `right_click_menu` pattern):
  Open, Copy URL, then for a service row Restart Service, Stop Service and Show Logs. No new
  hover button, so the row's text keeps its room (#618).
- **Restart:** `systemctl [--user] restart <unit>` through the spawn module; a refusal (a system
  unit without rights) is a toast with the reason and Copy Command, as Stop's is.
- **State:** the unit's `ActiveState` and `SubState`, asked once per scan for every service row
  together, not per row; the row shows `failed`, `restarting` (SubState `auto-restart`),
  `activating` or `stopped` beside its name, nothing when active.
- **Rows that vanish:** after a restart the listener's pid changes and the port is gone for the
  unit's `RestartSec`; the row stays, marked `restarting`, until it listens again or the unit
  stops.
- **Show Logs:** opens `journalctl [--user] -u <unit> -f` in a new terminal of the row's
  project, through `agents::start_in_terminal` as launch configs do.

### Out (explicitly deferred)
- Starting a stopped unit that no longer listens (its row is gone); editing units.
- Logs of a process with no unit.

## Reference (§20)
N/A — Marley-specific: #603's service rows. systemd's `systemctl restart`, `show -p ActiveState,
SubState` and `journalctl -u … -f` are the published interface.

### Prior art
- **Behavior maps:** #603 (AD-claude-603, L-claude-603, F-claude-603-a-long-trailing-state).
- **Published material:** `systemctl(1)`, `journalctl(1)`.
- **Code we already ship:**
  - `ports::stop` and `stop_unit` (`marley_workbench/src/ports.rs:242-296`), `hand_command` (313).
  - `marley_browser::service::unit_state` (`service.rs:440-458`), `--user` only today.
  - `render_port_row` and `stop_port` (`rail.rs:4669-4764, 3085-3110`); the thread row's
    `right_click_menu` (4311-4327).
  - `agents::start_in_terminal` (`agents.rs:350`) and `marley_agent::send_payload`, as
    `launch.rs:420-456` runs a command in a new terminal.

## UI proof
The scenario `script/e2e/615-restart-a-service-from-its-port-row.sh` (sway) starts a user unit of
its own (`systemd-run --user` running a small HTTP server in the scratch project, as #603's
scenario does), then:
- right-clicks its port row (`menu.png`: Restart Service, Stop Service, Show Logs);
- chooses Restart Service (`restarting.png` while it restarts, `restarted.png` with the new pid in
  the tooltip);
- makes the unit fail and shoots `failed.png`;
- chooses Show Logs (`logs.png`: a terminal running `journalctl --user -u … -f`).

## Locked-In Decisions
- D1 — The actions are in a right-click menu, not new hover buttons.
- D2 — Unit states are asked in one `systemctl show` per scan for all service rows.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user right-clicks a service's port row, the rail shall offer Restart Service, Stop Service and Show Logs. | Shot `menu.png` |
| REQ-002 | WHEN the user chooses Restart Service, Marley shall restart the unit, and the row shall read `restarting` until it listens again. | Shots `restarting.png`, `restarted.png` |
| REQ-003 | WHILE a service's unit is failed, its row shall say so. | Shot `failed.png` |
| REQ-004 | WHEN the user chooses Show Logs, Marley shall open the unit's journal, followed, in a terminal of the row's project. | Shot `logs.png` |

## Phase Plan
- **P1 Plan** — promote, recall, the design (the batched state query, the kept row).
- **P2 Code** — the menu, restart, state, logs; a review; the gate green.
- **P3 Test** — the scenario with its own unit, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
