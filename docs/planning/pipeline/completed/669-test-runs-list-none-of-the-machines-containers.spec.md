---
pipeline_id: 7753e0e9-32d1-4d70-93dc-29badcbbba1b
ticket: docs/planning/tickets/closed/TICKET-669-test-runs-list-none-of-the-machines-containers.md
status: Phase 4 — Complete PASS
title: "Test runs list none of the machine's containers"
type: chore
slice: the rail's ports (#521, #614) and the e2e harness (CONSTITUTION §7)
references: [docs/planning/pipeline/completed/614-container-ports-in-the-rail.spec.md, docs/planning/pipeline/completed/668-test-runs-act-on-nothing-of-the-users.spec.md, docs/t3code_architecture/05-terminal-browser-and-capture.md]
---

## Title
Test runs list none of the machine's containers. The rail lists, under Containers, every port a
container publishes that no project's folder holds (#614), read from the machine's `/proc` and
its `docker ps`. In a test run those are the user's containers, each with a Stop, and their number
moves every row below them (13 on the dev box pushed #640's Harness rows out of the window). A
setting, Containers in the Rail, on by default, lets the rail leave them out; each run's copy of
the settings turns it off, and #614's scenario, which tests that list, turns it back on.

## Scope
### In
- **`marley.rail_containers`** (`settings_content`, `default.json` `true`): whether the rail lists
  the container ports no project's folder holds. A container whose Compose folder is in a project
  shows under that project either way.
- **The rail** draws no Containers section while it is off, and draws it again when it is turned
  on, without a restart.
- **The Settings window's Marley page**: a Containers in the Rail toggle in the Layout section,
  after Rail Order.
- **`script/e2e.sh`**: the copy turns `marley.rail_containers` off, in the same Python block as
  #668's.
- **#614's scenario** turns it on in `setup`, and its stand-in proxy takes a port below any the
  machine's containers publish, so its row is the first under Containers whatever the machine
  runs.
- **#640's scenario**: `BUILD_Y` measured again with no Containers list above the Harness
  section (found in Test).

### Out (explicitly deferred)
- **The engines asked in a run.** The scan still runs `docker ps` and `podman ps` to name a
  project's containers; reading lists nothing of the user's and stops nothing.
- **641's Fleet panel**, which lists this machine's processes for the scenario's own `lab` host on
  127.0.0.1: read only, and the scenario's choice of host.

## Reference (§20)
N/A — Marley-specific: the rail's port rows are Marley's own (#521, #614); Zed lists no ports and
Warp's maps name none.

### Prior art
- **Behaviour maps:** T3 Code's `PortScanner` (`docs/t3code_architecture/05-terminal-browser-and-capture.md`
  §2.9) ties a listener to the terminal whose process tree owns it, so it never lists the machine's
  other servers; Marley lists the machine's containers on purpose (#614), and this ticket makes
  that a choice. `docs/warp_architecture/` and `docs/orca_architecture/` name no port list.
- **Published material:** none applies.
- **The code we ship:** `crates/marley_workbench/src/ports.rs` (`attribute_containers` returns the
  ports listed apart; `Ports::containers` serves them), `rail.rs` `render_containers`, the only
  reader; `MarleySettings` and the Marley settings page's existing toggles (`prompt_editor`,
  `sticky_command_header`) give the shape. No crate we build owns this seam.

## UI proof
`script/e2e/669-test-runs-list-none-of-the-machines-containers.sh` (`compositor sway`). Its
`setup` puts a fake `docker` first on the PATH that lists no container and records any `stop`,
and starts a stand-in `docker-proxy` for port 669, so the run has a container port of its own
whatever the machine runs. Shots: `669-01-off` (the run's copy: no Containers section);
`669-02-on` (the setting turned on in the run's settings: Containers with port 669 first);
`669-03-setting` (the Settings window's Marley page, searched for Containers). Then #614's
scenario runs and passes, its `apart` and `refused` shots on its own row, and #640's shows its
Harness rows in the window with its `BUILD_Y`.

## Locked-In Decisions
- D1 — **A setting, not a test-only variable**: the list is a real choice for a user with many
  containers (13 rows on the dev box push the rail's other sections down), and the harness already
  turns features off through the settings copy (#633 to #668).
- D2 — **On by default**: users keep #614's behaviour.
- D3 — **Hide at the rail, keep the scan**: `render_containers` is the only reader; the scan still
  names a project's containers.
- D4 — **#614's stand-in port is fixed low** (614): its proxy binds nothing, so any number works,
  and rows apart are sorted by port.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.rail_containers` is off, the rail shall list no container port that no project's folder holds. | `669-01-off` |
| REQ-002 | WHEN `marley.rail_containers` is turned on, the rail shall list those ports under Containers, without a restart. | `669-02-on` |
| REQ-003 | The Settings window's Marley page shall show Containers in the Rail with its description. | `669-03-setting` |
| REQ-004 | WHEN a test run copies the user's settings, the copy shall turn `marley.rail_containers` off. | the scenario's check |
| REQ-005 | WHILE a scenario turns it on, #614's scenario shall show, stop and refuse its own container, its row the first under Containers. | 614's checks and shots |
| REQ-006 | WHEN #640's scenario runs, its Harness rows shall show in the window with its `BUILD_Y`. | 640's shots |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the setting, the rail, the settings page, the harness; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its shots; 614's and 640's runs.
- **P4 Complete** — CHANGELOG, the ledger rows, the guide, ledger capture, the brain decision,
  close, archive, commit.
