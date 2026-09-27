---
pipeline_id: 774e0a8a-4066-4331-bfac-e9ffe6febe35
ticket: docs/planning/tickets/open/TICKET-569-stalled-or-looping-agents.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Stalled or looping agents: a flag on the rail row, never a stop"
type: feature
slice: prong 2, C1's attention (after #519, #547 and #566); use 4 of the System One layer, on #565
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/completed/565-system-one-layer.spec.md, docs/planning/pipeline/queued/566-stop-kind.spec.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/active/547-claude-code-events-slice-2.spec.md, docs/planning/pipeline/queued/542-rail-attention-order.spec.md, docs/orca_architecture/01-agents-and-sessions.md]
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
- **`marley_agent::stall`** (new, pure): the facts of a seat (`quiet_ms`, `tool_in_flight` and
  its tool, `cpu_active`, `repeats`, `held_by_marley`, `subagents`) and `judge(facts) ->
  Verdict`, one of `LongTask`, `Looping`, `Open` or `Calm`, by the rule table in the notes (first
  match wins). The repeats rule: the same `Tool: preview` line three times in one turn, or the
  same tool failing twice in a row, read from the turn's tool lines that #566's `TurnFacts` gains
  here (the last twelve, reset by a user's prompt). The process-tree CPU reader,
  `tree_cpu_in(proc_root, pid)`: the sum of `utime` and `stime` from `/proc/<pid>/stat` over the
  pid and its descendants (`/proc/<pid>/task/<tid>/children`), an unreadable entry skipped;
  `cpu_active` is the sum having grown since the last sample.
- **`marley_workbench::agent_events`**: a sampler that, while a working seat has been quiet 30
  seconds or more, reads the tree's CPU every 10 seconds off the main thread against
  `Terminal::pid()`; the asks at the configured quiet intervals through `system_one::ask` with
  the set `stall_kind/1` (a `choice`: long task, waiting for input no hook reported, stuck on a
  lock or the network, frozen, cannot tell; nouls `repeating` and `progress`), the use's own
  verdict handed with each ask; the flag written as seat labels the way #566 writes its kind
  (`flag`, `flag_source`, `flag_confidence`, through an `Upsert` stamped with the seat's own
  `last_event_ms`, so the quiet clock stands); the outcome line for each call (D8).
- **The rail**: a flagged row's state word reads `looping?` or `stalled?` in place of `working`
  (`seat_line`), with a warning mark at the row's end (`IconName::Warning`, `Color::Warning`)
  whose tooltip carries the facts and, for a stalled flag, the kind and its confidence; the
  activity line keeps the tool in flight. A flagged row sorts in #542's not-reporting class.
- **`act`**: one desktop notification per flag episode, `<project>: Claude Code may be stuck`
  with the reason as its body, through `notifications::notify` (the focus gate of #478, the
  click that shows the terminal).
- **Settings**: the use registered with #565 (`UseSpec { name: "stall_kind", deadline: 2 s }`),
  its mode in `marley.system_one.uses` (`off` by default), its dropdown on the Marley page's
  System One section; and `marley.stall_check_seconds` (`[60, 120, 240, 480]`) in
  `MarleySettingsContent`, `default.json` and the Marley page's Agents section.
- `script/e2e/569-stalled-or-looping-agents.sh`.

### Out (explicitly deferred)
- Stopping, interrupting, nudging or typing into the agent, and `session_stop`: never (Chad).
- Escalation to rustal-harness's foreman or manager (M10, #534): the label is the hook for it.
- The termios password check (a running program reading with echo off) as a fact: #551 adds it
  to `crates/terminal/src/pty_info.rs` for plain commands; once it has landed, `waiting for
  input` becomes a fact for agent terminals too, in a follow-up.
- Reading the user's keystrokes as "stepped in": the terminal taking the focus stands in.
- Agents that report no events (Codex, Gemini, OpenCode) and Agent Panel threads (ACP is live).
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
- **The code we already ship.** `marley_fleet::is_stale` (`crates/marley_fleet/src/attention.rs:41`)
  and `AttentionReason::Stale` (`:19`, "suspicion, not declaration"); the reducer's max-join on
  `last_event_ms` (`reducer.rs:205-207`), which lets a label ride an `Upsert` without moving the
  clock; `claude_events::seat_line` (`crates/marley_agent/src/claude_events.rs:189`, the `no
  update` form), the `lead_tool:` labels (`:47`, `:343-365`) and `fold`'s per-event handling
  (`:278-341`); `agent_events::next_quiet_change` (`crates/marley_workbench/src/agent_events.rs:46-64`)
  and `on_frame` (`:83-106`, which has the decoded event before folding it); the rail's minute
  timer (`rail.rs:342-376`) and quiet timers (`:426-438`), `render_terminal_row` (`:1319`) and
  the warning icon `thread_status_mark` draws (`:2044-2049`), `terminal_snapshot` (`:1717`);
  `Terminal::pid()` (`crates/terminal/src/terminal.rs:3277`, the PTY's foreground process group
  through `tcgetpgrp`, `pty_info.rs:37`). Zed refreshes a process with
  `ProcessRefreshKind::nothing().with_cmd().with_cwd().with_exe()` and no CPU
  (`pty_info.rs:90-94`), and `sysinfo` reads one process, not a tree, so Marley reads the tree from
  `/proc` as #521's design reads `/proc/net/tcp`. `notifications::notify` (`notifications.rs:59-82`)
  and its focus gate (`:66`); the Marley page's item pattern (`crates/settings_ui/src/marley_page.rs:45-66`);
  `TerminalSnapshot` and `TerminalRow` (`crates/marley_rail/src/marley_rail.rs:46-62`,
  `:216-235`). Does a crate we build own the seam? `marley_fleet` owns staleness and the labels,
  #565 the ask; nothing owns the repeats or the tree's CPU.

## UI proof
UI-AFFECTING: a word and a mark on the rail's agent rows, a tooltip, a notification.
`script/e2e/569-stalled-or-looping-agents.sh` (`compositor sway`: the pointer rests on the mark).
Fixtures: #519's stand-in `claude` with this scenario's steps; at one step it starts a child that
burns CPU (`python3 -c 'while True: pass'`) and at a later step ends it; the profile's settings:
`marley.stall_check_seconds` at `[10, 20, 40, 80]`, #565's layer enabled on `replay` with the
scratch repository listed and `uses.stall_kind` rewritten per step by #565's
`system_one_setting` helper; the replay file `$E2E_PROFILE/system_one/replay.jsonl` with rows
keyed by set and a `match` on the state; `busctl --user monitor org.freedesktop.Notifications`
into a log (L-claude-478). Shots:
- `569-01-long-task`: the mode `act`; a prompt, a `PreToolUse` (Bash `cargo test`), the child
  burning CPU, 45 s of quiet: `working`, no mark, and no row in the day's file.
- `569-02-looping`: the provider `rules`; three `PreToolUse` and `PostToolUse` pairs of the same
  `Bash: cargo test` in one turn: `looping?` and the mark, from the local rule alone.
- `569-03-stalled`: the provider `replay`, the mode `suggest`; a new prompt, the tool ended, the
  child gone, 15 s of quiet: the replay answers `frozen` at 0.91: `stalled?`, the mark.
- `569-04-tooltip`: the pointer on the mark: the facts (quiet 15 s, no tool in flight, no CPU) and
  `frozen (0.91)`.
- `569-05-cannot-tell`: the next prompt and the same quiet, the replay answering `cannot_tell`:
  no mark; the day's file has the row.
- `569-06-act-notification`: the mode `act`, the same quiet with `frozen` again: the mark, and one
  `Notify` in the bus log titled `repo: Claude Code may be stuck`.
- `569-07-cleared`: Enter (the next event): the mark gone; the outcome line names the event.
- `569-08-off`: the mode `off`, the same quiet: no mark, no row.
Checks: `mcp_agent fleet` while flagged shows the labels `flag: stalled:frozen`, `flag_source:
model`; the day's file's rows name the set `stall_kind/1`, carry the masked state (the prompt cut,
no transcript path), and each has its outcome line; the scenario's terminal shows nothing typed
into the stand-in but the scenario's own Enters; in `569-03` the Decisions view lists the call.

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
- D4 — The asks come at 1, 2, 4 and 8 minutes of quiet (the note's budget), configurable so a
  scenario can wait seconds; none while a tool is in flight with CPU under the tree, none while
  Marley itself holds the agent (a #525 approval or a #571 pause pending for that terminal), and
  none when the masked state hashes as the last one asked (the layer's dedupe).
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
  (the reducer's max-join keeps the clock), so the rail, `fleet_snapshot` and a later foreman
  read one thing. The seat's next event, a user's prompt or its end clears them.
- D8 — Modes as #565 defines them: `off` makes no call and writes no label; `shadow` shows the
  local `looping?` and logs the model's reading; `suggest` shows the model's flag; `act` adds the
  banner, once per episode, behind #478's gate. The outcome line records what ended the flag and
  whether the terminal took the focus in the active window meanwhile (Chad stepped in): the
  labels the golden report is fitted on.
- D9 — A flagged row sorts in #542's not-reporting class, with `no update in N m`. The state word
  slot is shared with #547's `no update` and #566's kinds: a flag wins over `no update` (it is
  finer), and #566's kinds belong to an idle seat, never a working one.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the use is not `off` and the provider is `rules`, WHILE a working seat's turn has run the same tool line three times, or the same tool has failed twice in a row, the system shall mark its row `looping?` from the local rule, with no model called. | Shot `569-02-looping`; the day's file: a `rules` row |
| REQ-002 | WHILE a working seat has a tool in flight and its process tree consumed CPU in the last sample, the system shall show no flag and ask nothing, however long it is quiet. | Shot `569-01-long-task`; the day's file has no row |
| REQ-003 | WHEN a working seat has been quiet for a configured interval with no tool in flight and no CPU under it, WHERE the use is not `off` and the project listed, the system shall ask the `stall_kind` set once for that interval with the masked state. | The day's file: one row per interval, the set named, the prompt cut, no transcript path |
| REQ-004 | WHERE the mode is `suggest` or `act`, WHEN the reading names a kind at or above the floor, the row shall read `stalled?` with a warning mark. | Shot `569-03-stalled` |
| REQ-005 | WHEN the pointer rests on the mark, the system shall show the facts and the reading with its confidence. | Shot `569-04-tooltip` |
| REQ-006 | WHEN the reading is `cannot_tell`, under the floor, no signal, refused or unavailable, the row shall show no flag. | Shot `569-05-cannot-tell` |
| REQ-007 | WHERE the mode is `act`, WHEN a row is flagged and its terminal is not in front, the system shall post one desktop notification for the episode. | Shot `569-06-act-notification`; the bus log: one `Notify` |
| REQ-008 | WHILE a row is flagged, the seat shall carry `flag`, `flag_source` and `flag_confidence`, and `fleet_snapshot` shall list them. | The run log: `mcp_agent fleet` |
| REQ-009 | WHEN the seat's next event arrives, the flag shall leave and the call's outcome shall be logged. | Shot `569-07-cleared`; the outcome line |
| REQ-010 | WHERE the mode is `off`, the system shall show no flag and make no call. | Shot `569-08-off`; the day's file |
| REQ-011 | The system shall never write to the agent's terminal or stop it on a flag. | Review of the diff (no path from a flag to `Terminal::input` or a stop); the scenario's terminal shows only its own Enters |
| REQ-012 | WHEN the Marley settings page opens, its System One section shall show the use's mode and its Agents section `Stall Check Seconds`. | The 515 scenario's page shot |
| REQ-013 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the e2e plan in the notes. At promotion: #565, #547 and
  #566 have shipped (the ask API, the replay file, `TurnFacts`, the label pattern); `brain_ask`.
- **P2 Code:** the ledger rows first (`crates/settings_content/src/marley.rs`,
  `crates/settings_ui/src/marley_page.rs`, `assets/settings/default.json`, each row widened for
  `stall_check_seconds` and the use's dropdown); `marley_agent::stall` and the turn's tool lines
  in `TurnFacts`; the `stall_kind/1` set in `marley_system_one::question`; the sampler, the asks,
  the labels and the outcomes in `agent_events`; the row, the mark, the tooltip and the class;
  the banner. fmt and clippy clean; a review of the diff against REQ-011 first.
- **P3 Test:** write and run the scenario and read every shot; `just regress` (519 and 547 sit in
  the set); `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_agent.md`, `marley_workbench.md`,
  `marley_rail.md` and `marley_system_one.md`; the plan's C1 status; ledger capture; close the
  ticket, archive, commit.
