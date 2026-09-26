# Agent permission modes: Claude Code's bypass or Codex's full access, as a setting — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-532-agent-permission-modes.md
- **Pipeline spec:** 532-agent-permission-modes.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "default to what the agents are doing but allow the dangerously
  bypass permissions config or codex full permissions." It answers the Orca survey's open
  question 5 (worktree agents' permissions; default, each CLI's own prompts). Queued overnight by
  the spec drafters.
- **Classification / tier:** feature, prong 2 (starting agent CLIs), with the settings page.
  Marley crates, plus small hunks in Zed-crate files that are Marley's already
  (`settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs`) and a getter in
  `terminal`. Size S to M.
- **Recall (§18.3):**
  - AD-claude-460: a `marley` setting's default is its enum's `#[default]`, and `default.json`
    carries no `marley` block; `Ask` is the default the same way. The content types live in
    `settings_content` because the settings macros resolve only there (the comment at the top of
    `settings_content/src/marley.rs`).
  - #515's spec: the page is `settings_ui/src/marley_page.rs`, and an enum dropdown needs
    `strum::VariantArray` and `strum::VariantNames` plus a renderer line.
  - #516's spec (active the same night): its Agents section and its flat keys
    (`marley.redact_secrets_for_agents`); this ticket joins both.
  - L-claude-515-dispatch-through-the-window-from-inside-an-action-001: `marley: open settings`
    dispatches through the window; the scenario opens the page with it.
  - L-claude-500: a clicked context menu starts on its first entry; the + menu's steps count
    from there.
  - L-claude-480: a fake acts the program out; here the fakes print the arguments they got.
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery:** each seam opened and checked on 2026-09-25, at commit `520a6e22a7`, with #516's
  working-tree changes read the same night.
  - `crates/marley_agent/src/marley_agent.rs:84-92` (`send_payload`, `launch_input`: the
    program and Enter), `:73-79` (`agent_kind_of` ignores arguments, so `claude --flag` is still
    Claude Code).
  - `crates/marley_workbench/src/agents.rs:188-221` (`start_cli`: the terminal, the startup
    handshake, then `launch_input` at `:209`), `:233-247` (`new_agent`), `:332-345` (`Start::run`,
    the picker's path); the rail's + menu calls the same `start_cli`.
  - `crates/marley_workbench/src/marley_workbench.rs:165-181` (`MarleySettings`,
    `from_settings`); `crates/settings_content/src/marley.rs:10-15` (`MarleySettingsContent`),
    `:19-40` (`MarleyLayout` with the strum derives for its dropdown); #516 adds
    `redact_secrets_for_agents` and `redaction_patterns` there.
  - `crates/settings_ui/src/marley_page.rs` (#515: `marley_page`, `layout_section`,
    `privacy_section`; #516 adds `agents_section`), `crates/settings_ui/src/settings_ui.rs:559`
    (`add_basic_renderer::<settings::MarleyLayout>(render_dropdown)`), each item `files: USER`.
  - `crates/settings/src/settings_store.rs:1111` (a project's settings file is parsed as
    `ProjectSettingsContent`), `crates/settings_content/src/project.rs:44` and `:90-96` (its
    fields and `flattened_deserialize!` list, no `marley`),
    `crates/settings_content/src/settings_content.rs:251` and `:408-422` (`marley` sits on
    `SettingsContent`, the user file's type).
  - `crates/project/src/project.rs:6579` (`project_group_key`), `:6604-6611` (`from_project`:
    the main worktree paths), `:6623` (`path_list`).
  - `crates/terminal/src/pty_info.rs:69-73` (`ProcessInfo { name, cwd, argv }`, `pub(crate)`),
    `crates/terminal/src/terminal.rs:3071-3080` (`foreground_process_command_name`, the one
    public reading of it), `:3615` (`foreground_process_command_from_argv`: an interpreter's
    script counts as the command, which lets a Python fake pass as `claude`).
  - `crates/marley_workbench/src/rail.rs:1666-1721` (`terminal_snapshot` builds the status line
    through `marley_agent::status_line`), `crates/marley_rail/src/marley_rail.rs:61-68`
    (`TerminalAgent`).
  - `crates/settings_content/src/agent.rs:777-781` and `:804-810` (`default_mode` for custom and
    registry agent servers).
  - Claude Code 2.1.283 and Codex 0.155.1, their binaries' help strings read with `grep -a`
    (quoted in the spec).
- **Decisions:** D1 to D6 in the spec.

### Design
- **Approach.**
  1. `settings_content/src/marley.rs`: `ClaudeCodePermissions { Ask, Bypass }` and
     `CodexPermissions { Ask, FullAccess }` (snake case, default `Ask`, the strum derives for the
     dropdowns); `AgentPermissionsContent { claude_code, codex }`; `MarleySettingsContent` gains
     `claude_code_permissions`, `codex_permissions` and `agent_permissions_by_project`
     (`HashMap<String, AgentPermissionsContent>`).
  2. `MarleySettings` resolves them into an `AgentPermissions` value with
     `for_project(folder: &Path, kind: AgentKind) -> LaunchMode`, the project's entry winning.
  3. `marley_agent`: `LaunchMode { Default, Bypass }`; `launch_input(kind, mode)` adds the
     agent's flags for `Bypass` (Claude Code one flag, Codex two); `permission_mark(kind, argv) ->
     Option<PermissionMark>` scans the arguments for the flags in the spec's scope (and the
     `=` forms, and the short forms `codex --help` lists, confirmed at promotion).
  4. `agents::start_cli`: the mode from `MarleySettings` for
     `project.project_group_key(cx).path_list()`'s first folder, then `launch_input(kind, mode)`.
  5. `terminal.rs` (Zed): `pub fn marley_foreground_argv(&self) -> Option<Vec<String>>` beside
     `foreground_process_command_name`, reading the same `info.current`.
  6. The rail: `TerminalAgent` gains `mark: Option<PermissionMark>`; `terminal_snapshot` sets it
     from `permission_mark(kind, argv)`; the status line puts "bypass" or "full access" after the
     agent's name, in the warning color.
  7. `marley_page.rs`: two `SettingItem`s in `agents_section()` (`json_path`
     `marley.claude_code_permissions` and `marley.codex_permissions`, `files: USER`);
     `settings_ui.rs` registers the two dropdown renderers.
- **File manifest.**
  - Marley: `crates/marley_agent/src/marley_agent.rs`, `crates/marley_workbench/src/agents.rs`,
    `crates/marley_workbench/src/marley_workbench.rs`, `crates/marley_rail/src/marley_rail.rs`,
    `crates/marley_workbench/src/rail.rs`, `script/e2e/532-agent-permission-modes.sh` (Test).
  - Zed crates: `crates/settings_content/src/marley.rs` (Marley's file in a Zed crate),
    `crates/settings_ui/src/marley_page.rs` (the same), `crates/settings_ui/src/settings_ui.rs`
    (two renderer lines), `crates/terminal/src/terminal.rs` (the getter).
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the rows for
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs` and
  `crates/settings_ui/src/settings_ui.rs` (#515's) gain #532's keys and items; the
  `crates/terminal/src/terminal.rs` row gains `marley_foreground_argv`.

### E2E plan
Fixtures: `$E2E_WORK/bin/claude` and `$E2E_WORK/bin/codex`, Python, printing
`started with: <arguments>` and waiting on stdin; a scratch repository with the hostile
`.zed/settings.json`; the run's user settings edited in the steps with a small Python helper
(the pattern of #501's setup), after which Marley reloads them.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-005 | the rail's +, Claude Code (the repository's own settings ask for bypass) | `532-01-default` |
| REQ-002, REQ-004 | write `agent_permissions_by_project` for the repository; the +, Claude Code | `532-02-bypass` |
| REQ-003, REQ-004 | write `codex_permissions: full_access`; the +, Codex | `532-03-full-access` |
| REQ-004 | the +, New Terminal; type `claude --dangerously-skip-permissions` | `532-04-typed` |
| REQ-006 | the palette, `marley: open settings` | `532-05-settings-page` |

What no scenario reaches: the real CLIs honoring their flags (Claude Code's disclaimer, Codex's
sandbox); the fakes prove what Marley passes, and each CLI's help text, read at promotion, what the
flags do.

### Risks
- A flag whose name changes in a CLI release: Marley would type one the CLI refuses, and the
  refusal shows in the terminal. The flags are read from the installed versions' help at
  promotion.
- Claude Code asks the user to accept a disclaimer the first time it starts in bypass mode; it
  shows in the terminal, and Chad answers it once.
- #516's keys or section could change before it ships; these settings follow whatever shape it
  lands with.
- The status line grows by a word; the rail already cuts a line to its width.
