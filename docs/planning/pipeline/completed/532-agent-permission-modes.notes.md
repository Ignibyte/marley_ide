# Agent permission modes: Claude Code's bypass or Codex's full access, as a setting — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-532-agent-permission-modes.md
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
- **Discovery:** each seam opened and checked on 2026-09-25, at commit `484a7f18cb`, with #516's
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
- **Decisions:** D1 to D9 in the spec.

### Promotion (2026-09-28)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #509's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore, over `62dcee61d1`) ✓; the binaries' strings read again ✓; spec and design
  updated ✓.
- **Recall, added:** AD-claude-440 (an agent CLI is read from argv; the launch writes only a known
  program name, which this keeps: the flags are Marley's own constants); L-claude-477 (the
  foreground process is seen once it writes output; the fakes print at start); #571's rule, which
  reads the seat's `permission_mode` and counts `bypassPermissions` and `dontAsk` as prompt-less;
  #519's `PERMISSION_MODE_LABEL` on every seat. AD-claude-460's "no `marley` block in
  `default.json`" no longer holds: the block exists, with every key but `layout` and `push`. The
  brain (consultation 55d4fcfeacfa450393b4f8e941cb2004): nothing on this seam.
- **What the code says now** (the Explore report, 2026-09-28):
  - One production launch path: `agents::start_cli` (`agents.rs:188-221`, `launch_input` at
    `:209`), from the rail's + (`Rail::new_agent`, `rail.rs:1539-1551`) and the picker
    (`Start::run`, `agents.rs:332-345`). A user's task in `.zed/tasks.json` could start a bypass
    without it; only the argument mark sees that.
  - `launch_input(kind)` (`marley_agent.rs:113-115`) and `agent_kind_of` (`:96-102`, arguments
    ignored); the tests that pin `b"claude\r"` (`:247`, `:261-268`) keep compiling.
  - `MarleySettings` (`marley_workbench.rs:222-249`) derives `Eq`; `from_settings` (`:265-313`);
    `SystemOneSettings::from_content` is the pattern for a nested block. `MarleySettingsContent`
    (`marley.rs:9-62`); its enums share one set of derives with `#[default]`; its one map is a
    `BTreeMap`, and `MergeFrom` has no `std::collections::HashMap`.
  - The page: `agents_section` (`marley_page.rs:49`), seven items, #571's dropdown the one to
    copy; the renderers at `settings_ui.rs:559-566`; `render_dropdown` shows `variants()[0]` when
    no file has a value, so `Ask` comes first.
  - A project's `.zed/settings.json`: `ProjectSettingsContent` (`project.rs:44`, no `marley`),
    stored with `marley` empty (`settings_store.rs:1110-1137`); a `marley` key there is dropped
    without an error.
  - `project_group_key` (`project.rs:6579`), `from_project` (`:6604-6611`, the main worktrees and
    the host), `PathList::paths()` sorted, `ordered_paths()` in the user's order.
  - `ProcessInfo { name, cwd, argv }` (`pty_info.rs:68-73`) is `pub(crate)`, refreshed off the
    main thread at each load; `foreground_process_command_name` (`terminal.rs:3190-3200`) is the
    one public reading; a name with `/` is no command (`:3710`), so a fake needs `#!/usr/bin/env
    python3`.
  - The rail: `terminal_snapshot` (`rail.rs:3557-3649`); rows without a seat read
    `status_line`, Claude Code rows with one `seat_line`, which names no agent; every line is
    `Color::Muted` (`:3975-3980`); #569's flag is a `Color::Warning` icon after the card
    (`:2399-2412`). `TerminalAgent` (`marley_rail.rs:132-139`) is `Copy`.
  - Settings in scenarios: each scenario carries its own `marley_setting` (570's JSON merge is
    the one for a map); Marley reloads the user file live.

