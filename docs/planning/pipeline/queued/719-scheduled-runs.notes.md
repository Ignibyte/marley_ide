# Scheduled runs from the harness, shown in Marley — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-719-scheduled-runs.md
- **Pipeline spec:** 719-scheduled-runs.spec.md

## Phase 1 — Plan (queued 2026-10-12)
- **Request:** MonoCode finding 6; Chad, 2026-10-11: "15.) harness yes"; 2026-10-12: the harness
  unpaused for #716 and #719.
- **The harness request** went to rustal-harness-a6 as MREQ-011 on 2026-10-12. Its proposed shape
  was accepted with item 15 changed (`schedule_tasks` gives the schedule, `paused`, the skipped
  count, the next and last run; each `schedule_runs` item gives the run id, `at_ms`, `outcome`,
  `first_line`, `session`). A second ask: a way for an actor script to report a run's outcome, for
  this ticket's scenario.
- **Seams mapped** by an Explore agent at `a72cbb7c87` (rail.rs `render_harness` 4897,
  `harness_section` 4944, `harness_row` 7498, `manager_entries` 7875; harness.rs `connected`
  1064-1123, `manager_posted` 628, `open` 1526, `read_tool` 1192; `harness_hosts.rs` `Followed` 182;
  `agent_activity.rs` 363-459). The design is written at promotion, once the harness's ticket lands.
- **The harness's answer to the second ask** (2026-10-12): an actor's `publish` step whose `value`
  holds `{"run_report": {"outcome": "finding"|"nothing", "summary": "<one line, at most 512
  bytes>"}}` reports the run's outcome; published with `state: "idle"`, the one step reports and
  ends the send. An actor's prompt is at most 512 bytes and gets no instruction line.
