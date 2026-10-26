---
pipeline_id: d9425938-58ff-4d71-adbd-ad5980b86cb2
ticket: docs/planning/tickets/open/TICKET-934-agent-activity-lists-rustys-calls.md
status: Phase 4 — Complete PASS
title: Agent Activity lists Rusty's calls
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/F-tools.md, docs/planning/pipeline/queued/933-tools-page-as-the-board.spec.md]
---

## Title
Calls that Claude Code makes to Rusty's tools, in the Agent Panel or in a Marley terminal, listed
in Agent Activity beside Marley's own, as the Tools board's `R brain_decide … page written` row
shows.

## Scope
### In
- **The calls in Agent Panel threads** (Marley crate, `agent_activity.rs`, new `follow_threads`).
  `init` registers `cx.observe_new::<AcpThread>`; each new thread is followed on
  `AcpThreadEvent::NewEntry` and `EntryUpdated`. A tool call is Rusty's when its Claude Code tool
  name (`_meta.claudeCode.toolName`, which claude-agent-acp sets on every tool call) is
  `mcp__rusty__<tool>`. When such a call reaches `Completed`, `Failed`, `Rejected` or `Canceled`
  and it was first seen unfinished in this run, it is logged once: the caller's mark from the
  thread's agent (`R` for Rusty's entry, `CC` for Claude Code, by #905's `AgentMark` from the
  thread's `AgentId`), the tool's own name, the summary from its raw input, and how it ended: the
  first line of its text result for `Completed`, the error's first line for `Failed`, "you
  refused" for `Rejected`, "canceled" for `Canceled`.
- **The calls in Marley terminals** (Marley crate, `agent_events.rs` `on_frame`). A decoded
  `PostToolUse` or `PostToolUseFailure` whose tool is `mcp__rusty__<tool>` is logged with the
  terminal's Claude Code as the caller (`CC`), the event's preview as the summary, and its
  `result` or `error`.
- **The plugin's hook** (Marley-owned, `crates/marley_workbench/claude_plugin/marley/hooks/event.py`).
  For a tool named `mcp__…`, `PostToolUse` adds `result`, the first line of the tool response's
  text, and `PostToolUseFailure` adds `error`; each cut to 120 characters and dropped first when
  the frame is over its bound. The plugin's version moves on (`plugin.json`, `marketplace.json`),
  as #570 did.
- **The decoder** (Marley crate, `crates/marley_agent/src/claude_events.rs` `HookEvent`): a
  `result: Option<String>` beside the `error` it has.
- **The row** (`agent_activity.rs`). `ActivityRow` gains `server: Server`, `Marley` (the default,
  so old lines read) or `Rusty`. A Rusty row whose caller is not Rusty's entry shows its tool as
  `rusty · <tool>`, so the server is never in doubt; under the `R` tile the tool stands alone, as
  the board draws it. `Outcome` gains `Failed { error: Option<String> }` for a call that ran and
  failed, drawn in the error colour. The result and error are redacted and cut as #933's lines
  are.
- **Which of Rusty's calls are listed** (`agent_activity.rs`, `rusty_lists`). Every call except
  those that only read: a tool name ending in `_list`, `_view`, `_search`, `_read`, `_get`,
  `_preview`, `_graph`, `_due` or `_status`, and `changes_since`. `brain_ask` is listed: it
  records a consultation. A `secret_…` tool is listed with its outcome only, never its result's
  text.

### Out (explicitly deferred)
- Rusty's entry, or any thread, running on Codex, and Codex in a terminal: how codex-acp names an
  MCP tool on its tool calls, and Codex's events in a terminal, need their own check. A follow-up
  ticket.
- Zed's own agent with the Rusty profile: its tool calls carry the bare tool name
  (`ContextServerTool::name`, `crates/agent/src/tools/context_server_registry.rs:305`), with no
  server. Telling Rusty's tools apart needs a Zed hunk that puts the server's id on the call; a
  follow-up ticket.
- Calls from programs outside Marley (a Claude Code with no Marley plugin, `rusty-cli`, another
  machine): Marley sees nothing of them. Rusty-side work: rusty-mcp would keep a log of each
  `tools/call` with its client's name and serve it (a tool such as `calls_since`), which Marley's
  own connection to Rusty could read. A Rusty ticket.
- Rusty's tools marking themselves read-only (`readOnlyHint` in rusty-mcp's `tools/list`), which
  would replace the suffix list: Rusty-side, the same Rusty ticket.
- Stop for Rusty's tools: the kill switch refuses Marley's own tools; it cannot refuse a call
  Marley does not carry.
- The row layout, the tiles and the result lines of Marley's own calls: #933.

## Reference (§20)
N/A — Marley-specific: Agent Activity is Marley's own log (#703). The calls are read from what
Zed's `acp_thread` already holds for each Agent Panel tool call (`ToolCall::meta`, `raw_input`,
`status`) and from #519's hook events; neither Zed's behaviour nor the ACP wire changes.

### Prior art
- `crates/marley_workbench/src/agent_activity.rs:1-7`, `:164-187`: the log covers Marley's
  write-tier tools only, through `AppCall::on_answer`. Grepped `rusty` and `brain` in it: none.
  Gap holds at HEAD.
