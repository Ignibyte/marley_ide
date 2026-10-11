# TICKET-719 — Scheduled runs

- **Ticket:** LOCAL #719 (feature; Marley's part small to medium, after the harness's)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** none yet. The scheduler is rustal-harness's; Marley's spec is written when the
  harness has said what it serves.
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 6; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"; Chad, 2026-10-11: "15.) harness yes"
- **Status:** open (deliberate: waits on rustal-harness)

## Summary
Recurring prompts (hourly, daily, weekdays) that run an agent in a background session, keep every
run, and raise something only when a run found something. Chad decided on 2026-10-11 that the
scheduler and the runs belong to rustal-harness, as agent hosting does since 2026-10-02: the harness
holds the tasks, fires them, runs each in a session it supervises, and keeps the run records. Marley
builds no scheduler and no timer.

Marley's part is to show the runs and their results:
- the rail's Harness section lists each followed harness's scheduled tasks (name, schedule, next
  run, last result), for every harness #740 follows;
- a run opens as its session (#534's session tab) or, for a seat, as its thread (#742);
- a run whose result is a finding raises a Needs you entry and a desktop notice, as the manager's
  records do (#688), and nothing otherwise;
- a task's history lists its runs, newest first, each with its time, outcome and first line.

T3 Code's design (report 06, item 9) is the reference the harness may take: no overlap with the
task's own last run, a one-minute floor, a grace window for fixed times, ids made from the task and
the time, and a delivery log with the prompt it sent.

**Due:** a request to rustal-harness, through its `docs/planning/MARLEY_REQUESTS.md`, for scheduled
tasks and runs read over `rh mcp` (a list with next and last run, a run's record, and a fleet event
when a run ends). This ticket's spec waits on its answer.

## Acceptance
On a followed harness with a daily task, the rail lists the task with its next run; a run that found
something raises one Needs you entry that opens the run, and the task's history lists every run.
