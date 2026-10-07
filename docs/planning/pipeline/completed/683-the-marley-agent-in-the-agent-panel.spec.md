---
pipeline_id: b835a537-6ace-4cde-a56d-7d08e5ffc9b2
ticket: docs/planning/tickets/open/TICKET-683-the-marley-agent-in-the-agent-panel.md
status: Phase 4 — Complete PASS
title: The Marley agent in the Agent Panel
type: feature
slice: prong 2 C; phase 1 item 5 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/intake/marley-agent-manager-foreman.md
  - docs/planning/pipeline/completed/681-docs-and-settings-tools.spec.md
  - docs/planning/pipeline/completed/682-settings-changes-accepted.spec.md
---

## Title
A "Marley" agent in the Agent Panel's new-thread menu: Claude Code, through the registry's
`claude-acp` adapter on the user's own login, told that its job is explaining and configuring
Marley with #681's and #682's tools, and kept from editing files or running commands. Off until
the user turns it on; offered once to someone signed in to Claude Code.

## Scope
### In
- **One Zed hunk** (revised in Code from three, see the notes): `marley.agent_session_meta`, a
  map in Marley's own settings block from an agent server's id to a JSON object, which
  `agent_servers` sends as `_meta` on that agent's `session/new`, `session/load` and
  `session/resume`.
- **`marley.assistant.enabled`** (off by default, in the Settings window's Marley page with the
  other switches). On, Marley adds a custom `Marley` entry to the agent servers' defaults (in memory,
  as it adds `context_servers.marley`) whose command is the one Zed resolves for the `claude-acp`
  registry adapter, with `CLAUDE_CODE_EXECUTABLE` set to the `claude` Marley launches and
  `ANTHROPIC_API_KEY` empty, and `marley.agent_session_meta.Marley`: the instructions appended to
  Claude Code's system prompt, Bash, Edit, Write, NotebookEdit and MultiEdit disallowed, bypass
  mode refused. Off, neither is there. `MARLEY_ASSISTANT_ADAPTER` names a stand-in ACP program.
- **The offer, once:** when the user's settings do not name `marley.assistant.enabled` and
  `claude auth status` says signed in, a notification: "Marley can help set itself up, through
  Claude Code" with Turn On and Not Now; either writes the choice to the user's settings, so it
  is never asked again.
- **The instructions:** what the agent is for (explaining Marley and Zed, finding and changing
  settings, finding keys), which Marley tools to use, that each settings change is the user's to
  accept, and that it edits no file and runs no command.

### Out (explicitly deferred)
- Codex and Zed's own agent (TICKET-687); the terminal version (TICKET-684).
- A Marley icon for the entry (the registry's Claude icon shows; Rusty's placeholder rule
  applies when Chad has one).
- Opening a real Marley thread in the scenario: it would run Claude Code on the user's own login
  and spend their plan; the first real thread is Chad's.

## Reference (§20)
Upstream Zed (the `agent_servers`, `project` and `settings_content` crates): Zed's Agent Panel
runs ACP agents from the registry and from custom commands, and ACP's `_meta` is the protocol's
own place for client-to-agent extension data; the hunks let a settings entry use it. The adapter
side is `@agentclientprotocol/claude-agent-acp`'s documented handling of `_meta.systemPrompt` and
`_meta.claudeCode.options`.

### Prior art
- **Behavior maps:** `docs/t3code_architecture/01-agents-and-providers.md` §2.1: T3 drives Claude
  Code through the Agent SDK with its own system-prompt append and tool limits
  (`makeClaudeQueryOptions`), the same knobs the adapter exposes through `_meta`.
