# New Agent in Worktree sits under New Agent Thread in the rail's + — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-598-worktree-agent-under-agent-thread.md
- **Pipeline spec:** 598-worktree-agent-under-agent-thread.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, testing with `docs/marley/walkthrough.md`: move New Agent in Worktree up to
  sit below New Agent Thread.
- **Classification / tier:** feature, one Marley file, UI order only.
- **Recall (§18.3):** AD-claude-453 (the rail's header menu reorders through Zed's multi-workspace);
  #510's spec placed the entry "after the Agent CLIs"; #527's placed Launch after it. No failure or
  prevention rule touches the menu's order.
- **Discovery:** `crates/marley_workbench/src/rail.rs`, `render_project_menu` (about line 3397):
  the builder chains `entry("New Terminal")`, `entry("New Browser Tab")`,
  `submenu("New Agent Thread")`, then `agent_cli_entries` (a separator, the "Agent CLIs" header,
  one item per CLI), then `worktree_agent_entries` (the "New Agent in Worktree" submenu, about
  line 3529), then `launch_entries`. The fix calls `worktree_agent_entries` before
  `agent_cli_entries`.
- **Decisions:** see the spec's D1 to D3.

### Visual check plan
- REQ-001, REQ-002: a sway scenario with a scratch git repository (`git init`, one commit), a fake
  `claude` first on the PATH so the CLI list is not empty, and a `.zed/marley.json` with one
  launch config so the Launch header shows; click the project's `+`; `menu.png` must read New
  Terminal, New Browser Tab, New Agent Thread, New Agent in Worktree, Agent CLIs, Claude Code,
  Launch, the config.
- REQ-003: review; the condition code is untouched.

### Risks
- `script/e2e/510-worktree-agents.sh` may walk the menu by position with the arrow keys. It is
  not run per ticket (§7), but read it at Code and note it if it depends on the old order.