### Design
- **Changed at promotion** (each item overrides the drafted design after it):
  - **The mark** is a chip at the row's end, beside #569's flag: `Label` "bypass" or "full
    access", `LabelSize::XSmall`, `Color::Warning`, with an id and a tooltip ("Claude Code asks
    for no permission: it runs in bypassPermissions, as its events report" or "…: started with
    `--dangerously-skip-permissions`"; "Codex runs with full access: no sandbox, no approvals
    (`--sandbox danger-full-access`)"). `status_line` and `seat_line` are left as they are.
  - **Where it comes from**: `marley_agent::permission_mark(kind, argv, reported) ->
    Option<PermissionMark>`, `PermissionMark { kind: MarkKind (Bypass, FullAccess), source:
    MarkSource (Reported, Arguments(&'static str)) }`, `Copy`, with `words()` and `tooltip()`.
    For Claude Code a reported mode decides (`bypassPermissions` marks, any other clears), else
    the arguments; for Codex the arguments. `terminal_snapshot` passes the seat's
    `PERMISSION_MODE_LABEL` and the argv from the terminal's new getter (read directly, not
    through the injectable `ForegroundCommand`, whose tests' fakes stay as they are).
  - **The settings**: `agent_permissions_by_project: Option<BTreeMap<String,
    AgentPermissionsContent>>`; `MarleySettings.agent_permissions: AgentPermissions` (`Eq`) with
    the defaults and the entries, their folders expanded with `~/`; `launch_mode(kind, folders)`
    takes the longest entry whose folder holds one of the project's main folders and sets that
    kind, else the default; `start_cli` gives it the group's main folders for a local project,
    none for a remote one. `default.json` gains the three keys.
  - **The getter**: `Terminal::marley_foreground_argv(&self) -> Option<Vec<String>>`, the cached
    `info.current`'s argv, beside `foreground_process_command_name`; no `/proc` read on the main
    thread.
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
- **File manifest** (as promoted).
  - Marley: `crates/marley_agent/src/marley_agent.rs` (`LaunchMode`, `launch_input(kind, mode)`,
    `PermissionMark`, `permission_mark`); `crates/marley_workbench/src/agents.rs` (`start_cli`'s
    mode, `AgentPermissions::launch_mode`); `crates/marley_workbench/src/marley_workbench.rs`
    (`MarleySettings.agent_permissions`); `crates/marley_rail/src/marley_rail.rs`
    (`TerminalAgent.mark`, the tests' literals); `crates/marley_workbench/src/rail.rs` (the mark in
    `terminal_snapshot`, the chip); `script/e2e/532-agent-permission-modes.sh` and
    `script/e2e/golden` (Test).
  - Zed crates: `crates/settings_content/src/marley.rs` (Marley's file in a Zed crate: the two
    enums, `AgentPermissionsContent`, the three keys), `crates/settings_ui/src/marley_page.rs`
    (the same: two items), `crates/settings_ui/src/settings_ui.rs` (two renderer lines),
    `crates/terminal/src/terminal.rs` (the getter), `assets/settings/default.json` (the three
    keys).
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the rows for
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `crates/settings_ui/src/settings_ui.rs` and `assets/settings/default.json` gain #532's keys and
  items; the `crates/terminal/src/terminal.rs` row gains `marley_foreground_argv`.

### E2E plan
As promoted, over the draft below: the fake `claude` sends #519's events through the plugin's
hook, reporting `bypassPermissions` when its arguments or its environment say so, else
`default`, and `default` again at an Enter; the chip replaces the status line's words; the
settings go in with a JSON merge like #570's `marley_setting`. Rows as promoted:

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-005 | the rail's +, Claude Code (the repository's own settings ask for bypass) | `532-01-default`; the run log's `started with:` |
| REQ-002, REQ-004 | `agent_permissions_by_project` for the repository; the +, Claude Code | `532-02-bypass` |
| REQ-003, REQ-004 | `codex_permissions: full_access`; the +, Codex | `532-03-full-access` |
| REQ-004 | a new terminal; `claude --dangerously-skip-permissions` typed, the fake's events off | `532-04-typed` |
| REQ-008 | an Enter in the bypass terminal (reports `default`); a new terminal whose fake reports `bypassPermissions` with no flag | `532-05-reported` |
| REQ-006 | the palette, `marley: open settings` | `532-06-settings-page` |
| REQ-007 | the gate and the golden set | `gates.sh --diff`; `just regress` |

