# What a stopped turn needs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-566-stop-kind.md
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

### Design
- **`TurnFacts`** (`claude_events`, kept in `Moving` and stored on the seat as one label,
  `turn`, in a compact `key=value` form the reader parses back; the reducer treats it as opaque):
  `started_ms`, `tools: BTreeMap<String, u32>` (lead), `subagent_tools: u32`, `failures: u32`,
  `interrupted: bool`, `permissions_asked: u32`, `permissions_pending: u32` (a `PermissionRequest`
  whose tool call has no `PostToolUse` or `PostToolUseFailure` yet), `last_edit_at: Option<u64>`
  (the sequence of the last Write, Edit or MultiEdit), `checked_after_edit: bool` (a Bash whose
  `PostToolUse` came after `last_edit_at`). `UserPromptSubmit` by the user resets it; a harness
  prompt does not.
- **`stop_kind`** (`crates/marley_agent/src/stop_kind.rs`, pure): `Verdict { Interrupted,
  Blocked, AsksYou, Open }`, `rules(prompt, message, &TurnFacts) -> Verdict`;
  `parts(prompt) -> Vec<String>` (split on sentence ends and list markers, at most six, each
  trimmed); `Kind` (the six plus `Interrupted`) with its label value and its row words;
  `apply_evidence(kind, facts) -> Kind` (the demotion of D1); `labels(kind, source, confidence,
  missing_parts)`; and the constants `STOP_KIND_LABEL`, `STOP_KIND_SOURCE_LABEL`,
  `STOP_KIND_CONFIDENCE_LABEL`, `STOP_PARTS_MISSING_LABEL`, which `take`'s `UserPromptSubmit` arm
  clears with message and error.
- **The set** (`marley_system_one::question::STOP_KIND_1`): the `kind` choice with the option
  descriptions of the spec, and a template noul `part_N_done` filled with each part's text.
- **The ask** (`agent_events::on_frame`): after `apply`, if the event is a lead `Stop`, the
  mode is not `off` and `rules` gives `Open`: `StateBuilder::new(project, detail)` with the facts
  and the two texts (the project from the terminal's workspace root, as #538's title takes it);
  `system_one::ask(&STOP_KIND, state, verdict, cx)` with the seat as the dedupe subject; the task
  is detached and, on its answer, updates the global: if the seat's `session_id` and turn are
  the ones asked (the labels carry the `prompt_id`), an `Upsert` with the seat's labels plus the
  kind labels is applied, and the rail's observers redraw. A `Rules` verdict other than `Open`
  sets the labels at once with `source: rules` and logs a row through the layer's `rules` path
  (D6 of #565: every verdict is a row).
- **The row**: `seat_line` reads `stop_kind` for an `Idle` seat by mode (`act`: the word; `suggest`:
  `idle · <word>?`; else `idle`); `seat_activity` prefixes `not covered: "<part>"` when
  `stop_parts_missing` is set. The mode reaches the pure function as an argument, read from
  `MarleySettings` in `terminal_snapshot`.
- **Outcomes**: `on_frame`, on a user's `UserPromptSubmit`, sends an `OutcomeRow` naming the
  seat's last stop call (kept in the global by seat) with `after_ms` and `prompt_chars`.
- **File manifest.** Marley: `crates/marley_agent/src/claude_events.rs`, `stop_kind.rs` (new),
  `marley_agent.rs` (the module); `crates/marley_system_one/src/question.rs`;
  `crates/marley_workbench/src/agent_events.rs`, `rail.rs`, `system_one.rs` (the use's
  registration), `script/e2e/566-stop-kind.sh`. Zed: `crates/settings_ui/src/marley_page.rs`
  (the dropdown row; its ledger row widened), `assets/settings/default.json` only if the block
  lists uses by name (565 decides; its row widened if so).
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `marley_page.rs` row names the
  `stop_kind` mode item.

### E2E plan
Fixtures: #519's scratch repository, HOME and stand-in `claude`, whose steps file gains the
sessions below; the replay file written by setup with rows matched on the prompt's text; the
layer enabled on `replay` with the repository listed; the mode set per step with
`system_one_setting uses {"stop_kind": "<mode>"}` before the step's stop.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-008 | mode `shadow`; SessionStart, prompt "Add a README to the project", PreToolUse Edit, PostToolUse, PreToolUse Bash `cargo test`, PostToolUse, Stop "Done; the tests pass." (replay: `done_checked` 0.91, parts 0.95) | `566-01-shadow` (the row; then `marley: decisions` for the row) |
| REQ-001, REQ-010 | mode `act`; the same session again (a new `session_id`) | `566-02-done-checked`; `mcp_agent fleet` shows the labels |
| REQ-002 | a prompt "Fix the typo", Edit, Stop "Fixed; tests pass." (replay `done_checked` 0.93) | `566-03-done-claimed` |
| REQ-003 | a prompt, Stop "Should I also add a license?" | `566-04-asks-you-by-rule`; the day's file: `source: rules`, no `replay` provider row |
| REQ-004 | a prompt, PreToolUse Write, PermissionRequest Write, Stop "I need permission to write the file." | `566-05-blocked-by-rule`; the log |
| REQ-005 | mode `suggest`; a prompt, Stop "Next I will wire the tests." (replay `still_going` 0.88) | `566-06-suggest` |
| REQ-006 | mode `act`; prompt "Add a README. Add a license.", Edit, Bash, Stop "Added the README." (replay `done_checked` 0.9, `part_1_done` 0.94, `part_2_done` 0.05) | `566-07-part-missing` |
| REQ-007 | a prompt with no replay row, Stop | `566-08-no-row`; the log: `NoSignal: no replay row` |
| REQ-009 | mode `off`; a prompt, Stop | `566-09-off`; the day's file unchanged |
| REQ-010 | the next prompt in `566-02`'s session | `mcp_agent fleet`: no `stop_kind`; the outcome line in the day's file |
| REQ-011 | a Stop whose message carries a `ghp_` token assembled at run time | the log row's masked state |
| REQ-012 | Test's gate and regression runs | the exit codes |

What no scenario reaches: a real model's readings (the replay rows stand in; the note's
recommendation 4 sets the bar before `act` is the default for anyone), and a real failing Bash
(the promotion measurement).

### Risks
- A Bash that fails: whether Claude Code reports it as `PostToolUseFailure` or as a
  `PostToolUse` with an error in `tool_response` decides `failures` and `checked_after_edit`.
  Measured at promotion on the installed 2.1.283; if the latter, `event.py` gains `failed: true`
  on such a `PostToolUse` (the summary's bound is unaffected: one short field).
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
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
