# MCP Servers as a card per agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-846-mcp-servers-and-agent-access.md
- **Pipeline spec:** 846-mcp-servers.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #876.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #877 (the run's Claude Code config is its own, so the page shows none of
  the user's servers).
- **Discovery:** a probe run: the page lists AGENT CLIS, then CLAUDE CODE's servers, CODEX's,
  ZED's, each a flat section.
- **Decisions:** D1 in the spec.

### Design
- `mcp_servers.rs`: `card(cx)`; `render` builds Claude Code's card (`render_agent(Claude)`,
  then `render_read` with its heading "Servers" and the plugins line), Codex's
  (`render_agent(Codex)`, `render_read`), and Zed's (heading "Zed", its rows); `render_read` takes
  an id key so the two Add Server buttons keep distinct ids; `render_agents` goes.
- **File manifest:** `mcp_servers.rs`; the scenario.

## Phase 2 — Code
- **Built:** `mcp_servers.rs`: `card(cx)`; the page builds Claude Code's card (`render_agent`, its
  servers under SERVERS with Add Server, the plugins line), Codex's (`render_agent`, SERVERS) and
  Zed's (its rows); `render_read` takes an id key (`claude`, `codex`) so each Add Server keeps its
  own id; `render_agents` went.
- **Gate:** green on the first run, 17 of 17.
- **Review:** the refresh, the forms and Remove are unchanged.


## Phase 3 — Test
- **Scenario:** `script/e2e/846-tools.sh` (sway).
- `846-01-servers`: three cards: Claude Code (2.1.307 at its path; Prompt tags on, IDE links,
  selections and mentions, the shared plugin; SERVERS with Add Server and "No MCP servers for the
  open projects (…/claude-config/.claude.json)"; the plugins line), Codex (not on Marley's PATH;
  App Servers; SERVERS), Zed (marley, running in repo, Marley's). REQ-001 met.
- `846-02-activity`: Agent Activity as it is.
- `846-03-day`: Marley Day, the cards. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 45618e5c.
- Ticket closed; pipeline archived.
