# C0: Marley's MCP server runs in the app — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-491-marley-mcp-in-the-app.md
- **Pipeline spec:** 491-marley-mcp-in-the-app.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; the agents' way in (plan D17). Prong 2's C0, pulled forward.
- **Classification:** feature; `marley_mcp` (deferred calls, the terminal family, wire names),
  `marley_workbench` (start, stop, answering calls; the plugin's files). No Zed path expected:
  the scenario points the bridge at its profile with `terminal_env`, so Zed's terminal needs
  no new variable.
- **Recall (§18.3):**
  - The never-logged-bearer rule (#370 to #375 lineage): the bearer never reaches a log, a
    tool result or a file other than the 0600 endpoint file.
  - AD-claude-482-…: the plugin is installed from the agent bar's chip with `claude plugin`
    from Marley's local marketplace; a new version must reach installed copies.
  - orchestration-shell.md §10: read tools loose, write tools granted; observation scopes
    (TICKET-043), superseded here for Marley's own tools (D5).
- **Discovery:** `marley_mcp`'s public surface (`transport::spawn`, `ServerData`, `Effect`,
  `Handled`, the const registry); Claude Code's plugin MCP rules from the docs agent.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
