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

## Phase 1 — Plan (promoted 2026-10-12)
- **The harness's side landed:** TICKET-118, commit `e7f38b1` (pushed; 797 tests and 87 audits
  green); the contract is rustal-harness `docs/SCHEDULE.md`. Two things differ from the queued
  spec: `schedule` is a reserved first segment, so the seen label is `finding.seen`; and the
  runtime stops a run's session once the run ends, so a run's session is `done` in the fleet,
  where the harness keeps it (its envelopes are rebuilt at startup).
- **Recall:** #688 raises a manager record's notice once and lists it until seen, the first read
  quiet; #716's `delegation.told` is the model for a seen label on a session, set with
  `session_label` through `harness::write_tool`, and its F-716 says to work out what the rail needs
  outside the window update (PR-claude-596); `marley_fleet::apply` keeps an ended session as
  `Done` with its labels. The brain had nothing on this seam (consultation
  98ffe106a4ea40c6b118e7b036ac8dcd).
- **Seams re-verified** at `2eafee70a2`: harness.rs `connected` 1064 (the poll loop, `POLL`),
  `change_seats` 296, `manager_posted` 628, `open_in_home` 1508, `open` 1526, `read_tool` 1192,
  `write_tool` 1179; rail.rs `render_harness` 4897, `harness_section` 4944, `harness_row` 7498,
  `harness_entries` 7799, `manager_entries` 7875, `observe_marks` ~990;
  `agent_activity.rs`'s tab (304, 410, 459); `threads_page.rs` `when` 178 (local time with chrono).

### Design
- **A new module `schedule_runs.rs`** (Marley crate) owns the feature, as `delegation.rs` does
  #716's: a global `Schedules` with, per `Slot`, the tasks from the last `schedule_tasks` read,
  when it was read, the finding runs already known (`None` until the slot's first fill), the first
  lines read per run, and the runs seen locally.
- **The poll:** `connected` calls `schedule_runs::poll(slot, cx)` after `follow_thread`; it reads
  `schedule_tasks` through `read_tool` when 15 s have passed. A refusal or an unknown tool sets no
  tasks and logs at debug: a harness without the scheduler shows none.
- **Findings:** `change_seats` calls `schedule_runs::follow(slot, cx)` beside
  `delegation::follow`. It lists the slot's sessions labelled `schedule.outcome: finding` without
  `finding.seen`; a run whose first line is unknown is read with `schedule_run {run}` in a spawned
  task. The first fill after a connection records the findings quietly; a later new one gets one
  desktop notice (tag `marley-finding-<run>`, title "<task> found something", body the first line)
  once its line is read.
- **The rail:** `harness_section` drops sessions labelled `schedule.run` from its rows and adds a
  row per task after them: a clock icon, the name, a line with the schedule in words and "next
  HH:MM" (a weekday's name when not today) or "paused", and a line with the last run: "running",
  or the outcome and its first line, colored (finding Warning, nothing Muted, failed Error). Muted
  while stale. A click opens the task's history.
- **Needs you:** `harness_entries` adds an entry per unseen finding: "<task> found something:
  <first line>", under the harness's name. `InboxTarget::Finding { slot, session, run }` opens the
  run's session (`harness::open_in_home`) and sets `finding.seen` with `session_label` (a request
  UUID); it marks the run seen locally first, so the entry goes at once, with or without writes.
  `observe_marks` adds `Schedules`.
- **The history tab** (`ScheduleView`, in `schedule_runs.rs`): opened in the Home group like Agent
  Activity, one per slot and task. It reads `schedule_runs {task, limit: 32}` on open and again when
  the task's last run changes; a row per run, newest first: the time (`Oct 12, 16:05`), the outcome
  in its color ("running" while it continues), the first line. A click opens the run's session.
