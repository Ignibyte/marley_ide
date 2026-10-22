# The System tab's other pages — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-885-more-system-pages.md
- **Pipeline spec:** 885-more-system-pages.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #884, taken whole: the splits so far left a ticket behind
  each one, and this one is Marley's own code in one file.
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** #855 made the tab; an Explore pass found each page's facts (MCP server,
  Browser hub, Fleet, Rusty, the plugin and CLIs, the shells).
- **Decisions:** D1, D2 in the spec.

### Design
- `system_pages.rs`: `Mark { Ok, Warn, Bad, Idle }` and `Row { mark, name, detail, value }`;
  `render_row(&Row)`; `Page` gains the seven; each page's rows from a function (`install_rows`,
  `trouble_rows`, `shell_rows`, `mcp_rows`, `chromium_rows`, `fleet_rows`, `developer_rows`);
  `Checks { git, clis }` read on the background executor by `check`; buttons per page.
- `clients.rs`: `server_url(cx)`, `paired_clients(cx)`. `browser.rs`: `BrowserHub::projects()`.
- **File manifest:** `system_pages.rs`, `clients.rs`, `browser.rs`, the scenario.

## Phase 2 — Code
- **Built:** `system_pages.rs`: `Mark` and `Row` (with `line` for a report); `Page` gains Install,
  Trouble, Shell, Mcp, Chromium, Fleet, Developers after Build and Data; `install_rows`,
  `trouble_rows` (with `version_row` from `agent_versions::found`, the plugin global, Rusty's
  `is_on`, `is_connected`, `unavailable` behind a `has_global`), `shell_rows`, `mcp_rows`,
  `chromium_rows`, `fleet_rows` (with `provider_kind`), `developer_rows`; `Checks` (git and the
  agent CLIs on Marley's search path) read by `check` on the background executor; Copy a report,
  Open the logs, Run the checks again. `clients.rs`: `server_url`, `paired_clients`.
  `browser.rs`: `BrowserHub::projects`.
- **Fix in Test:** the shell rows first said Marley's file runs before the user's; the module's
  own doc says bash's rcfile reads `~/.bashrc` first and zsh and fish hand back the user's
  folders. The rows now say that.
- **Gate:** green, 17 of 17, after the fix; the scenario re-run on it.
- **Review:** nothing writes or starts; `installed_clis` runs `which` off the main thread.


## Phase 3 — Test
- **Scenario:** `script/e2e/885-more-system-pages.sh` (sway): stand-in `claude` and `codex`
  printing versions; the page list's rows 29 px apart from #855's shot.
- `885-01-trouble`: shell integration on; Rusty off (quiet); Claude Code 2.1.200 and Codex 0.80.0
  found; the plugin not installed (warning); Gemini CLI and OpenCode found on the box's path; git
  found; Run the checks again and Copy a report. REQ-001 and REQ-002 met.
- `885-02-mcp`: running at `http://127.0.0.1:<port>/mcp`, the address file, no paired programs.
  REQ-001 met.
- `885-03a` to `885-03e`: Install and update (running commit, its binary, the install line, Open
  the logs), Shell integration, Marley's Chromium ("none started"), Fleet providers
  ("marley.fleet.providers is empty"), For developers. REQ-001 met.
- `885-04-day`: Marley Day, Troubleshooting. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation c292fe89.
- Ticket closed whole; pipeline archived.
