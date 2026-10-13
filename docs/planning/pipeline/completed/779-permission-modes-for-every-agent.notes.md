# The permission settings reach every agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-779-permission-modes-for-every-agent.md
- **Pipeline spec:** 779-permission-modes-for-every-agent.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-12: "we need to allow --dangerously-skip-permissions (on claude) and
  full permissions (on codex)"; asked where, "if the default settings allow bypass then the agents
  start with it. but the zed agent panel needs the ability to toggle through and change these
  permissions." Run autonomously with the batch.
- **Classification / tier:** feature; Marley crate plus two small Zed hunks
  (`settings_content/src/marley.rs`, `agent_servers/src/acp.rs`), both with ledger rows to extend.
- **Checklist:** pick ✓ (BACKLOG row removed), pre-flight ✓, recall ✓, mint ✓, prior art ✓,
  spec ✓, design ✓.
- **Recall (§18.3):**
  - AD-claude-532: two settings, off by default, start the CLIs without prompts; only the user's own
    settings set them; a remote project takes the defaults.
  - AD-claude-683 / AD-claude-687: the Marley entry's prompt and limits ride
    `marley.agent_session_meta` through one hunk in `agent_servers`; on Codex it runs codex-acp in
    read-only.
  - The brain (consultation 576203fa…): nothing on this seam.
- **Discovery:**
  - `agent_servers/src/acp.rs` `AcpConnectionDefaults::refresh_from_settings` reads the agent's
    `default_mode` from `AllAgentServersSettings` at connect and on every settings change; after
    `session/new` it sends `session/set_mode` when the agent offers the mode, else logs the modes
    it has.
  - `settings_content` derives `MergeFrom` for `CustomAgentServerSettings` as a whole-value
    replace, so a Marley default inside Zed's `agent_servers` loses to any user entry for the
    same agent (Chad's settings hold `"claude-acp": {"type": "registry"}`).
  - `assistant.rs` `session_meta` sends `allowDangerouslySkipPermissions: false`; the adapter then
    drops `bypassPermissions` from the session's modes, so neither a default nor the switch can
    reach it.
  - The adapters' mode ids, read from their packages (claude-agent-acp 0.89.1 in Marley's data
    folder; codex-acp 2.2.2 fetched with `npm pack` into the scratchpad): see the spec's prior art.

### Design
- `crates/settings_content/src/marley.rs` (Zed crate, row extended): `agent_default_modes:
  Option<BTreeMap<String, String>>` on `MarleySettingsContent`, documented as Marley's own, like
  `agent_session_meta`.
- `crates/agent_servers/src/acp.rs` (Zed crate, row extended): `marley_default_mode(id, cx)` beside
  `marley_session_meta`; `refresh_from_settings` takes it when the agent has no settings entry and
  `.or_else` after the entry's own `default_mode`. One `// Marley:` comment per hunk.
