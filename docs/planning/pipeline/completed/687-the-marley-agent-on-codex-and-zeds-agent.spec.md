---
pipeline_id: fbca47f9-08e8-4973-b892-bc3f6cb028ce
ticket: docs/planning/tickets/closed/TICKET-687-the-marley-agent-on-codex-and-zeds-agent.md
status: Phase 4 — Complete PASS
title: The Marley agent on Codex and on Zed's agent
type: feature
slice: prong 2 C; phase 1 item 5's second half of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/pipeline/completed/683-the-marley-agent-in-the-agent-panel.spec.md
  - docs/planning/pipeline/completed/684-the-marley-agent-in-a-terminal.spec.md
---

## Title
The Marley agent for people without Claude Code: Codex through the registry's `codex-acp` in its
read-only mode, and Zed's own agent through a "Marley" profile that has Marley's tools and no
file tools. `marley.assistant.agent` picks which one, and the offer names the one it found.

## Scope
### In
- `marley.assistant.agent`: `claude_code`, `codex` or `zed`. Unset means Claude Code, so #683's
  switch keeps working. The Settings window's Marley Agent section gains an Agent dropdown.
- The offer finds an agent in order: Claude Code signed in (`claude auth status`), then Codex
  signed in (`codex login status` exit 0), then a Zed default model whose provider is
  authenticated. It names that agent, and Turn On writes `enabled` and `agent`.
- **Codex:** the `Marley` entry in the Agent Panel runs the command Zed resolves for `codex-acp`
  with `INITIAL_AGENT_MODE=read-only` and `CODEX_CONFIG` holding the Marley agent's instructions
  as `developer_instructions` and `sandbox_mode: "read-only"`.
- **Zed's agent:** the settings' in-memory defaults gain `agent.profiles.marley` ("Marley").
  It has no built-in tools, `enable_all_context_servers: false`, and
  `context_servers.marley.tools` with the seven docs, settings, keymap and actions tools. The
  `Marley` entry and the `claude-acp` default are not added.
- The terminal command follows the agent: Codex runs as
  `codex --sandbox read-only -c developer_instructions=…`; with `zed` the palette does not list
  it.
- The instructions name `keymap_change` (#686), and their last paragraph is the same for every
  agent.
- Off leaves no trace (AD-661): no entry, no profile, no palette command.

### Out (explicitly deferred)
- MCP `instructions` in the system prompt of Zed's own agent. Zed drops the field (see Prior
  art). It would be a Zed hunk in `context_server` and `agent`, and every Write thread would get
  Marley's server text. It can be its own ticket.
- A one-click "Marley" entry for Zed's agent in the new-thread menu. `CreateThreadOptions` takes
  no profile, so the entry would need a hunk in `agent_ui`.
- Choosing an agent at each start when `agent` is unset. The plan's "else Codex, else Zed's
  agent" happens once, in the offer; see D1.

## Reference (§20)
Upstream Zed (`agent_servers`, `agent_settings`, `agent_ui`'s profile selector): a registry
agent's command through `AgentServerStore::get_external_agent`, and profiles from
`agent.profiles`, which the picker lists under "Custom Profiles". Marley adds entries to the
in-memory defaults as #683 and `mcp::offer_to_zeds_agents` do, and changes no Zed code.

### Prior art
- **The code we ship:**
  - `crates/settings_content/src/agent.rs:552` has `AgentProfileContent { name, tools,
    enable_all_context_servers, context_servers, default_model }`.
  - `crates/agent_settings/src/agent_profile.rs:145` is `is_context_server_tool_enabled`: an
    explicit tool entry wins over `enable_all_context_servers`.
  - `crates/agent/src/thread.rs:4172` is `enabled_tools`. MCP tools are keyed by the
    `context_servers` key, here `marley`.
  - `crates/agent_ui/src/profile_selector.rs:68`, `:354` and `:398`: the picker lists every
    profile in the merged settings and shows the custom ones under a header.
  - `crates/settings/src/settings_store.rs:920` is `update_default_settings`.
  - `crates/agent_servers/src/custom.rs:19,234` has `CODEX_ID = "codex-acp"`, and Zed's
    registry-only Codex key pass-through.
  - `crates/language_model/src/registry.rs:459` is `default_model`.
  - `crates/context_server/src/types.rs:276` is `InitializeResponse`, which has no
    `instructions` field, so Zed's agent drops the server's instructions.
  - `crates/agent/src/templates/system_prompt.hbs` has no prompt per profile.
- **Published material:**
  - codex-acp 2.1.1 (`agentclientprotocol/codex-acp`): `INITIAL_AGENT_MODE` picks the session's
    first mode (`src/AgentMode.ts:145`). `read-only` is a read-only sandbox, with approval on
    request.
  - In codex-acp, `CODEX_CONFIG` is "JSON object merged into the Codex session config"
    (`src/index.ts:80`, `src/CodexAcpClient.ts:913`), which `thread/start` carries as `config`.
  - Codex's App Server schema (`codex app-server generate-json-schema`, codex-cli 0.160.0) lists
    `developer_instructions` and `sandbox_mode` as config keys.
  - `codex login status` exits 0 when signed in and 1 when not, and prints to stderr.
- **Behavior maps:** none on this seam. `docs/zed_architecture/` has no page on profiles.

## UI proof
Two scenarios under `compositor sway`, since the offer is decided once per start:
- `script/e2e/687-the-marley-agent-on-codex.sh`, with shots `687-01-codex-offer`,
  `687-02-codex-thread`, `687-03-setting` and `687-04-codex-terminal`;
- `script/e2e/687-the-marley-agent-on-zeds-agent.sh`, with shots `687-05-zed-offer` and
  `687-06-profile`.

## Locked-In Decisions
- **D1:** `marley.assistant.agent` (`MarleyAssistantAgent { ClaudeCode, Codex, Zed }`).
  - Unset means Claude Code.
  - The offer finds an agent once (Claude Code, then Codex, then a Zed model), and Turn On writes
    the one it named.
  - Rejected: probing at every start when unset. That would run two programs at each start, and
    the entry would change with the login.
- **D2:** Codex is a custom `Marley` entry with the command Zed resolves for `codex-acp`, plus
  `INITIAL_AGENT_MODE=read-only` and `CODEX_CONFIG`.
  - The adapter runs its own bundled Codex (no `CODEX_PATH`), which matches its protocol and reads
    the user's `~/.codex` login.
  - Rejected: the user's `codex` (the adapter pins its App Server's protocol), and an API key from
    Marley (Marley never handles a token).
