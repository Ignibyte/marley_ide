# Stalled or looping agents: a flag on the rail row, never a stop — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-569-stalled-or-looping-agents.md
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
- **Discovery** (at `ca70b6488d` with #547's working-tree changes; promotion re-verifies):
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

### Promotion (2026-09-27)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #567's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore, at `c03828f765`) ✓; spec and design updated ✓.
- **Recall, added:** L-claude-566-the-folds-state-rides-on-the-seat-001 (anything the fold keeps
  across events is a label, and labels are what agents read in `fleet_snapshot`, so `shadow`
  lands none); PR-claude-an-answer-for-a-seat-names-the-session-prompt-and-stop-it-belongs-to-001
  and PR-claude-count-one-user-action-once-when-each-call-reports-it-001 (#566's landing
  guards); AD-claude-566 (a derived kind as seat labels, rules first); L-claude-521-gpuis-mutable-global-access-tells-every-observer-001
  (a write to a global the rail observes redraws every rail); L-claude-477 and L-claude-478 (the
  CPU from `/proc` on a timer; the notification proof). The brain (consultation
  dc1021383c964bd8889a36f4f380c72b): nothing on this seam.
- **What the code says now** (the Explore report, 2026-09-27; the drafted line numbers are stale):
  - #565's API is `ask(UseSpec, &Asking, cx) -> Task<Asked>`, `record`, `use_mode`, `detail` and
    `outcome` (#566, #567); `StateBuilder::new(detail, &mask)` runs inside `state_for`; `Reading`
    is `Off`, `Rules`, `Model`, `Refused` or `Unavailable`, a no-signal read a `Signal::Nothing`;
    sets live in `marley_system_one`'s root. The design's `ask(use, state, verdict)`,
    `Reading::NoSignal` and `question.rs` do not exist.
  - `TurnFacts` (`claude_events.rs:101-130`) counts tools by name and holds no tool lines;
    `Moving::from_seat` clones every label, so a label lives until the fold clears it; `end_turn`
    clears the tools in flight, the wait and #566's stop labels. The `lead_tool:` prefix is
    private.
  - `seat_line` (`:239-269`) picks `no update in N m` for a stale working seat, else #566's word for
    an idle one, else the status; `seat_activity` gives a working seat's tool line.
  - The only rail timer (`note_claude_code`, `rail.rs:412-446`) waits on `next_quiet_change`, which
    first fires at #547's threshold (30 minutes) and not at all with it at 0; the rail is one per
    window and only in the Marley layout. A write to `AgentEvents` notifies every rail and
    republishes the MCP snapshot.
  - `Terminal::pid()` is the PTY's foreground process group, which for an interactive `claude` is
    its own pid. Its tree holds long-lived children (MCP servers, a plugin's rust-analyzer burning
    CPU on its own), and each Bash tool call's `bash -c` in a session of its own; an idle `claude`
    itself burns about 1% of a core, and its spinner more.
  - Nothing reads `/proc/<pid>/stat` or `task/*/children` today; `procfs-core` is a workspace
    dependency with a `Stat` parser, and #521's `ports.rs` reads `/proc` off the main thread through
    `background_spawn(futures::future::lazy(..))`.
  - `notifications::notify` is private and tags every banner of a terminal
    `marley-terminal-<id>`, so a stall banner would replace Claude Code's own; `looking_at` is
    `pub(crate)`.
  - The rail's row end holds the bell dot over the close button's slot, hidden on hover, so a mark
    there could never show a tooltip; `thread_status_mark` draws `IconName::Warning` in
    `Color::Warning`; a tooltip needs a stateful element. `TerminalSnapshot` and `TerminalRow` have
    no `Default`, and the rail's tests hold seven literals of them.
  - No attention order exists (#542 is queued); #525's approvals, #571's pauses and #551's termios
    check are not built.
  - The Marley page edits a `u64` as a number field and has no editor for a list; its Agents
    section has five items and System One eleven.
  - The fixtures: 566's stand-in steps carry a session and a prompt id, and none can start or end
    a child; `fleet-labels` prints only the stop kind's keys; 535 proves a notification's text with a
    private D-Bus and a stand-in notification server, where 519's `busctl --user monitor` would let a
    banner reach the real desktop.

### Design
- **Changed at promotion** (the seams re-read on 2026-09-27; each item overrides the drafted design
  after it):
  - **The watch is its own global, not the rail's.** `marley_workbench::stall` holds a `StallWatch`
    global with the samples, each seat's episode and one timer task, armed by `agent_events` when a
    seat works and by itself. No rail observes it, so its ticks redraw nothing; only landing or
    clearing a flag writes `AgentEvents`. It runs in either layout and once per app.
  - **The CPU is the turn's tools'.** `tree_cpu_in(proc_root, pid, since_ticks)` sums `utime` and
    `stime` over `pid`'s descendants whose `starttime` comes after the turn's prompt, leaving out
    `pid` itself (Claude Code's spinner) and the children it had before the turn (MCP and language
    servers). `cpu_active` is that sum's growth between two samples above 2% of a core. The turn's
    start in boot-time ticks comes from `/proc/stat`'s `btime`. `procfs-core` parses the files.
  - **The rule table, conservative.** `judge(facts)`: repeats are `Looping`; a tool in flight with
    `cpu_active` is `LongTask`; quiet past the next check with no `cpu_active`, a tool in flight or
    not, is `Open`; otherwise `Calm`. Repeats are the same tool line three times in a row, or the
    same line failing twice in a row: an edit and a test run in turn are an ordinary cycle, not a
    loop. `held_by_marley` waits for #525 and #571.
  - **The checks double from one number.** `marley.stall_check_after_seconds` (60) sets the first
    check; the others come at twice, four and eight times it (1, 2, 4 and 8 minutes), and 0 turns
    the quiet checks off, the repeats rule still on. The settings page edits it as a number.
  - **Shadow shows nothing.** As #566, only `suggest` and `act` land labels, since agents read every
    label; `shadow` logs the rules' and the model's readings. D8 changes.
  - **The labels and their clearing.** `flag` (`looping` or `stalled:<kind>`), `flag_source`,
    `flag_confidence` land through an `Upsert` at the seat's own `last_event_ms`, guarded by the
    session, the prompt id and the stop count; the fold clears them at every event it takes, since
    any event is the agent moving again, and `agent_events` logs the outcome of the call that set
    them.
  - **The state.** Facts: the project, the agent, how long it has been quiet in words, the tool in
    flight, whether the tools burn CPU, the repeats, the subagents and the permission mode; texts:
    the prompt, the tool line and the terminal's last five non-empty lines, each masked and cut to
    300. A metadata-only project sends the facts alone.
  - **The set** `stall_kind/1` in `marley_system_one`'s root: the choice `kind` (`long_task`,
    `waiting_for_input`, `stuck`, `frozen`, `cannot_tell`) and the nouls `repeating` and `progress`.
    `long_task` and `cannot_tell` flag nothing.
  - **The row.** `seat_line` gains the flag's word (`looping?`, `stalled?`), which wins over `no
    update in N m` for a working seat, shown as a `FlagShown { Hidden, Shown }` from the mode.
    `TerminalSnapshot` and `TerminalRow` gain `flag: Option<String>`, the tooltip's text, and the
    row draws the warning mark as a child of its own beside the bell's slot, with an id for the
    tooltip.
  - **The banner.** `notifications::notify_stall(view, body, window, cx)`, `pub(crate)`, with a tag
    of its own (`marley-stall-<id>`) so it neither replaces nor is replaced by Claude Code's banners,
    behind the same focus gate, recorded for the click. Once per episode.
  - **The fixtures.** The scenario copies 566's stand-in, its steps able to `spawn` and `kill` a child
    (the CPU burner); `fleet-labels` prints the flag's keys too; the banner's proof is 535's
    private bus and stand-in notification server, the workspace switched away so the terminal is not
    in front.
- **`marley_agent::stall`** (new, pure):
  - `Facts { quiet_ms, tool_in_flight: Option<String>, cpu_active: Option<bool>, repeats:
    Option<Repeat>, subagents }`, `Repeat { line, times, failed }`;
    `judge(facts, next_check_ms) -> Verdict { Looping(Repeat), LongTask, Open, Calm }`: repeats
    first, then a tool in flight with `cpu_active`, then quiet past `next_check_ms` with no
    `cpu_active` (unknown counts as not active only once a sample pair exists), else calm.
  - `repeats(lines) -> Option<Repeat>` over `TurnFacts.tool_lines`: the same line three times in a
    row, or the same line ended by a failure twice in a row.
  - `checks(first_ms) -> [u64; 4]`, the first check and its doubles; `quiet_words(ms)`.
  - The labels `FLAG_LABEL`, `FLAG_SOURCE_LABEL`, `FLAG_CONFIDENCE_LABEL`, their list, `Flag {
    Looping, Stalled(Kind) }` with its label value, `labels(flag, source, confidence)`,
    `FlagShown { Hidden, Shown }`, `row_word(labels, shown)` and `tooltip(labels, shown)`.
  - `tree_cpu_in(proc_root, pid, since_ticks) -> Result<u64, io::Error>` and
    `boot_time_in(proc_root)`, through `procfs-core`'s `Stat`: `pid`'s descendants from
    `task/<tid>/children`, those with a `starttime` after `since_ticks`, their `utime + stime`;
    an entry that vanished mid-walk is skipped.
- **`claude_events`**: `TurnFacts.tool_lines`, the last twelve lead tool lines of the request,
  each with whether its end was a failure; the fold clears the flag's labels at every event it
  takes; `seat_line` takes a `FlagShown` beside `StopKindShown`, a flag's word winning over `no
  update in N m`; a public `tool_in_flight(labels)` for the lead's newest tool line.
- **`marley_system_one`**: `STALL_KIND_SET` (`stall_kind/1`) and `STALL_KIND` (`stall_kind`, 2 s).
- **`marley_workbench::stall`** (new): the `StallWatch` global (`samples` by seat: the last tree
  CPU and when; `episodes` by seat: the quiet episode's start, the checks asked, the flag's call,
  whether the banner went; the timer task); `watch(cx)`, armed from `on_frame` after a fold that
  leaves a working seat, and re-armed by the task for the next sample or check, dropped with no
  working seat; the tick: sample off the main thread (`background_spawn(futures::future::lazy)`),
  judge each working seat, `record` a `rules` row and land `looping` for a loop, `ask` the
  `stall_kind` set for an open seat at a check it has not asked, land `stalled:<kind>` for a kind
  other than `long_task` or `cannot_tell` at the floor, and in `act` post the banner once.
  Landing reuses #566's guard (the seat working, the same session, prompt id and stop count, the
  seat's own `last_event_ms`).
- **`agent_events`**: `watch` armed after each fold; the outcome of a cleared flag's call when an
  event clears it (`next event after N s: <event>`); `flag_shown(cx)` for the rail.
- **`notifications`**: `notify_stall`, the banner with its own tag.
- **The rail**: `terminal_snapshot` passes `flag_shown` to `seat_line` and gives the snapshot the
  tooltip; `render_terminal_row` draws the mark; `marley_rail`'s two types gain `flag`.
- **Settings**: `stall_check_after_seconds` in `MarleySettingsContent` (default 60),
  `MarleySettings`, `default.json` and the Marley page's Agents section (a number); the Stall Kind
  dropdown after Terminal Find in the System One section (twelve items); `uses.stall_kind: "off"`.
- **File manifest.**
  - Marley: `crates/marley_agent/src/stall.rs` (new), `marley_agent.rs` (the module),
    `claude_events.rs`, `crates/marley_agent/Cargo.toml` (`procfs-core`);
    `crates/marley_system_one/src/marley_system_one.rs` (the set);
    `crates/marley_workbench/src/stall.rs` (new), `agent_events.rs`, `notifications.rs`,
    `rail.rs`, `marley_workbench.rs` (the module and `MarleySettings`);
    `crates/marley_rail/src/marley_rail.rs` (the two fields and the test literals).
  - Zed paths with rows: `crates/settings_content/src/marley.rs` (the setting, the `uses`
    docstring), `crates/settings_ui/src/marley_page.rs` (the two items), `assets/settings/default.json`.
  - Scenarios: `script/e2e/569-stalled-or-looping-agents.sh` (new), `script/e2e/browser-fixture.sh`
    (`fleet-labels` prints the flag), `script/e2e/golden`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md` rows 55, 58 and 61 widen for
  `stall_check_after_seconds` and `uses.stall_kind` before those files are written.

### E2E plan
Fixtures: a scratch repository and HOME; 566's stand-in `claude` through `MARLEY_CLAUDE`, its steps
with a session and a prompt id each, and two actions besides events: `spawn` (the CPU burner,
`python3 -c 'while True: pass'`, a child started in the turn) and `kill`; the profile's settings:
`stall_check_after_seconds` 10 (checks at 10, 20, 40 and 80 seconds), #565's layer on with the
repository listed, the provider and `uses.stall_kind` set per step by 565's `system_one_setting`;
`replay.jsonl` written in setup (rows keyed `stall_kind/1` with a `match` on the state); 535's
private D-Bus and stand-in notification server for the banner.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-002 | mode `act`, provider `replay`; a prompt, `PreToolUse` Bash `cargo test`, `spawn` the burner; 25 s | `569-01-long-task`: `working`, no mark; no `stall_kind` row |
| REQ-001 | provider `rules`, mode `suggest`; the same `Bash: cargo test` ended three times in a row | `569-02-looping`: `looping?` and the mark; a `rules` row, no model call |
| REQ-003, REQ-004 | provider `replay`; a new prompt, the tool ended, `kill`; 15 s (replay: `frozen` 0.91) | `569-03-stalled`: `stalled?` and the mark; the row names `stall_kind/1`, the state masked and cut |
| REQ-005 | the pointer on the mark | `569-04-tooltip`: the facts and `frozen (0.91)` |
| REQ-006 | a new prompt (replay: `cannot_tell`); 15 s | `569-05-cannot-tell`: no mark; the row |
| REQ-007 | mode `act`; the workspace switched away; a new prompt; 15 s | the stand-in server's log: one banner `repo: Claude Code may be stuck` |
| REQ-008 | while flagged: `mcp_agent fleet-labels` | the run log: `flag 'stalled:frozen'`, `flag_source 'model'` |
| REQ-009 | the next event (`Stop`) | `569-07-cleared`: no mark; the outcome line |
| REQ-010 | mode `off`; a new prompt; 15 s | `569-08-off`: no mark, no new row |
| REQ-011 | the stand-in's log of what it read | the run log: the scenario's own Enters only |
| REQ-012 | the Marley settings page | `569-09-settings`: Stall Check After Seconds in Agents, Stall Kind in System One |
| REQ-013 | the gate and the golden set | the exit codes |

Not reachable by a scenario: a real frozen Claude Code (the stand-in's silence stands in), a real
model's reading (the replay rows stand in), and #542's attention order, not built yet.

### Risks
- The CPU rule rests on the tree Claude Code 2.1.283 has (its tools as children started in the
  turn, its servers from before); a later version that runs tools elsewhere reads as no CPU, which
  asks the model sooner and never flags on its own.
- The 2% threshold is a guess; an agent whose tool idles on the network legitimately reads open at
  a check, and the model's `stuck` is a flag, not a stop.
- `/proc` reads every sample for each quiet working seat: small trees, off the main thread.
- A banner only when the terminal is not in front, once per episode.
- If the slice runs long, the banner (REQ-007) splits off; the rules, the ask, the labels and the
  row are the first slice.

## Phase 2 — Code
- **Checklist** (no task tool): the ledger rows first (rows 55, 58 and 61, at promotion) ✓; the
  README marker present ✓; every manifest file written ✓; `cargo check` and `just clippy` on the
  touched crates, one at a time after #567's release install ended ✓; the review below ✓.
- **Built, to the manifest.**
  - `marley_agent::stall` (new, pure): the four flag labels and their list; `repeats` over the
    turn's tool lines; `Facts`, `Verdict` and `judge`; `checks` and `quiet_words`; `Stalled` and
    `Flag` with their label values; `labels`, `FlagShown`, `row_word` and `tooltip`; `active`,
    `ticks_since_boot`, `boot_time_in` and `tree_cpu_in` over `procfs-core`'s `Stat`.
  - `claude_events`: `TurnFacts.tool_lines` (the lead's last twelve tool ends, each cut to 120
    characters, with whether it failed; an interrupt is not one); the fold takes the flag's labels
    off at every event but a loop's while the loop goes on; `seat_line` takes a `FlagShown`, the
    flag's word first for a working seat.
  - `marley_system_one`: `STALL_KIND_SET` (`stall_kind/1`) and `STALL_KIND` (2 s).
  - `marley_workbench::stall` (new): the `StallWatch` global (the samples, the episodes, the calls
    waiting for their outcome, the boot time, whether the timer runs), `watch` and its tick, the
    sampling off the main thread, `decide`, `ask`, the banner, `Landing`, `note_tool_end` and
    `moved`. `agent_events` calls `moved` and `watch` after each fold and `note_tool_end` at a
    working seat's tool end, and gains `land_labels`, the guarded `Upsert` #566's
    `land_stop_kind` now uses too. `system_one` gains `project_name`, which the four places that
    named a project from its first folder now share. `notifications::notify_stall` posts under a
    tag of its own through the shared `post`. The rail gets the flag's word, the mark and its
    tooltip; `marley_rail`'s two types gain `flag`.
  - Settings: `stall_check_after_seconds` (default 60) in the content, `MarleySettings`,
    `default.json` and the Agents section; the Stall Kind dropdown in the System One section;
    `uses.stall_kind: "off"`.
- **Deviations from the design, and why.**
  - **A loop's flag stays while the loop goes on.** The design took every flag off at every
    event. A loop's next `PreToolUse` would then take its flag off, and nothing would put it back,
    since the loop is logged once. The fold keeps `looping` while the seat works and the newest
    tool line still repeats; a stall's flag leaves at any event, as designed.
  - **Runs of edits or reads of one file are not loops.** The plugin's line for `Read`, `Write`,
    `Edit`, `MultiEdit` and `NotebookEdit` names only the file, so three edits to one file read as
    three equal lines. Those tools count toward the failure rule only (the same line failing twice
    in a row).
  - **Outcomes in every mode but `off`.** The design logged an outcome when a flag left, so
    `shadow`, which lands no flag, would have logged readings with no outcome to fit a threshold
    on. A call that read something waits for its outcome in any mode: the next event for a stall
    reading, the loop's end for a loop, `still quiet N later` when the next check replaces it,
    and `the agent moved on before the reading came back` when the seat moved during the call.
  - **The flag's reason is a fourth label.** `flag_reason` carries what the flag rests on, in
    words, for the tooltip, which reads the labels alone.
  - **The CPU is read against the quiet.** A sample counts only beside another taken in the same
    quiet, so a tool that ran hot before the last event does not read as a long task after it.
  - **A late watch asks once.** A seat already quiet past several checks when the use is turned
    on is asked once, at the last check it passed, not once a tick for each.
  - **A tool's name is a fact, its line is text.** A fact is kept at every detail, so the state
    names the tool in flight and the looping tool by name alone (`tool_name`), and carries the
    line, which holds the command, as text a metadata-only project leaves out.
  - **The turn's start has a second's slack.** `/proc/stat`'s `btime` is whole seconds, so the
    turn's start in boot ticks can read up to a second late, which would leave out a tool started
    at once; the sampler takes a second off it.
  - **`tool_in_flight`** is the `tool` label the fold already keeps; no new function was needed.
- **Review** (the manifest against REQ-001 to REQ-013):
  - REQ-011: no path from a flag reaches `Terminal::input`, the PTY or a stop. The watch reads
    the terminal's pid and last lines; the banner's click activates the window, the workspace and
    the terminal, as a Claude Code banner's does.
  - Re-entrancy: `on_frame` ends its borrow of `AgentEvents` before `moved`, `watch` and
    `after_fold` run; the landing and the banner run from the app, outside any entity's update;
    the banner leases the window's root and reads only the terminal's focus.
  - Duplication: the guarded `Upsert` and the project's name were each written out in #566 and
    #567 and again here; both are now one function.
  - Provenance: nothing from Warp; no Zed function body copied. `procfs-core` was already a
    workspace dependency.
  - Clippy (`just clippy` on the six crates, all targets): `option_if_let_else` in `seat_line`
    (the flag's word first, then the old chain in `unwrap_or_else`), three long first doc
    paragraphs, `ask` over a hundred lines (split into `asking_for` and `answered`), and
    `run_tick` taking `&mut AsyncApp` (now `&AsyncApp`). `cargo check` found a name moved before
    its clone. All fixed; the last run is green.
  - Findings fixed: the samples from before the last event made a quiet tool look busy (fixed as
    above); a late watch asked once a tick for each check passed (fixed); the loop's flag vanished
    at its next event (fixed); edits to one file read as a loop (fixed); the command in flight went
    out as a fact, which a metadata-only project sends (fixed); a tool started in the turn's first
    second could read as older than the turn (fixed); a doc link from a public function to a
    private constant, which gate 14's `cargo doc` without private items rejects (fixed).

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan ✓; `fleet-labels` prints the flag's
  keys ✓; 569 in the golden set ✓; `just build` ✓; the scenario run and every shot read ✓; the
  reruns ✓; the golden set ✓; `gates.sh --diff` ✓.
- **The scenario**, `script/e2e/569-stalled-or-looping-agents.sh` (compositor sway): 566's
  stand-in `claude` through `MARLEY_CLAUDE`, one session with a prompt id per turn, its steps able
  to `sleep`, `say` a line on its terminal, `spawn` a child that burns a core (for at most 90
  seconds, its pid kept for the teardown) and `kill` it, and every line it reads logged; the
  profile's `stall_check_after_seconds` at 10 (checks at 10, 20, 40 and 80 seconds), #565's layer
  on with the repository listed, the provider and the mode set per step; `replay.jsonl` rows keyed
  `stall_kind/1` matched on the prompt; 535's private D-Bus and stand-in notification server, the
  window sent to workspace 2 for the banner. The real `claude` never ran. Run 1 passed every check;
  its tooltip shot pointed at the close button, so the mark's point came from its shots
  (207, 136) and run 2 passed every check with the tooltip shown.
- **The shots** (run 2, read one by one):
  - `569-01-long-task` (REQ-002): the row reads `working · Run the whole test su…` over
    `Bash: cargo test --workspace`, no mark, 25 seconds into the quiet with the burner on; the run
    log: no flag label, no `stall_kind` row two checks on.
  - `569-02-looping` (REQ-001): `looping? · Make the flaky t…` with the warning mark; the log:
    `flag 'looping'`, `flag_source 'rules'`, `flag_reason 'Bash: cargo test flaky ended 3 times in a
    row'`, one `rules` row (`tool: Bash`, `repeats: ended 3 times in a row`, `repeating: yes`), no
    other stall call.
  - `569-03-stalled` (REQ-003, REQ-004): `stalled? · Fix the flaky test…` with the mark; the row:
    `stall_kind/1` on `replay`, the facts (`quiet: 10 seconds`, `tool in flight: none`, `tools use
    the CPU: no`), the prompt cut to 300 characters with `…`, the terminal's `API_TOKEN` line
    `[redacted: secret]`, no transcript path, one ask for the check.
  - `569-04-tooltip` (REQ-005): the pointer on the mark shows `Stalled? frozen (0.91): quiet 10
    seconds, no tool in flight, no CPU in the turn's tools`; the mark stays beside the close button
    the hover shows.
  - `569-07-cleared` (REQ-009): after the next event, a `Stop`, the row is `idle` with the stop's
    message and no mark; the log: no flag label, and the outcome `the next event … later: Stop`;
    the loop's call had logged `the loop ended … later: Stop` at the step before.
  - `569-05-cannot-tell` (REQ-006): `working · Tidy the imports`, no mark; the row logged.
  - `569-06-act` (REQ-007): back from workspace 2, `stalled? · Update the chan…` with the mark; the
    stand-in server's log holds one banner, `repo: Claude Code may be stuck` with `stuck (0.84):
    quiet 10 seconds, no tool in flight, no CPU in the turn's tools`, through two checks away.
  - `569-08-off` (REQ-010): `working · Rename the module`, no mark; no new row.
  - `569-09a-agents`, `569-09-settings` (REQ-012): Stall Check After Seconds (10, the profile's)
    in the Agents section; Stall Kind (Off) after Terminal Find in the System One section.
  - REQ-008: the run log's `fleet-labels` lines carry `flag`, `flag_source`, `flag_confidence`
    and `flag_reason` while flagged, and none after.
  - REQ-011: the stand-in's log holds seven lines, each `"\n"`: the scenario's own Enters and
    nothing else reached the agent; the diff has no path from a flag to the terminal's input.
- **Focus**: the scenario ran in its own headless sway; the runner reported `hyprland: 0 Marley
  windows before the run, 0 after`, no rule added.
- **Not reachable by a scenario**: a real Claude Code that froze (the stand-in's silence stands
  in), a real model's reading (the replay rows stand in), and #542's attention order, not built.
- **The reruns** (`just regress`, the debug build): 519, 547, 565, 566 and 515 all passed (36,
  104, 214, 89 and 28 seconds): the fold, the fixture's wider `fleet-labels`, the layer, the stop
  kind through the shared `land_labels`, and the settings page.
- **The gate**: `script/gates.sh --diff` printed `GATE GREEN [diff]`, 16 passed and 0 failed
  (rustfmt, clippy on every target, audit, deny, shear, gitleaks, shellcheck, no-suppressions,
  source bans, rustdoc, the ledger, manifests, typos, semgrep, dylint, the receipt); its full log
  is kept in the scratchpad. The `SharedString` warnings it printed are Zed's `sidebar` crate's,
  at warn for Zed's crates: pre-existing — not in scope.
- **The golden set** with 569 added: `regress: all 41 passed`, 569 in 149 seconds.
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented** (§21): `CHANGELOG.md` (Added: the stall kind); `docs/marley/three-prong-plan.md`
  (S1: the stall kind shipped, the third use); `docs/marley_architecture/marley_agent.md` (the
  `procfs-core` dependency, `seat_line`'s `FlagShown`, `TurnFacts.tool_lines` and `ToolLine`, the
  new `stall` section, its consumers); `marley_rail.md` (`TerminalSnapshot::flag`);
  `marley_system_one.md` (the stall kind's set, its consumer, its scenario);
  `marley_workbench.md` (`notify_stall`, `land_labels`, the new stall section, `project_name`);
  `docs/marley/guide.md` ("The stall kind"). The Zed paths' rows in
  `docs/marley/zed-touchpoints.md` (55 `settings_content/src/marley.rs`, 58
  `settings_ui/src/marley_page.rs`, 61 `assets/settings/default.json`) were widened at promotion
  and describe what shipped: `stall_check_after_seconds`, `uses.stall_kind` off, the two items.
- **Knowledge appended** (§19): F-claude-569-the-command-in-flight-went-out-as-a-fact-001,
  F-claude-569-a-loops-flag-would-have-left-at-its-next-event-for-good-001,
  F-claude-569-edits-to-one-file-read-as-a-loop-001,
  F-claude-569-a-quiet-seat-read-busy-from-a-sample-before-its-last-event-001,
  F-claude-569-a-late-watch-would-ask-once-a-tick-for-each-passed-check-001;
  PR-claude-a-state-fact-holds-only-what-code-computed-001;
  L-claude-569-a-turns-tools-are-the-descendants-born-in-it-001,
  L-claude-569-proc-btime-is-whole-seconds-001,
  L-claude-569-a-stand-in-proves-a-cpu-rule-with-a-bounded-busy-child-001;
  AD-claude-569-the-stall-kind-flags-from-code-facts-first-and-never-acts-on-the-agent-001.
- **The brain**: consultation dc1021383c964bd8889a36f4f380c72b closed with `brain_decide`
  (`decisions/marley-569-the-stall-kind-flags-from-code-facts-first-and-never-acts-on-the-agent`),
  a follow-up due 2026-10-28: fit the 2% CPU threshold and the first check against a month of
  shadow rows and their outcomes.
- **Closed**: TICKET-569 moved to `tickets/closed/`, its pipeline doc pointed at `completed/`; no
  `BACKLOG.md` row was left (promotion removed it), and no queued spec named the queued pair.
- **The commit**: the Test phase's receipt matches the tree (no code changed after the green;
  docs only since).
