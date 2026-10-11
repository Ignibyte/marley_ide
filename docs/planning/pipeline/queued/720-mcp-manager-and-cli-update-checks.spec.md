---
pipeline_id: 08c4f17a-cba0-4377-9bb2-b38829a5a491
ticket: docs/planning/tickets/open/TICKET-720-mcp-manager-and-cli-update-checks.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: An MCP manager and agent CLI update checks
type: feature
slice: agents on their own tools; MonoCode findings item 7 (docs/planning/intake/monocode-findings.md), the first of two slices
references:
  - docs/planning/intake/monocode-findings.md
  - docs/planning/design-notes/tabs-everywhere-2026-10-11.md
  - docs/planning/pipeline/completed/648-agent-version-check.spec.md
  - docs/planning/pipeline/completed/709-load-the-shared-claude-code-plugin.spec.md
---

## Title
One center tab that shows every MCP server each agent sees (Claude Code's, Codex's, Zed's own)
and what Marley needs of each agent CLI, with a warning when a CLI is older than a part of Marley
the user turned on needs. Chad, 2026-10-11: "17.) build now if we can".

## Scope
### In
- **The tab, MCP Servers:** `marley: open mcp servers`, and a button on Home's Configure card.
  It opens in the shown group's active pane, or the group's open one comes forward (one per
  group, as #739's pages). Its files are read off the main thread when it opens, when it comes
  forward and on Refresh.
- **Agent CLIs, at the top:** for Claude Code and Codex, the program and version Marley found
  (#648's `agent_versions`), each part of Marley that rests on the CLI (#648's rows: prompt tags,
  the shared plugin, the IDE link, Codex's App Server) with the versions it needs and whether it
  is on. When the CLI is older than an on part needs, the row says so in the warning colour, with
  the update line (`claude update`; for Codex, its own updater). A CLI not found says "not on
  Marley's PATH".
- **Claude Code's servers:**
  - the user scope, `mcpServers` at the top of Claude Code's `.claude.json` (in
    `$CLAUDE_CONFIG_DIR`, else the home folder);
  - the local scope, `projects["<folder>"].mcpServers` for each open project's folder;
  - the project scope, each open project's `.mcp.json`, each server marked approved, denied or
    not yet asked from that project's `enabledMcpjsonServers` and `disabledMcpjsonServers`;
  - a line saying plugins add more, which `claude mcp list` shows.
- **Codex's servers:** `[mcp_servers.<name>]` in `$CODEX_HOME/config.toml` (else
  `~/.codex/config.toml`), off when `enabled = false`.
- **Zed's context servers:** the merged `context_servers` of the user's and each open project's
  settings, each with its status from the project's `ContextServerStore` (running, starting,
  error with its message, stopped, off). Marley's own `marley` and `rusty` are marked "Marley's".
  Configure in Settings opens Zed's MCP Servers page (`zed_actions::OpenSettingsAt`).
- **Each row:** the name, the scope, local command or remote URL, the command with its arguments
  or the URL with secret-looking query values hidden (as the browser tools hide them), and the
  names of its env variables and headers. No env or header value is ever shown.
