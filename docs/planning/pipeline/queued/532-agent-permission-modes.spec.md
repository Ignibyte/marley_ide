---
pipeline_id: 6b2ce487-e206-4b16-b65c-54ffdc41c5c0
ticket: docs/planning/tickets/open/TICKET-532-agent-permission-modes.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Agent permission modes: Claude Code's bypass or Codex's full access, as a setting"
type: feature
slice: prong 2 (starting agent CLIs), with the Marley settings page (#515, #516)
references: [docs/orca_architecture/README.md, docs/orca_architecture/01-agents-and-sessions.md, docs/planning/pipeline/completed/515-marley-settings-page.spec.md, docs/planning/pipeline/active/516-secret-redaction-for-agents.spec.md]
---

## Title
A setting, off by default, per agent and per project, makes Marley start Claude Code with
`--dangerously-skip-permissions` or Codex with its full access
(`--sandbox danger-full-access --ask-for-approval never`). An agent row in the rail says
"bypass" or "full access" whenever its terminal's agent runs with such a flag, however it was
started, so a bypass is never silent. The defaults sit in the Marley settings page's Agents
section.

## Scope
### In
- **The settings**, in the `marley` block beside #516's, flat as `marley.layout` is:
  `claude_code_permissions`, `"ask"` (the default, Claude Code's own prompts) or `"bypass"`;
  `codex_permissions`, `"ask"` (the default) or `"full_access"`; and
  `agent_permissions_by_project`, a map from a project's folder to either or both
  (`{"claude_code": "bypass", "codex": "full_access"}`), which wins over the default for that
  project. The content types live in `crates/settings_content/src/marley.rs`; `MarleySettings`
  resolves them.
- **The launch.** `marley_agent::launch_input` takes the mode and types
  `claude --dangerously-skip-permissions`, `codex --sandbox danger-full-access --ask-for-approval
  never`, or the program alone. `agents::start_cli` reads the mode for the workspace's project,
  keyed by its group's main folder (`Project::project_group_key`), so the rail's +, the New Agent
  picker and #510's worktree agents all take it.
- **The mark.** An agent row's status line reads `Claude Code · bypass · …` or
  `Codex · full access · …`, in the warning color, while the terminal's foreground process
  carries a bypass flag in its arguments: Claude Code's `--dangerously-skip-permissions` or
  `--permission-mode bypassPermissions`, Codex's `--sandbox danger-full-access` or
  `--dangerously-bypass-approvals-and-sandbox`. Zed's terminal gains a getter for the foreground
  process's arguments, which it reads already.
- **The page.** The Marley page's Agents section (#516) gains "Claude Code Permissions" and "Codex
  Permissions" as dropdowns over the two defaults, user settings only; the per-project map is set
  in `settings.json`.

### Out (explicitly deferred)
- Gemini CLI's `--yolo` and OpenCode's modes: Chad named Claude Code and Codex only.
- Agent Panel threads: Zed's own `agent_servers.<name>.default_mode` sets the mode they start in.
- The mode a running session reports: every hook event carries `permission_mode` (#519), which
  would also show a mode changed inside Claude Code; the row can follow it once #519 ships.
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

## UI proof
UI-AFFECTING: what the rail's + starts, the agent rows' mark and the settings page.
`script/e2e/532-agent-permission-modes.sh` (`compositor sway`: the + menu and the settings page
are clicked). Fixtures: fake `claude` and `codex`, Python scripts first on the PATH that print
`started with: <their arguments>` and wait; a scratch repository whose `.zed/settings.json` holds
`"marley": {"claude_code_permissions": "bypass"}`, which must change nothing; the run's user
settings with no permission keys at first. Shots:
- `532-01-default`: the rail's +, Claude Code: the terminal prints `started with:` and nothing
  more, and the row carries no mark.
- `532-02-bypass`: the scenario writes `agent_permissions_by_project` for the repository with
  `"claude_code": "bypass"` into the user settings; the +, Claude Code again: `started with:
  --dangerously-skip-permissions`, and the row reads `Claude Code · bypass` in the warning color.
- `532-03-full-access`: `"codex_permissions": "full_access"` in the user settings; the +, Codex:
  `started with: --sandbox danger-full-access --ask-for-approval never`, and the row reads
  `Codex · full access`.
- `532-04-typed`: a new plain terminal, `claude --dangerously-skip-permissions` typed by hand:
  the row reads `bypass` too.
- `532-05-settings-page`: `marley: open settings`: the Agents section shows Claude Code
  Permissions at Ask and Codex Permissions at Full Access.

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
  so a bypass typed by hand shows too, and nothing Marley remembers can go stale.
- D6: The keys sit flat in `marley` beside #516's (`redact_secrets_for_agents`) and the two
  defaults join #516's Agents section of the page.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE no permission setting applies, Marley shall start Claude Code and Codex with no permission flag. | Shot `532-01-default` |
| REQ-002 | WHEN the setting that applies to the project says bypass, Marley shall start Claude Code with `--dangerously-skip-permissions`. | Shot `532-02-bypass` |
| REQ-003 | WHEN Codex's setting says full access, Marley shall start Codex with `--sandbox danger-full-access --ask-for-approval never`. | Shot `532-03-full-access` |
| REQ-004 | WHILE an agent in a terminal runs with a bypass or full-access flag, however it was started, its rail row shall say so in the warning color. | Shots `532-02-bypass`, `532-03-full-access`, `532-04-typed` |
| REQ-005 | WHEN a project's `.zed/settings.json` asks for bypass, Marley shall start the agent without it. | Shot `532-01-default` |
| REQ-006 | WHEN the Settings window's Marley page opens, its Agents section shall show both settings with their values. | Shot `532-05-settings-page` |
| REQ-007 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, check #516 has shipped (its Agents section and keys),
  read `codex --help` and `claude --help` for the short and `=` forms the mark must also
  recognize, and re-verify the seams.
- **P2 Code:** the ledger rows first (`crates/settings_content/src/marley.rs`,
  `crates/settings_ui/src/marley_page.rs`, `crates/settings_ui/src/settings_ui.rs`,
  `crates/terminal/src/terminal.rs`); the content types and `MarleySettings`; `launch_input` and
  `start_cli`; the argument getter and the mark; the page's two items. fmt and clippy clean; a
  review of the diff.
- **P3 Test:** write and run the scenario, read every shot, `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
