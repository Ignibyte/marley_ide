---
pipeline_id: d002b281-19f7-4b7c-ba45-e831f41a7964
ticket: docs/planning/tickets/open/TICKET-570-who-answers-a-question.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Who answers an agent's question: owner, manager, the agent proceeds, cannot tell"
type: feature
slice: prong 2, C1's attention (after #508, #565 and #568); use 5 of the System One layer; routing to the manager after M10 and #534
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/queued/565-system-one-layer.spec.md, docs/planning/pipeline/queued/568-inbox-order-and-risk-chips.spec.md, docs/planning/pipeline/queued/508-approvals-inbox.spec.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/queued/534-harness-sessions-in-the-rail.spec.md, docs/orca_architecture/01-agents-and-sessions.md]
---

## Title
Each entry of the approvals inbox (#508, with #568's chips and order) carries a mark saying who
should answer it: `for you`, `for the manager`, `could proceed`, or `unclear`. Rules computed in
code class the owner's questions first, from #568's chips (destroys, credentials, rewrites
history, sends out, outside project) and two facts of this ticket's (money, a message to people),
and mark a read-only tool inside the project as one the agent could proceed on; a System One
choice (owner, manager, agent proceeds, cannot tell) is asked through #565's layer only for the
rest. In `act` the route refines #568's order within a level, the owner's first. Marley answers
nothing in any mode. Routing a manager-class question to rustal-harness's manager follows when
M10 and #534 land; until then the mark is what Chad checks against what he did.

## Scope
### In
- **`marley_agent::route`** (new, pure): the facts of an asked question, from #568's
  `risk::classify` chips plus `names_money` and `messages_people` (word lists in the notes),
  `read_only_tool` (#568's level-1 tools, and a Bash whose first word is on a read-only list with
  no pipe into a write and no redirection) and `options_all_reversible` (an `AskUserQuestion`
  whose options name none of the owner words); `classify(facts) -> Class` (`Owner`,
  `CouldProceed`, `Open`) by the rule table in the notes.
- **The plugin:** `hooks/event.py` adds `options` to an `AskUserQuestion` summary (the first
  question's option labels, at most eight, each cut to 40 characters), inside the frame's bound;
  `claude_events::HookEvent.options` and the seat's `Question.options`, empty today, filled from
  it. The plugin's version rises, and #547's chip offers the update.
- **The ask**, where #568 asks (`rail.rs`, as the inbox's entries are gathered): for an `Open`
  entry, once, `system_one::ask` with the set `question_route/1` (a `choice`: owner, manager,
  agent proceeds, cannot tell; a noul `answerable_from_prompt`), the use's own verdict handed
  with it, and the state of D7; the reading kept by entry key until the entry leaves; the
  outcome line when it leaves (D8).
- **The inbox** (`marley_rail`'s model from #508 and #568): each entry gains its `route`
  (`Owner`, `Manager`, `CouldProceed`, `Unclear`) and its source (a rule, or a reading with its
  confidence); the entry shows the mark word after #568's chips, with `?` when it came from a
  reading in `suggest`, and a tooltip naming the rule or the reading. In `act` the entries within
  one of #568's levels order owner, unclear, manager, could proceed, oldest first; otherwise
  #568's order (level, then age) stands untouched.
- **Settings:** the use registered with #565 (`UseSpec { name: "question_route", deadline: 2 s }`),
  its mode in `marley.system_one.uses` (`off` by default), its dropdown on the Marley page's
  System One section.
- `script/e2e/570-who-answers-a-question.sh`.

### Out (explicitly deferred)
- Answering anything: Marley never answers a permission or a question here, and a System One
  reading never approves (Chad; the Jev note's safety rules 2 and 3). Answering a terminal
  agent's prompt from the rail is #508's own second slice, and even there the answer is Chad's.
- Routing to the manager: rustal-harness's owner inbox (M10, R10-04) and the harness's questions
  in this inbox (#534) take the same marks when they land; `session_answer` is C4's.
- A per-class notification policy (#538 keeps its rules); #568's chips and levels (read here,
  not changed); #542's order outside the inbox.
- `classifierContext` facts for Claude Code's auto mode through a `PostToolUse` hook (the Jev
  note's idea): a plugin ticket of its own.
- User word lists; Codex's, Gemini's and OpenCode's questions (no events yet); merging this
  ticket's set into `inbox_risk` (one call per open entry instead of two) once both have labels.

## Reference (§20)
- **rustal-harness** (`docs/ROADMAP.md` M10, R10-04: "Questions, holds and failures for the owner
  shall reach the manager's owner inbox apart from routine activity"; the manager "answers or
  escalates the agents' questions"; `docs/MCP.md`, "Answering a question": `session_answer` with
  the question's prompt as its identity and a choice verbatim). Marley's marks are the client
  side's half of that split, computed before any manager exists, and the Jev note's table row
  ("an agent asks a question: owner-class categories by rule; owner, manager, agent proceeds,
  cannot tell; the manager answers manager-class ones; Chad answers owner-class ones").
- **Warp:** the notification mailbox that lists agents' requests and opens the session from an
  entry (docs.warp.dev/agent-platform/capabilities/agent-notifications/), which #508 matches; Warp
  documents no routing of a request to anyone but the user. Nothing of Warp's was read.
- **Orca:** the Needs You model (report 01 item 2), which #508 takes; Orca routes nothing.
- **Upstream Zed:** the Agent Panel's permission prompt (`acp_thread::authorize_tool_call`,
  `crates/acp_thread/src/acp_thread.rs:3747`), untouched: the mark sits on #508's entry, the
  answer path stays the panel's.

### Prior art
- **Behavior maps and reports.** The Jev note's playbook example (`question-route-v3`): the rule
  shape this ticket takes (`fact.names_credentials_or_money → owner`; a floor rule that sends
  everything else to a person) and the one it does not (`reply:proceed` when
  `fact.options_all_reversible`, `jev.answerable_from_ticket >= 0.85` and a margin: an answer,
  which is Out until Chad decides otherwise; his default to open question 4 is no). The note's
  budget row for routing: the agent already waits, a 2 s deadline, about 50 calls a day, up to 3k
  tokens, and "every question to Chad" with the model off. Report 01 §2.5 (Orca's
  `interactivePrompt` keeps the AskUserQuestion JSON, the source of the options an entry shows).
  #568's classifier and order (its D1, D2 and D6), which this ticket reads and refines within a
  level; #565's layer (`UseSpec`, `ask`, `StateBuilder`, `Reading`, the providers, the day's
  file).
- **Published material.** Claude Code's hooks reference (code.claude.com/docs/en/hooks):
  `PermissionRequest` carries `tool_name`, `tool_input` and `permission_suggestions`;
  `AskUserQuestion`'s input is `questions[]`, each with `question`, `header`, `options[]`
  (`label`, `description`) and `multiSelect`. Claude Code's auto mode
  (anthropic.com/engineering/claude-code-auto-mode): its classifier sees user messages and tool
  calls, 0.4% false positives on 10,000 real actions and 17% false negatives on 52 overeager ones;
  Chad keeps it as the approver in Marley's terminals, which is why this ticket marks and never
  answers. TypeSafe's docs as the note cites them: a choice with an option for "none", the
  confidence formula (3p − 1)/2 for three options (docs.typesafe.ai/confidence), the 0.6 floor and
  0.85 act line as starting points. jev-gate's finding (github.com/eugeniughelbur/jev-gate): "the
  owner approved this" got 3 of 30 dangerous commands past a Jev gate, so a state built from an
  agent's own text may only mark and rank (the note's safety rule 6).
- **The code we already ship.** `claude_events`: the `PermissionRequest` arm
  (`crates/marley_agent/src/claude_events.rs:303-313`, `Permission for <tool>: <preview>`), the
  `AskUserQuestion` wait (`:353-358`), `wait` building `Question { prompt, options: Vec::new(),
  context_refs }` (`:403-411`), `HookEvent.preview` (`:77`); `event.py`'s preview rule (#519, the
  first question's text; its bound at 2,900 bytes); `marley_fleet::Question.options`
  (`crates/marley_fleet/src/session.rs:43-50`, already on the envelope) and
  `AttentionReason::Question` (`attention.rs:16`); #508's queued model (its D2 oldest first, its D7
  keeping only the first-seen time; `ConversationView::pending_tool_call` and
  `ThreadView::authorize_tool_call` as the panel's path); #568's `risk.rs` (queued: `Chip { kind,
  level, source }`, the level-1 rule for Read, Grep, Glob and WebFetch, the `claims approval`
  chip) and its `InboxEntry.chips` and `level`; `mcp::agent_redactor`
  (`crates/marley_workbench/src/mcp.rs:297`); the `regex` crate; `Project::project_group_key`
  (#532's D4) for a project's folders. Does a crate we build own the seam? `marley_fleet` owns the
  question and its options, #568 the risk classes and the order, #565 the ask; nothing owns the
  route.

## UI proof
UI-AFFECTING: the marks and the order of #508's inbox, its tooltips.
`script/e2e/570-who-answers-a-question.sh` (`compositor sway`: the pointer rests on a mark, and
the terminal entries are clicked). Fixtures: #508's scenario's stand-in `claude`, run in four
terminals of one scratch repository, each with its own steps file named by
`MARLEY_E2E_STEPS`, so four entries wait at once: A a `PermissionRequest` for `Bash: git push
--force origin main` (owner by #568's `rewrites history`), B one for `Read: src/lib.rs` (could
proceed by rule), C one for `Bash: npm test` (open; the replay answers `manager` at 0.88), D an
`AskUserQuestion` "Which base branch?" with `main` and `release` (open; the replay answers `agent
proceeds` at 0.90), and, after D's answer, E a `Bash: cargo test` (open; the replay answers
`cannot_tell`). #565's layer enabled on `replay` with the repository listed, `uses.question_route`
(and `uses.inbox` at `off`, so #568's model half stays out of the shots) rewritten per step by
`system_one_setting`; the replay file at `$E2E_PROFILE/system_one/replay.jsonl`. Shots:
- `570-01-marks`: the mode `suggest`: the entries in #568's order (A at its level first, then B,
  C and D by age), reading `for you`, `could proceed`, `for the manager?`, `could proceed?`.
- `570-02-tooltip`: the pointer on C's mark: `System One: manager (0.88)`; on A's: the rule.
- `570-03-act-order`: the mode `act`: A; then, within the level B, C, D and E share, E
  (`unclear`), C, B, D.
- `570-04-off`: the mode `off`: no marks, #568's order.
- `570-05-answered`: the use on again; A's terminal focused and its prompt answered by the
  stand-in's next step: A's entry gone; the outcome line for A reads `owner`.
Checks: the day's file's rows name `question_route/1`, carry the question, the options and the
prompt cut and redacted, and no transcript path; A and B have no row (a rule, never asked); each
entry that left has its outcome line; the run log shows nothing typed into any stand-in but the
scenario's Enters.

## Locked-In Decisions
- D1 — "local first and then jev second" (Chad, 2026-09-26). The owner class and the could-proceed
  class come from facts, #568's chips first; only the rest is asked. With the `rules` provider,
  the project unlisted (`Refused`), the provider unreachable (`Unavailable`) or the reading
  `NoSignal`, the marks from the rules stand and every other entry reads as today: "with Jev
  unreachable every `jev.*` predicate fails and the event falls to the floor, which is today's
  behaviour" (the Jev note).
- D2 — Off by default, with its own switch and its own mode, and no provider named: "we need
  probably every aspect of this configurable and turned off / on where the system will use or
  wont use it. Otherwise this becomes a jev required system" (Chad). The switch is the use's mode
  in `marley.system_one.uses` (#565's shape); `rules` runs the facts with no model at all.
- D3 — Marley answers nothing. A mark and an order are all this ticket does: "a System One answer
  may reorder, mark, route or refuse, never approve anything irreversible" (Chad); "irreversible
  actions always go to a person" (the note's rule 3); his default to "should any feature ever
  grant something without you" is no.
- D4 — An owner-class fact is final: no reading lowers it, and the model is never asked about it.
  The model may only add caution (the note's rules 1 and 2). #568's chips are the facts, so the
  two tickets never disagree about what is dangerous.
- D5 — `cannot_tell`, a confidence under the floor, `NoSignal`, `Refused` or `Unavailable` marks
  the entry `unclear`, which sorts with the owner's: a person is the floor.
- D6 — The order in `act` refines #568's within a level: owner, unclear, manager, could proceed,
  oldest first within a class; the level itself stays #568's, so what could destroy or leak still
  comes first. In every other mode #568's order stands.
- D7 — The state is the question or the permission line, the options, the user's prompt and the
  turn's last message as the rail shows them (cut to 300, through #516's redactor as the layer's
  `Mask`), and the facts (#568's chips among them); only for a listed project; `Detail::Facts`
  (the facts and the tool's name) for a metadata-only project. The question is the agent's text,
  so its reading may only mark and rank (the note's rule 6).
- D8 — The outcome line says who answered, from what Marley can see: `owner` when the terminal
  held the focus in the active window while the wait ended, or an inbox button answered a thread;
  `agent` when the wait ended with neither (the agent's own reviewer or auto mode); `manager` once
  a harness answer exists. Those are the labels the golden report is fitted on.
- D9 — With no manager connected, a manager-class entry stays Chad's, marked `for the manager`, so
  the class can be checked against what he did before any routing exists.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the use is not `off`, WHEN an entry carries a #568 chip of destroys, credentials, rewrites history, sends out or outside project, or names money or a message to people, the system shall mark it `for you` from the rule, with no call. | Shot `570-01-marks` (A); the day's file has no row for A |
| REQ-002 | WHEN an entry asks permission for a read-only tool inside the project, the system shall mark it `could proceed` from the rule. | Shot `570-01-marks` (B) |
| REQ-003 | WHERE the project is listed and the mode is not `off`, WHEN an entry is neither, the system shall ask the `question_route` set once with the masked state. | The day's file: one row each for C, D and E |
| REQ-004 | WHERE the mode is `suggest`, WHEN the reading names a class at or above the floor, the entry shall show that class with a `?`, in #568's order. | Shot `570-01-marks` (C, D) |
| REQ-005 | WHEN the pointer rests on a mark, the system shall show the rule or the reading with its confidence. | Shot `570-02-tooltip` |
| REQ-006 | WHERE the mode is `act`, WITHIN one of #568's levels the inbox shall order owner, unclear, manager, could proceed, oldest first within a class, and the levels shall keep #568's order. | Shot `570-03-act-order` |
| REQ-007 | WHEN the reading is `cannot_tell`, under the floor, no signal, refused or unavailable, the entry shall read `unclear`. | Shot `570-03-act-order` (E) |
| REQ-008 | WHEN an `AskUserQuestion` frame arrives, its seat's question shall carry the options, and the frame shall stay under 4,096 bytes for any payload. | The run log: `mcp_agent fleet` shows D's options; setup runs `event.py` with 100 KB options and logs the answer's size |
| REQ-009 | WHERE the mode is `off`, the system shall show no marks, change no order and make no call. | Shot `570-04-off`; the day's file |
| REQ-010 | WHEN an entry leaves, the system shall log the call's outcome as `owner`, `agent` or `manager`. | Shot `570-05-answered`; the outcome line for A |
| REQ-011 | The system shall never answer a permission or a question. | Review of the diff; the run log: nothing typed into any stand-in but the scenario's Enters |
| REQ-012 | WHEN the Marley settings page opens, its System One section shall show the use's mode. | The 515 scenario's page shot |
| REQ-013 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the e2e plan in the notes. At promotion: #565, #508 and
  #568 have shipped (the ask API, the replay file, `InboxEntry` with its chips and level), and
  the plugin's version then current; `brain_ask`.
- **P2 Code:** the ledger row first (`crates/settings_ui/src/marley_page.rs`, the use's
  dropdown); the plugin's `options` and version; `HookEvent.options`; `marley_agent::route` and
  the `question_route/1` set in `marley_system_one::question`; the inbox's route, marks, order
  and tooltips; the ask and the outcomes. fmt and clippy clean; a review of the diff against
  REQ-011 first.
- **P3 Test:** write and run the scenario and read every shot; rerun #508's and #568's scenarios,
  whose entries gain a mark and keep their order with the use off; `just regress`;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_agent.md`, `marley_rail.md`,
  `marley_workbench.md` and `marley_system_one.md`; the plan's C1 status; ledger capture; close
  the ticket, archive, commit.