- `crates/marley_workbench/src/rusty.rs:283-286`: Marley's own connection to Rusty
  (`MARLEY_SERVER`) and Zed's agents' (`CONTEXT_SERVER`, `rusty`) both log by size only;
  `crates/marley_workbench/src/assistant.rs:20-30`: Rusty's entry on Claude Code or Codex carries
  Rusty's server in its session's `_meta`, so the agent's own process talks to Rusty and Marley
  never carries the call.
- claude-agent-acp 0.94.0 (Apache-2.0, installed under Marley's `external_agents/registry/npx/claude-acp`):
  `dist/tool-calls/renderer.js:326`, `:385`, `:397` put `claudeCode.toolName`, the Claude Code
  tool's name, in each tool call's `_meta`; `:598-603` splits `mcp__<server>__<tool>`.
- `crates/acp_thread/src/acp_thread.rs:1096-1121` `ToolCall` (`meta`, `raw_input`,
  `raw_output`, `status()` at `:1410`), `:1902-1917` `ToolCallStatus`, `:3601-3631`
  `AcpThreadEvent` (`NewEntry`, `EntryUpdated`). `crates/marley_workbench/src/rail.rs:2965-2977`
  subscribes to every thread's events; `thread_tab.rs:847-862` walks a thread's tool calls;
  `bookmarks.rs:34` uses `observe_new` on a Zed entity type, the hook this ticket uses for
  `AcpThread`.
- `crates/marley_workbench/claude_plugin/marley/hooks/event.py:94-131` `summary`: tool events
  carry `tool`, `preview` and `tool_use_id`; `error` only for `StopFailure`.
  `crates/marley_agent/src/claude_events.rs:82-110` `HookEvent` has `tool`, `preview`, `error`.
  `crates/marley_workbench/src/agent_events.rs:295-335` `on_frame` decodes each frame.
- rusty-mcp (`/srv/stacks/rusty-v3/crates/rusty-mcp/src/main.rs:2425-2445`, `:2516-2535`): the
  tool names and families; no `annotations` in its `tools/list`, and its `changes` table records
  changes, not calls or callers.
- #905's `AgentMark` (`R` for Rusty, from a thread's `AgentId`); #933's `Outcome` and row card.

## UI proof
Scenario `script/e2e/934-agent-activity-lists-rustys-calls.sh` (sway). Setup: Rusty off; a
stand-in ACP agent (#508's, in Python) as the custom agent server `Rusty` in the run's settings,
which on each prompt reports Claude Code's tool calls `mcp__rusty__brain_decide` (completed, "page
written"), `mcp__rusty__task_update` (failed, "no task 42") and `mcp__rusty__brain_search`
(completed); a stand-in `claude` first on the terminals' PATH through the scenario's `.bashrc`
that pipes a `PostToolUse` for `mcp__rusty__memory_add` through the plugin's real `event.py`.
- `934-01-thread`: Agent Activity after the thread's turn: an `R` row for `brain_decide` with
  "Ship the home page" and "page written", and an `R` row for `task_update` with "no task 42" in
  red; no `brain_search` row (REQ-001, REQ-003, REQ-004).
- `934-02-terminal`: after the terminal's stand-in ran: a `CC` row for `rusty · memory_add` with
  its result (REQ-002, REQ-005).
- `934-03-day`: Agent Activity in Marley Day (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — Marley lists what it can see without changing Rusty: Claude Code's own report of each call,
  in the Agent Panel and in a terminal. Everything else is Out, named.
- D2 — A call is Rusty's by Claude Code's name for it, `mcp__rusty__<tool>`, where `rusty` is the
  name Marley gives Rusty's server (`rusty::CONTEXT_SERVER`) and the one Rusty's entry passes.
- D3 — A call is logged once, when it finishes, and only when this run saw it unfinished, so a
  thread loaded from history adds no rows.
- D4 — Reads are not listed, as for Marley's own tools; Rusty does not mark its reads, so a suffix
  list stands in until it does (Out).
- D5 — The tile is the caller; a Rusty row from another agent names the server in its tool line.
- D6 — A secret tool's result text is never stored; every result and error is redacted and cut
  before it reaches the day file.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an Agent Panel thread's call to one of Rusty's tools that changes something finishes, Agent Activity shall list it with the thread's agent's tile, the tool, what it acted on and its result. | `934-01-thread`, `934-03-day` |
| REQ-002 | WHEN Claude Code in a Marley terminal finishes a call to one of Rusty's tools that changes something, Agent Activity shall list it with the `CC` tile, `rusty · <tool>`, what it acted on and its result. | `934-02-terminal`, `934-03-day` |
| REQ-003 | WHEN such a call fails, its row shall show the error in the error colour. | `934-01-thread` |
| REQ-004 | A call to one of Rusty's tools that only reads shall not be listed. | `934-01-thread` |
| REQ-005 | The plugin's hook shall carry the first line of an MCP tool's result or error, cut to 120 characters, and keep its frame under its bound. | `934-02-terminal`; the review of the diff |
| REQ-006 | A Rusty call shall be logged once, and a thread loaded from history shall add no rows. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `agent_activity.rs`, `agent_events.rs`, `marley_agent`'s `claude_events.rs`, the
  plugin's `event.py` and version; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
