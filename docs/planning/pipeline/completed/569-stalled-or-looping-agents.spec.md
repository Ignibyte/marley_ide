---
pipeline_id: 774e0a8a-4066-4331-bfac-e9ffe6febe35
ticket: docs/planning/tickets/closed/TICKET-569-stalled-or-looping-agents.md
status: Phase 4 — Complete PASS
title: "Stalled or looping agents: a flag on the rail row, never a stop"
type: feature
slice: prong 2, C1's attention (after #519, #547 and #566); use 4 of the System One layer, on #565
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/completed/565-system-one-layer.spec.md, docs/planning/pipeline/completed/566-stop-kind.spec.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/completed/547-claude-code-events-slice-2.spec.md, docs/planning/pipeline/queued/542-rail-attention-order.spec.md, docs/orca_architecture/01-agents-and-sessions.md]
---

## Title
An agent row gains a flag when its Claude Code has stalled or loops: `looping?` when the turn
repeats one tool line, `stalled?` with a kind (waiting for input, stuck, frozen) when a working
seat has gone quiet with nothing running under it. Facts computed in code decide first: #519's
events, the turn's repeats, the CPU of the process tree under the terminal, and Marley's own
pending approvals. A System One question is asked, through #565's layer, only for the quiet case
the facts leave open, at 1, 2, 4 and 8 minutes of quiet. The flag is a mark and a word on the row,
a label on the seat, and in `act` mode one desktop notification. Marley never stops, interrupts
or types into the agent.

## Scope
### In
- **`marley_agent::stall`** (new, pure): a working seat's facts (`quiet_ms`, the tool in flight,
  `cpu_active`, the repeats, the subagents) and `judge(facts, next_check_ms) -> Verdict`, one of
  `Looping`, `LongTask`, `Open` or `Calm`, by the rule table in the notes (first match wins). The
  repeats: the same `Tool: preview` line three times in a row, or the same line ended by a failure
  twice in a row, read from the turn's tool lines that #566's `TurnFacts` gains here (the last
  twelve, reset by a user's prompt); an edit and a test run in turn are an ordinary cycle, not a
  loop. The process-tree CPU reader, `tree_cpu_in(proc_root, pid, since_ticks)`: the sum of
  `utime` and `stime` over the descendants of Claude Code's pid started after the turn's prompt,
  Claude Code itself and its older children (MCP and language servers) left out, through
  `procfs-core`; `cpu_active` is that sum having grown by more than 2% of a core between two
  samples.
- **`marley_workbench::stall`** (new): a `StallWatch` global, which no view observes, with one timer
  task armed while a seat works: it samples the tree's CPU off the main thread for each working
  seat quiet half a check or more, and at each check asks through `system_one::ask` the set
  `stall_kind/1` (a `choice`: long task, waiting for input no hook reported, stuck on a lock or
  the network, frozen, cannot tell; nouls `repeating` and `progress`). A loop lands at once from
  the rule, logged as a `rules` row through `system_one::record`. The flag is written as seat
  labels the way #566 writes its kind (`flag`, `flag_source`, `flag_confidence`, through an
  `Upsert` stamped with the seat's own `last_event_ms`, so the quiet clock stands), only in
  `suggest` and `act`; the fold clears them at the seat's next event, and the outcome of the call
  that set them is logged (D8).
- **The rail**: a flagged row's state word reads `looping?` or `stalled?` in place of `working`
  (`seat_line`), with a warning mark (`IconName::Warning`, `Color::Warning`) as a child of its own at
  the row's end, whose tooltip carries the facts and, for a stalled flag, the kind and its
  confidence; the activity line keeps the tool in flight.
- **`act`**: one desktop notification per flag episode, `<project>: Claude Code may be stuck` with
  the reason as its body, through `notifications::notify_stall`: #478's focus gate and click,
  and a tag of its own so it neither replaces nor is replaced by Claude Code's own banners.
- **Settings**: the use (`UseSpec { name: "stall_kind", deadline: 2 s }`), its mode in
  `marley.system_one.uses` (`off` by default), its Stall Kind dropdown after Terminal Find on the
  Marley page's System One section; and `marley.stall_check_after_seconds` (60: the checks at 1, 2,
  4 and 8 minutes of quiet; 0 turns the quiet checks off) in `MarleySettingsContent`,
  `default.json` and the Marley page's Agents section.
