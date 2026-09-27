---
pipeline_id: af7542ce-9014-4f52-a789-abff58316e76
ticket: docs/planning/tickets/open/TICKET-566-stop-kind.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "What a stopped turn needs"
type: feature
slice: prong 2 (attention); the Jev note's use 1, on #519's seats and #565's layer
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/completed/565-system-one-layer.spec.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/queued/538-notifications-with-content.spec.md, docs/planning/pipeline/queued/542-rail-attention-order.spec.md]
---

## Title
A `Stop` folds a Claude Code seat to `idle` with its last message (#519). This use names what the
stop needs: `done · checked`, `done · claimed`, `asks you`, `blocked`, `still going`, or nothing
(cannot tell). Code decides the clear cases from the turn's own events; #565's layer is asked the
rest with the short updates the rail shows, and a noul per part of the prompt says which parts
the message covers. The kind rides on the seat as a label, so the row, `fleet_snapshot`, #538's
banner and #542's order read one thing. Off by default; shadow logs; suggest marks; act shows.

## Scope
### In
- **Turn facts** (`marley_agent::claude_events`, pure): the fold keeps a `TurnFacts` beside the
  seat, reset by a user's prompt: when the turn started; tools by name with counts (lead and
  subagents apart); tool failures (`PostToolUseFailure`), and whether one was an interrupt;
  permissions asked and how many never finished; whether a Bash ran after the last Write, Edit
  or MultiEdit and did not fail (`checked_after_edit`); the turn's duration at `Stop`.
- **The local verdict** (`marley_agent::stop_kind`, pure): `rules(prompt, message, facts) ->
  Verdict`: `Interrupted` when the turn ended by an interrupt; `Blocked` when a permission asked
  in the turn never finished (denied or unanswered); `AsksYou` when the message's last sentence
  ends in a question mark or opens with an asking phrase (`should I`, `do you want`, `would you
  like`, `which`, `shall I`, `let me know`, `can you confirm`); else `Open`, with
  `checked_after_edit` as the fact the model's `done_checked` needs. A verdict other than `Open`
  makes no call.