The draft's plan:

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
- Changed at promotion: the +'s Agent CLIs come from Marley's own PATH (the login shell's, since
  the runner sends Marley's stdout to a log), so the entries show where the real CLIs are
  installed, as on this box; the terminal's `.bashrc` puts the fakes first, so what runs is the
  fake, as in every Claude Code scenario since #566. The scenario prints the terminal's `command
  -v claude codex` before the first launch and stops if either is not the fake.
- A seat keeps its last reported mode until the next lead event: a Shift+Tab shows at the
  agent's next event, not at the keystroke.
- A flag whose name changes in a CLI release: Marley would type one the CLI refuses, and the
  refusal shows in the terminal. The flags are read from the installed versions' help at
  promotion.
- Claude Code asks the user to accept a disclaimer the first time it starts in bypass mode; it
  shows in the terminal, and Chad answers it once.
- #516's keys or section could change before it ships; these settings follow whatever shape it
  lands with. Settled at promotion: #516 shipped with its flat keys and the Agents section.
- The status line grows by a word; the rail already cuts a line to its width. Changed at
  promotion: the status lines stay as they are, and the chip takes the row's end, where the title
  gives way.

## Phase 2 — Code
- **Checklist** (no task tool): the five ledger rows first ✓; `settings_content/src/marley.rs` ✓;
  `default.json` ✓; `terminal.rs`'s getter ✓; `marley_agent` ✓; `AgentPermissions` and
  `start_cli` ✓; `MarleySettings` ✓; `TerminalAgent.mark` ✓; the rail's mark and chip ✓; the page's
  two items and their renderers ✓; fmt ✓; clippy ✓; the review ✓.
- **Built.**
  - The ledger (`docs/marley/zed-touchpoints.md`): the rows for `settings_content/src/marley.rs`,
    `settings_ui/src/marley_page.rs`, `settings_ui/src/settings_ui.rs`,
    `assets/settings/default.json` and `terminal/src/terminal.rs` gained #532's clauses before
    any of those files changed.
  - `settings_content/src/marley.rs`: `ClaudeCodePermissions { Ask, Bypass }` and
    `CodexPermissions { Ask, FullAccess }`, `Ask` the default, with the strum derives the other
    dropdown enums have; `AgentPermissionsContent { claude_code, codex }`; the keys
    `claude_code_permissions`, `codex_permissions` and `agent_permissions_by_project`
    (`BTreeMap<String, AgentPermissionsContent>`). `default.json`'s `marley` block: `"ask"`,
    `"ask"` and `{}`, with a comment each.
  - `terminal.rs` (Zed, additive): `Terminal::marley_foreground_argv()`, the cached
    `info.current`'s argv, beside `foreground_process_command_name`, with a `// Marley:` comment.
  - `marley_agent`: `LaunchMode { Ask, Bypass }`; `launch_input(kind, mode)` adds Marley's own
    constants, `--dangerously-skip-permissions` for Claude Code and `--sandbox danger-full-access
    --ask-for-approval never` for Codex, and nothing for the others; `BYPASS_MODE`;
    `PermissionMark { kind: MarkKind, source: MarkSource }` with `words()` and `tooltip()`;
    `permission_mark(kind, argv, reported)`: Claude Code's reported mode decides when there is
    one, else its arguments (`--dangerously-skip-permissions`, `--permission-mode
    bypassPermissions` and its `=` form); Codex's arguments (`--sandbox danger-full-access` and
    `-s`, the `=` and joined forms, a `-c`/`--config` of `sandbox_mode` in TOML, quoted or not,
    and `--dangerously-bypass-approvals-and-sandbox`); arguments after a `--` are not options.
    The three tests that pin the launch bytes pass `LaunchMode::Ask`.
  - `marley_workbench`: `agents::AgentPermissions` (`Eq`), from the content with the folders
    through `system_one::folder_path` (now `pub(crate)`), and `launch_mode(kind, folders)`: the
    longest entry whose folder holds one of the project's main folders and sets that agent, else
    the default; `start_cli` reads it for a local project's main folders, none for a remote one;
    `MarleySettings.agent_permissions`.
  - `marley_rail`: `TerminalAgent.mark: Option<PermissionMark>` (the struct stays `Copy`); the
    test's literal gains `mark: None`.
  - `rail.rs`: `terminal_snapshot` sets the mark from the seat's `PERMISSION_MODE_LABEL` and the
    terminal's argv; `render_terminal_row` draws the chip before #569's flag: a bordered pill as
    the inbox's chips are, `LabelSize::XSmall` in `Color::Warning`, its tooltip from `tooltip()`,
    `marley-rail-mark-<id>`.
  - `settings_ui`: the Agents section's Claude Code Permissions and Codex Permissions dropdowns
    (nine items now), `files: USER`, and their two renderer lines.
