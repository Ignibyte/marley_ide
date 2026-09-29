---
pipeline_id: 21a8dc41-30ed-4279-bc32-9799d37cc6af
ticket: docs/planning/tickets/open/TICKET-588-e2e-cleanup-after-the-compositor-exits.md
status: Phase 4 — Complete PASS
title: "The e2e runner finishes its cleanup, and says so, when its headless compositor exits partway"
type: chore
slice: the e2e harness (§7's visual check)
references: [docs/planning/pipeline/completed/560-conflict-chip.notes.md]
---

## Title
When the headless sway of a `compositor sway` run exits before the scenario ends, the runner's
cleanup stops on `SEAT_POINTER_PID: unbound variable` and never copies the run's Marley log.
The cleanup now finishes, the run prints one line naming the compositor's exit and its status and
fails, the run's Marley is stopped even without the compositor to find it by, and sway's own log
is kept at a level that says why it exited.

## Scope
### In
- `script/e2e.sh`: `sway_stop` reads `${SEAT_POINTER_PID:-}` (bash unsets a coproc's `_PID` once
  it reaps it, and the runner runs under `set -u`); sway runs under a wrapper that writes its exit
  status into the run's sway folder; the cleanup, finding that status before it asked sway to exit,
  prints `sway: the compositor exited during the run with status N …, so the run failed; see <log>`
  and makes the run exit 1; `launch_marley` keeps the pid of the Marley window it waited for, and
  `sway_stop` stops that pid when sway's tree can no longer name it, after checking the process is
  still the run's Marley (its command line holds the run's profile); sway starts with `--verbose`.
- `script/e2e/588-cleanup-after-the-compositor-exits.sh`: kills its own sway partway.

### Out (explicitly deferred)
- Why 508's sway exited in #560's golden run: the next exit's verbose log is what can say.
- The Hyprland backend, which the scenario's own compositor never is.

## Reference (§20)
N/A — Marley-specific: the runner is Marley's own test harness, and no Warp or Zed behavior bears
on it.

### Prior art
- **Code we already ship.** `script/e2e.sh` (`sway_start`, `sway_stop`, `cleanup`,
  `launch_marley`, the `pointer` helper's "the helper is gone" message); `script/regress`,
  which reports a failed scenario by the first line matching `FAIL|^no Marley window|^no .*: run|failed`.
- **Published material.** bash's manual: a coprocess's `NAME_PID` is unset once the shell reaps
  it; an `exit` inside an `EXIT` trap sets the script's status. sway(1): `--verbose` and
  `--debug`.
- None in Zed's crates: the runner is shell.

## UI proof
N/A — no UI delta: the change is the e2e runner's cleanup. Its check is the run of
`script/e2e/588-cleanup-after-the-compositor-exits.sh`, which kills its own sway after a first shot;
the runner's output, the files beside the shots and the processes left after it are read.

## Locked-In Decisions
- D1 — sway's status comes from a wrapper that outlives it (`sh -c 'sway …; echo $? > exit'`):
  `setsid -f` leaves no child to wait for, and the file is the one record that survives the run.
- D2 — A compositor that exits during the run fails the run, even when the scenario's steps never
  touched it again: shots taken after it are of nothing.
- D3 — The cleanup stops a Marley by the pid it saw only while that process still runs with the
  run's profile, so a reused pid is never killed.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the run's headless sway exits before the cleanup asks it to, the runner shall print one line naming the exit and its status, and the run shall exit non-zero. | The 588 run's output and exit status |
| REQ-002 | WHEN the pointer helper is gone, the cleanup shall still finish: Marley's log copied beside the shots, the profile and work folders removed. | The 588 run: `588-cleanup-after-the-compositor-exits.marley.log` present |
| REQ-003 | WHEN sway is gone, the cleanup shall stop the run's Marley, pointer helper and key holder. | `pgrep` for the run's profile, the helper and `wtype` after the run: none |
| REQ-004 | WHEN `just regress` runs a scenario whose compositor exited, it shall report it as failed with that line. | `just regress 588-cleanup-after-the-compositor-exits` |
| REQ-005 | WHEN a run ends normally, the runner shall stop sway as before and say so. | A run of `563-terminal-shortcut-note.sh` (sway, keys and clicks) passing with `sway: stopped` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `script/e2e.sh`; shellcheck; `script/gates.sh --diff` (no `.rs`: `--fast`).
- **P3 Test** — the 588 run, a regress run of it, and a normal sway run.
- **P4 Complete** — CHANGELOG, the e2e docs, ledger, close, archive, commit.