- **Published material:** ACP schema 1.7.0 v1 (`NewSessionRequest.meta`, `LoadSessionRequest`,
  `ResumeSessionRequest`; `Meta` is a JSON map); the adapter 0.81.0 to 0.87.0
  (`dist/acp-agent.js`): `_meta.systemPrompt` (a string, or an object whose `append` is added to
  the `claude_code` preset), `_meta.claudeCode.options.disallowedTools` (merged into the SDK's),
  `allowDangerouslySkipPermissions: false` (no bypass mode), `CLAUDE_CODE_EXECUTABLE` (the
  binary it runs). Anthropic's terms allow the unmodified Claude Code on the user's own
  subscription, which this is.
- **The code we ship** (an Explore read, 2026-10-07): `crates/agent_servers/src/acp.rs:1475-1495`
  (the three requests, no meta today), `:454-491` (`AcpConnectionDefaults` reading settings by
  agent id), `:4386` (Marley's MCP server reaches every external session);
  `crates/agent_servers/src/custom.rs:18,229-233` (`CLAUDE_AGENT_ID` and its env);
  `crates/project/src/agent_server_store.rs:294-489` (`reregister_agents`, registry lookup by the
  key at 377), `:1508-1623` (the project-side settings mirror);
  `crates/settings_content/src/agent.rs:764-826` (`CustomAgentServerSettings`);
  `crates/marley_workbench/src/mcp.rs:271-326` (`offer_to_zeds_agents`, the in-memory defaults
  pattern); `agent_versions.rs` (where Marley already runs `claude --version`).

## UI proof
`script/e2e/683-the-marley-agent-in-the-agent-panel.sh` (`compositor sway`). Fixtures: a fake
`claude` (`MARLEY_CLAUDE`) whose `auth status` prints `{"loggedIn": true}`; a scripted ACP agent
as `MARLEY_ASSISTANT_ADAPTER`, logging what it reads. Steps and shots: the offer at start
(`683-01-offer`); Turn On, the setting written (`683-02-turned-on`); the project's New Agent
Thread submenu with "Marley" in it (`683-03-menu`); a Marley thread, answered, and the agent's
log holding `session/new`'s `_meta` (`683-04-thread`); the switch off, the submenu without Marley
(`683-05-off`).

## Locked-In Decisions
- D1 — The prompt and the tool limits travel as ACP `_meta` from a settings field, not through a
  Marley proxy between Zed and the adapter: three small hunks against a JSON-RPC relay and an entry
  re-inserted at every registry refresh.
- D2 — The Marley entry lives in the in-memory defaults while the switch is on, never in the
  user's file, as `context_servers.marley` does.
- D3 — The offer asks once; the user's answer is the setting.
- D4 — Edit, Write, NotebookEdit, MultiEdit and Bash are disallowed, and bypass mode is refused;
  Read stays, so the agent can read a file the user names.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts with `claude auth status` signed in and the user's settings not naming `marley.assistant.enabled`, the system shall show the offer with Turn On and Not Now. | `683-01-offer` |
| REQ-002 | WHEN the user picks Turn On, the system shall write `marley.assistant.enabled: true` to the user's settings and not offer again. | The file; `683-02-turned-on` |
| REQ-003 | WHILE `marley.assistant.enabled` is on, the Agent Panel's new-thread menus shall list "Marley". | `683-03-menu` |
| REQ-004 | WHILE it is off, no menu shall list it. | `683-05-off` |
| REQ-005 | WHEN `marley.agent_session_meta` names an agent server, the system shall send that object as `_meta` on the agent's `session/new`. | The scripted agent's log holds the meta of a Marley thread |
| REQ-006 | The Marley agent's sessions shall carry the instructions as a system-prompt append, the five tools disallowed, and bypass refused. | The same log: the append, `disallowedTools`, `allowDangerouslySkipPermissions: false`; `683-04-thread` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the Zed hunks with their touchpoint rows; `marley.assistant`; `assistant.rs`
  (the entry, the offer, the instructions); the scenario; a review; `just gate-diff`.
- **P3 Test** — the scenario; every shot read.
- **P4 Complete** — docs, ledger, close, archive, commit, push, install.
