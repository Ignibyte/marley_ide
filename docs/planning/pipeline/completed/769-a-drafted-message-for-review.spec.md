---
pipeline_id: 9b2dc9cc-9894-4f0d-9936-6b3f8dee802f
ticket: docs/planning/tickets/open/TICKET-769-a-drafted-message-for-review.md
status: Phase 4 — Complete PASS
title: A drafted message for the person to review
type: feature
slice: agents together, after the finish (#716's Out: a drafted message), MonoCode's operator
references:
  - docs/planning/pipeline/queued/716-delegation-tools.spec.md
  - docs/planning/pipeline/completed/706-agent-panel-threads.spec.md
  - docs/planning/pipeline/completed/522-review-notes-to-the-agent.spec.md
---

## Title
An agent can write to another agent only by typing into it, which Marley refuses for an agent CLI
(`terminal_type`), or by posting into a thread (`thread_post`), which reaches only the Agent
Panel's threads and goes out at once. This ticket gives agents a draft instead: `draft_message`
opens the message in a tab of the person's, addressed to an agent in a terminal or a thread, and
nothing goes until the person presses Send. Split from #716 (Chad, 2026-10-11, "spec all"), after
MonoCode's operator, which drafts and the person sends.

## Scope
### In
- **The tool.** `draft_message { to: { terminal: <id> } | { thread: <id> }, text }`, a Write tool
  in a family of its own (`draft`), with the grant class `draft.write`. It checks the target
  exists, opens the draft and answers at once: `{ draft, to, opened: true }`. It never sends.
  - A terminal target is a `terminal_list` id whose terminal runs an agent CLI.
  - A thread target is a `thread_list` id, a thread in the Agent Panel or in a tab
    (`thread_tab::conversations_of`).
  - Refusals: `no_terminal`, `not_an_agent`, `no_thread`, `bad_argument` for an empty text.
- **The Draft tab,** in the target's window: "Draft from <who> to <target>", the text in an
  editable multi-line editor, Send and Drop.
  - **Send** delivers the editor's text as the person would: into the agent CLI's prompt with
    Enter (`terminal_drive::paste_then`, as Review notes do, #522), refused with a toast while the
    agent waits on a question; or into the thread's message box and sent (as `thread_post` does),
    refused while the person has unsent text there. The tab closes and a toast says where it went.
  - **Drop** closes the tab; nothing is sent.
- **The kill switch and the activity log** (#703) cover the tool as every Write tool.
- Docs: the guide's MCP tool table and a short "Drafts" passage; `INSTRUCTIONS` names the tool.

### Out (explicitly deferred)
- A draft to a person (mail, chat): no channel exists.
- Agents that start agents (#716), which waits on the harness.
- Drafts that survive a restart.

## Reference (§20)
N/A — Marley-specific: MonoCode's operator (`docs/planning/intake/monocode-findings.md`) drafts a
message for the person to send; Marley's version is an MCP tool and a tab. Upstream Zed's part is
the item and the editor.

### Prior art
- **The code we ship:** the MCP registry and dispatch (`marley_mcp/src/registry.rs`,
  `dispatch.rs:192-263`), the router (`marley_workbench/src/mcp.rs:525-608`) and the activity
  gate (`agent_activity.rs:147-161`); `terminal_list` and `terminal_with_id` (`mcp.rs:878-1008`);
  `thread_list`, `thread_named` and `post` (`thread_tools.rs:89-319`);
  `send_selection::target_of` and `terminal_drive::paste_then`; `review_notes::deliver`; the
  center-tab editor of `rusty/skills_tab.rs`; the caller's name (`click_pause::Who`).
- **Behavior maps:** MonoCode's findings (item 6, the operator).
- **Published material:** MCP tool annotations.

## UI proof
`script/e2e/769-a-drafted-message-for-review.sh`, under `compositor sway`. A stand-in `claude`
first on a terminal's PATH logs each line it reads (#555's); a scripted ACP agent thread in a tab
(#717's); the MCP stand-in client (`mcp_agent tool`) as the drafting agent.
- `769-01-draft`: `draft_message` to the terminal's Claude Code opens the Draft tab: the sender,
  the target, the text, Send and Drop; the call answered `opened`, the stand-in logged nothing.
- `769-02-sent`: text added in the editor, Send: the tab gone, a toast; the stand-in logged the
  whole edited text.
- `769-03-dropped`: a draft to the thread, Drop: the tab gone, the thread got nothing.
- `769-04-thread`: a second draft to the thread, Send: the thread shows it and the agent answers.

## Locked-In Decisions
- **D1 — The tool answers at once,** since an app call has 30 seconds (`APP_CALL_TIMEOUT_SECONDS`)
  and a draft can wait longer; the agent learns only that it opened.
- **D2 — Send is the person's path,** `paste_then` and `ThreadView::send`, not `terminal_type`,
  which refuses agent CLIs on purpose.
- **D3 — A family of its own,** `draft`, so a client's grants can leave it out.
- **D4 — Threads in tabs count,** through `conversations_of`.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `draft_message` with a terminal running an agent CLI or a thread, the system shall open a Draft tab naming the sender and the target, with the text editable, and answer `opened` without sending. | Shot 769-01 and the stand-in's log |
| REQ-002 | WHEN the person presses Send, the system shall deliver the editor's text to the target as the person would and close the tab. | Shots 769-02, 769-04 |
| REQ-003 | WHEN the person presses Drop, the system shall close the tab and send nothing. | Shot 769-03 |
| REQ-004 | IF the target does not exist or a terminal runs no agent CLI, THEN the tool shall refuse with `no_terminal`, `not_an_agent` or `no_thread`. | The scenario's refusal checks |
| REQ-005 | WHILE the kill switch is on, the tool shall be refused as every Write tool is. | The review |
