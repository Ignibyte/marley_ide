---
pipeline_id: 6b2ce487-e206-4b16-b65c-54ffdc41c5c0
ticket: docs/planning/tickets/closed/TICKET-532-agent-permission-modes.md
status: Phase 4 — Complete PASS
title: "Agent permission modes: Claude Code's bypass or Codex's full access, as a setting"
type: feature
slice: prong 2 (starting agent CLIs), with the Marley settings page (#515, #516)
references: [docs/orca_architecture/README.md, docs/orca_architecture/01-agents-and-sessions.md, docs/planning/pipeline/completed/515-marley-settings-page.spec.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/completed/571-pause-before-a-consequential-click.spec.md]
---

## Title
A setting, off by default, per agent and per project, makes Marley start Claude Code with
`--dangerously-skip-permissions` or Codex with its full access
(`--sandbox danger-full-access --ask-for-approval never`). An agent row in the rail carries a
"bypass" or "full access" mark whenever its terminal's agent runs with such a flag, however it
was started, or Claude Code reports it bypasses, so a bypass is never silent. The defaults sit in
the Marley settings page's Agents section.

## Scope
### In
- **The settings**, in the `marley` block beside #516's, flat as `marley.layout` is:
  `claude_code_permissions`, `"ask"` (the default, Claude Code's own prompts) or `"bypass"`;
  `codex_permissions`, `"ask"` (the default) or `"full_access"`; and
  `agent_permissions_by_project`, a map from a project's folder to either or both
  (`{"claude_code": "bypass", "codex": "full_access"}`), which wins over the default for that
  project. The content types live in `crates/settings_content/src/marley.rs`; `MarleySettings`
  resolves them. Changed at promotion: the map is a `BTreeMap` (what `MergeFrom` merges), its
  keys take `~/` as the System One layer's project lists do, and an entry applies to a local
  project whose main folder is its folder or inside it, the longest folder winning; a remote
  project takes the defaults. `default.json`'s `marley` block gains the three keys with their
  defaults.
- **The launch.** `marley_agent::launch_input` takes the mode and types
  `claude --dangerously-skip-permissions`, `codex --sandbox danger-full-access --ask-for-approval
  never`, or the program alone. `agents::start_cli` reads the mode for the workspace's project,
  keyed by its group's main folder (`Project::project_group_key`), so the rail's +, the New Agent
  picker and #510's worktree agents all take it.
- **The mark.** Changed at promotion: an agent row carries a chip at its end, "bypass" or "full
  access" in the warning color, with a tooltip naming what it means and where Marley read it,
  beside #569's flag; the status lines are left as they are (a row with Claude Code's events does
  not name the agent, and no line under a title is colored). For Claude Code with #519's events,
  the chip follows the mode its hook events report: `bypassPermissions` marks, any other mode
  clears it, whatever the arguments say, so a bypass Claude Code's own settings chose shows, and
  one the user left with Shift+Tab goes. Otherwise it follows the foreground process's
  arguments: Claude Code's `--dangerously-skip-permissions` or `--permission-mode
  bypassPermissions` (and its `=` form); Codex's `--sandbox danger-full-access` (and `-s`, and the
  `=` forms), a `-c`/`--config` of `sandbox_mode` to `danger-full-access`, or
  `--dangerously-bypass-approvals-and-sandbox`. Zed's terminal gains a getter for the foreground
  process's arguments, from the process info it already keeps.
- **The page.** The Marley page's Agents section (#516) gains "Claude Code Permissions" and "Codex
  Permissions" as dropdowns over the two defaults, user settings only; the per-project map is set
  in `settings.json`.

### Out (explicitly deferred)
- Gemini CLI's `--yolo` and OpenCode's modes: Chad named Claude Code and Codex only.
- Agent Panel threads: Zed's own `agent_servers.<name>.default_mode` sets the mode they start in.
- Marks for Claude Code's other modes (`auto`, `dontAsk`, `acceptEdits`): none bypasses its
  prompts the way `bypassPermissions` does; #571 reads them for its own rule.
- A full access Codex's own `config.toml` sets with no argument: Codex sends Marley no events, so
  its arguments are all Marley reads.
- A rail menu entry for the per-project value.
- Pre-trusting a new worktree's folder for the agent (#510's business).

## Reference (§20)
- **Warp:** its agent profiles set autonomy per profile ("Agent Decides", "Always Ask", "Always
  Allow"; everything on "Always allow" is its "YOLO mode"), for Warp's own agent only; third-party
  CLI agents keep their own configuration
  (docs.warp.dev/agent-platform/capabilities/agent-profiles-permissions/). Marley's setting covers
  the CLI agents it starts, and stays off until Chad turns it on.
- **Upstream Zed:** `agent_servers.<name>.default_mode`, the mode an Agent Panel thread starts in
  (`crates/settings_content/src/agent.rs:777`), kept for threads.
- **Orca:** its Agent Permissions switch (Yolo or Manual) over a table of each CLI's bypass flag,
  with bypass the default (report 01 §2.1). The table is the prior art; the default is what the
  survey says not to take (report 01 §4, README "What not to take").

### Prior art
- **Reports.** Report 01 §2.1 lists Orca's flags per agent (`src/shared/tui-agent-permissions.ts`;
  `DEFAULT_TUI_AGENT_ARGS` in `src/shared/tui-agent-launch-defaults.ts` makes bypass the default)
  and its switch that rewrites every untouched agent's arguments. Report 01 §4, report 05 §4 and
  report 06 §4 all skip bypass by default; report 06 notes it contradicts the approvals inbox and
  Marley's deny-by-default grants.
- **Published material.** The installed Claude Code 2.1.283, its bundle read and not run:
  `--dangerously-skip-permissions` is an "Alias for --permission-mode bypassPermissions"; the
  modes are `default`, `acceptEdits`, `plan`, `auto`, `bypassPermissions` and `dontAsk`; a first
  bypass start asks the user to accept a disclaimer once. The installed Codex 0.155.1, its binary
  read and not run: `--sandbox` takes `danger-full-access`; "`--ask-for-approval never`
  suppresses approval prompts"; its Full Access preset reads "Codex can edit files outside this
  workspace and access the internet without asking for approval";
  `--dangerously-bypass-approvals-and-sandbox` is "Intended solely for running in environments
  that are externally sandboxed". Warp's page above.
- **Code we already ship.** `marley_agent::launch_input` (`marley_agent.rs:90`) types the
  program's name and Enter, and `agents::start_cli` (`agents.rs:188`) is the one path every CLI
  launch takes. `MarleySettings` (`marley_workbench.rs:165`) resolves the `marley` block;
  #515's page and #516's Agents section hold its items, with dropdowns for an enum that derives
  `strum::VariantArray` and `strum::VariantNames`. Zed parses a project's `.zed/settings.json` as
  `ProjectSettingsContent` (`crates/settings/src/settings_store.rs:1111`), which has no `marley`
  key, so a repository cannot set these. The terminal already reads the foreground process's
  arguments (`crates/terminal/src/pty_info.rs:69-73`), but only its command's name is public
  (`crates/terminal/src/terminal.rs:3071`).
- **Read again at promotion** (2026-09-28, an Explore report over `62dcee61d1`): the launch has
  one production path, `agents::start_cli` (`agents.rs:188-221`, `launch_input` at `:209`), from
  the rail's + (`Rail::new_agent`, `rail.rs:1539-1551`) and the New Agent picker (`Start::run`,
  `agents.rs:332-345`); `send_selection`, `close_guard`, `rich_input` and the agent bar read the
  agent by name through `agent_in` and launch nothing. `MarleySettings` (`marley_workbench.rs:222`)
  derives `Eq`; `MarleySettingsContent` (`marley.rs:9-62`) holds #516's, #547's, #550's, #535's,
  #503's, #569's, #571's and #565's keys, each enum with the strum derives and `#[default]`, and
  its only map is a `BTreeMap` (`MergeFrom` has no `std::collections::HashMap`). The page's
  `agents_section` (`marley_page.rs:49`) has seven items, #571's Browser Click Pause Agents
  dropdown the one to copy; the renderers sit in `settings_ui.rs:559-566`. `default.json` has a
  `marley` block (`assets/settings/default.json:1664-1731`). A project's settings file is parsed
  as `ProjectSettingsContent` and stored with `marley` empty (`settings_store.rs:1110-1137`), so a
  repository cannot set `marley.*`. `ProcessInfo { name, cwd, argv }` is `pub(crate)`
  (`pty_info.rs:68-73`), refreshed off the main thread, and no public getter gives the argv. A
  Claude Code row with events reads `seat_line`, which names no agent (`claude_events.rs:265`);
  every line under a title is `Color::Muted` (`rail.rs:3975-3980`); #569's flag is an icon in
  `Color::Warning` after the card (`rail.rs:2399-2412`). Every seat carries `permission_mode`
  (`PERMISSION_MODE_LABEL`, `claude_events.rs:60-61`, from lead events), which #571 reads
  (`click_pause.rs:48-49`, `bypassPermissions` and `dontAsk` prompt-less for its rule).
- **The binaries, read again at promotion** with `grep -a`, never run: Claude Code 2.1.283 also
  has `--allow-dangerously-skip-permissions`, which makes bypass a mode the user can switch to
  mid-session without starting in it, and refuses a `bypassPermissions` restored from settings
  in a session "not launched with --dangerously-skip-permissions"; so the arguments cannot say
  what a session is doing now, and the reported mode can. Codex 0.155.1 has `--sandbox
  <SANDBOX_MODE>` ("Select the sandbox policy to use when executing model-generated shell
  commands"), the approval policy option, and `--dangerously-bypass-approvals-and-sandbox`
  ("EXTREMELY DANGEROUS. Intended solely for running in environments that are externally
  sandboxed"); no `--yolo` alias.

## UI proof
UI-AFFECTING: what the rail's + starts, the agent rows' mark and the settings page.
`script/e2e/532-agent-permission-modes.sh` (`compositor sway`: the + menu and the settings page
are clicked). Fixtures: fake `claude` and `codex`, Python scripts first on the terminal's PATH
that print `started with: <their arguments>` and wait; the fake `claude` also sends #519's hook
events through Marley's plugin hook, reporting `bypassPermissions` when its arguments ask for
bypass or its environment says so (as Claude Code's own settings would), else `default`, and on
an Enter reports `default` (as after Shift+Tab); a scratch repository whose `.zed/settings.json`
holds `"marley": {"claude_code_permissions": "bypass"}`, which must change nothing; the run's user
settings with no permission keys at first. Shots:
- `532-01-default`: the rail's +, Claude Code: `started with:` and nothing more; the row has no
  mark.
- `532-02-bypass`: `agent_permissions_by_project` written for the repository with
  `"claude_code": "bypass"`; the +, Claude Code: `started with: --dangerously-skip-permissions`,
  and the row carries the "bypass" chip in the warning color.
- `532-03-full-access`: `"codex_permissions": "full_access"`; the +, Codex: `started with:
  --sandbox danger-full-access --ask-for-approval never`, and the row carries "full access".
- `532-04-typed`: a new terminal, `claude --dangerously-skip-permissions` typed by hand with the
  fake's events off: the chip from the arguments alone.
- `532-05-reported`: an Enter in the bypass terminal, whose fake then reports `default`: its chip
  gone though its arguments still hold the flag; and a new terminal whose fake reports
  `bypassPermissions` with no flag: the chip.
- `532-06-settings-page`: `marley: open settings`: the Agents section shows Claude Code
  Permissions at Ask and Codex Permissions at Full Access.
The run log holds each fake's `started with:` line and the chip's tooltip text.

## Locked-In Decisions
- D1: Off by default. Each agent keeps its own permission prompts unless Chad turns this on
  (Chad, 2026-09-25; the survey's default; Orca's bypass by default is what not to take).
- D2: One choice per agent, named as each CLI names it: Claude Code's bypass
  (`--dangerously-skip-permissions`) and Codex's full access (`--sandbox danger-full-access
  --ask-for-approval never`, the two settings Codex's Full Access preset stands for). Codex's
  `--dangerously-bypass-approvals-and-sandbox`, which Orca passes, is not used: its own help says
  it is meant only for environments sandboxed from outside. The mark still recognizes it.
- D3: Per-project values live in the user's own settings, keyed by the project's folder, never in
  a project's `.zed/settings.json`. A repository that ships a settings file must not be able to
  turn off an agent's prompts, and Zed's parsing of project files keeps the `marley` key out.
- D4: A project's folder is its group's main folder (`Project::project_group_key`), so a worktree
  agent (#510) takes its project's setting.
- D5: The mark comes from the foreground process's own arguments, not from what Marley launched,
  so a bypass typed by hand shows too, and nothing Marley remembers can go stale. Changed at
  promotion: for Claude Code with #519's events, D7 comes first.
- D6: The keys sit flat in `marley` beside #516's (`redact_secrets_for_agents`) and the two
  defaults join #516's Agents section of the page.
- D7 (at promotion): a Claude Code seat's reported `permission_mode` wins over its arguments:
  the arguments say how it started, the hook events what it does now, and Claude Code can enter
  bypass from its own settings or leave it with Shift+Tab.
- D8 (at promotion): the mark is a chip at the row's end in the warning color, with a tooltip
  naming its source, not words in the status line: a row with events names no agent, and the
  row's lines are all muted.
- D9 (at promotion): a per-project entry matches a local project whose main folder is the
  entry's folder or inside it, the longest folder winning, with `~/` expanded, as the System One
  layer's project lists match; a remote project takes the defaults, since the map names local
  folders.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE no permission setting applies, Marley shall start Claude Code and Codex with no permission flag. | Shot `532-01-default` |
| REQ-002 | WHEN the setting that applies to the project says bypass, Marley shall start Claude Code with `--dangerously-skip-permissions`. | Shot `532-02-bypass` |
| REQ-003 | WHEN Codex's setting says full access, Marley shall start Codex with `--sandbox danger-full-access --ask-for-approval never`. | Shot `532-03-full-access` |
| REQ-004 | WHILE an agent in a terminal runs with a bypass or full-access flag, however it was started, its rail row shall carry the mark in the warning color. | Shots `532-02-bypass`, `532-03-full-access`, `532-04-typed` |
| REQ-005 | WHEN a project's `.zed/settings.json` asks for bypass, Marley shall start the agent without it. | Shot `532-01-default` |
| REQ-006 | WHEN the Settings window's Marley page opens, its Agents section shall show both settings with their values. | Shot `532-05-settings-page` |
| REQ-007 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |
| REQ-008 | WHILE Claude Code's hook events report its permission mode, its row's mark shall follow the reported mode: marked for `bypassPermissions`, unmarked for any other, whatever its arguments say. | Shot `532-05-reported` |

## Phase Plan
- **P1 Plan:** this spec; at promotion (2026-09-28): #516, #519 and #571 have shipped; the seams
  re-verified; the binaries' strings read again (never run).
- **P2 Code:** the ledger rows first (`crates/settings_content/src/marley.rs`,
  `crates/settings_ui/src/marley_page.rs`, `crates/settings_ui/src/settings_ui.rs`,
  `crates/terminal/src/terminal.rs`, `assets/settings/default.json`); the content types and
  `MarleySettings`; `launch_input` and `start_cli`; the argument getter, `permission_mark` and the
  chip; the page's two items. fmt and clippy clean; a review of the diff.
- **P3 Test:** write and run the scenario, read every shot, `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