- **Deviations from the design, and why.**
  - The launch mode is `LaunchMode { Ask, Bypass }`, the draft's `Default` named as the
    settings name it.
  - The mark's tooltip names the argument it read in its canonical form (`--sandbox
    danger-full-access` for any of the sandbox's forms), not the text typed.
  - The terminal's argv is read directly in `terminal_snapshot`, not through the injectable
    `ForegroundCommand`: the tests' fakes there stay as they are, and a display-only terminal
    gives no argv, so no mark.
- **The review** (against REQ-001 to REQ-008, re-entrancy, provenance, errors):
  - Checked: every launch goes through `start_cli` (the rail's + and the picker); a task or a
    typed command goes around it and is marked from its arguments. A repository's
    `.zed/settings.json` cannot reach the keys (the store keeps `marley` empty there). The words
    Marley types are its own constants, never text from a setting (AD-claude-440). `start_cli`
    reads the project and the settings inside the workspace's update, updating nothing; the rail
    reads the terminal and the seat in its refresh. No `unwrap`, no `let _ =` on a fallible call.
    Provenance: the flags come from the installed binaries' own strings and the published help;
    nothing from Warp; nothing from a Zed function body in a Marley crate.
- **Gates.** `rustfmt` on every touched file. `just clippy marley_agent marley_rail marley_workbench settings_content settings_ui terminal` (`--all-targets`, `-D warnings`): the first run found `permission_mark`'s first doc paragraph too long (split); the second found `render_terminal_row` at 117 lines, over the 100 with #509's turns and this chip (the chip and #569's flag moved to `permission_chip` and `stall_flag` beside `row_card`); the third is clean. The logs are in the scratchpad. No cargo ran while #509's release install ran its golden set; the edits began after its build had finished.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan as promoted ✓; 532 in the golden set
  ✓; `just build` ✓; the scenario run and every shot read ✓; the golden set ✓; `gates.sh --diff`
  ✓.
- **The scenario**, `script/e2e/532-agent-permission-modes.sh` (compositor sway, no Chromium):
  fake `claude` and `codex` (`#!/usr/bin/env python3`, first on the terminal's PATH), each
  writing `<program>: <its arguments>` to a launches log; the fake `claude` sends #519's
  SessionStart through the plugin's hook with `bypassPermissions` when its arguments ask for
  bypass or `STAND_IN_MODE` says so, else `default`, and at each Enter a UserPromptSubmit
  reporting `default`, logging each read with its arguments; `STAND_IN_EVENTS=0` sends nothing.
  The repository's `.zed/settings.json` asks for bypass and full access. Before the first launch
  the scenario writes the terminal's `command -v claude codex` to a file and stops unless both
  are the fakes (L-532). The `+` by Down steps from New Terminal (three to Claude Code, four to
  Codex); the settings through a JSON merge into the profile copy. The real `claude` and `codex`
  never ran. Run 1 passed its four checks, but its tooltip point missed the chip and its row
  point fell on the first Claude Code row, which took the Enter meant for the bypass one: the
  check counted lines in a log every fake writes (F-532). Run 2, with the points from run 1's
  shots and the check naming the fake that must read the Enter, passed all four and showed
  every step.