- `script/e2e/569-stalled-or-looping-agents.sh`, and `fleet-labels` printing the flag's labels.

### Out (explicitly deferred)
- Stopping, interrupting, nudging or typing into the agent, and `session_stop`: never (Chad).
- Escalation to rustal-harness's foreman or manager (M10, #534): the label is the hook for it.
- The termios password check (a running program reading with echo off) as a fact: #551 adds it
  to `crates/terminal/src/pty_info.rs` for plain commands; once it has landed, `waiting for
  input` becomes a fact for agent terminals too, in a follow-up.
- Reading the user's keystrokes as "stepped in": the terminal taking the focus stands in.
- Agents that report no events (Codex, Gemini, OpenCode) and Agent Panel threads (ACP is live).
- #542's attention order (queued): a flagged row takes its place in it when #542 lands.
- Marley's own holds as a fact (a #525 approval, a #571 pause): added when those land.
- Network activity as a fact; Orca's Ctrl+C and Escape inference (#519's Out).

## Reference (§20)
- **Orca** (report 01 §2.3, `src/shared/agent-status-freshness.ts`): a working row older than
  30 minutes becomes `unverifiable` ("No update in 34m", a dashed amber ring) and never `done`.
  #547 shipped that form; this ticket adds the finer, earlier flag, in the same spirit:
  "suspicion, not declaration" (`marley_fleet::AttentionReason::Stale`).
