---
pipeline_id: ad397fcf-e64c-49b5-a8b2-83ca5a46a4fb
ticket: docs/planning/tickets/open/TICKET-719-scheduled-runs.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Scheduled runs from the harness, shown in Marley
type: feature
slice: agents together (docs/planning/intake/monocode-findings.md item 6), the harness's scheduler (rustal-harness MREQ-011)
references:
  - docs/planning/pipeline/completed/688-a-notice-for-the-managers-reports.spec.md
  - docs/planning/pipeline/completed/740-one-harness-per-host.spec.md
  - docs/planning/pipeline/completed/742-a-thread-on-a-remote-seat.spec.md
---

## Title
Recurring prompts that rustal-harness runs on a schedule, shown in Marley: each followed harness's
tasks in the rail with their next run and last result, each task's history, each run's session,
and a Needs you entry with a desktop notice only when a run found something. Chad, 2026-10-11:
"15.) harness yes"; 2026-10-12: the harness unpaused for it.

## Scope
### In
- **The harness's side** (MREQ-011, accepted 2026-10-12 with item 15 changed): `rh schedule
  add|list|pause|resume|remove`; a task's name, prompt, profile, folder and schedule
  (`{"every": N}` minutes, `{"daily": "HH:MM"}`, `{"weekdays": "HH:MM"}`); runs that never
  overlap, a one-minute floor, a 10-minute grace for fixed times, run ids from the task and the time,
  a 30-minute limit; outcomes `finding`, `nothing`, `failed` (a run reports with `run_report`);
  the read tools `schedule_tasks`, `schedule_runs {task, limit?, before?}`, `schedule_run {run}`;
  a run's session labelled `schedule.task`, `schedule.run` and, at its end, `schedule.outcome`.
- **The rail:** under each followed harness's sessions, its scheduled tasks: the name, the
  schedule in words ("every 60 min", "daily 09:00", "weekdays 09:00"), the next run's time and the
  last run's outcome, "paused" for a paused task. A task's row opens its history.
- **The history tab:** a task's runs, newest first: the time, the outcome in its color and the first
  line; a click on a run opens its session (#534's tab).
- **Findings:** a run session whose `schedule.outcome` is `finding` raises one Needs you entry
  ("<task> found something: <first line>") and one desktop notice. Opening the entry opens the
  run's session and sets its label `schedule.seen`, so the entry does not come back after a
  restart. `nothing` and `failed` raise nothing.
- The tasks are read every 15 seconds while connected; a harness without the tools shows none.
- `docs/marley/guide.md` and the in-app guide.

### Out (explicitly deferred)
- Adding, pausing or removing a task from Marley (the harness's CLI does it).
- A run's own thread view for a seat profile (#742 opens one for the task's profile by hand).

## Reference (§20)
N/A — Marley-specific: it draws what rustal-harness serves. The reference for the scheduler's rules
is T3 Code's design (`docs/t3code_architecture/06-orchestration-mcp-and-automations.md`, item 9),
which the harness took.

### Prior art
- **The code we ship:** the poll loop `harness::connected` (harness.rs:1064-1123), the place a
  `schedule_tasks` read goes beside `follow_thread`; the manager records' notices and Needs you
  entries (#688: `manager_posted` 628, `manager_entries` in rail.rs 7875); `harness_section`
  (rail.rs:4944) and `harness_row` (7498); `harness::open` (1526); `agent_activity.rs`'s list tab
  (363, 410, 459) for the history; `harness::read_tool` (#716) for the history's reads and
  `write_tool` for `session_label`.
- **Behavior maps:** T3 Code's automations (report 06, item 9).
- **Published material:** MCP tools.

## UI proof
`script/e2e/719-scheduled-runs.sh`, under `compositor sway`, on a real `rh serve` with the
harness's scheduler: an actor profile and a task `--every 1` whose runs report `finding` and
`nothing` in turn: the actor's `publish` step with `{"run_report": {"outcome", "summary"}}`, which
the harness added for this test.

Shots:
- `719-01-rail`: the Harness section lists the task, "every 1 min", its next run, "paused" for a
  second, paused task.
- `719-02-finding`: after a run that found something: Needs you lists it; the desktop notice is in
  the private bus's log.
- `719-03-history`: the task's history, newest first, a finding and a nothing.
- `719-04-run`: a run's session tab from the history.

## Locked-In Decisions
- **D1:** the harness schedules, runs and records (Chad, 2026-10-11); Marley builds no timer.
- **D2:** Marley polls `schedule_tasks` every 15 seconds on its connection, as it reads the
  manager's thread, and reads findings from the run sessions' labels on the fleet stream.
- **D3:** a finding's "seen" lives on the run's session as the label `schedule.seen`, so nothing is
  stored in Marley and a restart does not raise it again.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a followed harness has scheduled tasks, the rail's Harness section shall list each with its schedule, next run and last outcome, and "paused" for a paused one. | Shot 719-01 |
| REQ-002 | WHEN a run ends with `finding`, the system shall list one Needs you entry and show one desktop notice. | Shot 719-02 and the bus's log |
| REQ-003 | WHEN a run ends with `nothing` or `failed`, the system shall raise nothing. | The bus's log |
| REQ-004 | WHEN the user opens a task, the system shall show its runs newest first with time, outcome and first line. | Shot 719-03 |
| REQ-005 | WHEN the user opens a run, the system shall open its session. | Shot 719-04 |
| REQ-006 | WHEN the user opens a finding's entry, the system shall set the run's `schedule.seen` label, so the entry does not return. | Review of the diff; the label in the fleet |

## Phase Plan
- **P1 Plan** — this spec; at promotion the design in the notes, once the harness's ticket has landed.
- **P2 Code** — `harness.rs` (the tasks per slot, the poll), `harness_hosts.rs` (the field),
  `rail.rs` (the rows, the entry), a new `schedule_tab.rs` (the history), `notifications`; the
  guides; `script/gates.sh --diff` green.
- **P3 Test** — the visual check.
- **P4 Complete** — CHANGELOG, architecture docs, knowledge, close, archive, commit.
