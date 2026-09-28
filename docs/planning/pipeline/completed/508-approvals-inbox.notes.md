# One approvals inbox in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-508-approvals-inbox.md
- **Pipeline spec:** 508-approvals-inbox.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "lets do 1 through 8", item 6 of the list after the browser
  waves: every pending permission prompt in one list in the rail. The Orca survey the same day
  added what the ticket should take (`docs/orca_architecture/README.md`, "What it changes in the
  queued sprint", #508) and moved it onto #519's events. The ticket doc is revised to match.
- **Classification / tier:** feature, prong 2 (attention). Marley crates only; no Zed path
  changes. Depends on #519. Size M.
- **Recall (§18.3):**
  - AD-claude-496: picks are staged and sent to the last terminal; the guard sits in that path.
  - L-claude-500: a clicked context menu starts on its first entry; the scenario's + menu steps
    count from there, as #501's do.
  - L-claude-501 and AD-claude-501: every Agent Panel agent takes Marley's server; the stand-in
    ACP agent in #501's scenario is the fixture to extend.
  - PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001 and
    PR-claude-state-another-entity-reads-is-kept-outside-render-001: the answer runs from a click
    handler into another entity (`ThreadView`), so it defers out of the rail's own update, and the
    rail reads the entries at refresh, never in render.
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery:** each seam opened and checked on 2026-09-25, at commit `484a7f18cb`.
  - `crates/acp_thread/src/acp_thread.rs:947` (`ToolCall`: `id`, `label`, `kind`, `status`,
    `raw_input`, `tool_name` public), `:1356` (`SelectedPermissionOutcome::new(option_id,
    option_kind)`), `:1402` (`AuthorizationKind`: `PermissionGrant`, `ActionChoice`), `:1417`
    (`ToolCallStatus::WaitingForConfirmation { options, respond_tx, kind, .. }`), `:2638`
    (`entries`), `:2700` (`is_waiting_for_confirmation`, elicitations included), `:3608`
    (`tool_call`), `:3697` (`request_tool_call_authorization` emits
    `ToolAuthorizationRequested`; `ToolAuthorizationReceived` follows the answer), `:3747`
    (`authorize_tool_call`).
  - `crates/acp_thread/src/connection.rs:580` (`PermissionOptions`: `Flat`, `Dropdown`,
    `DropdownWithPatterns`), `:599` (`first_option_of_kind`), `:628` (`allow_once_option_id`),
    `:633` (`deny_once_option_id`).
  - `crates/agent_ui/src/conversation_view.rs:286` (`Conversation` is `pub(crate)`), `:369`
    (its `pending_tool_call`: the first request across the conversation's sessions for a root,
    the session's own for a subagent), `:651` (`active_thread`), `:658`
    (`ConversationView::pending_tool_call`, public), `:669` (`root_thread_has_pending_tool_call`),
    `:688` (`root_thread_view`), `:694` (`thread_view`).
  - `crates/agent_ui/src/conversation_view/thread_view.rs:566` (`ThreadView`: `session_id`,
    `thread`, `workspace`, the agent's icon and name public), `:2482`
    (`ThreadView::authorize_tool_call(session_id, tool_call_id, outcome, window, cx)`: through the
    conversation to the thread, then follows the agent when the panel is following).
  - `crates/agent_ui/src/agent_panel.rs:4153` (`conversation_views`), `:3324` (`active_thread_id`).
  - `crates/marley_rail/src/marley_rail.rs:87-114` (`ThreadStatus`, `label`), `:118-133`
    (`thread_status`: a pending confirmation outranks an error and a run), `:140-155`
    (`thread_attention`).
  - `crates/marley_workbench/src/rail.rs:1561-1595` (`live_statuses`: `root_thread_has_pending_tool_call`
    per conversation), `:1520-1532` (`changes_the_row`: the rail refreshes on
    `ToolAuthorizationRequested` and `ToolAuthorizationReceived`), `:526` (`activate_terminal`),
    `:635` (`open_thread`), `:2140` (`render`).
  - `crates/marley_workbench/src/browser.rs:3645-3690` (`send_pick`: the last terminal from
    `LastTerminal`, `tray_error` for the reason, `terminal.paste`), `:5400-5405` (`LastTerminal`).
  - #519's spec: the seat of a terminal waiting on a PermissionRequest is `Waiting` with a
    `Question`, and its labels carry `tool` and `preview`.
  - `script/e2e/501-zeds-agents-drive-the-browser.sh:20-169` (the stand-in ACP agent, the custom
    agent server block, the + menu steps).
  - For the second slice: `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge:31`
    (`REQUEST_TIMEOUT = 40`), `crates/marley_mcp/src/marley_mcp.rs:80`
    (`APP_CALL_TIMEOUT_SECONDS = 30`); Claude Code 2.1.283's bundle: the `mcp_tool` hook type
    (`server`, `tool`, `input` with `${path}` substitution), a plugin's server named
    `plugin:marley:marley`, the PermissionRequest decision
    `{"hookSpecificOutput": {"hookEventName": "PermissionRequest", "decision": {"behavior": "allow"}}}`
    (or `"deny"` with `message` and `interrupt`), and an `mcp_tool` hook's result returned as
    `{ok, body, error}`, the tool's text in place of a command hook's stdout.
- **Decisions:** D1 to D9 in the spec.

### Promotion (2026-09-28)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #571's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore, at `82c438d09c`) ✓; spec and design updated ✓. No ledger rows: no Zed
  path changes.
- **Recall, added:** AD-claude-571 and PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001
  (the paused click the inbox now lists, and the rule for its buttons); #549's selection guard
  (`send_selection.rs:304`, `:330-340`), the pattern the pick's guard follows; L-claude-566 (a
  seat's state rides on its labels) and #569's `PERMISSION_MODE_LABEL`. The brain (consultation
  a9c8524153fa467eba6220d01a556f1f): nothing on this seam.
- **What the code says now** (the Explore report, 2026-09-28): `agent_ui`, `acp_thread`,
  `agent_servers` and `agent` are unchanged since the draft's check, so every Zed seam holds at
  its line. The findings that change the design:
  - `ThreadView` cannot be named outside `agent_ui` (`conversation_view` is `pub(crate)`; only
    `ConversationView` is re-exported), so a Marley struct cannot hold an `Entity<ThreadView>`;
    values come back through inference (`rail.rs:2104-2105` does so today).
  - The panel gates Allow on a sandbox escalation with surprising Unicode
    (`pending_allow_blocked_by_confusables`, `thread_view.rs:2503-2536`, and
    `sandbox_confusables_block_allow`, `:9083-9097`); `authorize_tool_call` has no such check,
    and `ToolCall.sandbox_authorization_details` is public (`acp_thread.rs:961`).
  - `allow_once_option_id` and `deny_once_option_id` give ids only; `first_option_of_kind`
    gives the option with its kind, which `SelectedPermissionOutcome::new` needs.
  - The seat has no `preview` label; a PermissionRequest waits with the question "Permission for
    <tool line>", an AskUserQuestion with its question, and `waiting_on` is private. The wait ends
    when the waited call's id ends (a PermissionRequest takes it from the matching in-flight
    tool), the turn ends, or the session changes; a request with no matching PreToolUse waits
    on "" and no tool end clears it.
  - `send_pick` (`browser.rs:5110-5154`) has no guard; #549's `send_selection` has one. Rich input
    (#481) and the agent bar's Attach File (#477) paste with no check, from the terminal itself.
  - #571's paused clicks are a second kind of approval the rail does not show.
  - `RailSnapshot` is `{ projects, focus, filtering }`, and `refresh` notifies only when it
    changed, so the inbox lives in it. The row list follows the header and the filter; a section
    above the projects pushes every row down, and scenario 549 clicks its terminal row while its
    seat waits (`549.sh:171`).
  - #501's stand-in ACP agent answers any message with an id and no method with an error, and
    sends no permission request; #519's fake's PermissionRequest (Write README.md, after its
    PreToolUse) is the shape that clears by PostToolUse.

### Design
- **Changed at promotion** (each item overrides the drafted design after it):
  - **The inbox holds the conversation, not the thread view.** A thread entry keeps the
    `ConversationView`'s weak handle, the session id, the tool call id and the allow-once and
    deny-once options (id and kind, from `first_option_of_kind`); the answer finds the thread view
    at the click (`thread_view(&session)`) and calls `authorize_tool_call` on it, deferred out of
    the rail's update. `agent-client-protocol` joins the workbench's dependencies for the ids.
  - **No buttons on a sandbox escalation** (D3): an entry whose tool call carries
    `sandbox_authorization_details` opens its thread, where the panel's own gate applies.
  - **Terminal entries are every waiting seat** (D8): permissions and questions, the seat's
    question as what it asks, mapped to the rail's terminal by the seat's id (the view's entity
    id). A seat in a docked terminal, which the rail does not list, has no entry.
  - **Paused clicks** (D9): `BrowserHub::paused()` lists each tab's sentence; an entry's Allow and
    Refuse call `answer_pause`, and its click `show_paused`. The hub already notifies the rail.
  - **Ages.** The rail keeps when it first saw each entry's key, pruned at each refresh, and a
    timer refreshes every 30 seconds while the inbox shows, so the ages move: `now`, `1 m`,
    `1 h 5 m`.
  - **The pick's guard** (`send_pick`) returns before the caption is taken while the last
    terminal's seat waits, with the reason in the tray.
  - **Placement.** The section sits between the filter and the row list and pushes the rows down
    while it shows; the scenarios that click a row while a seat waits are rerun (519, 535, 549,
    566), and their points change where they must.
- **Approach.**
  1. `marley_rail` (pure): `InboxEntry { key, project, agent, ask, waited, answers }`, where `key`
     is a thread's key with its tool call id or a terminal's id and `answers` says whether Allow
     and Deny show; `RailSnapshot` gains `inbox`, sorted by when each entry was first seen, and a
     function gives the section's rows (none when the list is empty).
  2. `rail.rs`: `build_snapshot` gathers the entries. For each group's workspaces, the Agent
     Panel's conversation views give `pending_tool_call` (session, tool call id, options); the
     session's `ThreadView` gives the thread, whose `tool_call` gives the label's markdown source,
     the tool name and the raw input (cut to one line). #519's `AgentEvents` gives the waiting
     terminal seats. The rail keeps a map from each entry's key to when it was first seen, pruned
     at each refresh; ages are rendered from it with a one-minute timer while the list shows.
  3. Answering: the button's handler takes the entry's `ThreadView` and option (kind and id,
     picked by `allow_once_option_id` or `deny_once_option_id`) and calls
     `ThreadView::authorize_tool_call` inside the thread view's window, deferred out of the rail's
     update. The thread's `ToolAuthorizationReceived` refreshes the rail, which drops the entry.
  4. A click on an entry: a thread's opens it (`open_thread`), a terminal's shows it
     (`activate_terminal`).
  5. The pick guard: `send_pick` asks `AgentEvents` whether the last terminal's seat waits on a
     permission, and if so sets `tray_error` ("Claude Code in <terminal> is waiting for your
     answer. Answer it, then Send.") and returns before the paste.
- **File manifest** (as promoted). Marley only: `crates/marley_rail/src/marley_rail.rs` (the
  entries, their kinds and order, `RailSnapshot.inbox`, the tests' literals);
  `crates/marley_workbench/src/rail.rs` (gathering the entries, first-seen and the age timer, the
  section, the answers and the clicks); `crates/marley_workbench/src/browser.rs` (`paused()`, the
  pick's guard); `crates/marley_workbench/src/agent_events.rs` (a `waiting(view)` query);
  `crates/marley_workbench/Cargo.toml` (`agent-client-protocol` to the dependencies);
  `script/e2e/508-approvals-inbox.sh` and `script/e2e/golden` (Test), and the reruns' points
  where the section moves them.
- **Ledger rows.** None: no Zed path changes. The rail calls public Zed API only.

### E2E plan
As promoted, over the draft below: the stand-in ACP agent is the scenario's own (501's pattern),
answering Zed's replies and sending a `tool_call` update and a `session/request_permission` with
`allow_once` and `reject_once` on each prompt, logging the answer; the fake `claude` is 569's
stand-in with a PreToolUse before its PermissionRequest (Bash `rm -rf build`), so its PostToolUse
clears the wait; the layer on the `rules` provider with the click consequence in `shadow` and
`all_agents`, for one paused click. One row joins the table:

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-009 | the fixture's client clicks Delete account in the background; the inbox's Refuse | `508-08-paused-click`; the client's output reads that the user refused |

The draft's plan:

Fixtures: the scratch repository and HOME of #519's scenario; the stand-in ACP agent from #501's,
extended to request a permission per prompt and log the answer to `$E2E_WORK/stand-in.log`; #519's
fake `claude`; `browser-fixture.sh`'s offline Chromium and a local page with one button.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | the rail's +, the agents' submenu, the stand-in; type a prompt and Enter | `508-01-thread-entry` |
| REQ-002 | a second terminal: `claude`, Enter through to its PermissionRequest | `508-02-two-entries` |
| REQ-003 | click Allow on the thread's entry; then a second prompt and Deny | `508-03-allowed`, `508-04-denied`; `stand-in.log` holds `allow_once`, then `reject_once` |
| REQ-004 | none; review of the entry's rule against `AuthorizationKind::ActionChoice` and `PermissionOptions` without the pair | the review in Phase 2's notes |
| REQ-005 | click the terminal's entry | `508-05-terminal-opened` |
| REQ-007 | New Browser Tab on the page; pick the button; Send, while the fake waits | `508-06-pick-refused` |
| REQ-006 | Enter in the fake's terminal: the PostToolUse of the Bash it asked for | `508-07-cleared` |

What no scenario reaches: an action-choice prompt and a sandbox escalation, which only Zed's
native agent raises (it needs a real model), so REQ-004 and REQ-010 are proven by review.

### Risks
- `agent_ui` is upstream's busiest crate; `pending_tool_call` and `ThreadView::authorize_tool_call`
  could move at a merge. They are public and used by Zed's own UI, and the review at promotion
  re-reads them.
- A subagent's prompt reaches the inbox only through its conversation's `pending_tool_call`, and
  the rail subscribes to root threads, so such an entry may appear at the next refresh rather
  than at once.
- The label's markdown can be long; the entry keeps one line.

### The second slice: answering a terminal agent from the rail
Not specced here; the notes record the design so it can be ticketed after this one ships.
- **Mechanism.** The plugin registers a second PermissionRequest hook of type `mcp_tool` on
  `plugin:marley:marley`, calling a new tool, `approval_wait`, with the session id, the tool
  and its input (`${…}` substitution). The tool puts the prompt in the inbox with Allow and Deny
  and holds until one is clicked, then returns the decision JSON Claude Code expects, or `{}` (no
  decision) when its wait ends, so Claude Code's own dialog takes over.
- **Timeouts.** The bridge gives up after 40 s and the server waits 30 s for the app, so the wait
  must stay under both, or both grow for this one tool. Claude Code gives `mcp_tool` hooks 600 s by
  default.
- **First measurement.** Whether Claude Code draws its own dialog while a PermissionRequest hook
  runs. If it does not, Chad at the terminal waits out the hook, and the wait must be short or
  opt-in. Measure on 2.1.283 with a hook that sleeps 20 s before building anything.
- **What Orca teaches (report 04 §3.2).** Give each prompt an id and a revision, so a late answer
  cannot answer a newer prompt; show "unconfirmed" when an answer's fate is unknown; refuse a
  second answer while one is in flight.
- **Trust.** The answer goes over the MCP bearer, so it is authenticated, unlike #519's frames; a
  new write grant (`approval.answer`) gates the tool.
- **Limits.** An agent on a remote host cannot reach the loopback server without an `ssh -R`
  forward; Codex's approvals need their own hook.
- **Size.** M.

## Phase 2 — Code
- **Checklist** (no task tool): the README marker ✓; no Zed paths, so no ledger rows ✓; every
  manifest file written ✓; `cargo check` and `just clippy` on the touched crates, one at a time
  after #571's release install ended ✓; the review below ✓.
- **Built.**
  - `marley_rail`: `InboxKind { Thread, Terminal, Click }`, `InboxEntry { key, kind, agent,
    project, ask, waited, answers }`, `waited_words(seconds)` (`now`, `3 m`, `1 h 5 m`) and
    `RailSnapshot.inbox`; the tests' one literal gains the field.
  - `rail.rs`: `inbox_entries` per group (each Agent Panel conversation's `pending_tool_call`
    through `thread_entry`; each terminal's waiting seat; each Browser tab's paused click, once a
    page), `InboxTarget` in the workbench's snapshot (the conversation's weak handle, the session,
    the tool call and the two options for a thread; the terminal's id; the click's page),
    `note_inbox` (first-seen per key, the order, the ages, a 30-second refresh while any entry
    waits), `render_inbox` and `render_inbox_entry` (the section between the filter and the rows:
    "Needs you" and the count, a card per entry with the agent, the project, what it asks and its
    age, Deny or Refuse and Allow under an entry that answers in place), `answer_inbox`,
    `answer_thread` (deferred, the thread view found at the click, the panel's own
    `authorize_tool_call`) and `open_inbox_entry` (the thread, the terminal, or `show_paused`).
  - `browser.rs`: `pause_sentence` and `show_paused` are `pub(crate)`; `send_pick` keeps a pick
    and its caption while the last terminal's Claude Code waits, with the reason in the tray.
  - `agent_events.rs`: `waiting(view)`.
  - `Cargo.toml`: `agent-client-protocol` moves to the dependencies, for the ids and kinds.
- **Deviations from the design, and why.**
  - **The thread's key is the conversation's** (`parent_id`), the key the rail's thread rows and
    `open_thread` use, so the entry's click opens the same row's thread.
  - **The option kinds come from `first_option_of_kind`**, since `allow_once_option_id` and
    `deny_once_option_id` give ids only and `SelectedPermissionOutcome::new` needs the kind.
  - **A paused click's entry names "Browser tab"** as its agent and the card's sentence as what it
    asks, which already says who wants to click what.
  - **An entry's buttons sit under its card**: the rail is narrow, and the card's end holds the age.
- **Checks.** `cargo check` clean on the first run; `just clippy` (all targets) found a tuple
  made into an array by hand (now `<[_; 2]>::from`), `build_snapshot` a line over a hundred (the
  ports' line folded into the literal) and three references that need not be mutable. The last
  run is green.
- **Review** (REQ-001 to REQ-010):
  - The answer runs out of the rail's update (`defer_in`), and nothing reads or updates the rail
    while the thread's view updates its thread.
  - A button's click stops there (PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001),
    so Allow does not also open the thread.
  - A prompt without the allow-once and deny-once pair, or a sandbox escalation, has no buttons
    (REQ-004, REQ-010): `answers` is `None`, and the entry opens its thread.
  - Nothing keeps a thread view: the handle is the conversation's, weak, and the view is found
    again at the click, so a closed thread answers nothing.
  - The pick's guard returns before the caption is taken and the pick marked sent, so Send
    works once the agent is answered.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan as promoted ✓; 508 in the golden
  set ✓; `just build` ✓; the scenario run and every shot read ✓; two fixes from the shots ✓; the
  golden set and the reruns ✓; `gates.sh --diff` ✓.
- **The scenario**, `script/e2e/508-approvals-inbox.sh` (compositor sway): the offline Chromium
  and a served page with Save profile and Delete account, whose log goes into the page's title;
  the scenario's own stand-in ACP agent ("Stand-in", a custom agent server in the run's settings),
  which at each prompt sends a `tool_call`, asks `session/request_permission` with `allow_once`
  and `reject_once` options, logs the answer and ends the turn; #566's stand-in `claude` in
  Marley's terminal, its first Enter sending UserPromptSubmit, a PreToolUse for Bash
  `rm -rf build` and its PermissionRequest, its second the PostToolUse and Stop, and every line
  it reads logged; the layer on the `rules` provider with the click consequence in `shadow` and
  `browser_click_pause_agents` at `all_agents`, for one paused click by the fixture's client. The
  real `claude` never ran.
- **The runs.** Run 1 stopped at its first check: the guessed Allow point fell between the
  buttons, and the shots gave every point after it. Run 2 passed all four checks, but its shots
  showed two faults the checks let through:
  - the pick never happened: the fixture's navigate just before it had put the Agent chip in the
    toolbar, which moved Pick left, so the click on Pick's old place fell on the chip and the
    page took the click, the typed space and Enter as three presses of Save profile. The check
    that the terminal read nothing passed with nothing to send (F-claude-508 below). The
    navigate went (the page is the run's own), the pick follows #496's hover first, and a new
    check reads the pick listed and unsent;
  - the paused click's entry was missing from the inbox in its shot, while the tab's card and the
    toast showed it: the rail refreshes on the hub's events, and a pause only notified. The
    entry came with the next refresh, in time for the Refuse click to find it. Fixed in
    `browser.rs`: `pause_click` and `end_pause` emit `PageStatusChanged`, which the rail already
    refreshes on (the Phase 1 design's "the hub already notifies the rail" was wrong about how
    the rail listens).

  And one from run 1's shots: the thread's entry drew the terminal icon Zed's panel gives a
  custom agent, where its thread's row draws the sparkle. `thread_entry` now takes the agent's
  icon and name from `agents::thread_icon` and `agents::thread_agent_name`, as the rows do.
  Run 3 found the chip in a shot taken right after the Pick click; run 4 passed all six checks.
- **The shots** (run 4, read one by one; the rail cropped and enlarged where it is small):
  - `508-01-thread-entry` (REQ-001): "Needs you 1" between the filter and the rows; the entry
    "Stand-in · repo", "Edit README.md", `now`, with the thread row's sparkle, and Deny and Allow
    under it; the thread's row below "Stand-in · waiting" with its mark.
  - `508-02-two-entries` (REQ-002): "Needs you 2": the thread's entry first, then "Claude Code ·
    repo", "Permission for Bash: rm -rf build", `now`, with no buttons; the rows pushed down, the
    terminal's "waiting · Clean the build".
  - `508-03-allowed` (REQ-003): Allow in the inbox; "Needs you 1", Claude Code's entry alone; the
    thread reads "The edit was allow." and its row "Stand-in · idle"; the stand-in logged
    `answer: allow`.
  - `508-04a-second-prompt` (REQ-002): the second prompt's entry second, under the terminal's
    older one, with Deny and Allow; the panel shows the same prompt with its own Allow and Reject.
  - `508-04-denied` (REQ-003): Deny in the inbox; the tool call failed (the red mark), "The edit
    was reject."; the stand-in logged `answer: reject`.
  - `508-05-terminal-opened` (REQ-005): the terminal's entry clicked; the terminal's row selected,
    its Claude Code waiting in the terminal.
  - `508-06a-pick-staged`: Pick, then a click on Save profile: the tray holds pick 1, "button
    “Save profile”", the page's log empty.
  - `508-06-pick-refused` (REQ-007): the caption and Enter: the tray says "Claude Code in that
    terminal waits for your answer: answer it there, th…", the pick and its caption stay; the
    agent reads pick 1 "not sent"; the terminal read only the scenario's one Enter.
  - `508-08-paused-click` (REQ-009): the fixture's client clicks Delete account: "Needs you 2",
    Claude Code's entry, then "Browser tab · repo", "e2e wants to click button “Delete…", with
    Refuse and Allow; the tab's card and the toast say the same. Refuse in the inbox: the client
    read "Marley paused this click: e2e wants to click button “Delete account”, which deletes (its
    name), on 127.0.0.1:…. The user refused it; ask them before you try again".
  - `508-07-cleared` (REQ-006): the terminal's entry, then Enter: the stand-in's PostToolUse and
    Stop; no "Needs you" section, the rows back at the top, the terminal's row "idle · Clean the
    build", "The build is clean.".
- **Focus**: the scenario ran in its own headless sway; the runner reported `hyprland: 0 Marley
  windows before the run, 0 after`, no rule added.
- **Not reachable by a scenario**: an action-choice prompt and a sandbox escalation (REQ-004,
  REQ-010), which only Zed's native agent raises with a model behind it: proven by review of
  `thread_entry` (no buttons without both the allow-once and reject-once options, or with
  `sandbox_authorization_details`).
- **The golden set** with 508 added (`just regress`, the debug build): `regress: all 43 passed`,
  508 in 77 seconds. The set holds the scenarios the design named for a rerun because the section
  can move their rows (519, 535, 549, 566), and 569 and 571: all passed with their points as
  they were. 501, which is not in the set, passed by name (37 seconds).
- **The gate**: `script/gates.sh --diff` printed `GATE GREEN [diff]`, 16 passed and 0 failed.
  After the golden set, `InboxEntry.agent`'s doc comment was corrected (a held click's entry
  names `Browser tab`, and its `ask` names the caller), and the gate ran again: `GATE GREEN
  [diff]`, 16 passed and 0 failed. Both full logs are kept in the scratchpad.
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented** (§21): `CHANGELOG.md` (Added: the inbox); `docs/marley/three-prong-plan.md` (C1:
  the approvals inbox shipped); `docs/marley_architecture/marley_rail.md` (the inbox's entries and
  kinds, `waited_words`, the one order the workbench decides); `marley_workbench.md` (the inbox's
  bullets in the rail section, the hub's `PageStatusChanged` for a held click, `waiting(view)`,
  the pause's inbox bullet, the pick's guard, a known limit); `docs/marley/guide.md` ("Needs you"
  in the rail's section, the guard in the pick tray's). No path outside the Marley-owned set
  changed, so no row in `docs/marley/zed-touchpoints.md` applies. The guide's "What is planned"
  section is stale throughout and is left to the documentation pass after the remaining tickets.
- **Knowledge appended** (§19): F-claude-508-a-held-click-reached-the-inbox-only-with-another-refresh-001,
  F-claude-508-the-inbox-drew-a-custom-agent-with-the-panels-icon-001,
  F-claude-508-the-pick-check-passed-with-nothing-picked-001;
  PR-claude-a-state-another-view-lists-is-announced-by-an-event-001;
  L-claude-508-the-agent-chip-moves-the-browser-toolbar-001,
  L-claude-508-a-stand-in-acp-agent-asks-for-permission-001;
  AD-claude-508-one-inbox-lists-every-agent-that-waits-on-the-user-001.
- **The brain**: consultation a9c8524153fa467eba6220d01a556f1f closed with `brain_decide`
  (`decisions/marley-508-one-inbox-at-the-top-of-the-rail-lists-every-agent-that-waits-on-the-user`),
  a follow-up due 2026-10-28: whether answering a terminal agent from the rail, the second slice,
  is wanted after a month of the inbox.
- **Closed**: TICKET-508 moved to `tickets/closed/`, its pipeline doc pointed at `completed/`; no
  `BACKLOG.md` row was left (#568's and #534's rows name #508 as built on). The references to the
  queued pair in `completed/549` and in the queued 555, 568 and 570 specs now point at
  `completed/`.
- **The commit**: the Test phase's receipt matches the tree (docs only since the second green).
