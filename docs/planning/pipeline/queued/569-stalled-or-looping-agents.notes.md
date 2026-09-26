# Stalled or looping agents: a flag on the rail row, never a stop — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-569-stalled-or-looping-agents.md
- **Pipeline spec:** 569-stalled-or-looping-agents.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-26, approving the seven ranked uses of the Jev note
  (`docs/planning/design-notes/jev-system-one-2026-09-25.md`). Use 4: "a choice (long task,
  waiting for input no hook reported, stuck on a lock or the network, frozen, cannot tell); nouls
  for repeating and progress; a flag on the row, later an escalation to rustal-harness's foreman;
  never a stop; waits on #519; M". Drafted by the second spec drafter beside #565 (the layer) and
  #566 to #568 (uses 1 to 3), and aligned to their pairs once they were on disk.
- **Classification / tier:** feature, prong 2 (C1's attention). Marley crates, plus the three
  additive Zed paths a Marley setting takes (`stall_check_seconds`) and the page's dropdown for
  the use's mode. Size M.
- **Recall (§18.3):**
  - AD-claude-519 (hook events ride in band into `marley_fleet`; the seat is the events' fold):
    the flag rides as labels the way #566's kind does, stamped so the quiet clock stands (D7).
  - L-claude-477 (a quiet foreground process is seen only after output): the tree's CPU is read
    from `/proc` on a timer, not from the terminal's process info, which refreshes on output.
  - L-claude-478 (Omarchy's notifications go to Quickshell; `busctl --user monitor` sees a
    `Notify`): the scenario's proof of REQ-007.
  - AD-claude-478 (a notification only for a terminal not in front): the same gate here.
  - PR-claude-474 (a hook frame is output until its nonce says otherwise; showing a field needs
    no check, acting on one does): a flag shows and never acts, so an unauthenticated frame can at
    worst mark a row.
  - #547's notes: the minute timer and `next_quiet_change`, which this ticket's intervals extend;
    `end` leaves a failed seat alone, so a failed seat is never asked about.
  - #565's D1, D2 and D5 (the verdict handed with each ask; `rules` answers it; a set is
    compiled in, named and versioned) and #566's D3 (a derived kind as seat labels).
  - Brain: not consulted in this drafting session; promotion asks.
- **Discovery** (at `51bfe04034` with #547's working-tree changes; promotion re-verifies):
  - `crates/marley_agent/src/claude_events.rs`: `HookEvent` (55-90, `tool`, `preview`,
    `tool_use_id`, `is_interrupt`), `fold` (141-159), `seat_status` (172), `seat_line` (189-209,
    the `no update` form), `seat_activity` (214-224), `Moving::take` (278-341), `tool_starts`
    (343-365), `tool_ends` (367-393), `end_turn` (414-422). #566 (queued) adds `TurnFacts` here,
    reset by a user's prompt, with tools by name and counts; this ticket adds the tool lines.
  - `crates/marley_workbench/src/agent_events.rs`: `AgentEvents { snapshot }` (21-23), `seat`
    (30-34), `next_quiet_change` (46-64), `on_frame` (83-106), `end` (112-133), `forget` (136-145).
  - `crates/marley_workbench/src/rail.rs`: `refresh` (309-336), `note_claude_code` (342-376, the
    minute timer), `note_output` (426-438), `render_terminal_row` (1319-1380: the bell dot at the
    row's end, 1347-1364), `terminal_snapshot` (1717-1784), `no_update_after_ms` (1788), `row_card`
    (1982-2024), `thread_status_mark` (2035-2060).
  - `crates/marley_fleet/src/session.rs`: `Session.labels` (66-70), `last_event_ms` (73);
    `reducer.rs`: `Upsert` (24-40), `apply` (147-208, the max-join at 205-207); `attention.rs`:
    `is_stale` (41), `reason_for` (46-56).
  - `crates/marley_workbench/src/notifications.rs`: `notify` (59-82), the gate (66),
    `show_sender` (86-100).
  - `crates/terminal/src/terminal.rs`: `pid` (3277), `foreground_process_command_name` (3082),
    `last_n_non_empty_lines` (2635); `pty_info.rs`: `ProcessIdGetter::pid` through `tcgetpgrp`
    (37), the refresh kind without CPU (90-94).
  - `crates/settings_content/src/marley.rs` (10-29), `crates/settings_ui/src/marley_page.rs`
    (`agents_section` 42-91), `assets/settings/default.json` (1664-1674),
    `crates/marley_workbench/src/marley_workbench.rs` (`MarleySettings` 168-198).
  - #565 (queued): `UseSpec { name, set, deadline }`, `system_one::ask(use, state, verdict, cx)
    -> Task<Reading>`, `StateBuilder::new(project, detail).fact(..).text(..)`, `Detail::{Full,
    Facts}`, `Reading::{Off, Rules, Model, NoSignal, Unavailable, Refused, Shadow}`, the day's
    file `<data>/system_one/calls-<date>.jsonl` with `CallRow` and `OutcomeRow`, the replay file
    `<data>/system_one/replay.jsonl` (rows `{set, match?, state_hash?, answers, repeat?}`), the
    scenario helper `system_one_setting <key> <json>`, and `marley.system_one.uses`.
  - rustal-harness `docs/SUPERVISION.md` ("Stalls": `stall_ms` from 1,000 to 3,600,000 ms; the
    `supervision_stalled` event; the labels `supervision`, `stall_ms`) and `docs/FLEET.md` (a
    stalled session keeps its state; heartbeats at most every 10 s while activity advances).
- **Decisions:** D1 to D9 in the spec.

### Design
- **The rule table** (`marley_agent::stall::judge`, first match wins):

  | # | When | Verdict |
  |---|---|---|
  | 1 | `held_by_marley` (a #525 approval or a #571 pause pending for the terminal) | `LongTask` |
  | 2 | `repeats`: one tool line three times this turn, or the same tool failed twice in a row | `Looping` |
  | 3 | a tool in flight and `cpu_active` | `LongTask` |
  | 4 | `quiet_ms` at or past the next configured interval, no tool in flight, no `cpu_active` | `Open` |
  | 5 | a tool in flight, no `cpu_active`, `quiet_ms` past the interval | `Open` |
  | 6 | otherwise | `Calm` |

  Row 5 exists because a tool waiting on the network or a lock burns no CPU; the model's choice
  has "stuck on a lock or the network" for it, and the tool line is in the state.
- **The facts.** `quiet_ms` from `Session.last_event_ms`; the tool in flight from the `lead_tool:`
  labels (the newest); `repeats` from `TurnFacts.tool_lines` (the last twelve `Tool: preview`
  lines since the last user prompt, each `PostToolUseFailure` marked); `cpu_active` from the
  sampler; `held_by_marley` from #525's and #571's pending state, when they exist (a function
  returning `false` until then); `subagents` from the label.
- **The sampler.** `AgentEvents` keeps `tree_cpu: HashMap<seat, (ticks, Instant)>`. The rail's
  minute timer is generalized: `next_quiet_change` also returns the next configured interval and,
  while any working seat has been quiet 30 s or more, a 10 s tick; the tick runs `tree_cpu_in`
  on the background executor (`cx.background_spawn(futures::future::lazy(...))`, the pattern
  `autosuggest::read_history` uses) for each such seat's `Terminal::pid()`, then updates the map.
- **The ask.** At an interval, `Open` builds the state with `StateBuilder::new(project,
  detail)`: facts `quiet` (in words, "3 minutes"), `tool in flight`, `cpu active`, `repeats`,
  `subagents`, `permission mode`; texts `prompt` and `tool` (the labels, cut to 300) and `screen`
  (the last five non-empty lines), each through the layer's `Mask` (#516's redactor), dropped
  under `Detail::Facts`. The use's verdict (`Open`) goes with the ask; `system_one::ask` answers a
  `Reading`; only `Model` with a kind other than `cannot_tell` at or above the floor flags. The
  call's id is kept with the seat for the outcome line.
- **The flag.** Labels `flag` (`looping`, or `stalled:<kind>`), `flag_source` (`rules` or
  `model`), `flag_confidence`, written through an `Upsert` of the seat's labels whose `ts_ms` is
  the seat's `last_event_ms`, so `apply`'s max-join leaves the clock alone; cleared the same way at
  the seat's next event (`on_frame`), a user prompt, `end` or `forget`; clearing writes the
  outcome line: `next_event`, `new_prompt`, `ended`, plus `focused: true` when the terminal held
  the focus in the active window since the flag (the rail's refresh notes it).
- **The row.** `terminal_snapshot` reads the labels: the state word becomes `looping?` or
  `stalled?`; `TerminalSnapshot` and `TerminalRow` gain `flag: Option<String>` (the tooltip's
  text); `render_terminal_row`'s end draws the warning mark beside or in place of the bell dot,
  with a `Tooltip::text`. #542's class function reads the flag as not reporting. The state word
  slot is shared with #547's `no update` (a flag wins) and #566's kinds (an idle seat's, never a
  working one's).
- **The banner.** `act` only: `notifications::notify` (crate-visible since #551, or made so
  here), called by the rail when a flag is set, once per episode.
- **File manifest.** Marley: `crates/marley_agent/src/stall.rs` (new), `marley_agent.rs` (the
  module), `claude_events.rs` (`TurnFacts.tool_lines`), `crates/marley_agent/Cargo.toml`
  (nothing new: `std::fs` reads `/proc`); `crates/marley_workbench/src/agent_events.rs`,
  `rail.rs`, `notifications.rs`, `marley_workbench.rs` (`stall_check_seconds` in
  `MarleySettings`); `crates/marley_rail/src/marley_rail.rs`;
  `crates/marley_system_one/src/question.rs` (the `stall_kind/1` set) and the workbench's
  `system_one.rs` (the `UseSpec`); `script/e2e/569-stalled-or-looping-agents.sh`. Zed:
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`.
- **Ledger rows.** The three Zed paths' rows in `docs/marley/zed-touchpoints.md` gain
  `stall_check_seconds` and the use's dropdown (#569) before their edits.

### E2E plan
Fixtures: a scratch repository; a HOME whose `.bashrc` puts `$E2E_WORK/bin` first on the PATH;
the stand-in `claude` of #519 with this scenario's `steps.json`, whose steps may also run a
command (`spawn`: the CPU burner; `kill`: end it); `CLAUDE_CONFIG_DIR` in a scratch folder; the
profile's settings: `marley.stall_check_seconds: [10, 20, 40, 80]`, #565's block enabled with
the repository listed, the provider and `uses.stall_kind` rewritten per step by
`system_one_setting`; `$E2E_PROFILE/system_one/replay.jsonl` written by setup (rows keyed by
`stall_kind/1` and a `match` on the state: `frozen` 0.91 for a state holding `quiet: 15
seconds`, `cannot_tell` for one holding the second prompt's words); `busctl --user monitor
org.freedesktop.Notifications` into a log.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-002 | mode `act`, provider `replay`; `claude`; Enter: a prompt and a `PreToolUse` (Bash `cargo test`) and `spawn` the burner; settle 45 | `569-01-long-task`; `grep -c stall_kind` on the day's file is 0 |
| REQ-001 | provider `rules`; Enter three times: `PostToolUse`, then two more `PreToolUse`/`PostToolUse` pairs of the same line; settle 3 | `569-02-looping`; the day's file: a `rules` row |
| REQ-003, REQ-004 | provider `replay`, mode `suggest`; Enter: a new prompt, `PostToolUse`, `kill` the burner; settle 15 | `569-03-stalled`; the row (set, masked state) |
| REQ-005 | `pointer_to` the mark; settle 2 | `569-04-tooltip` |
| REQ-006 | Enter: a new prompt (replay: `cannot_tell`); settle 15 | `569-05-cannot-tell`; the row |
| REQ-007 | mode `act`; Enter: a new prompt; settle 15 | `569-06-act-notification`; the bus log: one `Notify` |
| REQ-008 | `mcp_agent fleet` | the run log: `flag: stalled:frozen`, `flag_source: model` |
| REQ-009 | Enter: `Stop`; settle 3 | `569-07-cleared`; the outcome line |
| REQ-010 | mode `off`; Enter: a new prompt; settle 15 | `569-08-off`; no new row |
| REQ-011 | the stand-in's own log of what it read on stdin | the run log: the scenario's Enters only |
| REQ-012 | the golden set's 515 run | its page shot |

Not reachable by a scenario: a real frozen Claude Code (the stand-in stands for its silence), and
the thirty real minutes of #547 (its setting stands in).

### Risks
- Reading `/proc` for a process tree every 10 s: bounded by the tree's size (a Claude Code
  session's is small); `tree_cpu_in` takes its root, so the reads never touch the user's own
  files, and a process that exits mid-walk is skipped.
- A tool that legitimately waits (a `WebFetch` on a slow site) reads as open at the interval; the
  state carries the tool line, and the model's "stuck on the network" kind is a flag, not a stop.
- The Agents section grows by one item and the System One section by a dropdown; #515's
  scenario clicks the Layout dropdown, which stays above both (#547's notes).
- If #566 lands with `TurnFacts` in another shape, the tool lines go beside it in `agent_events`
  instead; the rule table is unaffected.
- If the slice runs long, the banner (REQ-007) splits off; the rule table, the ask, the labels and
  the row are the first slice.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
