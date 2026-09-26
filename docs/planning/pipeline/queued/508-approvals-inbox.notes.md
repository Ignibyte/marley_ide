# One approvals inbox in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-508-approvals-inbox.md
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
- **Discovery:** each seam opened and checked on 2026-09-25, at commit `520a6e22a7`.
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
- **Decisions:** D1 to D7 in the spec.

### Design
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
- **File manifest.** Marley only: `crates/marley_rail/src/marley_rail.rs`,
  `crates/marley_workbench/src/rail.rs`, `crates/marley_workbench/src/browser.rs` (the guard),
  `crates/marley_workbench/src/agent_events.rs` (#519's module: a `waiting_on_permission(view)`
  query), `script/e2e/508-approvals-inbox.sh` (Test).
- **Ledger rows.** None: no Zed path changes. The rail calls public Zed API only.

### E2E plan
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

What no scenario reaches: an action-choice prompt, which only Zed's native agent raises (it needs
a real model), so REQ-004 is proven by review.

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