- **The question set** `stop_kind/1` (in `marley_system_one::question`, pinned to `jev-1.13.0`):
  `kind`, a choice of `done_checked` ("the message reports the work done and names a check that
  ran: tests, a build, a command whose result it cites"), `done_claimed` ("the message reports
  the work done and names no check"), `asks_you` ("the message asks the user something or offers
  choices and waits"), `blocked` ("the message says it cannot continue: something it needs, a
  permission, a failure it could not fix"), `still_going` ("the message describes work under
  way or a next step it will take itself"), `cannot_tell`; and `part_N_done` nouls, one per part
  of the prompt (its sentences or list items, at most six): "The message says this part of the
  request is done: '<part>'", true when the message states or directly implies it, false when it
  is not mentioned or is left for later. With `Facts` detail (a metadata-only project) the parts
  are not asked.
- **The state** (D2): `project` (its name), `agent`, `tools` (`Bash 2, Edit 1, Read 3`),
  `failed tools`, `permissions pending`, `checked after edit` (yes or no), `turn` (its length in
  words) as facts; `prompt` and `message` as text, each the frame's 300 characters, masked.
- **The ask**: in `agent_events::on_frame`, after a lead `Stop` folds, when the use's mode is
  not `off` and the local verdict is `Open`: the state is built, `system_one::ask` runs with a
  2-second deadline, and the answer lands as labels through an `Upsert` of the seat's labels:
  `stop_kind` (`done_checked`, `done_claimed`, `asks_you`, `blocked`, `still_going`,
  `interrupted`; absent for cannot tell, no signal or unavailable), `stop_kind_source` (`rules`
  or `model`), `stop_kind_confidence`, and `stop_parts_missing` (the parts whose noul read
  absent, joined). A `done_checked` answer with `checked_after_edit` false is written as
  `done_claimed` (D1). The next user prompt clears the four labels.
- **The row** (`claude_events::seat_line`): in `act`, the word in place of `idle`: `done ·
  checked`, `done · claimed`, `asks you`, `blocked`, `still going`, `interrupted`; in `suggest`,
  `idle · asks you?`; in `shadow` and with no label, `idle`. `seat_activity` for an idle seat with
  `stop_parts_missing` shows `not covered: "<part>"` before the message.
- **Outcomes**: when the user's next prompt arrives in the seat, an outcome line names the
  stop's call with the delay and the prompt's length; when a seat with a `blocked` or `asks_you`
  kind gets a prompt within a minute, the line says so. These are the free labels the note counts
  on.
- The use registered with #565 (`UseSpec { name: "stop_kind", deadline: 2 s }`), its mode in
  `marley.system_one.uses`, its dropdown on the Marley page's System One section.
- `script/e2e/566-stop-kind.sh` on the `replay` provider.

### Out (explicitly deferred)
- The banner's wording (`asks you`) and the rail's order: #538 and #542 read the label when
  they land; this ticket changes no notification and moves no row.
- Acting on a kind: replying, nudging, restarting (the harness decider, M10).
- Stalls and loops while a turn runs (the note's use 4), routing a question to an owner or the
  manager (use 5).
- Codex, Gemini and OpenCode, whose hooks send no events yet.
- A Bash exit code from `tool_response`: `event.py` gains a `failed` flag on `PostToolUse` only
  if promotion finds that a non-zero exit arrives that way rather than as
  `PostToolUseFailure` (the notes' first measurement).
- Fitted thresholds and the golden report; the compiled-in 0.5 floor and the noul band stand.

## Reference (§20)
- **Warp:** its agent notifications give each tab an icon for the agent's state, "working,
  blocked, completed, or errored" (docs.warp.dev/agent-platform/capabilities/agent-notifications/,
  #519's reference); Marley's `idle` splits into what the stop needs. No Warp code was read.
- **Upstream Zed:** the Agent Panel's thread status (`marley_rail::thread_status`: waiting for
  confirmation, error, generating), which the rail shows for threads; a terminal row's stop gains
  a finer word the same way #519 gave it the four states.
- **Orca:** one "needs you" state and a "done and not yet seen" state (report 01 item 2, report
  05 §2.10); this use tells them apart on the message, which Orca never reads.

### Prior art
- **Behavior maps.** Report 01 §2.2 and item 2 (Orca's states from hooks; `interactivePrompt`
  kept for a question), report 05 §3 item 4 (acknowledge per turn); the Jev note's use 1 and
  its recommendation 4 (act "once recall on the classes that need Chad reaches 95% (an
  abstention counts as needing him) and 'done with evidence' is 90% precise"); #542's classes
  (needs you, done and not seen, working, not reporting, idle) which the label feeds.
- **Published material.** jev-gates' done check (github.com/rashedInt32/jev-gates, in the
  note): finished parts of a task scored 0.94 or more and a silently skipped part 0.03, the
  pattern the parts nouls follow. TypeSafe's guidance in the note: split a decision into narrow
  questions with written criteria (95.0% against 62.6% on one broad question), a rule on the
  question is followed where the same rule in a preamble is not, a phrase computed in code helps
  where numbers hurt, and "Describe situations, not degrees" for a score's levels. Claude Code's
  hooks reference (code.claude.com/docs/en/hooks): `Stop` carries `last_assistant_message`,
  `PostToolUseFailure` carries `is_interrupt`, `PermissionRequest` names the tool.
- **The code we already ship.** `crates/marley_agent/src/claude_events.rs`: `HookEvent` (55),
  `fold` (141), `Moving::take` with the `Stop` arm (314-319) and `PermissionRequest` arm
  (303-313), `tool_ends` with the interrupt (367-393), `end_turn` (414), the labels
  (`PROMPT_LABEL` 35, `TOOL_LABEL` 37, `MESSAGE_LABEL` 39, `WAITING_ON_LABEL` 51, the tool
  prefixes 47-49), `seat_status` (172), `seat_line` (189) and `seat_activity` (214), which the
  row reads. `crates/marley_workbench/src/agent_events.rs`: `AgentEvents` (21), `seat` (30),
  `on_frame` (83: the fold and `apply`, the place the ask starts), `now_ms` (73).
  `crates/marley_fleet/src/session.rs:55-77` (`Session::labels`, opaque chips rendered and never
  matched by the fleet) and `reducer.rs:17` (`Upsert` carries labels). The rail:
  `crates/marley_workbench/src/rail.rs:1717-1785` (`terminal_snapshot`: the seat at 1735,
  `seat_status` at 1743, `seat_line` at 1769, `activity` at 1781); the row's lines at 1371.
  The hook: `crates/marley_workbench/claude_plugin/marley/hooks/event.py:57-90` (`summary`:
  `prompt`, `tool`, `preview`, `tool_use_id`, `is_interrupt`, `message`, `error`). #565's layer
  (`system_one::ask`, `StateBuilder`, the log and replay) and its `Mode`. Does a crate we build
  own the seam? `marley_fleet` owns the labels, `claude_events` the fold and the row's words,
  #565 the ask; this ticket adds the facts, the rules and one question set.

## UI proof
UI-AFFECTING: the agent's rail row, and the Decisions view.
`script/e2e/566-stop-kind.sh` (`compositor sway`, for the Decisions view's rows). #519's
stand-in `claude` acts out stops; the profile enables #565's layer on `replay` with the
repository listed and `uses.stop_kind` set per step by 565's `system_one_setting` helper; the
replay file holds rows keyed by set and a `match` on the state. Shots:
- `566-01-shadow`: mode `shadow`; a prompt, an Edit, a Bash, then `Stop` "Done; the tests pass":
  the row reads `idle · Add a README…` as today; the Decisions view lists the call with `would
  show: done · checked (0.91)`.
- `566-02-done-checked`: mode `act`; the same session replayed: `done · checked · Add a README…`.
- `566-03-done-claimed`: a turn with an Edit and no Bash after it, the replay answering
  `done_checked` 0.93: the row reads `done · claimed` (code's fact demoted it).
- `566-04-asks-you-by-rule`: `Stop` "Should I also add a license?": `asks you`; the log's row has
  `source: rules` and no provider call.
- `566-05-blocked-by-rule`: a `PermissionRequest` for Write, then `Stop` with the tool never
  finished: `blocked`, no call.
- `566-06-suggest`: mode `suggest`; a stop the replay reads `still_going` 0.88: `idle · still
  going?`.
- `566-07-part-missing`: a prompt with two parts ("Add a README. Add a license."), the replay's
  `part_2_done` at 0.05: the activity line `not covered: "Add a license."`.
- `566-08-no-row`: a stop with no replay row: the row stays `idle`; the log's row reads
  `NoSignal: no replay row`.
- `566-09-off`: mode `off`; a stop: `idle`, and no row is written.
Checks: `mcp_agent fleet` (#547's command) shows the seat's `stop_kind` label after
`566-02`; `holds` on the day's file for each expected reading; the outcome line after the next
prompt in `566-02`'s session.

## Locked-In Decisions
- D1: Code decides the clear cases and holds the evidence ("local first and then jev second"):
  a question in the message is `asks you`, a permission never finished is `blocked`, an
  interrupt is `interrupted`, none of which asks the model; `done · checked` needs code's
  `checked_after_edit`, so the model can demote a claim but never promote one.
- D2: The state is the rail's short updates and nothing more (Chad, 2026-09-26): the prompt and
  the message as the frame cut them (300 characters), masked by #516's redactor; the tools'
  names and counts; the facts as a phrase. No transcript, no file, no tool output. A
  metadata-only project sends the facts and the tool names, and the parts are not asked.
- D3: The kind is a seat label. Labels are the fleet's opaque chips (`Session::labels`), the
  rail already reads the seat, `fleet_snapshot` publishes labels (#547 D5), and #538 and #542
  can read `stop_kind` without a new seam.
- D4: Display only. The use changes a word and an activity line; it sends no notification and
  moves no row (those are #538's and #542's, reading the label), and nothing acts on a kind
  (the note's safety rule 6: a use that reads agent output only displays, ranks or routes).
- D5: The modes read as #565 defines them: `off` makes no call and writes no label; `shadow`
  logs and shows nothing; `suggest` shows the word after `idle` with a question mark; `act` shows
  it in place of `idle`. A `cannot_tell`, a no-signal or an unavailable reading leaves `idle`.
- D6: One ask per `Stop`, deduped by the state's hash, with a 2-second deadline that blocks
  nothing: the row reads `idle` until the answer lands. About 300 calls and 5 cents a day at
  five agents (the note's budget row).
- D7: The parts nouls are the check that a turn skipped something (jev-gates' pattern): a part
  under the noul band's floor is named on the row; a part inside the band says nothing.
- D8: Outcomes are the user's next actions, logged and never shown: the delay to the next
  prompt and its length. They are the labels the golden report (Out) will fit thresholds on.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the use is in `act`, WHEN a `Stop` arrives whose replay reads `done_checked` and a command ran after the turn's last edit without failing, the row shall read `done · checked`. | Shot `566-02-done-checked` |
| REQ-002 | WHEN the model reads `done_checked` and no command ran after the last edit, the row shall read `done · claimed`. | Shot `566-03-done-claimed` |
| REQ-003 | WHEN the message ends in a question, the row shall read `asks you` and the system shall make no call. | Shot `566-04-asks-you-by-rule`; the log row's source and the replay row left unconsumed |
| REQ-004 | WHEN a permission asked in the turn never finished before `Stop`, the row shall read `blocked` and the system shall make no call. | Shot `566-05-blocked-by-rule`; the log |
| REQ-005 | WHERE the use is in `suggest`, the row shall read `idle · <kind>?`. | Shot `566-06-suggest` |
| REQ-006 | WHEN a part of the prompt reads absent, the activity line shall name it as not covered. | Shot `566-07-part-missing` |
| REQ-007 | WHEN the reading is no signal, unavailable or cannot tell, the row shall read `idle` and the log shall say which. | Shot `566-08-no-row`; the log row |
| REQ-008 | WHERE the use is in `shadow`, the row shall read `idle` and the Decisions view shall list the call with the word it would have shown. | Shot `566-01-shadow` |
| REQ-009 | WHERE the use is `off`, the system shall make no call and write no row. | Shot `566-09-off`; the day's file |
| REQ-010 | WHEN a kind is set, `fleet_snapshot` shall carry `stop_kind`, `stop_kind_source` and `stop_kind_confidence` on the seat, and the next user prompt shall clear them. | The run log: `mcp_agent fleet` after `566-02` and after the next prompt |
| REQ-011 | WHEN the state's text holds a secret, the system shall send and log it masked. | The log row of a stop whose message carries a token assembled at run time |
| REQ-012 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion: #565 shipped;
  `brain_ask`; the live measurement of how a failing Bash reaches the hooks on the installed
  Claude Code (a Python pty, as #519's live check).
- **P2 Code:** the settings page's dropdown row (the ledger row for `marley_page.rs`, first);
  `TurnFacts` and the rules in `marley_agent`; the set in `marley_system_one`; the ask, the labels
  and the outcomes in `agent_events`; the row's words; fmt and clippy clean; a review of the
  diff against each REQ and D1.
- **P3 Test:** write and run the scenario and read every shot; rerun 519 and 547 (their rows'
  words are unchanged with the use off); `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_agent.md`,
  `marley_workbench.md` and `marley_system_one.md`; the plan's C1 row; the ledger capture; close
  the ticket, archive, commit.
