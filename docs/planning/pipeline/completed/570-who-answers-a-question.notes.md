# Who answers an agent's question: owner, manager, the agent proceeds, cannot tell — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-570-who-answers-a-question.md
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
- **Decisions:** D1 to D11 in the spec.

### Promotion (2026-09-28)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #568's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore, over #568's tree, before its commit `8d136c2bc0`) ✓; the prior-art sweep
  ✓; spec and design updated ✓. Three Zed paths will change (`marley.rs`, `marley_page.rs`,
  `default.json`), each with its ledger row widened before its edit.
- **Recall, added:** AD-claude-568-inbox-risk-chips-come-from-code-and-a-reading-only-adds-001
  and #568's D10 (a rule's verdict is a `rules` row, so an outcome labels every entry);
  F-claude-568-a-mode-switch-that-moved-no-level-would-not-have-redrawn-001 (a mark's `?` rides
  in the rail's snapshot); L-claude-568-unused-results-wants-every-returned-value-used-001 and
  L-claude-568-a-seat-holds-one-wait-so-each-entry-needs-its-own-terminal-001;
  PR-claude-drop-the-unbounded-fields-first-001 (the options join the frame's drop order). The
  brain (consultation fb34b66c94c849e4b6584aaf4676e2b4): nothing on this seam beyond #508's and
  #568's decisions.
- **What the code says now** (the Explore report, 2026-09-28):
  - #568's `risk::classify(&Action) -> Vec<Chip>`, `Chip { kind, source }`, the level from the
    kind; `ToolClass::Read` holds `Read`, `Grep`, `Glob`, `LS`, `WebFetch` and `WebSearch`, and the
    Agent Panel's read, search, fetch and think kinds. A read gets no `outside project`, so a
    "read inside the project" needs its own path check; the tokenizer splits no redirection, and
    `words` drops symbols, so a currency sign needs its own test. The tokenizer, `program`,
    `words`, `holds_any` and the path resolution are private to `risk.rs`, in the same crate.
  - With `uses.inbox` off, `inbox_entries` builds no `Waiting`, no chips and level 0 (a held
    click too, since #568's last fix), and the order is by first seen.
  - `InboxEntry` derives `Eq`: a mark's confidence stays beside the rail's reading, as #568's
    probability does. `note_inbox` sorts; `follow_risk` starts asks from `refresh`.
  - The plugin is at 1.3.0 in `plugin.json` and `marketplace.json`; `event.py` sends an
    AskUserQuestion's first question as its preview and no options; the frame's bound is 2,900
    bytes with the drop order `transcript_path`, `cwd`, `message`, `preview`, `prompt`, and a
    summary still too large prints `{}`. `HookEvent` ignores fields it does not know; `wait`
    takes no options; nothing reads `Question.options` today.
  - An AskUserQuestion folds on its PreToolUse (the question, or "A question") and ends on the
    PostToolUse with its id; a PermissionRequest during a question replaces it.
  - #565: `Question::Choice`, `choice_verdict`; `cannot_tell` and `none` abstain; the floor is
    0.5; no signal is `Signal::Nothing`. The replay row for a choice: `{"type": "choice",
    "choice", "confidence", "probabilities"}`.
  - The seat's `message` label is cleared at each prompt and set only at a stop.
  - The rail knows the focused terminal and whether it holds the focus (`Focus.terminal`,
    `terminal_focused`), the thread the panel shows with the focus, and the window's activity.

### Design
- **Changed at promotion** (each item overrides the drafted design after it):
  - **`route.rs`'s shape.** `Route { Owner, Manager, CouldProceed, Unclear }` (`words`: `for
    you`, `for the manager`, `could proceed`, `unclear`; `rank`: 0, 1, 2, and 0 for unclear),
    `RouteSource { Rule(&'static str), Reading }`, `RouteMark { route, source }`, all `Copy` and
    `Eq`; `Facts<'a> { action: &Action<'a>, chips: &[Chip], question: Option<&str>, options:
    &[String] }`; `classify(&Facts) -> Class { Owner(&'static str), CouldProceed(&'static str),
    Open }`, the rule's name its words (`rewrites history`, `names money`, `a read inside the
    project`); `route_of_choice(option) -> Route` (`owner`, `manager`, `agent_proceeds`, else
    `Unclear`). `risk.rs`'s tokenizer, `program`, `words`, `holds_any`, `resolve` and
    `is_allowed` become `pub(crate)`. Money: a `$`, `€` or `£` next to a digit, or the words;
    people: the words and `gh pr create`, `gh issue comment`.
  - **The facts with #568 off.** `inbox_entries` reads each tool entry into a `Waiting` while
    either use is on; `mark` classifies it once, sets the entry's chips and level only while
    `inbox` is on, and while `question_route` is on sets its rule's mark and keeps its
    `RouteAsking` in the snapshot.
  - **The state.** Facts: `tool`, `agent`, `project`, `chips` (#568's words, or `nothing`); texts:
    `ask`, `options` (joined) for a question, `prompt` (the seat's `prompt` label) for a terminal.
  - **The asks and rows** mirror #568's: `Rail.route: HashMap<String, Route…>` and
    `follow_route`, called from `refresh` after `follow_risk`; a rule's mark is `record`ed with
    `choice_verdict("route", "owner")` or `"agent_proceeds"`; an open entry is asked once, its
    reading landing by key and ask.
  - **The mark** rides in `InboxEntry.route` from the rule, or from the reading in `suggest` and
    `act`, and `RailSnapshot.route_suggests` carries the `?`; `render_chips` draws it after the
    chips, and the line shows when there is a chip, a mark or a button. The tooltip: "Marley's
    rule: rewrites history", or "System One: manager (0.88)".
  - **The order** in `act`: `note_inbox` sorts by `(Reverse(level), rank, first seen)`, the
    levels #568's, all 0 while `inbox` is off.
  - **The outcome** (D8): `follow_route` notes at each refresh whether each terminal entry's
    terminal is the rail's focused terminal holding the focus in an active window, and whether a
    thread entry's thread is the one the panel shows with the focus; when an entry with a row
    leaves, `owner` if so or if an inbox button answered it, else `agent`, with how long it
    waited.
  - **The plugin**: `options` in the drop order between `cwd` and `message`, as the drafted design
    says; the version to 1.4.0 in both files.
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
- **File manifest** (as promoted). Marley: `crates/marley_agent/src/route.rs` (new),
  `marley_agent.rs` (the module), `risk.rs` (its helpers shared in the crate),
  `claude_events.rs` (`HookEvent.options`, `wait`); the plugin's `hooks/event.py`, `plugin.json`,
  `marketplace.json`; `crates/marley_rail/src/marley_rail.rs` (`InboxEntry.route`,
  `route_suggests`); `crates/marley_system_one/src/marley_system_one.rs` (`QUESTION_ROUTE_SET`,
  `QUESTION_ROUTE`); `crates/marley_workbench/src/rail.rs` (the facts with #568 off, the mark,
  the asks and rows, the order, the outcome); `script/e2e/browser-fixture.sh` (the fleet listing
  prints a question's options); `script/e2e/570-who-answers-a-question.sh` and
  `script/e2e/golden` (Test). Zed: `crates/settings_content/src/marley.rs` (the docstring),
  `crates/settings_ui/src/marley_page.rs` (the item, 15 in the section),
  `assets/settings/default.json` (the default and its comment).
- **Ledger rows.** Rows 10, 13 and 16 of `docs/marley/zed-touchpoints.md` gain the
  `question_route` use, each before its path is edited.

### E2E plan
As promoted, over the draft below: #568's mechanics (the stand-in's case by argument, the
palette's new terminal, `marley_setting`, the replay file before the layer is turned on), with
`uses.inbox` in `shadow`; the cases `push`, `read`, `test`, `ask` (whose second step ends the
question and asks E) and the `read` stand-in ending its own wait after 40 seconds; the spec's UI
proof lists the shots.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-002, REQ-003, REQ-004 | `suggest`; A, B, C and D wait | `570-01-marks`; the day's file: `rules` rows for A and B, calls for C and D |
| REQ-005 | the pointer on C's mark, then on A's | `570-02a-tooltip`, `570-02b-tooltip` |
| REQ-008 | the fleet listing while D waits; setup's hook fed 100 KB of options | the run log |
| REQ-006, REQ-007 | Enter in D (its wait ends, E asks); `act` | `570-03-act-order` |
| REQ-009 | `off` | `570-04-off`; no new row |
| REQ-010 | `act`; A's entry clicked, Enter in A; B's stand-in ends its own wait | `570-05-answered`; the outcomes `owner` and `agent` |
| REQ-011 | the stand-ins' logs of stdin | the run log |
| REQ-012 | the Marley page, scrolled | `570-06-setting` |
| REQ-013 | the gate and the golden set | the exit codes |

The draft's plan:

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
- Changed at promotion: the marks add a word to the chips' line; #508's and #568's scenarios
  run with the use off, so their points hold, and Test reruns both.
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
- **Checklist** (no task tool): the README marker ✓; ledger rows 10, 13 and 16 widened before the
  Zed paths were edited ✓; every manifest file written but the fixture's (Test's, since #568's
  release install was running the golden set, which sources it) ✓; no cargo until that install
  ended, then `just clippy` on the six touched crates, clean on the first run ✓; the review below ✓.
- **Built.**
  - `marley_agent::route` (new, pure): `Route` (`words`, `rank`), `RouteSource`, `RouteMark`,
    `Class` (`mark`), `Facts`, `classify` (the owner's chips; money, by words and, outside a
    command, a currency sign next to a figure; a message to people; a claim of approval leaving
    the entry open; a local read tool whose every path is inside the project; a command whose
    every simple command is a read-only program with no redirection), `route_of_choice`.
    `risk.rs`'s `Command`, `commands`, `program`, `arguments`, `resolve`, `words` and `holds_any`
    are `pub(crate)`.
  - `claude_events`: `HookEvent.options`; `wait` takes the options, which an AskUserQuestion's
    PreToolUse passes, into the seat's `Question.options`.
  - The plugin: `event.py` sends the first question's option labels (at most eight, each cut to
    40) and drops them under the bound after the working directory; `plugin.json` and
    `marketplace.json` at 1.4.0.
  - `marley_rail`: `InboxEntry.route`, `RailSnapshot.route_suggests`.
  - `marley_system_one`: `QUESTION_ROUTE_SET` (`question_route/1`: the choice `route` over
    `owner`, `manager`, `agent_proceeds`, `cannot_tell`, and the noul `answerable_from_prompt`) and
    `QUESTION_ROUTE` (`question_route`, 2 s).
  - `rail.rs`: `RiskScope` names which uses read the entries, and `mark` classifies each tool
    entry once for both: the chips and level while `inbox` is on, the rule's mark and the
    `RouteAsking` (facts `tool`, `agent`, `project`, `chips`; texts `ask`, `options`, `prompt`;
    the rules' `choice_verdict`) while `question_route` is on; `seat_waiting` carries the options
    and the prompt; a held click is the owner's by its class. `follow_route`, after `follow_risk`
    in `refresh`: who had each entry's terminal or thread in front with the focus in an active
    window, the outcome (`owner` or `agent after … s`) when an entry with a row leaves, a `rules`
    row or one ask per new entry, its reading (`route_reading`) landing by key and ask.
    `note_inbox`: readings' marks in `suggest` and `act`, `route_suggests`, and in `act` the sort
    `(Reverse(level), rank, first seen)`, an unmarked entry ranking as unclear. `render_route`
    draws the mark after the chips with its tooltip; `answer_inbox` counts an answer from the
    inbox as the owner's.
  - Zed paths: the `uses` docstring names `question_route`; the Marley page's Question Route
    after Inbox Risk (15 items); `default.json`'s `question_route: "off"` and a line in its
    comment.
- **Deviations from the design, and why.**
  - **A claim of approval leaves an entry open** rather than letting a rule say it could proceed:
    text that claims an approval is the agent's own, and the note's rule 6 keeps it out of any
    shortcut.
  - **A web read is not "a read inside the project"**: `WebFetch`, `WebSearch` and the panel's
    fetch reach off the machine, so they go to the model as open entries.
  - **The tooltip of a reading with no answer** says "System One could not tell": `cannot_tell`,
    a reading under the floor and a refusal all read `unclear` with no confidence to show.
- **Review** (REQ-001 to REQ-013):
  - REQ-011 first: a reading sets only a mark and a rank; the one change to `answer_inbox` notes
    who answered, and no code path answers a permission or a question.
  - With `inbox` off, the chips are still computed for the route and never shown; with
    `question_route` off, no mark, no row and no ask, and `follow_route` drops what it kept.
  - The asks start from `refresh`, outside any other entity's update; a reading lands through
    `update_in` and refreshes the rail itself; the `?` rides in the snapshot
    (F-claude-568-a-mode-switch-that-moved-no-level-would-not-have-redrawn-001).
  - The state: the facts are what code computed; the ask, the options and the prompt are text,
    masked and cut by the layer; no path or working directory is a fact.
  - Found nothing to fix.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan as promoted ✓; the fixture's fleet
  listing of a waiting question's options ✓; 570 in the golden set ✓; `just build` ✓; the
  scenario run and every shot read ✓; the golden set ✓; `gates.sh --diff` ✓.
- **The scenario**, `script/e2e/570-who-answers-a-question.sh` (compositor sway, no Chromium):
  #566's stand-in `claude` by case in four terminals from `palette "workspace: new terminal"`: A
  `push` (`git push --force origin main`), B `read` (`Read: src/lib.rs`, its request sent 10
  seconds after its first Enter, once the focus has moved on, and its wait ended by the stand-in
  itself 150 seconds later), C `test` (`npm test`), D `ask` (an AskUserQuestion "Which base
  branch?" with `main` and `release`, whose next step ends it and asks E, `cargo test`); the
  replay rows for `question_route/1` on `npm test` (`manager` at 0.88), "Which base branch"
  (`agent_proceeds` at 0.90) and `cargo test` (`cannot_tell`), written before the layer is turned
  on; `uses.inbox` in `shadow`, the route's mode per step; setup feeds the plugin's hook 100 KB of
  options. The real `claude` never ran. Run 1 passed all 12 checks, but its two tooltip shots
  showed no tooltip, the pointer beside the marks rather than on them; run 2, with the points
  measured from run 1's shot, passed all 12 again and showed both.
- **The shots** (run 2, read one by one; the rail cropped and enlarged):
  - `570-01-marks` (REQ-001, REQ-002, REQ-004): `suggest`, in #568's order: A with `rewrites
    history`, `sends out` and `for you`; C `for the manager?`; D "Which base branch?" `could
    proceed?`; B `could proceed`. The day's file: `rules` rows for A and B, `replay` calls for C
    and D, D's state with `options: main, release`, no state naming a working directory.
  - `570-02a-tooltip`, `570-02b-tooltip` (REQ-005): on C's mark, "System One: for the manager
    (0.88)"; on A's, "Marley's rule: rewrites history".
  - `570-03-act-order` (REQ-006, REQ-007): D answered in its terminal and E asked; `act`: A; then,
    in the level C and E share, E `unclear` above the older C `for the manager`; then B `could
    proceed`.
  - `570-04-off` (REQ-009): no marks, #568's order (A, C, E, B); no new row.
  - `570-05-answered` (REQ-010): A's entry clicked, its terminal forward with the focus, Enter
    there: A gone, its terminal "idle · Publish the branch"; B gone once its stand-in ended the
    wait; the day's file holds `owner after … s` and `agent after … s`.
  - `570-06-setting` (REQ-012): the Marley page's System One section: Inbox Risk, then Question
    Route with its description, set to Act by the run, with the mark of a value off its default.
  - The run log (REQ-008, REQ-011): the fleet listing shows D's seat with `options ['main',
    'release']`; the hook's answer to 100 KB of options is 742 bytes, keeping `options` (8),
    `preview`, `session_id`, `tool` and `tool_use_id` while the 3,000-character `cwd` went; the
    stand-ins read only the scenario's Enters.
- **Focus**: the scenario ran in its own headless sway; the runner reported `hyprland: 0 Marley
  windows before the run, 0 after`, no rule added.
- **Not reachable by a scenario**: a manager's answer (rustal-harness's M10 and #534); an Agent
  Panel entry of Zed's own agent, which needs a model, so the thread's focus rule and its inbox
  answer are proven by review of `follow_route`.
- **The gate**: `script/gates.sh --diff` printed `GATE GREEN [diff]`, 16 passed and 0 failed; the
  full log is kept in the scratchpad.
- **The golden set** with 570 added (`just regress`, the debug build): `regress: all 45 passed`,
  570 in 211 seconds (its read's stand-in waits 150), and 508 and 568, whose inboxes run with the
  route off, in 77 and 123.
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented** (§21): `CHANGELOG.md` (Added: the question route, the plugin at 1.4.0);
  `docs/marley/three-prong-plan.md` (S1: the sixth use shipped, the manager's half waiting on
  M10 and #534); `docs/marley_architecture/marley_agent.md` (the `route` module,
  `HookEvent.options`, the consumer); `marley_system_one.md` (the `question_route/1` set);
  `marley_rail.md` (an entry's route, `route_suggests`); `marley_workbench.md` (who should answer
  in the inbox's bullets, the plugin's options); `docs/marley/guide.md` (the plugin's version and
  what 1.4.0 adds, "The question route" under System One). The Zed paths' rows in
  `docs/marley/zed-touchpoints.md` (10, 13 and 16, the last widened for the comment's line)
  describe what shipped.
- **Knowledge appended** (§19): L-claude-570-an-outcome-read-from-the-focus-needs-its-wait-to-start-unwatched-001,
  L-claude-570-a-choice-that-abstains-reads-as-no-signal-001;
  AD-claude-570-the-inbox-marks-who-should-answer-and-answers-nothing-001. No F-block: nothing
  was found wrong in Code or Test (the first run's tooltip shots missed the marks by their
  points, a scenario's guess, not a fault).
- **The brain**: consultation fb34b66c94c849e4b6584aaf4676e2b4 closed with `brain_decide`
  (`decisions/marley-570-the-inbox-marks-who-should-answer-rules-first-and-answers-nothing`), a
  follow-up due 2026-10-28: the rules' marks and the readings against a month of outcomes.
- **Closed**: TICKET-570 moved to `tickets/closed/`, its pipeline doc pointed at `completed/`; no
  `BACKLOG.md` row was left.
- **The commit**: the Test phase's receipt matches the tree (docs only since).
