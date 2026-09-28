# What a stopped turn needs — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-566-stop-kind.md
- **Pipeline spec:** 566-stop-kind.spec.md

## Phase 1 — Plan
- **Request:** the Jev note's use 1 (2026-09-25), the first use its recommendations turn on:
  "a choice (done and checked, done by claim only, asks Chad, blocked, still going, cannot tell);
  a noul per part of the last prompt", changing "the rail row's word, the notification's wording
  ('asks you'), the order". Chad approved all seven uses on 2026-09-26 with the rules quoted in
  #565's D1 to D4. The notification and the order belong to #538 and #542, so this ticket puts
  the kind where both can read it and changes the row.
- **Classification / tier:** feature, prong 2 (attention). Marley crates: `marley_agent`,
  `marley_system_one`, `marley_workbench`. One Zed path with a row: `marley_page.rs` (the
  dropdown). Depends on #565 and #519 (shipped). Size S to M.
- **Recall (§18.3):**
  - AD-claude-519-claude-codes-hook-events-ride-in-band-into-marley-fleet-001: the seat is the
    one model the rail and the server read; the kind is a label on it (D3).
  - F-claude-519-a-frame-bound-dropped-the-shown-fields-before-the-unbounded-ones-001 and
    PR-claude-drop-the-unbounded-fields-first-001: the frame's prompt and message are what the
    hook kept after bounding; the state uses them as they are (300 characters) and nothing longer.
  - PR-claude-474 and AD-claude-474 (#519 D4): a frame is display data; a label set from it may
    be shown, never acted on (D4).
  - L-claude-482: `claude -p` drops a hook's `terminalSequence`; the promotion measurement runs an
    interactive session in a pty, as #519's did.
  - L-claude-480: an e2e fake acts the program out so the result shows; #519's stand-in and its
    steps file are the fixture, extended with this ticket's sessions.
  - The 547 spec's D5: `fleet_snapshot` publishes the fold's labels as they are; the new labels
    ride the same way (REQ-010).
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery** (2026-09-26, at the working tree of `ca70b6488d` with #547 in Test):
  - `crates/marley_agent/src/claude_events.rs`: `HookEvent` (55-90), `decode` (123), `fold`
    (141-159), `seat_status` (172), `seat_line` (189-209: the state word, the subagents, the
    prompt), `seat_activity` (214-224: the message for an idle seat), `Moving` (227),
    `note_session` (251), `take` (278-341: `UserPromptSubmit` ends the turn and clears message
    and error at 290-299; `PermissionRequest` at 303; `Stop` at 314; `StopFailure` at 320),
    `tool_starts` (343), `tool_ends` (367-393: `is_interrupt` ends the turn idle at 388),
    `end_turn` (414-422), `set` and `clear` (424-430), `into_events` (432: an `Upsert` with the
    labels, then a `QuestionRaised`). The turn facts live beside `Moving` and ride as labels only
    where a reader needs them.
  - `crates/marley_workbench/src/agent_events.rs`: `AgentEvents { snapshot }` (21), `seat` (30),
    `on_frame` (83-106: the foreground check, decode, fold against the previous seat, `apply`),
    `end` (112), `forget` (136), `without` (149: the reducer never removes a seat, so a snapshot
    without one is rebuilt from upserts).
  - `crates/marley_workbench/src/rail.rs:1717-1785` (`terminal_snapshot`), `:309` (`refresh`,
    observing `AgentEvents` at 245), `:1371` (the row's subtitle and activity lines).
  - `crates/marley_fleet/src/session.rs:55-77` (`Session`, `labels` a `BTreeMap`), `reducer.rs:17`
    (`SessionEvent::Upsert { labels, .. }`), `:147` (`apply`).
  - `crates/marley_workbench/claude_plugin/marley/hooks/event.py:40-56` (`preview`), `:57-90`
    (`summary`: no `tool_response` field today; `is_interrupt` at 74).
  - #538's spec (queued): `event_line(project, kind, event)` builds the banner's title; it will
    read `stop_kind` for `asks you`. #542's spec (queued): the classes in `marley_rail`; `asks_you`
    and `blocked` sort as needs you, `done_*` as done and not seen, `still_going` as working.
  - #565's spec: `UseSpec`, `StateBuilder` (facts and masked text, `Detail`), `ask`, `Mode`,
    the log and replay rows, `system_one_setting` in its scenario.
- **Decisions:** D1 to D8 in the spec.

### Promotion (2026-09-27)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #565's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore) ✓; the failing-Bash question answered from the hooks reference ✓; spec
  and design updated ✓.
- **Recall, added:** F-claude-547-a-scenarios-click-ran-the-real-claude-001 and
  PR-claude-name-the-fakes-the-app-runs-001 (the scenario sets `MARLEY_CLAUDE` to #519's stand-in,
  so the real Claude Code never runs); AD-claude-565-the-system-one-layer-is-a-pure-core-behind-an-adapter-off-by-default-001
  and L-claude-565-jevs-answer-shape-as-recorded-001 (a choice answers `choice`, `confidence`
  and `probabilities`; a noul `noul` alone); L-claude-565-a-poll-in-setup-dies-under-set-e-001 and
  L-claude-565-a-scenario-reaches-the-users-own-keyring-001 (the scenario's setup and the layer's
  key, as 565's); F-claude-565-the-check-read-its-workspace-inside-that-workspaces-update-001 (the
  ask must not read an entity inside that entity's update). The brain (consultation
  586b6b97fa4c4be58571ada24caee8c6): no decision on the stop kind.
- **What #565 shipped that changes this design.** Its question sets are `&'static` and it left
  dynamic questions to their first user, which is this ticket. The `part_N_done` nouls would carry
  the prompt's own parts; since a caller never supplies its own questions, the parts go in the
  state instead, and seven sets cover zero to six parts (Changed at promotion). Its `ask`
  answers a use's own verdict only when the provider is `rules`, so a stop the rules settle needs
  a way to log a `rules` row whatever the provider (#565's D6: every verdict is a row). Its log
  defines `OutcomeRow` and nothing writes one yet, so the adapter gains the write. Each use's mode
  is `marley.system_one.uses.<name>`, and the Marley page's Check item is the pattern for a Stop
  Kind item.
- **The live measurement, replaced.** The plan's measurement ran the installed Claude Code in a
  pty. The real `claude` is not started from this session, so the question it answered, how a
  failing Bash reaches the hooks, is answered from Claude Code's published hooks reference
  (below).
- **What the hooks reference says** (code.claude.com/docs/en/hooks and the hooks guide, read
  2026-09-27): "`PostToolUse` | After a tool call succeeds" and "`PostToolUseFailure` | After a
  tool call fails". Whether a Bash whose command exits non-zero is a failed call is not said, and
  neither page gives `PostToolUse`'s `tool_response` for Bash or `PostToolUseFailure`'s fields.
  Every hook input carries `prompt_id` ("absent until the first user input"), and `Stop` carries
  `last_assistant_message` and `stop_hook_active`. `PermissionDenied` fires only "when auto mode
  denies a tool call", so a user's refusal in the prompt sends no event. So the design holds
  either way. `checked_after_edit` is a Bash whose call ended after the turn's last edit with no
  failure event for its `tool_use_id`. A failing Bash reported as a `PostToolUse` would count as a
  check, and a wrong `done · checked` then needs the model too, since its `done_checked` requires
  the message to name a check that ran. `event.py` reads nothing from `tool_response` (Out, as the
  spec had it).

### Design
- **Changed at promotion** (the seams re-read on 2026-09-27; each item overrides the drafted design
  after it):
  - **The facts live in a label.** `Moving` is rebuilt from the seat on every fold
    (`claude_events.rs:155`, `:236`), so `TurnFacts` is the `turn` label, parsed at the start of
    `take` and written back at its end. It counts per `tool_use_id`: a `PermissionRequest` whose
    call has no `PostToolUse` or `PostToolUseFailure` yet is pending. The `Stop` arm's `end_turn`
    wipes `waiting_on` and the tool labels, so `Blocked` reads the facts, not the wait (an
    `AskUserQuestion` also waits). `checked_after_edit` is a lead Bash that ended by `PostToolUse`
    after the turn's last Write, Edit or MultiEdit (or, with no edit in the turn, any such Bash).
    `event.py` sends no `tool_response` (`:57-91`), so a failure is a `PostToolUseFailure`.
  - **A user's prompt resets, a harness's does not.** The `UserPromptSubmit` arm clears the message
    for harness-injected prompts too (`:298`); the turn facts and the stop labels reset only for a
    prompt the user typed.
  - **`Interrupted` comes from the interrupt**, which ends the turn in `tool_ends` (`:390`) without
    a `Stop`; `Blocked` and `AsksYou` come at the lead `Stop`.
  - **The row's mode is an argument.** `marley_agent` depends only on base64, `marley_fleet` and
    serde, so `seat_line(seat, now_ms, no_update_after_ms, shown)` takes a local `StopKindShown {
    Hidden, Suggest, Act }`, which `rail.rs::terminal_snapshot` (`:2200`) computes from
    `MarleySettings.system_one` (`enabled` and `uses["stop_kind"]`), as it reads
    `no_update_after_ms` (`:2271`).
  - **Shadow writes no label.** `fleet_snapshot` publishes every label (`mcp.rs:145`,
    `marley_mcp/src/tools.rs:33`), so a label would show the kind to agents in shadow; shadow and
    off write none, and only `suggest` and `act` do.
  - **Landing an answer.** `on_frame` runs inside the terminal view's update
    (`notifications.rs:43`); the ask reads the view it is given, its workspace through
    `marley_workspace()` (not updated there) and globals. The answer lands later through a new
    `agent_events::land_stop_kind(session, prompt_id, labels, cx)`: it re-reads the seat and
    applies an `Upsert` with the seat's full labels, state, title and transport plus the kind's
    labels only when the seat is still there, `Idle`, and on the same `prompt_id`. An `Upsert`
    would recreate a forgotten seat and revive a done one (`reducer.rs:149-178`).
  - **The parts stay in the state, the questions compiled in.** #565's sets are `&'static` and a
    caller never supplies its own questions (its D5). So the parts are text lines of the state
    (`part 1: …`, masked and cut like any text), and `STOP_KIND` is seven static `UseSpec`s, by part
    count (0 to 6), whose sets hold the `kind` choice and nouls `part_1_done` to `part_N_done`
    that ask about "the part of the request the state labels `part N`". A metadata-only project
    has no text lines, so it asks `STOP_KIND[0]`.
  - **The adapter gains three things** (`system_one.rs`): `detail(asking, cx)`, the project's
    `Detail` or refusal, which the use needs before it picks a set; `record(spec, asking, reading,
    cx) -> Asked`, a `rules` row for a verdict the rules settled, whatever the provider (#565's
    D6); and `outcome(call, text, cx)`, an `OutcomeRow` line.
  - **The settings.** A Stop Kind dropdown after Check on the Marley page (`marley_page.rs:386`, the
    map-keyed pattern; the section grows to nine items); `default.json`'s `uses` gains
    `"stop_kind": "off"` (#565 lists uses by name), with the `Default:` docstring in
    `settings_content/src/marley.rs:103`. Ledger rows 55, 58 and 61 widen.
  - **The scenario.** #519's stand-in `claude` is inline in its scenario and #547's copy sets
    `MARLEY_CLAUDE` (`547-*.sh:107`): 566 copies #547's, with `MARLEY_CLAUDE` set, and
    `system_one_setting` from 565. Its steps carry `prompt_id`. `mcp_agent fleet` prints four labels
    (`browser-fixture.sh:746-753`), so a new `fleet-labels` command prints a seat's labels, leaving
    `fleet`'s output to 547.
  - **What the view and log say.** `Reading` has no no-signal variant, so a stop with no replay row
    logs `kind: no signal (no replay row)`; a shadow row in Decisions reads `would show: kind:
    done_checked 0.91; …`.
  - **The message keeps its end** (D9, new). `event.py` cuts every message to its first 299
    characters and an ellipsis (`cut`, `:36-38`), so a question or a status at the end of a long
    final message never reaches the rules or the model, and Claude Code's final messages often
    end with one. A message over 300 characters now keeps its start, ` … `, and its last whole
    sentences up to 147 characters: still 300 characters, as Chad's rule sets, and the row still
    shows the start. Whole sentences and a start cut at a word, since the hook cuts before
    Marley's redaction runs (PR-claude-redact-the-whole-text-before-cutting-it-001, found at
    Code's recall). The plugin goes to 1.3.0 so an installed 1.2.0 is offered the update (#547), and
    547's stand-in writes the version Marley ships into its list instead of `1.2.0`.
  - **`Interrupted` gets a criterion** (REQ-012, new): the drafted table had none. The gate moves
    to REQ-014, after the long message's REQ-013.
- **`TurnFacts`** (`claude_events`, kept in `Moving` and stored on the seat as one label,
  `turn`, in a compact `key=value` form the reader parses back; the reducer treats it as opaque):
  `started_ms`, `tools: BTreeMap<String, u32>` (lead), `subagent_tools: u32`, `failures: u32`,
  `interrupted: bool`, `permissions_asked: u32`, `permissions_pending: u32` (a `PermissionRequest`
  whose tool call has no `PostToolUse` or `PostToolUseFailure` yet), `last_edit_at: Option<u64>`
  (the sequence of the last Write, Edit or MultiEdit), `checked_after_edit: bool` (a lead Bash
  whose `PostToolUse` came after `last_edit_at`, or with no edit in the turn, any such Bash), and
  the pending calls' ids, which a tool's end takes out. `UserPromptSubmit` by the user resets it;
  a harness prompt does not. The `prompt_id` the seat already carries as a label (`:267`) names
  the turn.
- **`stop_kind`** (`crates/marley_agent/src/stop_kind.rs`, pure): `Verdict { Interrupted,
  Blocked, AsksYou, Open }`, `rules(prompt, message, &TurnFacts) -> Verdict` (`Interrupted` from
  `facts.interrupted`, `Blocked` from pending permissions, `AsksYou` from the message's last
  sentence);
  `parts(prompt) -> Vec<String>` (split on sentence ends and list markers, at most six, each
  trimmed); `Kind` (the six plus `Interrupted`) with its label value and its row words;
  `apply_evidence(kind, facts) -> Kind` (the demotion of D1); `labels(kind, source, confidence,
  missing_parts)`; and the constants `STOP_KIND_LABEL`, `STOP_KIND_SOURCE_LABEL`,
  `STOP_KIND_CONFIDENCE_LABEL`, `STOP_PARTS_MISSING_LABEL`, which `take`'s `UserPromptSubmit` arm
  clears with message and error.
- **The sets** (`marley_system_one`'s root, beside `CHECK_SET`): `STOP_KIND_SETS`, seven
  `QuestionSet`s (`stop_kind/1`, `jev-1.13.0`), the `kind` choice with the spec's option
  descriptions plus nouls `part_1_done` to `part_N_done` about "the part of the request the
  state labels `part N`"; and `STOP_KIND`, seven `UseSpec`s (`stop_kind`, 2 s) over them. The
  replay matches on the set's id, which the seven share, and on the state.
- **The ask** (`agent_events::on_frame`, after `apply`): on a lead `Stop`, or a lead interrupt
  that ended the turn, with the layer on and the use's mode not `off`: `rules` over the seat's
  prompt, message and `turn` facts. The use fills an `Asking` (#565): the subject is the seat, the
  project and folders are the terminal's workspace's, read through `marley_workspace()` (not
  under update there); the facts and texts are the spec's, and `system_one::detail` says whether
  the parts go in. A verdict other than `Open` goes to `system_one::record` as a `rules` row and,
  in `suggest` or `act`, lands at once. `Open` runs `system_one::ask(&STOP_KIND[parts], …)`, and
  its detached task calls `land_stop_kind(session, prompt_id, labels, cx)` in `suggest` or `act`,
  which applies the `Upsert` only while the seat is there, `Idle`, and on the asked `prompt_id`;
  the rail's observers redraw. `shadow` lands nothing.
- **The row**: `seat_line(seat, now_ms, no_update_after_ms, shown)` reads `stop_kind` for an
  `Idle` seat by `StopKindShown` (`Act`: the word; `Suggest`: `idle · <word>?`; `Hidden`: `idle`);
  `seat_activity` prefixes `not covered: "<part>"` when `stop_parts_missing` is set.
  `terminal_snapshot` computes `shown` from `MarleySettings.system_one` (`enabled` and
  `uses["stop_kind"]`).
- **Outcomes**: `on_frame`, on a user's `UserPromptSubmit`, calls `system_one::outcome` with
  the seat's last stop call (kept in `AgentEvents` by seat) and a line such as `next prompt after
  12 s, 48 characters` (`, after asks you` when the kind was `asks_you` or `blocked` and the
  prompt came within a minute).
- **The message's end** (`event.py`): `cut_ends(text, limit)` for the three message fields: the
  tail is the longest run of whole sentences at the end (each starting after `. `, `? ` or `! `)
  that fits in 147 characters, the head the rest of the 300 cut back to a word; with no such
  tail, `cut` as today. `plugin.json` 1.3.0.
- **File manifest.**
  - Marley: `crates/marley_agent/src/claude_events.rs` (`TurnFacts` in the fold, the stop
    labels cleared on a user's prompt, `StopKindShown` and `seat_line`'s argument,
    `seat_activity`'s `not covered`), `stop_kind.rs` (new), `marley_agent.rs` (the module);
    `crates/marley_system_one/src/marley_system_one.rs` (`STOP_KIND_SETS`, `STOP_KIND`);
    `crates/marley_workbench/src/agent_events.rs` (the ask, `land_stop_kind`, the last stop call
    by seat, the outcome), `system_one.rs` (`detail`, `record`, `outcome`), `rail.rs`
    (`terminal_snapshot`'s `shown`), `claude_plugin/marley/hooks/event.py` (`cut_ends`),
    `claude_plugin/marley/.claude-plugin/plugin.json` (1.3.0).
  - Zed paths with rows: `crates/settings_ui/src/marley_page.rs` (the Stop Kind dropdown, the
    section at nine items), `crates/settings_content/src/marley.rs` (the `uses` docstring's
    default), `assets/settings/default.json` (`uses.stop_kind`).
  - Scenarios: `script/e2e/566-stop-kind.sh` (new), `script/e2e/browser-fixture.sh`
    (`mcp_agent fleet-labels`), `script/e2e/547-claude-code-events-slice-2.sh` (the shipped
    version), `script/e2e/golden` (the 566 entry).
- **Ledger rows.** `docs/marley/zed-touchpoints.md` rows 55 (`marley.rs`), 58 (`marley_page.rs`)
  and 61 (`default.json`) name the `stop_kind` use, before those files are written.

### E2E plan
Fixtures: #519's scratch repository and HOME, #547's copy of the stand-in `claude` reached
through `MARLEY_CLAUDE`, its steps carrying a `prompt_id` per prompt; the replay file written by
setup with rows matched on the prompt's text; the layer enabled on `replay` with the repository
listed; the mode set per step with 565's `system_one_setting` before the step's stop.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-008 | mode `shadow`; SessionStart, prompt "Add a README to the project", PreToolUse Edit, PostToolUse, PreToolUse Bash `cargo test`, PostToolUse, Stop "Done; the tests pass." (replay: `done_checked` 0.91, parts 0.95) | `566-01-shadow` (the row; then `marley: decisions` for the row) |
| REQ-008 | after `566-01` | `mcp_agent fleet-labels`: no `stop_kind` |
| REQ-001, REQ-010 | mode `act`; the same session again (a new `session_id`) | `566-02-done-checked`; `mcp_agent fleet-labels` shows the labels |
| REQ-002 | a prompt "Fix the typo", Edit, Stop "Fixed; tests pass." (replay `done_checked` 0.93) | `566-03-done-claimed` |
| REQ-003 | a prompt, Stop "Should I also add a license?" | `566-04-asks-you-by-rule`; the day's file: `source: rules`, no `replay` provider row |
| REQ-004 | a prompt, PreToolUse Write, PermissionRequest Write, Stop "I need permission to write the file." | `566-05-blocked-by-rule`; the log |
| REQ-005 | mode `suggest`; a prompt, Stop "Next I will wire the tests." (replay `still_going` 0.88) | `566-06-suggest` |
| REQ-006 | mode `act`; prompt "Add a README. Add a license.", Edit, Bash, Stop "Added the README." (replay `done_checked` 0.9, `part_1_done` 0.94, `part_2_done` 0.05) | `566-07-part-missing` |
| REQ-007 | a prompt with no replay row, Stop | `566-08-no-row`; the log: `kind: no signal (no replay row)` |
| REQ-009 | mode `off`; a prompt, Stop | `566-09-off`; the day's file unchanged |
| REQ-010 | the next prompt in `566-02`'s session | `mcp_agent fleet-labels`: no `stop_kind`; the outcome line in the day's file |
| REQ-011 | a Stop whose message carries a `ghp_` token assembled at run time | the log row's masked state |
| REQ-012 | mode `act`; a prompt, PreToolUse Bash, PostToolUseFailure with `is_interrupt` | `566-10-interrupted`; the day's file: `source: rules`, no call |
| REQ-013 | a Stop whose message is 600 characters of summary ending "Should I also add a license?" | `566-11-long-message`: `asks you`; the log row's state holds the start, ` … ` and the question |
| REQ-014 | Test's gate and regression runs | the exit codes |

What no scenario reaches: a real model's readings (the replay rows stand in; the note's
recommendation 4 sets the bar before `act` is the default for anyone), and a real failing Bash,
whose hook the reference leaves open (the Promotion entry) and the real `claude` is not started
to see.

### Risks
- A Bash that fails: whether Claude Code reports it as `PostToolUseFailure` or as a
  `PostToolUse` with an error in `tool_response` decides `failures` and `checked_after_edit`, and
  the reference does not say. If it is the latter, a failing test run counts as a check, and a
  wrong `done · checked` then needs the model too, since its `done_checked` requires the message
  to name a check that ran. A ticket that can see a real failing Bash adds `failed` to
  `event.py`.
- A permission the user refuses: `PermissionDenied` fires only in auto mode, so a refusal in the
  dialog either sends a `PostToolUseFailure` for the call, and the model reads the message, or
  sends nothing, and the call stays pending, so the rules read `blocked`. Both are sound.
- The plugin's version: 1.3.0 makes the agent bar offer the update to anyone on 1.2.0, and the
  new cut reaches a session only after it. Until then the rules read the start of a long message,
  as today's row does.
- The prompt's parts from a 300-character cut: a long prompt loses its tail, so the parts cover
  what the frame kept; the row says `not covered` only for a part it asked about.
- `turn` as one label: the reducer copies labels through `Upsert`, so the facts survive the
  fold; a reader that does not know the label ignores it (labels are opaque).
- A late answer after the next prompt: the labels carry the `prompt_id` asked, and an answer for
  an older turn is dropped.
- Cost: about 300 calls a day at five agents (the note); the dedupe drops a repeated stop of the
  same state.
- The words on a row: `done · checked · <prompt>` is longer than `idle`; the rail cuts the prompt
  as it does today (#519's Test found the state word must come first).

## Phase 2 — Code
- **Checklist** (no task tool): the ledger rows 55, 58 and 61 first ✓; `event.py` and the plugin's
  1.3.0 ✓; `marley_system_one`'s seven sets ✓; `marley_agent`'s `stop_kind.rs` and the fold ✓;
  the adapter's `use_mode`, `detail`, `record`, `outcome` ✓; `agent_events`' ask, landing and
  outcomes ✓; the rail ✓; the settings page, `default.json` and the docstring ✓; check, fmt and
  clippy ✓; the review ✓.
- **Built.**
  - `event.py`: `cut_ends`, for the three message fields. A message over 300 characters keeps its
    start, cut back to a word, then ` … ` and its last whole sentences up to 147 characters; a
    last sentence longer than that leaves `cut` as it was. `plugin.json` and `marketplace.json`
    at 1.3.0.
  - `marley_system_one`: `MAX_PARTS`, `STOP_KIND_SETS` (`stop_kind_0/1` to `stop_kind_6/1`: the
    `kind` choice and `part_1_done` to `part_N_done`), `STOP_KIND` (seven `UseSpec`s named
    `stop_kind`, 2 s) as statics, so a use picked at run time is `&'static`, and `stop_kind(parts)`.
  - `marley_agent::claude_events`: `TurnFacts` (serde, the `turn` label as JSON, `of(labels)`),
    kept in `Moving` and written back on every event; `PROMPT_ID_LABEL` and `TURN_LABEL`; `take`
    gains `now_ms`; `note_tool_end` counts failures, the first lead interrupt (a stop), edits and
    a Bash after the last edit; a `PermissionRequest` adds its call to `pending`; `end_turn` also
    clears the four stop labels; `seat_line` and `seat_activity` take a `StopKindShown`.
  - `marley_agent::stop_kind` (new, pure): the label constants, `Verdict` and `rules(message,
    facts)`, `parts(prompt)` (sentences and numbered items, the opening bullet's items, two words
    at least, no `:` intro, no cut last part, six at most), `Kind` with `value`, `from_value` and
    `words`, `apply_evidence`, `Source`, `labels`, `StopKindShown`, `row_word`, `not_covered`,
    `state_facts` with the turn's length in words.
  - `marley_workbench::system_one`: `state_for` (the masked state, shared with `ask`), `use_mode`,
    `detail`, `record` (a `rules` row whatever the provider), `outcome` (an `OutcomeRow`),
    `choice_verdict`, and `project_of` made `pub(crate)`.
  - `marley_workbench::agent_events`: `last_stops` by seat; `after_fold` after every fold (the
    outcome on a user's prompt, the ask on a lead `Stop` or on the interrupt that ended a running
    turn, the last stop forgotten on `SessionStart`, `end` and `forget`); `ask_stop_kind` (the
    rules, the project from the view's workspace, the parts when the project sends text, `record`
    or `ask`); `Landing` (the kind read against the facts, the parts not covered, the last stop
    kept for its outcome when the call read something); `land_stop_kind` (an `Upsert` at the
    seat's own time, only while the seat is idle, on the same `prompt_id`, with no stop since);
    `stop_kind_shown` for the rail.
  - `rail.rs`: `terminal_snapshot` passes `shown`; the settings observer refreshes the rows.
  - Settings: the Stop Kind item after Check (`marley_page.rs`, nine items), `uses.stop_kind:
    "off"` in `default.json`, the docstring's default in `marley.rs`.
- **Deviations from the plan, and why.**
  - The stop labels clear at every turn's start and end, not only on a user's prompt: with the
    use in shadow or off at a later stop of the same request, an earlier stop's kind would have
    shown on the row (found writing `end_turn`). REQ-010 still holds.
  - Each set has its own id, `stop_kind_N/1`, not one shared `stop_kind/1`: #565's `QuestionSet`
    says an id names exactly what was asked, and the seven ask different questions. A replay row
    names the set of its prompt's part count.
  - `checked_after_edit` counts any Bash that finished, the lead's or a subagent's, after any
    edit, and `NotebookEdit` is an edit: a subagent's test run is a check, and the fact is only
    the half of `done · checked` that code holds; the model needs the message to name the check.
  - The landing checks a stop counter in the facts as well as the `prompt_id`, so a late answer
    for one stop never lands on the next stop of the same prompt.
  - `rules` takes the message and the facts, not the prompt, which no rule reads.
  - An outcome is kept only for a call that read something: a refused or failed call has no
    reading to fit a threshold on.
  - A turn with no prompt the user typed (after a resume) says `turn: unknown` rather than a
    length from 1970.
  - The rail refreshes on a settings change, so a new mode redraws the rows at once.
  - An outcome belongs to the session that stopped: a prompt in another session of the same
    terminal logs none (found drafting the scenario, whose sessions change without
    `SessionStart`).
  - D9 keeps whole sentences at the end and cuts the start at a word, not a fixed 150 and 147
    characters: the hook cuts before Marley redacts
    (PR-claude-redact-the-whole-text-before-cutting-it-001, found at Code's recall).
- **Review against each REQ and D1.** REQ-001, 002: `apply_evidence` demotes a model's
  `done_checked` without `checked_after_edit` (D1: the model can take a check away, never add
  one). REQ-003, 004, 012: a settled verdict makes no call and logs a `rules` row through
  `record`. REQ-005, 008: `row_word` by `StopKindShown`; shadow lands no label. REQ-006:
  `Landing::read` maps each `part_N_done` read `no` to its part. REQ-007: a no-signal,
  unavailable or refused reading lands nothing. REQ-009: `use_mode` off returns before anything.
  REQ-010: `end_turn` clears the labels. REQ-011: the state goes through `state_for`'s mask.
  REQ-013: `cut_ends`. Re-entrancy: `ask_stop_kind` runs inside the terminal view's update and
  reads only its workspace, which no update holds then, and globals; the landing is a global
  update in a later task. Provenance: nothing from Warp; no Zed function body carried over. Zed
  hunks: the settings item, the default and the docstring, each additive, the rows written first.
- **Found and changed at review.**
  - Parallel lead tools each report the one interrupt, which would have counted two stops and
    logged two rows: only the first interrupt of a turn counts, and the ask needs the seat to have
    been running (`before`).
  - A stop's outcome could have been logged against a prompt in another session of the same
    terminal, since a seat is the terminal's and a new session need not send `SessionStart`
    first: the last stop keeps its session, and a prompt in another one logs nothing.
- **Check, fmt and clippy.** #565's release install ended first (regress: all 38 passed,
  installed at 7780aa2f2d; its binary was built before any of this ticket's source changed).
  `cargo check -p marley_agent -p marley_system_one`, then `-p marley_workbench -p settings_ui`,
  clean after an unused `pop` result; `rustfmt` on the touched files; `just clippy marley_agent
  marley_system_one marley_workbench settings_ui settings_content` green after three first doc
  paragraphs were split (`too_long_first_doc_paragraph`) and `detail` became a `map_or_else`
  (`option_if_let_else`). Logs in the scratchpad's `566/`.


## Phase 3 — Test
- **Checklist** (no task tool): the scenario ✓; `mcp_agent fleet-labels` ✓; 547's stand-in ✓;
  `just build` ✓; the runs ✓; every shot read ✓; 515, 519, 547 and 565 again ✓; the golden
  entry ✓; the gate ✓; the golden set ✓.
- **The scenario**, `script/e2e/566-stop-kind.sh` (`compositor sway`, a headless sway of its own,
  so no key or click reached the user's session). #547's stand-in `claude`, named through
  `MARLEY_CLAUDE`, runs the plugin's real `event.py` for thirteen steps, each in a session of its
  own with a prompt id, one per Enter. The profile turns the layer on with the `replay` provider
  and the scratch repository listed; `replay.jsonl`, written in setup, holds six rows keyed by
  `stop_kind_N/1` and a `match` on the state (a decoy row for the question repeats, so a stop the
  rules settle would read `done · claimed` were it asked). The mode moves by 565's
  `system_one_setting`. `browser-fixture.sh` gains `mcp_agent fleet-labels`, which prints each
  seat's four stop labels, or `no stop kind`. `script/e2e/golden` gains the 566 entry (39).
- **Runs.** Run 1 passed every check up to the long message's, which failed on the check's own
  wording: the tail is the longest run of whole sentences that fits in 147 characters, so it held
  the message's last two sentences and ` … ` came before `I went through…`, not before the
  question. The check now names the start, ` … I went through`, and `does. Should I also add a
  license?`; each checked row's reading and state now print in the run's log. Run 2: every check
  passed (exit 0). Run 1 also showed Decisions' header saying `1 calls today`, #565's wording,
  which now says `1 call` for one (`decisions.rs`).
- **The shots** (run 2, read at full size and cropped to the row):
  - `566-01a-shadow-row`: the row reads `idle · Add a README to the proj…` over `Done; the tests
    pass.`; `fleet-labels`: `idle, no stop kind` (REQ-008).
  - `566-01-shadow`: Decisions: `1 call today · 0¢ spent of the 50¢ budget`, provider `replay ·
    jev-1.13.0`, key `not needed`, and the row `stop_kind · repo · replay · jev-1.13.0  would
    show: kind: done_checked 0.91; part 1 done: yes (0.95)` (REQ-008).
  - `566-02-done-checked`: `done · checked · Add a READM…`; `fleet-labels`: `stop_kind
    'done_checked', stop_kind_source 'model', stop_kind_confidence '0.91'` (REQ-001, REQ-010).
    After the next prompt: `working, no stop kind`, and an outcome row `next prompt after …`
    (REQ-010).
  - `566-03-done-claimed`: `done · claimed · Fix the typo in…` over `Fixed the typo; the tests
    pass.`: the replay said `done_checked` 0.93, and no command ran after the edit (REQ-002).
  - `566-04-asks-you-by-rule`: `asks you · Add a README`; the row: `stop_kind act rules
    stop_kind_0/1: kind: asks_you (rules)` (REQ-003).
  - `566-05-blocked-by-rule`: `blocked · Write the config file` over `I need permission to write
    the…`; the row: `kind: blocked (rules)` (REQ-004).
  - `566-06-suggest`: `idle · still going? · Wire the tests` (REQ-005).
  - `566-07-part-missing`: `done · checked · Add a READM…` over `not covered: "Add a license." ·…`;
    `fleet-labels`: `stop_parts_missing '["Add a license."]'` (REQ-006).
  - `566-08-no-row`: `idle · Tidy the imports`; the row: `kind: no signal (no replay row); part 1
    done: no signal (no replay row)` (REQ-007).
  - `566-09-off`: `idle · Rename the module`; the day's call rows unchanged (REQ-009).
  - `566-10-interrupted`: `interrupted · Run the slow tests`; the row: `kind: interrupted (rules)`
    (REQ-012).
  - `566-11-long-message`: `asks you · Summarize the chan…`; the row's state: `message: I went
    through the crates … and … I went through the crates one by one and wrote down what each of
    them does. Should I also add a license?`, the start cut at a word and the last two sentences
    whole, `kind: asks_you (rules)` (REQ-013).
  - The token step (no shot): the row's state reads `message: I set GITHUB_TOKEN=[redacted:
    secret] in the deploy environment.`, and the token is nowhere in it (REQ-011). The rail's row
    shows the message as the frame carried it, as it has since #519: the user's own screen.
  - `566-12-settings`: the Settings window's System One section: Check `Act`, then Stop Kind `Act`
    with its reset arrow and its description, then Decisions.
  - The shots show the stand-in terminal's default title, the user and host, which is why they
    stay in the scratchpad.
- **Again.** `just regress` on 515, 519, 547 and 565: all 4 passed. 547's agent bar offers `Update
  Marley's plugin` before the click and nothing after, the stand-in writing the shipped 1.3.0.
- **The gate.** `script/gates.sh --diff`: 16 passed, 0 failed, `GATE GREEN [diff]` (rustfmt,
  clippy on the scope with every target, cargo-audit, cargo-deny, cargo-shear, gitleaks,
  shellcheck, no-suppressions, source-bans, rustdoc, the Zed ledger, manifests, typos, semgrep,
  dylint, and the receipt); the full log is the scratchpad's `566/gate1.log`.
- **The golden set.** `just regress`: all 39 passed, `566-stop-kind` in 89 s (REQ-014).
- **The focus report.** Every run was in a headless sway of its own; nothing reached the user's
  session, and no scenario started the real `claude` (`MARLEY_CLAUDE` names the stand-in) or
  touched the keyring.
- **Out of reach**: a real model's readings (the replay rows stand in), and a real failing Bash,
  whose hook the reference leaves open.
- **Pre-existing, not in scope:** none. `marley_workbench.md`'s `unparseable` (#547's text), which
  typos flagged once this change touched the file, now reads `unparsable`.
- **Verdict:** PASS.


## Phase 4 — Complete
- **Documented.** `CHANGELOG.md` Added (the stop kind, and the plugin's 1.3.0). The architecture
  record: `marley_agent.md` (`TurnFacts` and the `turn` label, the row's `StopKindShown`, a
  section for `stop_kind.rs`, its consumers); `marley_system_one.md` (the seven sets and uses, the
  consumers, the scenario); `marley_workbench.md` (the stop kind under the hook events, the
  adapter's `use_mode`, `detail`, `record` and `outcome`, the plugin's `cut_ends` at 1.3.0, and
  #547's `unparseable` now `unparsable`); `docs/marley/guide.md` (the plugin's third bullet at
  1.3.0, a section "The stop kind" under System One, the uses line); `three-prong-plan.md`'s S1
  row. `zed-touchpoints.md` rows 55 (`marley.rs`), 58 (`marley_page.rs`) and 61 (`default.json`)
  checked against what shipped.
- **Knowledge appended.** F-claude-566-a-cut-that-kept-a-messages-end-could-part-a-secret-from-its-name-001
  (under the existing PR-claude-redact-the-whole-text-before-cutting-it-001),
  F-claude-566-parallel-tools-counted-one-interrupt-twice-001,
  F-claude-566-an-outcome-crossed-sessions-in-one-terminal-001,
  F-claude-566-an-earlier-stops-kind-would-show-at-a-later-stop-001,
  F-claude-566-decisions-said-1-calls-today-001;
  PR-claude-count-one-user-action-once-when-each-call-reports-it-001,
  PR-claude-an-answer-for-a-seat-names-the-session-prompt-and-stop-it-belongs-to-001;
  L-claude-566-the-folds-state-rides-on-the-seat-001,
  L-claude-566-a-use-whose-questions-vary-needs-a-set-per-shape-001,
  L-claude-566-the-hooks-reference-leaves-a-failing-bash-open-001,
  L-claude-566-an-edit-script-fed-through-the-heredoc-it-edits-ends-early-001;
  AD-claude-566-the-stop-kind-is-rules-first-and-rides-on-the-seat-001.
- **The brain.** Consultation 586b6b97fa4c4be58571ada24caee8c6 closed with
  `decisions/marleys-stop-kind-rules-first-the-kind-as-seat-labels-marley-566`, follow-up by
  2026-10-27.
- **Closed and archived.** TICKET-566 to `tickets/closed/`, the pair to `completed/`, and 569's
  references to `completed/566` and `completed/547`.
- **Commit.** The receipt of Test's green still matches the tree: nothing under `crates/`,
  `script/e2e/`, `script/gates.sh` or the manifests changed after it.
