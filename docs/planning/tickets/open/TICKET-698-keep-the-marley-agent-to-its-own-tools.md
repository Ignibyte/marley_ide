# TICKET-698 — Keep the Marley agent to its own tools on Claude Code and Codex

- **Ticket:** LOCAL #698 (bug)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** (set at promotion)
- **Source ticket:** found while planning #696
- **Status:** open

## Summary
The Marley agent's instructions say it edits no file and runs no command. On Claude Code, only
Claude Code's own file and shell tools are disallowed. Zed hands every agent the whole `marley`
MCP server, so the entry can still call `terminal_run`, `terminal_type` and the browser's write
tools, which run commands and type into programs. Codex's read-only sandbox doesn't cover MCP
tools either. Zed's agent is already limited to the eight tools by its profile.

Now that the agent is on by default (#696), make the limit true:
- **Claude Code:** add every served Marley tool outside `PROFILE_TOOLS` to `disallowedTools` as
  `mcp__marley__<tool>`, built from `marley_mcp`'s registry so a new tool is covered.
- **Codex:** find whether `CODEX_CONFIG.mcp_servers.marley.disabled_tools` (or the adapter's
  equivalent) holds against the server ACP passes, and use it.

## Acceptance
The Marley entry's session `_meta` shall name every Marley tool but its eight as disallowed, and
a Codex Marley entry's config shall do the same.
