---
pipeline_id: a0ad069e-00fb-42c2-b6ca-cb36dc30edb9
ticket: docs/planning/tickets/open/TICKET-706-agent-panel-threads.md
status: Phase 4 — Complete PASS
title: Agent Panel threads over MCP
type: feature
slice: Marley's MCP server (docs/planning/intake/zed-control-over-mcp.md), the fourth of #703 to #707
references:
  - docs/planning/pipeline/completed/704-editors-read-and-open.spec.md
  - docs/planning/pipeline/completed/705-editors-edit.spec.md
---

## Title
An agent sees the Agent Panel's threads, reads them, posts into one, and answers another's pending
permission, each behind the threads area's mode.

## Scope
### In
- **`thread_list`** (read): the live threads in every window's Agent Panel. Each has its id (the
  thread's key), its title, its agent, its project, whether it runs, and its pending permission:
  the tool's words, and whether an agent may answer it.
- **`thread_read`** (read): a thread as Markdown (its messages and tool calls), redacted, paged.
  It is listed in Agent Activity.
- **`thread_post`** (act): it types a message into a thread and sends it, as the user would. A
  thread that is running queues it.
- **`thread_answer`** (sensitive): Allow Once or Reject Once on a thread's pending permission. A
  sandbox escalation is refused outright (`sandbox_escalation`), as the rail refuses it, since
  the Agent Panel guards those behind its surprising-Unicode check.
- **The area mode** `marley.agent_control.threads` (`ask_first` by default), as #704's editors.

### Out (explicitly deferred)
- Starting a thread, choosing its agent, or closing one.
- Allow Always answers.

## Reference (§20)
Upstream Zed, agent_ui and acp_thread, through public APIs:
- `AgentPanel::conversation_views`, `ConversationView::{root_thread_view, pending_tool_call,
  thread_view, parent_id}`;
- `AcpThread::{title, status, to_markdown}`;
- `ThreadView::{message_editor, send, authorize_permission_request}`;
- the rail's `thread_entry` and `answer_thread` (#508), which already answer from outside the
  panel.

No Zed hunk. ACP's `session/request_permission` and its option kinds.

### Prior art
- **The code we ship:** `rail.rs`'s `live_threads`, `thread_entry` (its sandbox rule) and
  `answer_thread`, plus `agents::thread_agent_name`; #704's `agent_control` and paging.
- **The Explore pass (2026-10-09):** every method above is `pub`, and `ThreadView` and
  `MessageEditor` can't be named outside `agent_ui` but their methods work on the entities the
  panel hands out.

## UI proof
`script/e2e/706-agent-panel-threads.sh`, under `compositor sway`. #697's scripted ACP agent
(`MARLEY_ASSISTANT_ADAPTER`) runs the Marley entry, and it asks for permission when told to.
#704's scripted MCP client (`e2e-agent`) is the other agent.

Shots:
- `706-01-post-asked`: `thread_post` asks first (threads area, `ask_first`).
- `706-02-posted`: after Allow, the thread shows the posted message and the agent's answer.
- `706-03-answer-asked`: the thread waits on its permission prompt, and `thread_answer` asks every
  time.
- `706-04-answered`: after Allow, the prompt is answered: the agent reports "Answered: allow".

## Locked-In Decisions
- **D1:** a thread's id is its key (`ThreadId::to_key_string`), as the rail's rows use.
- **D2:** a post goes through the message editor and `ThreadView::send`, so the panel's own queue
  holds while the thread runs.
- **D3:** answers are sensitive, ask every time unless `allow`, and are never given to a sandbox
  escalation.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `thread_list`, the system shall answer each live thread with its id, title, agent, project, whether it runs, and any pending permission. | The client's reply |
| REQ-002 | WHEN an agent calls `thread_read`, the system shall answer the thread's messages and tool calls as Markdown, redacted and paged, and list the read. | The client's reply |
| REQ-003 | WHEN an agent calls `thread_post` and the user allows it, the system shall send the message into that thread as the user would. | Shots 706-01, 706-02 |
| REQ-004 | WHEN an agent calls `thread_answer` on a pending permission, the system shall ask the user every time unless the area is `allow`, then answer the prompt; a sandbox escalation shall be refused. | Shots 706-03, 706-04 |
| REQ-005 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the registry's `thread` family, `Area::Threads` and its setting, and
  `thread_tools.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check, every shot read.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close, archive,
  commit.
