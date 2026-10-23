# Agent Activity lists Rusty's calls — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-934-agent-activity-lists-rustys-calls.md
- **Pipeline spec:** 934-agent-activity-lists-rustys-calls.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, Tools).
- **Classification / tier:** feature, M (the review said M–L; the Marley side is M once the
  Codex, Zed-agent and outside-client paths are Out). Marley crates and Marley's own plugin only.
  Depends on #933 (the row card, `Outcome`, the caller's mark) and #905 (`AgentMark`).
- **Recall (§18.3):**
  - AD-claude-519: Claude Code's hook events ride in band into Marley as OSC 777 frames; the
    terminal path reuses them.
  - F-claude-519 / PR-claude-drop-the-unbounded-fields-first: `event.py` keeps its frame under
    2,900 bytes by dropping fields; the new `result` and `error` are bounded (120) and join the
    drop order before the fields the rail shows.
  - L-claude-508: a stand-in ACP agent in Python, a custom entry in the run's `agent_servers`,
    sends `tool_call` and `tool_call_update` with no model; reused, with `_meta.claudeCode`.
  - PR-claude-687 / F-claude-687: the scenario proves the thread runs its stand-in (its log file)
    before it types into it, and opens the thread from New Agent Thread's submenu, never with
    `agent: new thread`.
  - F-claude-903: the terminal's stand-in `claude` goes first on the terminals' PATH through the
    `.bashrc` handed to `terminal_env HOME`.
  - L-claude-698: codex-acp's MCP handling was read once for #698; how it names a tool on its
    tool calls was not, which is why Codex is Out.
  - F-claude-865: a hook called from a render never reads that view; here the thread observer
    reads the thread inside its own event callback, after the thread's update, never from a
    render.
- **Discovery:**
  - `crates/marley_workbench/src/agent_activity.rs`: `init` (calls `follow_threads`),
    `follow_threads`, `ActivityRow.server`, `Outcome::Failed`, `rusty_lists`, a `record` that the
    two paths share with `log`.
  - `crates/marley_workbench/src/agent_events.rs`: `on_frame` hands Rusty tool events to
    `agent_activity`.
  - `crates/marley_agent/src/claude_events.rs`: `HookEvent.result`.
  - `crates/marley_workbench/claude_plugin/marley/hooks/event.py`, `.claude-plugin/plugin.json`,
    `claude_plugin/.claude-plugin/marketplace.json`.
- **Decisions:** D1–D6 in the spec.

### Design
- `follow_threads(cx)`: `cx.observe_new(|_: &mut AcpThread, _, cx: &mut Context<AcpThread>| …)`,
  as `bookmarks.rs:34` does for `Terminal`; inside it `cx.subscribe_self::<AcpThreadEvent>(…)`
  (gpui `app/context.rs:112`), detached, and `cx.on_release` to drop the thread's state. A
  `RustyCalls` global holds, per thread entity id, the ids of Rusty calls seen unfinished
  (`HashSet<ToolCallId>`) and those already logged.
- On `NewEntry` and `EntryUpdated(index)`: read `thread.entries()[index]` (bounds-checked with
  `get`); for `AgentThreadEntry::ToolCall(call)`, take `call.meta` → `claudeCode` → `toolName`;
  `strip_prefix("mcp__rusty__")`. Unfinished statuses add the id to the seen set; a finished
  status with the id in the seen set and not in the logged set logs the row and moves it.
- The summary: #703's `summary(&raw_input)`, with `title` and `slug` added to `SUMMARY_FIELDS`.
  The result: the first text line of `call.content()` (`ToolCallContent::ContentBlock` text), else
  of `raw_output` as a string; the error for `Failed` the same way.
- The caller: the thread's `connection().agent_id()` through #905's `AgentMark::from_agent_id`,
  and its display name for `who`.
- Terminal path: in `on_frame`, after `claude_events::decode`, when `event.event` is
  `PostToolUse` or `PostToolUseFailure` and `event.tool` starts with `mcp__rusty__`, call
  `agent_activity::record_rusty(…)` with `AgentMark::ClaudeCode`, `who` "Claude Code", the
  preview, and `event.result` or `event.error`. The frame is still folded into the fleet as now.
- `event.py`: in `summary`, for `PostToolUse` with a `tool` starting `mcp__`, read
  `tool_response` (a string, or a list of `{type: "text", text}` blocks, or an object with
  `content`) and keep its first non-empty line cut to 120 as `result`; for `PostToolUseFailure`,
  `error` cut to 120. Add `result` and `error` to the drop loop's list before `preview`.
- File manifest: `agent_activity.rs`, `agent_events.rs` (Marley crate `marley_workbench`),
  `claude_events.rs` (Marley crate `marley_agent`), the plugin files (Marley-owned); the scenario.
  No Zed path, so no `docs/marley/zed-touchpoints.md` row.

### Visual check plan
- REQ-001, REQ-003, REQ-004: open a thread of the custom agent `Rusty` from New Agent Thread's
  submenu; wait for the stand-in's log to show `session/new`; type a prompt and Enter; the
  stand-in reports the three calls; open Agent Activity; `934-01-thread` shows the `R` rows for
  `brain_decide` ("page written") and `task_update` (red "no task 42"), and no `brain_search`.
- REQ-002, REQ-005: New Terminal; run `claude`, whose stand-in pipes a `PostToolUse` JSON for
  `mcp__rusty__memory_add` (`tool_response` "memory saved") through `event.py` and prints the
  sequence; `934-02-terminal` shows the `CC` row `rusty · memory_add` with "memory saved". The
  scenario also decodes the frame `event.py` printed and checks it carries `result`.
- Marley Day: `934-03-day`.
- REQ-006: the review of the diff (the seen set). Reopening the thread from history in the run
  and checking no row is added would need the thread store's history; taken as far as the diff.

### Risks
- A newer claude-agent-acp could move `toolName`; the match also accepts the call's `title` when
  it is exactly `mcp__rusty__<tool>`, and the scenario's stand-in follows 0.94.0's shape.
- `observe_new` on `AcpThread` fires for every thread, Zed's agent's too; the callback does no
  work until a Rusty tool call shows.
- A user who renames Rusty's server in a project's `.mcp.json` (not `rusty`) is not matched; the
  name Marley and Rusty's entry use is the one matched (D2).
- Plugin version bump: the scenario's `installed_plugins.json` must list the new version so the
  agent bar offers no update, as #508's does with `SHIPPED`.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