- **Guides:** `docs/marley/guide.md` and the in-app guide gain a "Scheduled runs" article; CHANGELOG.

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_workbench/src/schedule_runs.rs` | Marley | new: `Schedules`, `poll`, `follow`, the rows' data, `ScheduleView` |
| `crates/marley_workbench/src/marley_workbench.rs` | Marley | the module, the global's init |
| `crates/marley_workbench/src/harness.rs` | Marley | `connected` polls; `change_seats` follows |
| `crates/marley_workbench/src/rail.rs` | Marley | task rows, run sessions hidden, the Finding entry and its target, the observer |
| `crates/marley_workbench/guide/index.html`, `docs/marley/guide.md` | Marley | the article |
| `script/e2e/719-scheduled-runs.sh` | Marley | the scenario |

No Zed crate changes, so no ledger row.

### Visual check plan
| REQ | The scenario | Shot |
|---|---|---|
| REQ-001, REQ-007 | A real `rh serve` on a scratch root with an actor profile; task `checks` `--every 1` whose actor reports `finding` then `nothing` in turn; task `nightly` `--daily 03:00`, paused. Marley follows it with writes on. | `719-01-rail`: both task rows, "every 1 min · next …", "daily 03:00 · paused"; no run session among the sessions |
| REQ-002, REQ-003 | Wait for the first run's end (`finding`), then the second's (`nothing`). | `719-02-finding`: one Needs you entry "checks found something: …"; the bus log holds one notice |
| REQ-004 | Click the `checks` row. | `719-03-history`: the runs newest first, nothing then finding |
| REQ-005, REQ-006 | Click the finding's Needs you entry. | `719-04-run`: the run's session tab; the entry gone; `rh fleet` shows `finding.seen` |

Not reached: a host harness over SSH (the code is per slot, as #740's is); a run that fails by its
limit (the harness's suite runs it).

### Risks
- A stopped session's tab may read no lines if the harness serves none for an ended session; the
  shot tells, and the tab says so if so.
- An `--every 1` task fires about each minute: the scenario waits up to 150 s per run.

## Phase 2 — Code
- **Built:** `schedule_runs.rs` (new): `Schedules` per slot; `poll` (`schedule_tasks` every 15 s
  through `harness::read_tool`); `follow` from `change_seats` (unseen findings from the session
  labels, `read_line` with `schedule_run`, one `notify` per run that ended after Marley connected);
  `mark_seen` (local at once, then `session_label` `finding.seen`); `schedule_words`,
  `last_run_words`, `outcome_of`; `open_history` and `ScheduleView`. `harness.rs`: the poll in
  `connected`, the follow in `change_seats`. `rail.rs`: run sessions left out of
  `harness_section`, `schedule_row`, `open_schedule`, `finding_entries` with
  `InboxTarget::Finding` (both exhaustive matches and the open handler), `Schedules` observed
  (13 subscriptions). Guides (in-app article `harness-schedule` and its nav line; `guide.md`),
  CHANGELOG, the architecture note.
- **Deviations from the plan:** the notice rule is "the run's session last changed after
  `Schedules` was made", not "the first fill after a connection is quiet": the seed and the first
  `follow` happen in the same update, and the time rule also covers a reconnect without a flag.
  The notice's tag is the run, so two findings of one task are two banners.
- **Review:** no entity is read while it updates (the rows and the history click go through
  `in_group`'s deferral); a finding whose line cannot be read keeps an empty line, so it is not
  read again on each event; the poll only writes `Schedules` when it reads, every 15 s, so the rail
  does not redraw each second. Clippy found six things in the new module (a `map().unwrap_or_else`,
  an `if let` that is a `map_or_else`, an unused `self`, three `&mut` that need only `&`); fixed.
- **Gate:** the first run of `script/gates.sh --diff` was red on gate:7 alone: new RustSec
  advisories against `ammonia` 4.1.4 (RUSTSEC-2021-0157) and `quinn-proto` 0.11.17
  (RUSTSEC-2026-0338, -0340, -0343), published since #716's gate. Not this ticket's code, but a red
  gate writes no receipt; both have semver-compatible fixes, so `cargo update -p … --precise`
  took `ammonia` 4.1.7 and `quinn-proto` 0.11.18 (fixed at the source, no ignore entry), with a
  CHANGELOG Fixed entry. The second run: GATE GREEN [diff], 17 of 17, receipt written.

## Phase 3 — Test
- **Scenario:** `script/e2e/719-scheduled-runs.sh` under `compositor sway`, on a real `rh serve` of
  the harness's `rh` at TICKET-118 (`e7f38b1`), with harness writes on. An actor profile reports
  through its `publish` step; the scenario swaps its script after the first run, so `checks`
  (`--every 1`) reports `finding` and then `nothing`; `nightly` (`--daily 03:00`) is paused. A
  private session bus logs the banners.
- **The runs found:** the first run's sessions failed to open ("launch process proof was not
  confirmed"): the scenario had exported the private bus before `rh serve`, and the runtime
  launches through the user's bus (L-claude-719-start-the-harness-before-a-private-session-bus-001).
  `rh serve` now starts first. That run's runtime outlived its deleted root; the teardown now
  stops it by its pid. The click points estimated from #716's layout hit at once.
- **The final run**, on the gated tree and a rebuilt `marley`; every check passed:
  - **719-01-rail** (REQ-001, REQ-007): under HARNESS, "checks · every 1 min · next 16:34 · no run
    yet" and "nightly · daily 03:00 · paused · no run yet", each with a clock; no session rows.
  - **719-02-finding** (REQ-002): Needs you 1, "checks · Harness: checks found something: two
    test…"; the task's last run "finding: two tests fail in the parser" in the warning color; no
    run session among the sessions; one banner "checks found something|two tests fail in the
    parser" in the bus's log.
  - REQ-003: after the second run (`nothing`), still one banner.
  - **719-03-history** (REQ-004): the `checks` tab in the Default group: "every 1 min · next 16:36";
    "Oct 12, 16:35 · nothing · the checks passed" above "Oct 12, 16:34 · finding · two tests fail in
    the parser"; the task's row reads "nothing: the checks passed".
  - **719-04-run** (REQ-005, REQ-006): the entry opened the run's session tab, "tester-c96a67e5",
    showing "checking the parser"; Needs you is gone; `rh fleet` shows `"finding.seen": "1"`.
- **Not reached:** a host harness over SSH (the code is per slot, as #740's); a run that fails by
  its limit (the harness's suite runs it); a restart with an unseen finding (the label and the
  notice rule are read from the code: an unseen finding from before the connection is listed with
  no notice).

## Phase 4 — Complete
- **Documented:** CHANGELOG (Added; Fixed for the two dependencies); `docs/marley/guide.md` and the
  in-app guide (`harness-schedule` and its nav line); `docs/marley_architecture/marley_workbench.md`
  ("Scheduled runs"); item 6 of `docs/planning/intake/monocode-findings.md` marked built. No Zed
  path changed, so no touchpoint row.
- **Knowledge:** L-claude-719-start-the-harness-before-a-private-session-bus-001,
  AD-claude-719-scheduled-runs-are-the-harnesss-and-a-findings-seen-is-its-sessions-label-001.
  No product bug was found, so no F block.
- **Brain:** consultation 98ffe106a4ea40c6b118e7b036ac8dcd decided; f3f0613f0d4b4680b03396e3b1b7e35d
  and 47c89f588fa145528b01f952192e75ba were the same question asked again, closed with no decision.
- **Closed:** TICKET-719; the pair archived to `completed/`.
