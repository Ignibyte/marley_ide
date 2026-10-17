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
