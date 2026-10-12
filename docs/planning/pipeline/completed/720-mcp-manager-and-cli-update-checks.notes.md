# An MCP manager and agent CLI update checks — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-720-mcp-manager-and-cli-update-checks.md
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

### Promotion (2026-10-11)
- Moved from `queued/` to `active/` at `8ca014d6a8`; the BACKLOG row removed; the ticket in
  progress. Brain: consultation 48b325c654f8459182e3e5691635e148, nothing on this seam beyond
  #648's version rows (a second query, 9987026a…, closed as no decision).
- **Recall since the draft:** #755 shipped `safe_write` (the second slice's writes) and
  L-claude-755-terminal-lists-id-…; #754's scenario set `CLAUDE_CONFIG_DIR` in `setup`, the
  pattern this scenario follows. The e2e runner sets `MARLEY_CODEX` to a missing program and
  `allow_untested_versions.claude_prompt_tags`, but not `CLAUDE_CONFIG_DIR`.
- **Seams checked again** (an Explore pass):
  - `versions.rs`: `Range { from, before: Option<Version> }` (23-60; `before` exclusive, no
    `to`), `contains` drops a prerelease; `Integration` 64-78; the rows 85-186:
    `claude_prompt_tags` 2.1.283 up to 2.2.0, `codex_app_server` 0.155.1 up to 0.162.1 (0.162.0
    included), the three IDE rows `NOT_YET_CHECKED` (from 0.0.0, before 0.0.0),
    `claude_shared_plugin` 2.1.287 and later; `INTEGRATIONS` 189-196, `integrations_of`
    199-203, `parse_version` 213-230, `Found` 233-241, `verdict` 278-293, `reasons` 310-335.
  - `agent_versions.rs`: the cited ranges hold; `init` is called at `marley_workbench.rs:987`.
    The `AgentVersions` global keeps each CLI's `Check` (`path`, `found`); `wanted` (344-358) is
    private and only the chip uses it; nothing notifies today.
  - `home_page.rs`: `render_configure` is at 404-445, five filled buttons dispatched through the
    page's focus handle.
  - `threads_page.rs`: `actions!` 47-59, `init` 61-71 (called at `marley_workbench.rs:1007`),
    `open` 186-200 (`items_of_type` in the group, else `add_item_to_center`), `Item` 663-683.
  - Zed: `ContextServerStatus` 52-68 (`Starting`, `Running`, `Stopped`, `Error`,
    `AuthRequired`, `ClientSecretRequired`, `Authenticating`); `server_ids` 614,
    `status_for_server` 556, `settings_for_server` 570 (command, URL and headers for a custom
    server, also when it is off), `is_server_enabled` 600; `Project::context_server_store`
    (`project.rs:2266`); `ProjectSettings::get_global(cx).context_servers`
    (`HashMap<Arc<str>, ContextServerSettings>`); `ContextServerCommand { path, args, env,
    timeout }`. `OpenSettingsAt { path, target }` (`zed_actions/src/lib.rs:145-156`).
  - Marley's servers: `mcp::CONTEXT_SERVER` (`"marley"`, 271) and `rusty::CONTEXT_SERVER`
    (`"rusty"`, 93), both in the defaults layer.
  - Config paths: `agent_notify::codex_config` (65-69, private, the file); no Rust reads
    `.claude.json` yet. Read from Claude Code 2.1.296's binary: its global config is
    `join(CLAUDE_CONFIG_DIR || homedir(), ".claude.json")`, so inside the folder when the
    variable is set and not empty, else `~/.claude.json` (not `~/.claude/`).
  - `marley_browser::observe::redact_url` (288, public) hides a URL's secret-looking query values
    and its user and password. `marley_mcp::redact::Redactor::new(&[])` masks keys and tokens in
    text.
  - A notification with a button: `show_app_notification` with `MessageNotification`
    (`agent_trust.rs:276-299`), which also reaches windows opened later; a "shown once" flag in
    `KeyValueStore::global(cx).scoped(..)` (`shortcut_note.rs:51-79`).
  - Codex 0.162.0 has `codex update`.