- **The shots** (run 2, read one by one; the rails cropped side by side):
  - `532-00-menu`: the `+` menu after three Downs: New Terminal, New Browser Tab, New Agent
    Thread, the Agent CLIs header, Claude Code (selected), Codex, Gemini CLI, OpenCode.
  - `532-01-default` (REQ-001, REQ-005): the new terminal's `claude` and `started with:` with
    nothing after, though the repository's own settings ask for bypass; its row reads `idle`
    with no chip.
  - `532-02-bypass` (REQ-002, REQ-004): with the repository's entry, `claude
    --dangerously-skip-permissions` and `started with: --dangerously-skip-permissions`; the new
    row carries `bypass` in the warning color, the first Claude Code row none.
  - `532-02b-tooltip`: on the chip, "Claude Code asks for no permission: its events report
    bypassPermissions".
  - `532-03-full-access` (REQ-003, REQ-004): `codex --sandbox danger-full-access
    --ask-for-approval never`, the fake printing the same, and the Codex row, `Codex · waiting`,
    carrying `full access`.
  - `532-04-typed` (REQ-004): `STAND_IN_EVENTS=0 claude --dangerously-skip-permissions` typed in
    a new terminal: its row, `Claude Code · waiting` (no events, so no seat), carries `bypass`
    from its arguments.
  - `532-05-reported` (REQ-008): the bypass terminal, clicked and sent an Enter, prints `mode
    now: default`: its row reads `working · Carry on` with no chip, though its arguments still
    hold the flag; the terminal whose fake reported `bypassPermissions` with no flag carries
    `bypass`; the typed one and Codex keep theirs.
  - `532-06-settings-page` (REQ-006): the Marley page's Agents section ends with Claude Code
    Permissions at Ask and Codex Permissions at Full Access, with the mark of a value off its
    default.
  - The run log: the launches `claude: `, `claude: --dangerously-skip-permissions`, `codex:
    --sandbox danger-full-access --ask-for-approval never`, `claude:
    --dangerously-skip-permissions` (typed) and `claude: ` (the reported bypass); the one read
    came from the fake started with `--dangerously-skip-permissions` and events on.
- **Focus**: the scenario ran in its own headless sway; the runner reported `hyprland: 0 Marley
  windows before the run, 0 after`, no rule added.
- **Not reachable by a scenario**: the real CLIs honoring their flags (Claude Code's first-bypass
  warning, Codex's sandbox), and a real Claude Code's Shift+Tab; the fakes prove what Marley
  types and what it reads back, and the binaries' own strings, read at promotion, what the
  flags do. A remote project, which the runner cannot open.
- **The gate**: `script/gates.sh --diff` printed `GATE GREEN [diff]`, 16 passed and 0 failed, on the first run; the full log is kept in the scratchpad.
- **The golden set** with 532 added: all 47 passed on the debug build (`just regress`), 532 in 97 s with its four checks.
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented.** `CHANGELOG.md` (Added: agent permission modes); `docs/marley/workbench-shell.md`
  (W4's record: #532's permission modes); `docs/marley/guide.md` ("Permission modes" under Agent
  CLIs in terminals: the two settings and what Marley types, the per-project map, the chip and
  where it reads from, what it cannot see); `docs/marley_architecture/marley_agent.md` (the
  surface, `launch_input`'s constants, `permission_mark`); `marley_workbench.md` (the permission
  modes under Agent CLIs); `marley_rail.md` (`TerminalAgent::mark`). The five touchpoint rows,
  written before the Zed edits, describe what shipped: the keys and enums, the two dropdowns,
  their renderers, the three defaults and the argv getter.
- **Knowledge appended.** `failures.md`:
  F-claude-532-the-enter-check-counted-a-log-every-stand-in-writes-001. `prevention-rules.md`:
  PR-claude-a-check-on-a-shared-log-names-its-writer-001. `lessons.md`:
  L-claude-532-the-plus-lists-marleys-path-and-types-into-the-terminals-001,
  L-claude-532-claude-codes-arguments-say-how-it-started-not-its-mode-001.
  `architecture-decisions.md`:
  AD-claude-532-agents-start-with-their-prompts-unless-your-own-settings-say-otherwise-001.
- **Brain.** Consultation `55d4fcfeacfa450393b4f8e941cb2004` closed with `brain decide`
  (`decisions/marley-starts-agent-clis-with-their-prompts-unless-the-users-own-settings-say-otherwise-and-marks-a-bypass-on-the-rail`),
  follow-up by 2026-10-28.
- **Closed.** TICKET-532 moved to `tickets/closed/`, its pipeline link at `completed/`; the
  backlog row went at promotion.
- **The receipt.** `GATE GREEN [diff]` on the tree the golden set ran on; only docs changed
  since.
