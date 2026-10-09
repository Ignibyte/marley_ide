# Agent Panel threads over MCP — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-706-agent-panel-threads.md
- **Pipeline spec:** 706-agent-panel-threads.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09: full control of Zed over MCP; the intake's fourth ticket.
- **Classification:** feature, Marley crates plus the settings paths (rows extended).
- **Recall (§18.3):**
  - #508's rail answers a thread's permission from outside the panel (`thread_entry`,
    `answer_thread`) and gives no answer to a sandbox escalation.
  - #697's scripted ACP agent runs the Marley entry in scenarios (PR-687).
  - AD-704 and AD-705 set the modes and the levels.
- **Discovery:** `rail.rs` `live_threads`/`thread_entry`/`answer_thread`;
  `agents::thread_agent_name`; `ConversationView::pending_tool_call`; `AcpThread::to_markdown`;
  `ThreadView::send`; `MessageEditor::set_text`.

### Design
- **`registry.rs`:** `Family::Thread` (`thread`, served), with `list` and `read` (Read) and
  `post` and `answer` (Write, `thread.write`), and their schemas. The pinned list and the count
  (50) are updated. `dispatch.rs` defers `Family::Thread`. `mcp.rs` grants `thread.write` at
  start and routes `thread_*` to `thread_tools`.
- **`agent_control.rs`:** `Area::Threads`, with its mode from `marley.agent_control.threads`.
- **`thread_tools.rs` (new):**
  - `live`: every window's workspaces' `AgentPanel`s and their `conversation_views`, each
    conversation once, keyed by `parent_id().to_key_string()`.
  - `list` builds the rows: the title (`AcpThread::title`, else "New Thread"), the agent
    (`thread_agent_name`), the project (the workspace's first visible worktree), `running`
    (`ThreadStatus::Generating`), and `pending` with the tool's label and `answerable` (no
    sandbox details).
  - `read`: `log`, then `to_markdown`, redacted, then `page_from`.
  - `post`: the thread's window; `message_editor.set_text`; then `ThreadView::send`.
  - `answer`: the pending call. None is `no_pending`; a sandbox escalation is
    `sandbox_escalation`. It answers through `authorize_permission_request` with the AllowOnce or
    RejectOnce option, as `answer_thread` does.
- **Settings (rows extended first):** `MarleyAgentControlContent.threads`, `default.json` and
  the Agent Control section's Threads dropdown.
- **File manifest:**
  - Marley crates: `marley_mcp` (`registry.rs`, `dispatch.rs`); `marley_workbench`
    (`thread_tools.rs` new, `agent_control.rs`, `mcp.rs`, `marley_workbench.rs`).
  - Zed paths: `settings_content/src/marley.rs`, `default.json` and `marley_page.rs`.
  - Docs: the touchpoints and the guide.
  - The scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | A Marley thread on the scripted agent gets "hello". The client runs `thread_list` (the thread, idle) and `thread_read` ("Noted: hello") | the replies checked |
| 003 | `thread_post` "posted by e2e-agent" in the background | 706-01-post-asked |
| 003 | Allow for This Session | 706-02-posted: the message and "Noted: posted by e2e-agent" |
| 004 | The client posts "ask permission" (no question now); the scripted agent requests permission; `thread_answer` allow in the background | 706-03-answer-asked |
| 004 | Allow for This Session (it answers this one call) | 706-04-answered: "Answered: allow" |

### Risks
- **The scripted agent's permission request:** ACP's `session/request_permission` shape must
  match Zed's client. It is built from the ACP types Zed's own tests use, and the first run shows
  it.
- **An agent answering its own prompt:** an agent blocked on its own permission prompt can't call
  a tool, and every answer asks the user unless the area is `allow`.

## Phase 2 — Code
- **Built:**
  - **Settings:** `MarleyAgentControlContent.threads`, its `default.json` line, and the Agent
    Control section's Threads dropdown. The three touchpoint rows were extended first.
  - **`registry.rs`:** `Family::Thread` (served), with `thread_list`/`thread_read` (Read) and
    `thread_post`/`thread_answer` (Write, `thread.write`) in `thread_schemas`; count 50.
    `dispatch.rs` defers the family.
  - **`mcp.rs`:** grants `thread.write` at start and routes `thread_*`.
  - **`agent_control.rs`:** `Area::Threads`.
  - **`thread_tools.rs` (new):** `live`, `thread_named`, `title_of`, `pending_of`, `list`,
    `read`, `post` and `give_answer`.
  - **`guide.md`:** the tools' lines.