- **D3:** Zed's agent gets the `marley` profile in the defaults while the switch is on and
  `agent` is `zed`.
  - The profile has no system prompt. Zed has no prompt per profile and drops MCP
    `instructions`, so the tools' own descriptions carry the guidance.
  - Rejected: writing to the user's `AGENTS.md` (their file, and it would reach every thread),
    and a Zed hunk for MCP instructions (see Out).
- **D4:** Detection never touches a token. It reads `claude auth status`'s `loggedIn`,
  `codex login status`'s exit status, and the registry's default model with
  `provider.is_authenticated`.
- **D6:** When the profile leaves (switch off, or another agent), a user `agent.default_profile`
  of `marley` goes back to `write`. Zed's delete does the same
  (`manage_profiles_modal.rs:463`). Left alone, every new thread of Zed's agent would get no
  tools (`thread.rs:4176`).
- **D5:** The terminal command follows `agent`.
  - Codex gets `--sandbox read-only -c developer_instructions=<the instructions as a TOML basic
    string>`.
  - `zed` hides the command, since Zed's agent has no terminal interface.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Claude Code is signed out, Codex is signed in and the switch is undecided, the system shall offer the Marley agent through Codex | shot `687-01-codex-offer` |
| REQ-002 | WHEN Turn On is clicked on that offer, the system shall write `marley.assistant.enabled: true` and `agent: "codex"` | the scenario's check of the settings file |
| REQ-003 | WHEN a Marley thread starts on Codex, the system shall run the adapter with `INITIAL_AGENT_MODE=read-only` and a `CODEX_CONFIG` that holds the Marley agent's instructions and `sandbox_mode: "read-only"` | shot `687-02-codex-thread` and the scenario's check of the stand-in's environment |
| REQ-004 | WHILE `agent` is `codex`, the Settings window's Marley Agent section shall show Agent: Codex | shot `687-03-setting` |
| REQ-005 | WHEN `marley: open marley agent in terminal` runs with `agent` set to `codex`, the system shall start `codex` with `--sandbox read-only` and the instructions as `developer_instructions` | shot `687-04-codex-terminal` and the check of the fake's arguments |
| REQ-006 | WHEN neither Claude Code nor Codex is signed in and Zed's default model is authenticated, the system shall offer the Marley agent through Zed's agent | shot `687-05-zed-offer` |
| REQ-007 | WHILE the switch is on and `agent` is `zed`, the profile selector of Zed's agent shall list Marley, a profile with no built-in tools, with only the seven Marley tools enabled | shot `687-06-profile` and the check of `settings_read agent.profiles.marley` |
| REQ-008 | WHEN the switch turns off, the system shall take the Marley profile out of the settings, and WHERE the user's `agent.default_profile` names it, set that back to `write` as Zed's own profile delete does | the checks of `settings_read` and of the settings file after the switch is off |

## Phase Plan
- **P1 Plan:** this spec, and the design in the notes.
- **P2 Code:** the setting and its dropdown, `assistant.rs` per agent, the instructions, a review
  of the diff, and `script/gates.sh --diff` green.
- **P3 Test:** the two scenarios, with every shot read.
- **P4 Complete:** CHANGELOG, the guide and the architecture docs (§21), the ledger (§19), close,
  archive and commit.