- `crates/marley_workbench/src/agents.rs` (Marley): the mode ids (`CLAUDE_AGENT_BYPASS =
  marley_agent::BYPASS_MODE`, `CODEX_ACP_FULL_ACCESS = "agent-full-access"`);
  `AgentPermissions::starts_without_prompts(kind)` (the defaults' bypass); `thread_modes(cx)`,
  the map D2 names; `sync_thread_modes(cx)`, which writes it into the defaults when it differs
  from the `ThreadModes` global, called on the settings and the `Assistant` global.
- `crates/marley_workbench/src/assistant.rs` (Marley): `session_meta` sends
  `allowDangerouslySkipPermissions: true`; `entry_agents(cx)`, each entry's name and the agent it
  runs on; doc comments.
- `crates/marley_workbench/src/marley_workbench.rs` (Marley): the sync's observers in `init`.

### Visual check plan
| REQ | Set-up and action | Shot |
|---|---|---|
| 001 | Settings at bypass/full access; New Agent → the `claude-acp` stand-in in this project | `779-01-claude-bypass` |
| 004 | Shift+Tab in its message box | `779-02-cycled` |
| 002 | New Agent → the `codex-acp` stand-in | `779-03-codex-full` |
| 003 | `marley: talk to marley` (the entry on the stand-in, Claude's modes) | `779-04-marley-bypass` |
| 005 | Both settings set to ask in the run's settings; a new `claude-acp` thread | `779-05-ask` |
REQ-006 (the user's own default wins) is the hunk's order, read in the review: the scenario would
need a second profile.

### Risks
- A real adapter that names its modes differently (an older codex-acp's `full-access`) gets no
  mode change; Zed logs the modes it has. The ids are the ones the shipped versions use.
- The default applies to every thread of the agent, Zed's panel included: that is the request.

## Phase 2 — Code
- **Built:**
  - `settings_content/src/marley.rs` (Zed crate, row extended): `agent_default_modes`.
  - `agent_servers/src/acp.rs` (Zed crate, row extended): `marley_default_mode`; in
    `refresh_from_settings`, an agent with no settings entry takes it, and one with an entry takes
    it `.or_else` after its own `default_mode`. Two `// Marley:` comments.
  - `agents.rs`: `CODEX_AGENT`, `CODEX_ACP_FULL_ACCESS`, `thread_mode(kind)`, `thread_modes(cx)`
    (Claude Agent, Codex and the entries `assistant::entry_agents` names, each kept while
    `AgentPermissions::launch_mode(kind, &[])` asks for no prompts), the `ThreadModes` global and
    `sync_thread_modes`, observing the settings and the `Assistant` global and deferred once at
    `init`.
  - `assistant.rs`: `entry_agents`; `session_meta` sends `allowDangerouslySkipPermissions: true`;
    the module and function docs.
- **Deviations:** `AgentPermissions::starts_without_prompts` was not needed: `launch_mode(kind,
  &[])` is the defaults' answer already, a remote project's rule.
- **Review:**
  - REQ-001/002/003: the map holds `claude-acp`/`codex-acp` and the entries while their setting is
    bypass/full access; Zed refreshes an agent's defaults on every settings change, and sends
    `session/set_mode` after `session/new` when the agent offers the mode.
  - REQ-005: with both settings at ask the map is empty; the hunk then leaves the mode as before.
  - REQ-006: the hunk took the user's `default_mode` first (`or_else`); reversed in the Test
    phase (D5).
  - The sync writes only a changed map, so its own write wakes it once and stops; it reads the
    `Assistant` global, which `assistant::init` sets after `agents::init`, through `try_global`.
  - Only the user's own settings decide (#532): `MarleySettings::get_global` reads no project
    settings.
  - Upstream discipline: two hunks, each additive, rows written before the edit.
- **Gate:** `script/gates.sh --diff` GATE GREEN [diff], 17 of 17 (`gate-779.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/779-permission-modes-for-every-agent.sh` under `compositor sway`: one
  Python stand-in run as `claude-acp` and `codex-acp` (custom agent servers in the run's settings)
  and as the Marley entry (`MARLEY_ASSISTANT_ADAPTER`), offering claude-agent-acp 0.89.1's modes
  (Bypass Permissions only while `_meta` does not refuse it) or codex-acp 2.2.2's, logging each
  message; `claude_code_permissions` `bypass`, `codex_permissions` `full_access`,
  `list_cli_sessions` off.
- **The runs found:**
  - "claude" in New Agent picked the Claude Code terminal entry first (the real `claude`, typed
    with `--dangerously-skip-permissions` and its `bypass` chip, #532's path); the scenario now
    types "claude agent", and takes Codex's Thread row with Down.
  - The note that Ctrl+Alt+N opened Marley's picker from a terminal (#563) stays up over the
    thread's bottom right, where the mode switch and the send button are, so the first shots
    showed neither. The scenario closes the note.
  - Reading Zed's `ModeSelector` for the missing switch showed that every pick is saved as the
    agent's `default_mode`. With the user's default first (the plan), one Shift+Tab would have
    ended the setting's effect on every later thread: the hunk now puts Marley's mode first (D5,
    REQ-006 rewritten), with a step that proves it.
- **The final run**, every check passed (`e2e exit 0`, 7 checks); the bottom bars read in crops:
  - **779-01-claude-bypass** (REQ-001): a Claude Agent thread in a repo tab, "Message claude-acp",
    the switch reading **Bypass Permissions**; the stand-in logged `session/set_mode`
    `bypassPermissions`.
  - **779-02-cycled** (REQ-004): after Shift+Tab, the switch reads **Default**; logged `default`.
  - **779-03-second-claude** (REQ-006): a second Claude Agent thread, after that pick: **Bypass
    Permissions** again; a second `bypassPermissions` request.
  - **779-03-codex-full** (REQ-002): a Codex thread reading **Full access**; logged
    `agent-full-access`.
  - **779-04-marley-bypass** (REQ-003): "Message Marley", **Bypass Permissions**; the Marley
    session's `_meta` held `allowDangerouslySkipPermissions: true` and it logged
    `bypassPermissions`. The rail lists Marley, Codex and two Claude Agent threads.
  - **779-05-ask** (REQ-005): both settings at ask, a new Claude Agent thread reads **Default**;
    no new `bypassPermissions` request.
- **Seen, not in scope:** the #563 note covering a thread's mode switch and send button whenever
  the New Agent picker opens from a terminal by its key; worth a look of its own.
- **Gate after the change and the scenario:** `script/gates.sh --diff` again on the final tree.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Changed, #779); `docs/marley/guide.md` (Permission modes: threads, the
  mode switch, what a pick does); `docs/marley_architecture/marley_workbench.md` (Threads' modes);
  `docs/marley/zed-touchpoints.md` (both rows describe what shipped, Marley's mode first).
- Ledger appends: `AD-claude-779-marleys-permission-settings-start-threads-ahead-of-zeds-remembered-mode-001`,
  `L-claude-779-zeds-mode-selector-saves-each-pick-as-the-agents-default-001`. No `F-`: the
  ordering was caught in the Test phase before shipping, recorded as D5 and in the lesson.
- Brain: consultation 576203fa… closed with `brain decide`
  (`decisions/marleys-permission-settings-start-agent-threads-ahead-of-zeds-remembered-mode`).
- Ticket closed; pipeline archived; committed on `marley/workbench-shell`.