- **rustal-harness** (`docs/SUPERVISION.md`, "Stalls"; `docs/ROADMAP.md` M10 R10-01): a working
  session whose activity count has not changed for its `stall_ms` is stopped and restarted under a
  policy. The harness's stall is a mechanism over declared counters that stops; Marley's flag is a
  display over a terminal that never stops (Chad's rule for use 4), and the seat's label is what
  a foreman reads when M10 lands. The kinds are the Jev note's (use 4, and the table in "How it
  relates to rustal-harness": "long task, waiting on input, stuck, frozen, cannot tell").
- **Warp:** N/A. Its agent notifications carry working, blocked, completed and errored; its docs
  describe no stall detection. Nothing of Warp's was read.
- **Upstream Zed:** none. The Agent Panel's thread status is live over ACP and needs no flag.

### Prior art
- **Behavior maps and reports.** Orca report 01 §2.3 (the decay; interrupt and answer inference,
  which stay Out) and §2.4 (the Smart sort's `unverifiable` class, which #542 takes as "not
  reporting"). The harness's `docs/SUPERVISION.md` (`stall_ms`, `supervision_stalled`, the
  `supervision` label) and `docs/FLEET.md` (a stalled session keeps its state and its
  `last_event_ms` stops advancing). The Jev note's budget row for this loop: no block on the
  agent, "a flag within 30 s", about 100 calls a day "asked at 1, 2, 4 and 8 minutes of quiet",
  up to 4k tokens, about $0.02 a day, and "the 2 s quiet heuristic" when the model is off or down.
  #565's layer (`UseSpec`, `ask`, `StateBuilder`, `Reading`, the `rules` and `replay` providers,
  the day's file and its outcome rows) and #566's pattern for a derived kind as seat labels.
- **Published material.** TypeSafe's docs as the note cites them: a `choice` needs a "none" option
  when the list may not cover the case (docs.typesafe.ai/api), Jev "cannot say 'don't know'
  without an option for it" (docs.typesafe.ai/model-jaggedness/jev-1.13), the 0.6 floor and 0.85
  act line as starting points (docs.typesafe.ai/patterns/confidence-routing). `proc(5)`:
  `/proc/<pid>/stat` fields 14 and 15 (`utime`, `stime`, in clock ticks) and
  `/proc/<pid>/task/<tid>/children`. Claude Code's hooks reference: an Escape fires no hook
  (#519's Out, Orca's recording), which is why the quiet case exists.
- **The code we already ship** (the lines as of promotion, `c03828f765`). `marley_fleet::is_stale`
  (`crates/marley_fleet/src/attention.rs:41`) and `AttentionReason::Stale` (`:19`, "suspicion, not
  declaration"); the reducer's max-join on `last_event_ms` (`reducer.rs:205-207`), which lets a
  label ride an `Upsert` without moving the clock. `claude_events.rs`: `TurnFacts` (`:101-130`),
  `take` (`:349-435`), `tool_starts` (`:437-467`), `note_tool_end` (`:497-524`), `end_turn`
  (`:544-556`, which clears #566's stop labels), `seat_line` (`:239-269`, the `no update` form and
  #566's word) and `seat_activity` (`:271-293`). `stop_kind.rs`'s pattern for a derived kind:
  `StopKindShown`, `row_word`, `labels`. `agent_events.rs`: `next_quiet_change` (`:84-102`),
  `on_frame` (`:122-157`) and `after_fold` (`:162-193`), `land_stop_kind` (`:416-451`, the guarded
  `Upsert` at the seat's own time), `note_outcome` (`:208-232`). The rail: `note_claude_code`, the one
  timer (`rail.rs:412-446`), `note_output` (`:496-508`), `terminal_snapshot` (`:2202-2275`),
  `render_terminal_row` (`:1583-1676`, the bell's slot at `:1610-1628`), `thread_status_mark`'s
  warning icon (`:2628-2633`), the port row's tooltip (`:1855`); `marley_rail`'s `TerminalSnapshot`
  (`:48-66`) and `TerminalRow` (`:264-285`). `Terminal::pid()` (`crates/terminal/src/terminal.rs:3386`,
  the PTY's foreground process group through `tcgetpgrp`, `pty_info.rs:37`); Zed refreshes a
  process with no CPU (`pty_info.rs:90-94`) and `sysinfo` reads one process, not a tree, so Marley
  reads the tree from `/proc` as #521 reads its listeners, with `procfs-core`'s `Stat`.
  `notifications::notify` (`notifications.rs:69-92`, its gate at `:76`, its tag shared by a
  terminal's banners); the Marley page's Agents section (`crates/settings_ui/src/marley_page.rs:49-143`)
  and System One section (`:259-537`); #565's `ask`, `record`, `use_mode`, `detail` and `outcome`
  (`system_one.rs:607-787`). Does a crate we build own the seam? `marley_fleet` owns staleness and
  the labels, #565 the ask; nothing owns the repeats or the tree's CPU.

## UI proof
UI-AFFECTING: a word and a mark on the rail's agent rows, a tooltip, a notification, and two
settings items. `script/e2e/569-stalled-or-looping-agents.sh` (`compositor sway`: the pointer rests
on the mark). Fixtures: 566's stand-in `claude`, named through `MARLEY_CLAUDE`, its steps able to
`spawn` a child that burns CPU (`python3 -c 'while True: pass'`) and to `kill` it; the profile's
settings: `marley.stall_check_after_seconds` at 10 (checks at 10, 20, 40 and 80 seconds), #565's
layer on with the scratch repository listed, the provider and `uses.stall_kind` set per step by
565's `system_one_setting`; `replay.jsonl` with rows keyed `stall_kind/1` and a `match` on the
state; 535's private D-Bus and stand-in notification server. Shots:
- `569-01-long-task`: the mode `act`; a prompt, a `PreToolUse` (Bash `cargo test`), the child
  burning CPU, 25 s of quiet: `working`, no mark, and no `stall_kind` row in the day's file.
- `569-02-looping`: the provider `rules`, the mode `suggest`; the same `Bash: cargo test` ended three
  times in a row: `looping?` and the mark, from the local rule alone.
- `569-03-stalled`: the provider `replay`; a new prompt, the tool ended, the child gone, 15 s of
  quiet: the replay answers `frozen` at 0.91: `stalled?`, the mark.
- `569-04-tooltip`: the pointer on the mark: the facts (quiet, no tool in flight, no CPU) and
  `frozen (0.91)`.
- `569-05-cannot-tell`: the next prompt and the same quiet, the replay answering `cannot_tell`: no
  mark; the day's file has the row.
- `569-06-act`: the mode `act`, the workspace switched away, the same quiet with `frozen` again:
  the mark, and one banner in the stand-in server's log titled `repo: Claude Code may be stuck`.
- `569-07-cleared`: the next event: the mark gone; the outcome line names the event.
- `569-08-off`: the mode `off`, the same quiet: no mark, no row.
- `569-09-settings`: the Marley page: Stall Check After Seconds in Agents, Stall Kind in System One.
Checks: `mcp_agent fleet-labels` while flagged shows `flag 'stalled:frozen'`, `flag_source
'model'`; the day's file's rows name `stall_kind/1`, carry the masked state (the prompt cut, no
transcript path), and each flag's call has its outcome line; the stand-in's log shows nothing typed
into it but the scenario's own Enters.

## Locked-In Decisions
- D1 — "local first and then jev second" (Chad, 2026-09-26). The rule table decides from facts:
  a tool in flight with CPU under the tree is a long task; repeats are a loop; only a quiet seat
  with no tool and no CPU is open, and only then is the model asked. With the `rules` provider,
  the project unlisted (`Refused`), the provider unreachable (`Unavailable`) or the reading
  `NoSignal`, the facts alone stand: `looping?` from the repeats, nothing for the open case until
  #547's thirty minutes.
- D2 — Off by default, with its own switch and its own mode, and no provider named: "we need
  probably every aspect of this configurable and turned off / on where the system will use or
  wont use it. Otherwise this becomes a jev required system" (Chad). The switch is the use's mode
  in `marley.system_one.uses` (#565's shape), and any provider the layer has serves it; `rules`
  runs the facts with no model at all.
- D3 — Never a stop (Chad; the Jev note's use 4: "a flag on the row, later an escalation to
  rustal-harness's foreman; never a stop"). The flag is a mark, a word, a label and at most a
  notification. No path leads from a flag to `Terminal::input`, an interrupt or `session_stop`.
- D4 — The asks come at 1, 2, 4 and 8 minutes of quiet (the note's budget), set by one number,
  `stall_check_after_seconds`, so a scenario can wait seconds; none while a tool in flight burns
  CPU in the turn's processes, and none when the masked state hashes as the last one asked (the
  layer's dedupe). Marley's own holds join as a fact when #525 and #571 land.
- D5 — The state is facts computed in code and the seat's own short texts as the rail shows them
  (the prompt and the tool line, cut to 300, through #516's redactor as the layer's `Mask`), plus
  the terminal's last five non-empty lines as text; only for a project on #565's allow list;
  `Detail::Facts` for a metadata-only project. Never files or transcripts (Chad's answer,
  2026-09-26).
- D6 — The choice carries `cannot_tell`, and `cannot_tell`, a confidence under the floor (0.5,
  #565's starting point; refitted from labels), `NoSignal` or `Unavailable` flags nothing: a
  suspicion earns a `?`, an abstention earns silence.
- D7 — The flag rides on the seat as labels, as #566's kind does: `flag`, `flag_source`,
  `flag_confidence`, written through an `Upsert` stamped with the seat's own `last_event_ms`
  (the reducer's max-join keeps the clock) and guarded by the session, the prompt id and the stop
  count, so the rail, `fleet_snapshot` and a later foreman read one thing. The fold clears them at
  the seat's next event, whatever it is: any event is the agent moving again.
- D8 — Modes as #565 defines them: `off` makes no call and writes no label; `shadow` logs the
  rules' and the model's readings and shows nothing, since agents read every label (promotion:
  #566's lesson); `suggest` shows the flag; `act` adds the banner, once per episode, behind #478's
  gate. The outcome line records what ended the flag and
  whether the terminal took the focus in the active window meanwhile (Chad stepped in): the
  labels the golden report is fitted on.
- D9 — The state word slot is shared with #547's `no update` and #566's kinds: a flag wins over
  `no update` (it is finer), and #566's kinds belong to an idle seat, never a working one. A
  flagged row takes #542's not-reporting class when #542 lands.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the use is in `suggest` or `act`, WHILE a working seat's turn has ended the same tool line three times in a row, or the same line has failed twice in a row, the system shall mark its row `looping?` from the local rule, with no model called. | Shot `569-02-looping`; the day's file: a `rules` row |
| REQ-002 | WHILE a working seat has a tool in flight and the turn's processes consumed CPU in the last sample, the system shall show no flag and ask nothing, however long it is quiet. | Shot `569-01-long-task`; the day's file has no row |
| REQ-003 | WHEN a working seat has been quiet for a configured interval with no tool in flight and no CPU under it, WHERE the use is not `off` and the project listed, the system shall ask the `stall_kind` set once for that interval with the masked state. | The day's file: one row per interval, the set named, the prompt cut, no transcript path |
| REQ-004 | WHERE the mode is `suggest` or `act`, WHEN the reading names a kind at or above the floor, the row shall read `stalled?` with a warning mark. | Shot `569-03-stalled` |
| REQ-005 | WHEN the pointer rests on the mark, the system shall show the facts and the reading with its confidence. | Shot `569-04-tooltip` |
| REQ-006 | WHEN the reading is `cannot_tell`, under the floor, no signal, refused or unavailable, the row shall show no flag. | Shot `569-05-cannot-tell` |
| REQ-007 | WHERE the mode is `act`, WHEN a row is flagged and its terminal is not in front, the system shall post one desktop notification for the episode. | Shot `569-06-act`; the stand-in notification server's log: one banner |
| REQ-008 | WHILE a row is flagged, the seat shall carry `flag`, `flag_source` and `flag_confidence`, and `fleet_snapshot` shall list them. | The run log: `mcp_agent fleet-labels` |
| REQ-009 | WHEN the seat's next event arrives, the flag shall leave and the call's outcome shall be logged. | Shot `569-07-cleared`; the outcome line |
| REQ-010 | WHERE the mode is `off`, the system shall show no flag and make no call. | Shot `569-08-off`; the day's file |
| REQ-011 | The system shall never write to the agent's terminal or stop it on a flag. | Review of the diff (no path from a flag to `Terminal::input` or a stop); the scenario's terminal shows only its own Enters |
| REQ-012 | WHEN the Marley settings page opens, its System One section shall show the use's mode and its Agents section `Stall Check After Seconds`. | Shot `569-09-settings` |
| REQ-013 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the e2e plan in the notes. At promotion: #565, #566 and
  #567 shipped (the ask API, the replay file, `TurnFacts`, the label pattern, `UseSpec` by value);
  `brain_ask`; the seams re-read through an Explore agent (the notes' Promotion entry).
- **P2 Code:** the ledger rows 55, 58 and 61 first; the setting, the dropdown and the defaults;
  `marley_agent::stall` and `TurnFacts.tool_lines`; the `stall_kind/1` set; `marley_workbench::stall`,
  the watch, the samples, the asks, the labels and the outcomes; `notify_stall`; the row, the mark
  and the tooltip. fmt and clippy clean; a review of the diff against REQ-011 first.
- **P3 Test:** write and run the scenario and read every shot; rerun 519, 547, 565, 566 and 515;
  the golden set; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_agent.md`, `marley_workbench.md`,
  `marley_rail.md` and `marley_system_one.md`; the plan's S1 row; the ledger capture; close the
  ticket, archive, commit.
