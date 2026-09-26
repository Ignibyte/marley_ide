# Zed's own agents get Marley's tools, the browser among them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-501-zeds-agents-drive-the-browser.md
- **Pipeline spec:** 501-zeds-agents-drive-the-browser.spec.md

## Phase 1 — Plan (2026-09-25)
- **Request:** Chad: "the built in zed agents should be extended or made to be able to drive the
  browser as well if it hasnt already".
- **Classification:** feature, small; `marley_workbench` (`mcp.rs`, `claude_plugin.rs`). No Zed
  path.
- **Recall (§18.3):**
  - The brain (consultation `03d995363927486d8e1718e72b59c2a6`): nothing on this seam.
  - #491: the server runs in the app on loopback with a bearer and writes `mcp-endpoint.json`;
    the plugin's bridge reads it and serves over stdio; Claude Code uses it that way.
  - Zed: `ContextServerStore::maintain_servers` starts every enabled server of
    `ProjectSettings::context_servers` (and the registry's); the Zed Agent's Write profile has
    `enable_all_context_servers: true`; `mcp_servers_for_project` hands the configured stdio and
    HTTP servers to `session/new`. The layout (`apply_defaults`) already patches defaults with
    `SettingsStore::update_default_settings`.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, pre-flight ✓, recall ✓, prior
  art ✓, spec ✓, design ✓.

### Design
- `claude_plugin.rs`: the bridge's text as `pub(crate) const BRIDGE`, which `FILES` uses too.
- `mcp.rs`: when the server has started, `offer_to_zeds_agents(data_dir)`: off the main thread,
  write `<data dir>/mcp/marley-mcp-bridge` (0755); then `update_default_settings` inserts
  `context_servers.marley = Stdio { command: that path, env: { MARLEY_MCP_ENDPOINT:
  <data dir>/mcp-endpoint.json } }`. A write that fails is logged and leaves Zed's agents
  without the tools, as before.
- **Manifest:** `crates/marley_workbench/src/mcp.rs`, `claude_plugin.rs`;
  `script/e2e/501-zeds-agents-drive-the-browser.sh`. No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline) | Shot or log |
|---|---|---|
| REQ-001 | `agent: open settings` | `501-01-servers` |
| REQ-002 | a stand-in ACP agent (custom agent server in the run's settings) logs `session/new`'s `mcpServers` | the run log |
| REQ-003 | its thread from the rail's +, the prompt "open <url>"; it calls `browser_navigate` through the `marley` server it was handed | `501-02-driven` |
| (regression) | #492's scenario | its shots |

The Zed Agent itself needs a model to call a tool, which the offline run has none of; its side
is shown by the server and its tools in its settings (REQ-001).

### Risks
- The stand-in speaks just enough ACP (initialize, session/new, session/prompt) for the run.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built:** `claude_plugin.rs`: the bridge's text as `pub(crate) const BRIDGE`, which the
  plugin's `FILES` uses. `mcp.rs`: `offer_to_zeds_agents`, called when the server has started:
  `write_bridge_in` writes `<data dir>/mcp/marley-mcp-bridge` (0755) off the main thread, then
  `update_default_settings` inserts `context_servers.marley` (`CONTEXT_SERVER`) as a stdio
  server running it, `MARLEY_MCP_ENDPOINT` naming `<data dir>/mcp-endpoint.json`; a failed write
  is logged.
- **Deviations from the design:** none.
- **Review:** no bearer in any setting (the bridge reads the file); the default only, so a
  user's own entry replaces it; nothing registers when the server did not start. REQ-001 rests on
  the Write profile's `enable_all_context_servers`; REQ-002 on `mcp_servers_for_project`, which
  reads the merged `context_servers`.
- **Checks:** `cargo clippy -p marley_workbench --all-targets -- -D warnings` clean; `cargo fmt`
  clean.

## Phase 2 — closeout
Phase 2 PASS (autonomous).

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/501-zeds-agents-drive-the-browser.sh` (`compositor sway`, offline
  Chromium): the scratch repository; a stand-in external agent (just enough ACP in Python:
  initialize, session/new, session/prompt) set as the custom agent server "Stand-in" inside the
  run's settings' own `agent_servers` block; a loopback page. Steps: Marley's log line and the
  running bridge; the rail's +, New Agent Thread, Stand-in; the prompt "open <url>".
- **The run log:**
  - REQ-001: "Zed's own agents reach Marley's tools through the context server marley, which
    runs <profile>/mcp/marley-mcp-bridge", and `python3 <profile>/mcp/marley-mcp-bridge`
    running: Zed's context server store started it for the project, as it does every server the
    Zed Agent's Write profile takes tools from.
  - REQ-002: `session/new mcpServers: [{"name": "marley", "command": "<profile>/mcp/marley-mcp-bridge",
    "args": []}]`, its env naming only `MARLEY_MCP_ENDPOINT`: no bearer in what Zed handed over.
  - REQ-003: `browser_navigate: {"did": "went to http://127.0.0.1:…/index.html", …, "title":
    "Driven by an agent"}`.
- **Shots (in the scratchpad, `e2e-501/`), each read:**
  - `501-01-agents`: the rail's + menu, New Agent Thread's submenu: Zed Agent, Claude Agent,
    Stand-in (selected).
  - `501-02-driven` (REQ-003): the Stand-in's thread in the Agent Panel with the prompt and its
    answer "I opened http://… in Marley's Browser tab.", and a Browser tab "Driven by an agent"
    showing the page; the rail lists the thread.
- **What the first run found:** `agent: open settings` now opens Zed's Settings window at its AI
  page (`OpenSettingsPage`), which the run's shot did not show, so REQ-001's proof is the run log
  instead, and the registration logs a line; the submenu opens on its first entry, so the
  stand-in is two steps down (`L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001`).
- **Regressions:** `492-browser-tools.sh`: its shots and log as before, 20 tools listed.
- **Focus report:** every run in the headless sway; "hyprland: 0 Marley windows before the run,
  0 after; the run added no rule and did not reload it".
- **Not reached by a scenario:** the Zed Agent calling a tool needs a language model, which the
  offline run has none of; the store running `marley` is what the Write profile's tools come
  from.
- **Gate:** `just gate-diff` — 16 passed, 0 failed, `GATE GREEN [diff]`; the receipt matches the
  tree.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, 492 ✓, gate ✓.

## Phase 3 — closeout
Phase 3 PASS (autonomous).

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` (#501 under Added); `docs/marley_architecture/marley_workbench.md`
  (Marley's MCP server: the context server), `marley_mcp.md` (Zed's own agents as clients). No
  path outside the Marley-owned set changed.
- **Knowledge appended:** `L-claude-501-zeds-agents-take-tools-from-the-projects-context-servers-001`,
  `AD-claude-501-marleys-server-is-a-default-context-server-001`.
- **Brain:** consultation `03d995363927486d8e1718e72b59c2a6` closed with
  `decisions/marleys-mcp-server-is-a-default-context-server-for-zeds-own-agents`.
- **Ticket:** closed.
