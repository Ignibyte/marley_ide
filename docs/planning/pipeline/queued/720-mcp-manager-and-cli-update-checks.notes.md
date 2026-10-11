# An MCP manager and agent CLI update checks — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-720-mcp-manager-and-cli-update-checks.md
- **Pipeline spec:** 720-mcp-manager-and-cli-update-checks.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** MonoCode findings item 7, "An MCP manager and agent CLI update checks", queued on
  2026-10-09 for future use. Chad, 2026-10-11: "17.) build now if we can". His rule the same day
  that everything opens as a tab
  (`docs/planning/design-notes/tabs-everywhere-2026-10-11.md`) moves it from a Settings page to a
  center tab.
- **Classification:** feature. Marley crates; no Zed path in this slice.
- **Recall (§18.3):**
  - #648 (completed): the version table is a list of tested ranges, not minimums: "a version
    that is too old and one that is too new produce the same result". Its only warning is the
    agent bar's chip, under a terminal whose foreground is `claude` or `codex`, for parts the
    user turned on. The scenario harness turns `claude_prompt_tags` on for every run.
  - #709 (completed): the shared plugin's row, `claude_shared_plugin`, 2.1.287 and later.
  - #653: the IDE rows ship `NOT_YET_CHECKED`, so they are off on every version.
  - #552 and L-claude-725: Codex's home is `CODEX_HOME`; a scenario points outside locations at
    its own folders.
  - L-claude-648-a-stand-in-can-print-its-version-from-its-own-file-name: a stand-in CLI prints a
    version for #648's reader.
  - L-claude-659-reach-a-settings-item-by-the-settings-search and
    L-claude-660-a-missed-click-outside-a-zed-modal-closes-it-silently: driving Zed's settings
    window in a scenario.
- **Facts found (2026-10-11, names and structure only, no values read out):**
  - Installed: Claude Code 2.1.296, Codex 0.162.0 (outside `codex_app_server`'s tested range,
    newer).
  - `~/.claude.json` holds a top-level `mcpServers` and, under `projects`, an `mcpServers` per
    project, with `enabledMcpjsonServers`, `disabledMcpjsonServers` and `mcpContextUris`.
    Claude's `settings.json` holds no MCP keys.
  - `~/.codex/config.toml` holds `[mcp_servers.<name>]` tables with `command`, `args`, `env`,
    `startup_timeout_sec`, `tool_timeout_sec`.
  - `claude mcp`: `list` (no JSON), `get`, `add` (`--scope local|user|project`, `--transport`,
    `--env`, `--header`), `add-json`, `remove` (`--scope`), no enable or disable. `codex mcp`:
    `list --json`, `get --json`, `add` (`--url` or a command, `--env`,
    `--bearer-token-env-var`), `remove`, no scope.
  - Marley never writes `~/.claude.json`, Claude's `settings.json` or Codex's `mcp_servers`. Codex
    in a terminal gets no Marley tools; it reaches them only as an ACP agent through Zed
    (`agent_servers/src/acp.rs:1671`).
- **Discovery (Explore, 2026-10-11):**
  - `crates/marley_agent/src/versions.rs`: `Range` 24-60, `Integration` 65-78, the rows 85-184,
    `INTEGRATIONS` 187-194, `parse_version` 211-228, `verdict` 276-291, `chip_label` 295-303,
    `reasons` 308-333.
  - `crates/marley_workbench/src/agent_versions.rs`: `init` 87-102 (called from
    `marley_workbench.rs:878`), `locate` 171-202, `read_version` 206-246, `store` 250-285,
    `check_if_stale` 288-298, `is_on`/`found`/`program`/`observe` 319-338, `wanted` 344-358,
    `chip` 373-431.
  - `crates/marley_workbench/src/home_page.rs`: `open_here` 62-78, `render_configure` 386-420.
  - `crates/marley_workbench/src/threads_page.rs`: `actions!` 27-35, `init` 37-44, `open` 46-60,
    `Item` 437-457, the template for the tab.
  - Zed: `mcp_servers_page.rs`, `context_server_store.rs`, `settings_content/src/project.rs`
    506-604, as the spec's prior art cites.

### Design
- **`crates/marley_agent/src/versions.rs`** (Marley crate, pure): `Range::older(version) -> bool`
  (below `from`); `older_than(found, cx-free rows)` listing the on rows a version is older than.