- **Deviations:**
  - **`post` inserts instead of setting the text.** `MessageEditor::set_text` exists only under
    test-support, so `post` uses `insert_text`. It refuses `draft_in_progress` while the user has
    unsent text in that thread: inserting would have sent the user's draft along.
  - **`give_answer` reports a lost answer.** It returns `no_pending` when the request is gone by
    the time it answers (answered meanwhile, or the window closed) instead of reporting success.
- **Review:**
  - No entity is updated inside its own update: the tools run from the admit's `cx.update`, and
    `window.update` → `view.update` is the only nesting.
  - A sandbox escalation is refused before any option is chosen. An agent waiting on its own
    prompt can't call a tool, and every answer asks unless the area is `allow`.
  - The read is redacted with the user's agent redactor and paged like `editor_read`.
- **Gate:**
  - `706-gate-1.log` RED (clippy: the module doc's first paragraph too long).
  - `706-gate-2.log` GATE GREEN [diff]. cargo-deny printed "couldn't check if the package is
    yanked" (the registry timed out), which the gate does not count.

## Phase 3 — Test
- **Scenario:** `script/e2e/706-agent-panel-threads.sh`, under `compositor sway`.
  - The scripted ACP agent is #697's, extended: told "ask permission", it sends a pending tool
    call and `session/request_permission` with `allow_once` and `reject_once` options, then
    reports the option it got.
  - The client is #704's, rewritten for the four thread tools.
- **First run (`shots-706a`): every check passes, every shot shows its criterion.**
  - **Checks:**
    - `thread_list` → `hello agent=Marley project=repo running=False pending=None`;
    - `thread_read` → `## User / hello / ## Assistant / Noted: hello`;
    - the post → `sent=True`, and the agent's log holds the text;
    - the second post went without a question (the session was allowed);
    - `thread_list` while it waits → `running=True pending=Write notes.txt answerable=True`;
    - `thread_answer` → `answered Write notes.txt allowed=True`, and the agent's log holds
      `"optionId": "allow"`.
  - **706-01-post-asked (REQ-003):** "e2e-agent wants to use thread_post", "send "posted by
    e2e-agent" into the thread "hello"", with the three buttons over the panel's thread.
  - **706-02-posted (REQ-003):** the thread shows "posted by e2e-agent" as a user message and
    "Noted: posted by e2e-agent" under it.
  - **706-03-answer-asked (REQ-004):**
    - the thread shows "Write notes.txt" with Allow and Reject and "Awaiting Confirmation…";
    - the rail's Needs you counts 1 and the row reads "Marley · waiting";
    - the question "e2e-agent wants to use thread_answer", "allow Write notes.txt in the thread
      "hello"" is asked, although the session was allowed to post.
  - **706-04-answered (REQ-004):** the tool call has no buttons, "Answered: allow" under it,
    Needs you gone, and the row reads idle.
  - **Not covered by a scenario:** the `sandbox_escalation` refusal. The scripted agent can't
    send Zed's sandbox details, which only Zed's native agent builds. The branch reads the same
    field the rail's row does (#508), so the review covers it.
  - Chad's Hyprland untouched.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md` (Added);
  - `docs/marley_architecture/marley_workbench.md` (a section for `thread_tools.rs`) and
    `marley_mcp.md`;
  - the guide (Phase 2).
  - The three touchpoint rows describe the shipped `threads` setting, its default and its
    dropdown.
- **Knowledge appended:** AD-claude-706-agents-post-and-answer-through-the-panels-own-paths-001.
  No bug was found in Test; the two Code deviations are design fixes, recorded in the AD.
- **Brain:** no `rusty` MCP server in this repository's sessions; the decision is in the ledger.
- **Ticket:** closed; the BACKLOG row left at promotion.
- **Gate:** `706-gate-3.log`, GATE GREEN [diff], on the tree committed.
