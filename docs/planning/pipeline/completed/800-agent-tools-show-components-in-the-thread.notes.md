# Agents show components in the thread — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-800-agent-tools-show-components-in-the-thread.md
- **Pipeline spec:** 800-agent-tools-show-components-in-the-thread.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17 (see #799).
- **Discovery (drafted by /spec; re-verified at promotion):**
  - Tool content: `acp_thread::ToolCallContent` (`crates/acp_thread/src/acp_thread.rs`),
    rendered by `render_any_tool_call`, `render_tool_call` and `render_tool_call_content` in
    `crates/agent_ui/src/conversation_view/thread_view.rs`; `meta` is carried and unread.
  - The name precedent: `render_numbered_read_file_output` (`tool_name == "read_file"`).
  - Zed's agent's MCP tools: `ContextServerTool::run`
    (`crates/agent/src/tools/context_server_registry.rs`) keeps text and images only.
  - Marley's MCP: `crates/marley_mcp/src/registry.rs` (`Family`, `is_served`, `REGISTRY`,
    `tool_schemas`), `crates/marley_mcp/src/dispatch.rs` (`tools_call`, deferred app calls),
    `crates/marley_workbench/src/mcp.rs` (`answer`, `offer_to_zeds_agents`).
  - Tab precedent: `review_guide_open` (`crates/marley_workbench/src/review_tools.rs`) and
    `guide_tab::open`.
- **Open for promotion:** each agent's reported tool name (Claude Code's ACP adapter, Codex's,
  Zed's agent), and the caller identity the bridge sends for a terminal agent.
- **Recall (§18.3):** PR-claude-798-read-no-pane-bounds-in-render-001 (the hook draws in render:
  it reads only the tool call's own data); #733's `review_guide_open` (a tool that opens a tab);
  #520's caller identity (`caller_terminal`), which tells a terminal agent's call.
- **Promoted 2026-10-17**; seams re-read: `render_tool_call` in `thread_view.rs` (the body's
  `.children(tool_output_display)`), `ToolCall { name, tool_name, raw_input, label }`,
  `MarleyThreadHost` in `agent_ui.rs`, `Family` and `REGISTRY` in `marley_mcp/src/registry.rs`,
  the deferred arm in `dispatch.rs`, `answer` in `marley_workbench/src/mcp.rs`.
- **Checklist (no TaskCreate):** pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓, spec ✓,
  design ✓.

### Design
- **`marley_mcp/src/registry.rs` (Marley):** `Family::Component` (`component`, served),
  `component_list` and `component_show` (read tier), `component_schemas`; `dispatch.rs` defers
  the family to the app.
- **`marley_workbench/src/component_tools.rs` (Marley, new):** `answer` (list from
  `marley_components::catalog`; show: parse, answer the error with its path, or open a Component
  tab beside a terminal caller, or say the thread draws it), `tool_call_view` (the thread's
  drawing, from the call's input), `ComponentTab`; `init` sets `agent_ui::MarleyToolCallView`.
- **`agent_ui.rs` (Zed, additive):** `MarleyToolCallView { draw }`, a `Global`.
- **`thread_view.rs` (Zed, additive):** in `render_tool_call`, a completed call the global draws
  gets the drawing under its header, whether the row is open or not; the name is `tool_name`, else
  `name`, else the label's text.

### Visual check plan
- `script/e2e/800-agent-tools-show-components-in-the-thread.sh`, `compositor sway`. A scripted ACP
  agent as Marley's own agent (#744's set-up) answers a prompt with four completed tool calls named
  as the agents name them: `mcp__plugin_marley_marley__component_show` (Claude Code's plugin),
  `marley.component_show` (a dotted name), `component_show` (Zed's agent's context-server tool),
  and one with an unknown type; shots of the thread (REQ-001, REQ-003 row). The row's disclosure
  opened shows the raw input (REQ-002). A terminal runs a stand-in MCP client through the plugin's
  bridge: `component_list` printed (REQ-005), `component_show` with a bad tree prints
  `shown: false` and the path (REQ-003 answer), with a good tree opens a Component tab (REQ-004).

## Phase 2 — Code
- **Built:** `Family::Component` with `component_list` and `component_show` (read tier) and
  their schemas in `marley_mcp/src/registry.rs`, deferred to the app in `dispatch.rs`;
  `marley_workbench/src/component_tools.rs` (the answers, the thread's drawing, `ComponentTab`,
  `init` setting the global); the route in `mcp.rs`; `agent_ui.rs`'s `MarleyToolCallView`; the hook
  in `thread_view.rs`'s `render_tool_call`.
- **Deviations:** `component_list`'s description leaves out templates until #801 adds them. The
  thread names a call by `tool_name`, then `name`, then the label's text: Claude Code's adapter
  reports an MCP tool by its title alone.
- **Review of the diff:** the hook reads only the call's own fields and runs only for a completed
  call, so a half-streamed input never shows an error; the drawing has an element id per entry; the
  tab opens in the caller terminal's pane through the window that holds it
  (`browser::window_of`), never inside another window's update.
- **Gate run 1 (`800-gate-1.log`):** red on gate:14, a module doc linking the private
  `tool_call_view`. **Gate run 2 (`800-gate-2.log`): GATE GREEN [diff]**, after the scenario.

## Phase 3 — Test
- **Scenario:** `script/e2e/800-agent-tools-show-components-in-the-thread.sh` (`compositor sway`):
  a stand-in agent CLI through the plugin's bridge in the repo's terminal, and Marley's own agent
  on a scripted ACP agent answering with four completed `component_show` calls.
- **Shots, each read (`scratchpad/shots-800/`):**
  - `800-07-list`: `agent list` prints "types: row column card divider text heading code table
    key_value stat bar_chart sparkline badge progress callout" (REQ-005).
  - `800-05-answer`: `agent bad` prints `{"shown": false, "error": {"path": "$.children[1]",
    "message": "The type `gauge` is not in the catalog. …"}}` (REQ-003, the answer).
  - `800-06-terminal`: `agent good` opens a "Deploy" tab beside the terminal with the staging
    card, its stat and an 80% rollout bar (REQ-004).
  - `800-01-thread`: the thread draws the table under `mcp__plugin_marley_marley__component_show`
    (the title alone), the bar chart under "Show a component" (`_meta.tool_name`
    `marley.component_show`), the stats under `component_show`, and under the last call the error
    "$.children[0]: The type `gauge` is not in the catalog…" (REQ-001, REQ-003, the row).
  - `800-04-raw`: the first row opened: its raw input as JSON and "Output: shown" (REQ-002).
  - `800-02-more`, `800-03-error`: the thread fits one screen, so these repeat `800-01-thread`.
- Real Claude Code, Codex and Zed-agent sessions are stood in by the names their ACP tool calls
  carry; a live session against a model is out of a scenario's reach (no model, no network).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` ("Agents draw components"); `docs/marley/guide.md` ("Agents draw
  components (#800)"). Touchpoints rows: `agent_ui.rs` (`MarleyToolCallView`), `thread_view.rs`
  (the hook); gate:16 green.
- **Knowledge:** nothing new beyond #798's and #799's lessons, which the scenario followed.
- **Brain:** no decision recorded.