- **The notification:** when Marley finds a Claude Code or Codex older than a part the user
  turned on needs, at start or when the program changes, a notification names the CLI, its
  version, the part and the version it needs, with Open MCP Servers. It shows once per CLI and
  version (remembered in Zed's key-value store).
- **`marley_agent::versions`:** `Range` tells "older than its range" apart from "newer than it",
  so the tab and the notification warn only on older. The agent bar's chip stays as #648 made it.
- **Docs:** the guide (the tab, the notification), `docs/marley_architecture/marley_workbench.md`.

### Out (explicitly deferred)
- **Editing (the second slice):** add, change and remove a server for Claude Code and Codex
  through a form in the tab, writing through #755's safe write. Drafted in the notes; it needs a
  number, and #755 lands first.
- Editing Zed's context servers in the tab: Zed's own page does it, and the tab opens it.
- Servers that Claude Code plugins add (Marley's own `marley` plugin among them).
- Checking a registry for the newest release of a CLI: that reaches outside the box and would
  need its own switch, off by default.
- Gemini CLI and OpenCode: #648 does not detect them.
- Adding Marley's server to Codex's config (Codex in a terminal has no Marley tools today; it
  reaches them only as an ACP agent through Zed).

## Reference (§20)
Upstream Zed, the Settings window's MCP Servers page (`crates/settings_ui/src/pages/mcp_servers_page.rs`)
and `project::context_server_store`: a list of each server with its status and Configure. Behavior
kept: Zed's page stays the place to edit Zed's servers; the tab reads the same settings and store
and links to it. The other CLIs have no Zed analog; their files are read as each CLI documents
them (`claude mcp`, `codex mcp`). No Warp analog read.

### Prior art
- **The code we ship:**
  - `crates/settings_ui/src/pages/mcp_servers_page.rs`: the list (134-149), each row
    (151-238), the toggle (319-375), add (523-626), remove (628-653) and the form
    (`open_mcp_server_form` 859-887, `pub(crate)`). Registered at `page_data.rs:8887-8898` under
    `context_servers`. `zed_actions::OpenSettingsAt { path: "context_servers" }` opens it, as the
    Agent Panel's "Add Server…" does (`agent_panel.rs:5755-5761`).
  - `crates/project/src/context_server_store.rs`: `ContextServerStatus` 52-68, `server_ids` 614,
    `status_for_server` 556, `configuration_for_server` 560, `is_server_enabled` 600.
  - `crates/settings_content/src/project.rs:506-604`: `ContextServerSettingsContent` (Stdio,
    Http, Extension) and `ContextServerCommand`.
  - Marley: `marley_agent::versions` (`Range` 24-60, `INTEGRATIONS` 187-194, `verdict`
    276-291); `marley_workbench::agent_versions` (`init` 87-102, `locate` 171-202, `chip`
    373-431, `wanted` 344-358, `found`/`observe` 319-338); `home_page.rs` (`open_here` 62-78,
    `render_configure` 386+); `threads_page.rs` (an action, `open`, `Item`); `mcp.rs:266-329`
    (Marley's default `context_servers.marley`); `rusty.rs:807-855` (`rusty`).
  - `agent_notify.rs:66-70` (Codex's config path, `CODEX_HOME`), `claude_plugin.rs:115-117`
    (Claude's config folder, `CLAUDE_CONFIG_DIR`); `toml_edit` and `serde_json` with
    `preserve_order` are already in the workspace.
- **Behavior maps:** the MonoCode findings (item 7); T3 survey 01's provider updates section on
  telling the user an agent is too old.
- **Published material:** Claude Code's MCP docs (scopes user, local and project; `.mcp.json`);
  the installed CLIs' help: `claude mcp` (list with no JSON, `add`, `add-json`, `remove` with
  `--scope`), `codex mcp` (`list --json`, `add`, `remove`); MCP 2025-06-18.

## UI proof
`script/e2e/720-mcp-manager-and-cli-update-checks.sh`, under `compositor sway`. The run points
Claude Code and Codex at its own folders (`CLAUDE_CONFIG_DIR`, `CODEX_HOME`), so nothing of the
user's is read:
- `.claude.json` with a user-scope stdio server whose env holds `API_TOKEN`, and a local-scope
  HTTP server for the repo with an `Authorization` header;
- the repo's `.mcp.json` with one server, approved;
- Codex's `config.toml` with two servers, one `enabled = false`;
- the run's settings with a stand-in stdio MCP server in `context_servers` (a Python script that
  answers `initialize` and `tools/list`) beside Marley's defaults;
- `MARLEY_CLAUDE` naming a stand-in that prints `2.1.280 (Claude Code)`, with
  `marley.claude_code_shared_plugin: true`; `MARLEY_CODEX` one that prints
  `codex-cli 0.157.0`.

Shots:
- `720-01-notice`: after start, the notification naming Claude Code 2.1.280 and the shared
  plugin's 2.1.287, with Open MCP Servers.
- `720-02-tab`: the tab from that button: Agent CLIs with Claude Code's warning row and Codex's
  rows; Claude Code's three scopes; Codex's two servers, one off; Zed's servers with the stand-in
  running and `marley` marked Marley's; env and header names, no values.
- `720-03-refresh`: a third Codex server written into `config.toml` by the scenario, then
  Refresh: it shows.
- `720-04-again`: `marley: open mcp servers` again: the same tab in front, one tab.
- `720-05-zed-page`: Configure in Settings: Zed's MCP Servers page.

## Locked-In Decisions
- **D1:** a center tab (Chad, 2026-10-11, tabs everywhere). Rejected: a Settings page, as the
  finding said, or a sub-page of the Marley settings page (`MarleyPageViews`).
- **D2:** the tab shows; editing is the second slice. Rejected: one ticket for both (a form, three
  file formats, the safe write and the questions in the notes make it about twice this one).
- **D3:** files are read directly (`.claude.json`, `.mcp.json`, `config.toml`), not through
  `claude mcp list` (it prints no JSON, and it starts every server to check its health).
- **D4:** values never show: env and header values are left out of what the tab holds, not just
  of what it draws.
- **D5:** the warning is for "older than needed" only. A newer, untested CLI keeps #648's chip,
  since the user can do nothing about it but wait.
- **D6:** the notification shows once per CLI and version, so a user who stays on an old version
  is not told at every start.
- **D7:** Zed's servers are read from the merged settings and the stores, and edited on Zed's
  page. Rejected: a second editor for Zed's settings; a Zed hunk to open `open_mcp_server_form`
  for one server.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley finds a Claude Code or Codex older than a part of Marley the user turned on needs, the system shall show a notification naming the CLI, its version, the part and the version needed, once per CLI and version. | Shot 720-01 |
| REQ-002 | WHEN the user runs `marley: open mcp servers` or presses Open MCP Servers, the system shall open the MCP Servers tab in the shown group, or bring that group's open one forward. | Shots 720-02, 720-04 |
| REQ-003 | The tab shall show each agent CLI's program and version and each part of Marley that rests on it, marking a CLI older than an on part needs, with how to update. | Shot 720-02 |
| REQ-004 | The tab shall list Claude Code's user, local and project servers and Codex's servers, each with its scope, its command or URL, whether it is on, and its env and header names. | Shot 720-02 |
| REQ-005 | The tab shall list Zed's context servers with their status and mark Marley's own. | Shot 720-02 |
| REQ-006 | The tab shall never show an env or header value. | Shot 720-02; the review of the diff |
| REQ-007 | WHEN the user presses Refresh, the system shall read the files again and show what changed. | Shot 720-03 |
| REQ-008 | WHEN the user presses Configure in Settings, the system shall open Zed's MCP Servers settings page. | Shot 720-05 |
| REQ-009 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, every cited seam checked
  again.
- **P2 Code** — `marley_agent::versions` (older than the range); `mcp_servers.rs` (new, the
  readers and the tab) and its action; `agent_versions.rs` (the notification); `home_page.rs`
  (the button); the guide and the architecture note; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
