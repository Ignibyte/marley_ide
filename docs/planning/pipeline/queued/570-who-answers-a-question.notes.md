# Who answers an agent's question: owner, manager, the agent proceeds, cannot tell — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-570-who-answers-a-question.md
- **Pipeline spec:** 570-who-answers-a-question.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-26, approving the seven ranked uses of the Jev note
  (`docs/planning/design-notes/jev-system-one-2026-09-25.md`). Use 5: "owner-class categories by
  rule, then a choice (owner, manager, agent proceeds, cannot tell); ranks the inbox; routes once
  rustal-harness M10's manager runs; waits on #508, #534; M". Drafted by the second spec drafter
  beside #565 and #566 to #568, and aligned to #568 (the inbox's risk chips and order) once it
  was on disk: the two tickets share one inbox, so this one reads #568's chips as its facts and
  refines the order only within a level.
- **Classification / tier:** feature, prong 2 (C1's attention). Marley crates and the plugin,
  plus the page's dropdown for the use's mode. Size M.
- **Recall (§18.3):**
  - AD-claude-519 (the seat is the events' fold; labels are rendered, the fold's own labels are
    matched by their owner): the route is kept beside the entry, not in the seat.
  - F-claude-519 and PR-claude-drop-the-unbounded-fields-first-001: the new `options` field joins
    the frame's drop order after the transcript path and the working directory, before the
    message, so a long option list never pushes out what the row shows.
  - AD-claude-516 and PR-claude-redact-the-whole-text-before-cutting-it-001: the state's texts are
    redacted whole, then cut.
  - L-claude-482 (a plugin change reaches an install only through an update): #547's chip.
  - #508's queued notes: the entries' sources and the clearing rule (D5 there), which this ticket's
    outcome line reads; #568's D1 to D3 and D6 (the chips and the order this ticket builds on);
    #565's D1 and D5 (the verdict handed with each ask; a compiled-in set).
  - Brain: not consulted in this drafting session; promotion asks.
- **Discovery** (at `ca70b6488d` with #547's working-tree changes; promotion re-verifies):
  - `crates/marley_agent/src/claude_events.rs`: `HookEvent` (55-90), the `PermissionRequest` arm
    (303-313), `tool_starts` with `AskUserQuestion` (353-358), `wait` (403-411), `tool_line`
    (461-467), `one_line` (470).
  - `crates/marley_workbench/claude_plugin/marley/hooks/event.py`: the summary's fields and the
    drop order (#519's D, the notes' Reds); `hooks.json`; `plugin.json` 1.2.0.
  - `crates/marley_fleet/src/session.rs`: `Question { prompt, options, context_refs }` (43-50);
    `reducer.rs`: `QuestionRaised` (51-58), `apply` (147-208).
  - `crates/marley_workbench/src/agent_events.rs`: `on_frame` (83-106).
  - #508's spec (queued): the inbox's model in `marley_rail`, entries from the Agent Panel
    (`ConversationView::pending_tool_call`, `ThreadView::authorize_tool_call`) and from #519's
    seats, oldest first, `Rail::activate_terminal` (`rail.rs:577`).
  - #568's spec (queued): `marley_agent::risk::classify(tool, preview, cwd, project_folders) ->
    Vec<Chip>` with `Chip { kind, level, source }` (destroys 5, credentials 5, rewrites history 5,
    sends out 4, installs 4, outside project 3, claims approval +1); a Read, Grep, Glob or WebFetch
    with no chip is level 1, any other tool with no chip level 2; `InboxEntry.chips` and `level`;
    the order by level then age; the ask in `rail.rs` where the entries are gathered, keyed by
    the entry; `UseSpec { name: "inbox", deadline: 600 ms }`.
  - #565 (queued): `UseSpec`, `system_one::ask`, `StateBuilder`, `Detail`, `Reading`, the day's
    file, the replay file and `system_one_setting`.
  - `crates/marley_workbench/src/mcp.rs`: `agent_redactor` (297-303), `for_agents` (306-314).
  - `crates/settings_ui/src/marley_page.rs` (42-91).
  - rustal-harness `docs/MCP.md` ("Answering a question": `session_answer { id, choice, prompt }`;
    a Codex approval offers `approve`, `deny`, `cancel`) and `docs/ROADMAP.md` M10 (R10-04, the
    owner inbox; D130: one manager per project on the pinned Codex adapter).
- **Decisions:** D1 to D9 in the spec.

### Design
- **The facts** (`marley_agent::route`, over an `Asked { tool, preview, question, options,
  chips, level, project_folders, cwd }`, the chips and level from #568):
  - `owner_chip`: a #568 chip of `destroys`, `credentials`, `rewrites history`, `sends out` or
    `outside project`.
  - `names_money`: the preview or question matches, case-insensitively, `\$[0-9]|€|£|pay|
    charge|billing|invoice|card|wallet|stripe|paypal|purchase|subscription`.
  - `messages_people`: `mail|sendmail|slack|discord|tweet|post |gh (pr|issue) comment|gh pr
    create|sms|notify-send|reply`.
  - `read_only_tool`: #568's level 1 (`Read`, `Grep`, `Glob`, `WebFetch`), `WebSearch`, `LS`, or
    a `Bash` at level 2 whose command's first word is one of `ls`, `cat`, `head`, `tail`, `wc`,
    `rg`, `grep`, `find`, `git status`, `git log`, `git diff`, `git show`, `pwd`, `which`,
    `echo`, `env`, with no pipe into a write and no redirection.
  - `options_all_reversible`: an `AskUserQuestion` whose options name none of the words above.
- **The rule table** (`classify`, first match wins): 1 `owner_chip`, `names_money` or
  `messages_people` → `Owner`; 2 `read_only_tool` → `CouldProceed`; 3 else `Open`.
- **The ask.** #508's `InboxEntry` (with #568's fields) gains `route: Option<Route>` and
  `source` (`Rule(&'static str)` or `Reading { confidence }`). Where #568 asks for its chips
  (`rail.rs`, as the entries are gathered), an `Open` entry with no route reading yet asks
  `system_one::ask` once for `question_route`, keyed by the entry's key (#508's identity), with
  `StateBuilder::new(project, detail)`: facts `tool`, `chips` (#568's names, or `nothing`),
  `level`, `read only`, `options reversible`; texts `ask` (the permission line or the question),
  `options` (joined), `prompt` and `message` (the seat's labels, cut to 300), each through the
  layer's `Mask`; the use's verdict (`Open`) with it. The reading is kept beside #568's by entry
  key. In `suggest` the mark carries `?`; in `act` the class counts for the order; in `shadow`
  the reading is logged and the entry shows no mark from it.
- **The order.** `marley_rail`'s inbox rows sort by `(level desc, route_rank, first_seen)` in
  `act`, with `Owner` and `Unclear` at rank 0, `Manager` 1, `CouldProceed` 2, and #568's model
  urgency inside the level as it defines it; otherwise by `(level desc, first_seen)`, #568's own.
- **The outcome.** When an entry leaves (#508's clearing rule), the workbench writes the call's
  outcome line: `owner` if the entry's terminal held the focus in the active window at any point
  between the entry's first-seen time and its clearing (the rail's refresh notes focus per row),
  or an inbox button answered a thread; `manager` when a harness answer exists (later); else
  `agent`.
- **The plugin.** `event.py`: for `AskUserQuestion`, `options` = the first question's
  `options[].label`, at most eight, each cut to 40; dropped in the size loop after the transcript
  path and the working directory, before the message. `HookEvent.options: Option<Vec<String>>`;
  `Moving::tool_starts` passes them to `wait`, which fills `Question.options`.
- **File manifest.** Marley: `crates/marley_agent/src/route.rs` (new), `marley_agent.rs` (the
  module), `claude_events.rs`; the plugin's `hooks/event.py`, `plugin.json`,
  `marketplace.json`; `crates/marley_rail/src/marley_rail.rs` (the inbox model from #508 and
  #568); `crates/marley_workbench/src/rail.rs`, `agent_events.rs`;
  `crates/marley_system_one/src/question.rs` (the `question_route/1` set) and the workbench's
  `system_one.rs` (the `UseSpec`); `script/e2e/browser-fixture.sh` (the stand-in's `fleet` prints
  a question's options); `script/e2e/570-who-answers-a-question.sh`. Zed:
  `crates/settings_ui/src/marley_page.rs` (the use's dropdown).
- **Ledger rows.** The `marley_page.rs` row gains the `question_route` dropdown (#570) before
  its edit.

### E2E plan
Fixtures: a scratch repository; a HOME whose `.bashrc` puts `$E2E_WORK/bin` first on the PATH;
#508's stand-in `claude`, reading its steps from the file `MARLEY_E2E_STEPS` names, started in
four terminals through the rail's + with the variable set per terminal (each terminal's `.bashrc`
export differs by a counter file the stand-in advances); `CLAUDE_CONFIG_DIR` in a scratch folder;
the profile's settings: #565's block enabled on `replay` with the repository listed,
`uses.inbox` at `off` (so #568's model half stays out of the shots) and `uses.question_route`
rewritten per step by `system_one_setting`; `$E2E_PROFILE/system_one/replay.jsonl` written by
setup (C: `manager` 0.88; D: `agent proceeds` 0.90; E: `cannot_tell`, each matched on the ask's
words).

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-002, REQ-003, REQ-004 | mode `suggest`; in A, B, C and D press Enter to each terminal's `PermissionRequest` or `AskUserQuestion`; settle 3 | `570-01-marks`; the day's file: rows for C and D, none for A and B |
| REQ-005 | `pointer_to` C's mark, settle 2; then A's | `570-02-tooltip` (two shots, `-a` and `-c`) |
| REQ-006, REQ-007 | mode `act`; in D press Enter (the `PostToolUse` that ends its wait) and again (E's `PermissionRequest`); settle 3 | `570-03-act-order` |
| REQ-008 | `mcp_agent fleet` before D's answer; setup runs `event.py` with 100 KB `options` | the run log |
| REQ-009 | mode `off`; settle 3 | `570-04-off` |
| REQ-010 | mode `act`; click A's entry (its terminal comes forward and takes the focus); Enter in A (the stand-in's `PostToolUse` ends the wait); settle 3 | `570-05-answered`; the outcome line for A: `owner` |
| REQ-011 | the stand-ins' own logs of stdin | the run log |
| REQ-012 | the golden set's 515 run | its page shot |

Not reachable by a scenario: a real manager's answer (M10, `session_answer`), and a real Claude
Code's auto mode allowing a call without input (`agent` is proven by review of the outcome rule).

### Risks
- Four stand-ins in one window: the rail's rows and the inbox's entries move the y coordinates
  #508's and #568's scenarios click (L-claude-498); Test reruns both and reads their logs.
- Two calls per open entry, #568's and this ticket's, each deduped by the layer; the note's
  budget for routing (about 50 a day) stands. A later version of one set can carry both
  questions once both have labels (Out).
- The word lists are a floor, not a wall: a permission an agent phrases around them is `Open`
  and goes to the model, whose reading may only mark. What the lists miss reaches Chad as today.
- A plugin version bump beside #538's (which removes `notify.sh`): whichever lands second bumps
  again; the update chip covers both.
- If #568 lands with a different entry identity or chip shape, promotion re-keys the reading and
  the `owner_chip` fact on it.
- If the slice runs long, the outcome lines (REQ-010) split off; the marks and the order are the
  first slice.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
