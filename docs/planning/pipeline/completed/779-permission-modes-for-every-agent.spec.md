---
pipeline_id: 75647664-62d7-4268-9799-f55b1498c63e
ticket: docs/planning/tickets/open/TICKET-779-permission-modes-for-every-agent.md
status: Phase 4 — Complete PASS
title: The permission settings reach every agent, and a thread switches its mode
type: feature
slice: agents in the Marley layout (#532's permission modes, #734's threads in tabs, #683/#687's entries)
references: [532, 683, 687, 696, 734]
---

## Title
`marley.claude_code_permissions: "bypass"` and `marley.codex_permissions: "full_access"` start
the agent CLIs Marley types without their prompts (#532), but not the agents in threads: Claude
Agent and Codex threads start in their adapter's default mode, and the Marley and Rusty agents
refuse bypass outright. Chad, 2026-10-12: "if the default settings allow bypass then the agents
start with it. but the zed agent panel needs the ability to toggle through and change these
permissions."

## Scope
### In
- A new Claude Agent (`claude-acp`) thread starts in `bypassPermissions` while
  `claude_code_permissions` is `bypass`; a new Codex (`codex-acp`) thread in `agent-full-access`
  while `codex_permissions` is `full_access`; anywhere they start (a tab, Zed's panel).
- The Marley and Rusty agents start in the same mode as the adapter they run on, by the same
  setting.
- The Marley and Rusty agents on Claude Code offer Bypass Permissions among their modes whatever
  the setting, so the thread's mode switch reaches it (they sent `allowDangerouslySkipPermissions:
  false`).
- A thread's mode switch (the selector, Shift+Tab) moves through every mode the agent offers, bypass
  and full access included.
- While a setting asks for no prompts, it decides how each new thread of that agent starts, even
  after a mode was picked in another thread (Zed's mode selector saves each pick as the agent's
  `default_mode`); while it is ask, that remembered mode applies, as before.

### Out (explicitly deferred)
- `agent_permissions_by_project` for threads: an adapter's default mode is per agent, not per
  project, so threads take the default, as a remote project's CLIs do.
- Zed's own agent (tool permissions are its profiles), Gemini and OpenCode threads, and harness
  seats' thread entries (#742).
- Lifting the Marley and Rusty agents' disallowed tools: bypass removes their prompts, not their
  tool limits.

## Reference (§20)
Upstream Zed, `agent_servers`: an agent server's `default_mode` setting, which
`AcpConnectionDefaults` applies with `session/set_mode` after `session/new` when the agent offers
it, and `agent_ui`'s mode selector with `agent::CycleModeSelector` (Shift+Tab). Marley's map comes
before the agent's `default_mode`, which Zed's mode selector rewrites on every pick.

### Prior art
- Behavior maps: `docs/zed_architecture/` (agent servers and the panel); nothing for modes in
  Warp's or Orca's maps.
- Published material: ACP's session modes (`session/new`'s `modes`, `session/set_mode`); the
  adapters' own mode ids read from their packages: `claude-agent-acp` 0.89.1 (`default`,
  `acceptEdits`, `plan`, `dontAsk`, `bypassPermissions`, bypass only while
  `allowDangerouslySkipPermissions` is not false and the settings do not disable it) and
  `codex-acp` 2.2.2 (`read-only`, `workspace-write`, `agent`, `agent-full-access`).
- The code we ship: `agent_servers::acp::AcpConnectionDefaults` (`refresh_from_settings`, the
  default-mode request), `CustomAgentServerSettings::default_mode`, `agent_ui::mode_selector`,
  #683's `marley.agent_session_meta` hunk (the model for a Marley map read by `agent_servers`),
  #532's `AgentPermissions`, `marley_agent::BYPASS_MODE`. Rejected: Marley writing `default_mode`
  into Zed's `agent_servers` defaults, because settings merge an agent's entry whole (`MergeFrom`
  on an enum replaces), so a user's `"claude-acp": {"type": "registry"}` would drop it.

## UI proof
`script/e2e/779-permission-modes-for-every-agent.sh` (`compositor sway`): stand-ins named
`claude-acp` and `codex-acp` (custom agent servers in the run's settings, one scripted ACP agent
offering each adapter's modes and logging every message), the Marley entry on the same script
offering Claude's modes; both settings at bypass and full access.
- `779-01-claude-bypass`: a Claude Agent thread in a tab, its mode switch reading Bypass
  Permissions; the agent's log holds `session/set_mode` `bypassPermissions` (REQ-001).
- `779-02-cycled`: Shift+Tab in its box: the switch reads the next mode, logged (REQ-004).
- `779-03-second-claude`: a second Claude Agent thread after that pick: Bypass Permissions again,
  logged (REQ-006).
- `779-03-codex-full`: a Codex thread reading Full access; logged `agent-full-access` (REQ-002).
- `779-04-marley-bypass`: a Marley conversation reading Bypass Permissions; its `session/new`
  `_meta` holds `allowDangerouslySkipPermissions: true` (REQ-003).
- `779-05-ask`: both settings back to ask: a new Claude Agent thread reads Default, with no new
  `session/set_mode` (REQ-005).

## Locked-In Decisions
- D1 — Marley's defaults travel as `marley.agent_default_modes`, an in-memory default map from an
  agent server's id to a mode id, which one hunk in `agent_servers` reads before the agent's own
  `default_mode` (as #683's `agent_session_meta`).
- D5 — Marley's mode comes first: Zed's `ModeSelector::set_mode` saves every pick as the agent's
  `default_mode`, so with the user's default first one Shift+Tab would end the setting's effect for
  every later thread. Found while reading the selector in the Test phase; first planned the other
  way.
- D2 — Marley fills it from the permission settings' defaults, from the user's own settings only
  (#532): `claude-acp` and every Marley entry on Claude Code → `bypassPermissions`; `codex-acp`
  and every Marley entry on Codex → `agent-full-access`; nothing while the setting is ask.
- D3 — The Marley and Rusty entries on Claude Code send `allowDangerouslySkipPermissions: true`, so
  Bypass Permissions is always among their modes; their start mode follows D2.
- D4 — The map is written only when it changes, so writing it does not wake its own observer.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `claude_code_permissions` is `bypass`, WHEN a Claude Agent thread starts, the system shall start it in Bypass Permissions. | shot `779-01-claude-bypass`, the agent's log |
| REQ-002 | WHILE `codex_permissions` is `full_access`, WHEN a Codex thread starts, the system shall start it in Full access. | shot `779-03-codex-full`, the log |
| REQ-003 | WHILE the setting for its adapter allows it, WHEN a Marley or Rusty conversation starts, the system shall start it in that adapter's bypass or full access mode. | shot `779-04-marley-bypass`, the log |
| REQ-004 | WHEN the user presses Shift+Tab in a thread's message box, the system shall move the thread to the next mode its agent offers. | shot `779-02-cycled`, the log |
| REQ-005 | WHILE both settings are ask, WHEN a thread starts, the system shall leave it in its agent's own default mode. | shot `779-05-ask`, the log |
| REQ-006 | WHILE a permission setting asks for no prompts, WHEN a thread of its agent starts after a mode was picked in another thread, the system shall still start it in bypass or full access. | shot `779-03-second-claude`, the log |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `settings_content/src/marley.rs` and `agent_servers/src/acp.rs` (Zed crates, their
  ledger rows extended first), `agents.rs`, `assistant.rs`, `marley_workbench.rs`; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