- **What differs from the draft, and where the design takes it:**
  - `Range::older` compares with `from` only; the rows' `before` plays no part (D5).
  - `.claude.json` sits in the home folder, not the config folder, when `CLAUDE_CONFIG_DIR` is
    unset (the draft's risk, settled).
  - The arguments of a command can hold a token too: the tab masks the command line with the
    built-in redactor rules, as it hides URL query values (D8, below).
  - "When it comes forward" is read as: the action or the Home button brings an open tab forward
    and reads the files again.

### Design (as built)
- **`marley_agent::versions`:** `Range::older(&Version) -> bool` (the release below `from`;
  `NOT_YET_CHECKED`'s 0.0.0 makes nothing older), and `update_command(AgentKind)`:
  `claude update`, `codex update`.
- **`agent_versions.rs`:** `wanted` becomes `pub(crate)`; `outdated(kind, cx)` lists the wanted
  integrations the found version is older than; after each stored check and each settings change,
  `warn_if_outdated` shows an app notification for a CLI and version not yet warned (a session
  set, then the key-value store, scope `marley-agent-versions`), with Open MCP Servers.
- **New `mcp_servers.rs`** (Marley crate):
  - `actions!(marley, [OpenMcpServers])`, `init`, `open` (one per group, as the Threads page).
  - Readers, pure, run in `background_spawn`: `claude_rows_in(claude_json, folders)` and
    `codex_rows_in(config)`. Env and header maps are parsed as `IgnoredAny` values, so no value
    is ever held (D4). A missing file is no rows; an unreadable one is the section's error line.
  - `Row { name, scope, target (Local command line, masked | Remote URL, redacted), env_names,
    header_names, state }`; `State`: on, off, approved, denied, not asked, or a status.
  - Zed's rows from the open projects' stores (the page's window and every other window),
    grouped by name with each project's status; `marley` and `rusty` marked Marley's.
  - The Agent CLIs section from `agent_versions` (program, version, each integration's range,
    whether it is on, the warning and the update command).
  - Refresh, and Configure in Settings (`OpenSettingsAt { path: "context_servers" }`).
- **`home_page.rs`:** MCP Servers on the Configure card. **`marley_workbench.rs`:** the module
  and its `init`. **`agent_notify.rs`:** `codex_config` becomes `pub(crate)`.
- **D8:** the command line is masked with the built-in redaction rules and a URL with
  `redact_url`, so a token passed as an argument or a query value does not show either.

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

## Phase 2 — Code (2026-10-11)
- **Built:**
  - `marley_agent::versions`: `Range::older` and `update_command`.
  - `agent_versions.rs`: `outdated`, `warn_if_outdated` (after each stored check and on settings
    changes; a session set and the key-value store's `marley-agent-versions` scope), `wanted` and
    `capitalized` made `pub(crate)`.
  - `mcp_servers.rs` (new): the action, `open`, the tab, the readers, Zed's rows.
  - `home_page.rs`: MCP Servers on the Configure card. `marley_workbench.rs`: the module and
    `init`. `agent_notify::codex_config` made `pub(crate)`.
  - `mcp::window_workspaces`, moved from `turn_undo` (#714's helper) so both callers share it.
  - The guide's "The agent's version" paragraph and a new "MCP Servers" section; the in-app
    guide's Agent versions card and a new MCP Servers card.
- **Deviations from the design:**
  - The tab follows each open project's context server store (`ServerStatusChangedEvent`), so a
    status moves without Refresh; the design had Zed's rows only drawn from the stores.
  - Env and header maps are read through `Names` (a map visitor keeping keys, skipping values)
    rather than `BTreeMap<String, IgnoredAny>`, which clippy refuses (`zero_sized_map_values`).
  - The notification's text: "Run claude update to update it." without backticks, which print
    literally in a label (seen in the first 720-01 shot).
- **The review found:** `refresh`, called from `new` inside the action's workspace update, read
  every workspace of the window, that one included: a double lease on the first open. Fixed before
  any run: `spawn_in`, then `update_in` (F-claude-720-a-tab-read-its-workspace-…).
  - D4 by construction: no row field can hold a value; the `.claude.json` text itself is read in
    the background and dropped there.
  - Remote projects add no folders; a project with no folder still lists its store's servers.
- **Clippy found and fixed:** an unused `workspace` field, `struct_field_names` (`ClaudeProject`'s
  fields renamed with serde renames), `zero_sized_map_values`, `needless_pass_by_value`,
  `needless_pass_by_ref_mut` (twice), a missing doc and Debug, `used_underscore_binding`.
- **The gate:** the first run red on gate:1 (rustfmt: edits after the last `./script/clippy`) and
  gate:21 (dylint: `SharedString::new_static` over `"…".into()`, five times in `State::words`).
  Fixed at the source and `cargo fmt --all`; the second run red on gate:2 (rustfmt had turned a
  match arm into a block without `;`, clippy's `semicolon_if_nothing_returned`); fixed, clippy and
  the fmt check clean; the third run green, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/720-mcp-manager-and-cli-update-checks.sh`, under
  `compositor sway`, exit 0; the check "the warning is remembered for claude 2.1.280" passes.
  Calibrated on its first run (the notification's button at 1206,933; Refresh 1383,96; Configure
  1507,96). Shots read:
  - `720-01-notice`: bottom right, "Claude Code 2.1.280 is too old for parts of Marley you use",
    "Prompt tags: 2.1.283 or later", "The shared plugin: 2.1.287 or later", "Run claude update to
    update it.", and Open MCP Servers (REQ-001).
  - `720-02-tab`: the tab from that button, one tab beside the terminal and a rail row. Agent
    CLIs: Claude Code 2.1.280 at the stand-in, prompt tags and the shared plugin in the warning
    colour with "needs … or later", the IDE rows "not turned on, tested on no version yet", the
    update line; Codex 0.157.0, App Servers "not turned on, tested on 0.155.1 up to 0.162.1"
    (REQ-003). Claude Code: `notes` user on, `notes-mcp --stdio --api-key=[redacted: secret]`,
    "env: API_TOKEN, NOTES_DIR"; `tracker` local, repo on,
    `https://tracker.example.com/mcp?team=core&token=…`, "headers: Authorization"; `docs`
    project, repo approved; the `/somewhere/else` entry not listed. Codex: `search` on with
    "env: SEARCH_API_KEY", `old-wiki` off. Zed: `marley` running in repo, Marley's, with its env
    names; `stand-in` running in repo (REQ-002, REQ-004, REQ-005). No value of the fixtures
    (`not-a-real-…`) shows anywhere (REQ-006).
  - `720-03-refresh`: after the scenario appended `[mcp_servers.third]` and clicked Refresh,
    `third` user on `third-mcp` under Codex (REQ-007).
  - `720-04-again`: `marley: open mcp servers` again: still one MCP Servers tab, in front
    (REQ-002).
  - `720-05-zed-page`: Configure in Settings: Zed's Settings window at User / AI / General / MCP
    Servers, listing `marley` and `stand-in` (REQ-008).
- **What no scenario shows:** the second start that says nothing for a version already warned of;
  the run checks the key-value store holds it instead.
- **The last run** came after the green gate, on a fresh build of that tree: exit 0, the same
  shots.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #720); the guide ("The agent's version" and a new "MCP
  Servers" section) and the in-app guide (Agent versions, a new MCP Servers card and its nav
  entry); `docs/marley_architecture/marley_workbench.md` (the versions section and a new MCP
  Servers section). No Zed path.
- **Knowledge appended:**
  - AD-claude-720-mcp-servers-is-a-tab-over-each-clis-own-files-names-never-values-001;
  - F-claude-720-a-tab-read-its-workspace-inside-the-action-that-opened-it-001 (prevented by
    PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001, no new rule);
  - L-claude-720-claude-json-sits-in-the-home-folder-unless-claude-config-dir-001.
  - The `SharedString::new_static` lint is already in the lessons (line 4522); no new entry.
- **Brain:** consultation 48b325c654f8459182e3e5691635e148 closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed. #768's spec is
  queued for the second slice.