- **New `crates/marley_workbench/src/mcp_servers.rs`** (Marley crate):
  - **Readers, pure and off the main thread**, each `*_in(path)` so the scenario's folders are
    used:
    - `claude_servers_in(config_dir, project_folders)` → user, local and project rows from
      `.claude.json` and each folder's `.mcp.json`;
    - `codex_servers_in(codex_home)` → rows from `config.toml` through `toml_edit`.
    - A row: `{name, source, scope, kind: Local { command, args } | Remote { url }, env_names,
      header_names, on, note}`; values are dropped as each file is parsed (D4).
  - **Zed's rows** from `SettingsStore::merged_settings().project.context_servers` and each open
    project's `ContextServerStore` (`server_ids`, `status_for_server`, `is_server_enabled`), on
    the main thread.
  - **`McpServers`** (`Item + Render + Focusable`): sections Agent CLIs, Claude Code, Codex, Zed;
    Refresh; Configure in Settings. `actions!(marley, [OpenMcpServers])`, and `open_here` as
    `home_page::open_here` does; it re-reads when it becomes the active item.
  - The URL query values hidden with the browser tools' rule (the redaction `browser_tools` uses
    for URLs, shared rather than written twice).
- **`agent_versions.rs`:** after a version is stored, the on rows it is older than; a
  notification per CLI and version not yet in the key-value store (`marley-agent-versions` /
  `warned`), with Open MCP Servers.
- **`home_page.rs`:** an MCP Servers button on the Configure card.
- **`marley_workbench.rs`:** the module and its `init`.
- **Docs:** the guide (a section "MCP servers and agent versions", and a line in "The agent's
  version"); `docs/marley_architecture/marley_workbench.md`.
- **File manifest:** `marley_agent` (`versions.rs`); `marley_workbench` (`mcp_servers.rs` new,
  `agent_versions.rs`, `home_page.rs`, `marley_workbench.rs`); the guide; the architecture note;
  the scenario. No Zed path.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Starts with the stand-in `claude` at 2.1.280 and the shared plugin on | 720-01-notice |
| 002, 003, 004, 005, 006 | Clicks Open MCP Servers in the notification | 720-02-tab |
| 007 | Appends `[mcp_servers.third]` to the run's `config.toml`, clicks Refresh | 720-03-refresh |
| 002 | Runs `marley: open mcp servers` | 720-04-again |
| 008 | Clicks Configure in Settings | 720-05-zed-page |
| 009 | — | The gate |

The fixtures' env and header values are made-up strings. The Test phase reads 720-02 for their
absence, and the review checks that the rows never hold them.

### Risks
- **`.claude.json`'s place:** with `CLAUDE_CONFIG_DIR` set, Claude Code keeps it inside that
  folder; otherwise in the home folder. Check against the installed Claude Code at promotion.
- **A large `.claude.json`:** it holds every project's history; it is parsed off the main thread
  and only the MCP keys are kept.
- **Statuses of Zed's servers** belong to each project's store; a server Zed runs in two projects
  shows once per project. The tab groups rows by name and lists each project's status.
- **Other scenarios:** the installed CLIs are not older than any row (Claude Code 2.1.296 against
  2.1.283 and 2.1.287; Codex 0.162.0 against 0.155.1), the IDE rows' empty range never reads as
  older, and a stand-in that prints no version reads as unknown, not older. A scenario whose
  stand-in prints an old version (#648's) now also gets the notification in its corner; its
  shots are not run again (§7).

## The second slice (drafted; needs a number): editing servers (M)
- **Scope:**
  - Add, Edit and Remove on Claude Code's and Codex's rows, and Add per section: a form with the
    name, the scope (Claude Code: user, local for one open project, project for one open
    project's `.mcp.json`), local command and arguments or remote URL, and env and header pairs.
  - An existing value never comes back into the form: its field shows "set", is kept unless a
    new value is typed, and can be cleared.
  - Remove asks first.
- **Writes, through #755's `safe_write::replace_in`** (Chad, 2026-10-11: writes follow #755's
  safe write):
  - Codex's `config.toml` through `toml_edit`, comments and other tables kept;
  - a project's `.mcp.json` through `serde_json` with `preserve_order`;
  - `.claude.json` for the user and local scopes, the same way, only the MCP keys touched;
  - one backup each, and a refusal when the file changed since the tab read it, offering Reload.
- **The question for Chad, with the default:** Claude Code rewrites `.claude.json` often and
  keeps its own `.claude.json.backup`. If a running Claude Code writes its own copy over an
  outside change, Marley's edit could be lost.
  - *Default: the safe write, as Chad's rule says, plus a check at that slice's promotion: edit
    the file while a Claude Code session runs and see whether the change survives.*
  - If it does not survive, the user and local scopes go through `claude mcp add-json --scope`
    and `claude mcp remove --scope` instead, so Claude Code's own writer makes the change.
- **Proof:** a scenario: add a Codex server and see it in `config.toml` with the comment kept and
  a backup; change a Claude user-scope server's argument; remove a project-scope server; a value
  kept through an edit; a file changed under the form refused.
- **Acceptance:** each CLI's servers are added, changed and removed from the tab, each write kept
  safe; values stay hidden.
