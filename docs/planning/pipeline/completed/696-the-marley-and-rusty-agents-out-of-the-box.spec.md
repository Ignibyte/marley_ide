---
pipeline_id: 78506b7d-d282-41a6-9f3a-42ba80eefd29
ticket: docs/planning/tickets/open/TICKET-696-marley-and-rusty-agents-out-of-the-box.md
status: Phase 4 — Complete PASS
title: The Marley and Rusty agents out of the box
type: feature
slice: prong 2 C; the Marley agent of docs/planning/intake/marley-agent-manager-foreman.md, on Chad's 2026-10-08 answer
references:
  - docs/planning/pipeline/completed/683-the-marley-agent-in-the-agent-panel.spec.md
  - docs/planning/pipeline/completed/687-the-marley-agent-on-codex-and-zeds-agent.spec.md
  - docs/planning/pipeline/completed/643-rusty-switch-and-connection.spec.md
---

## Title
The Marley agent is on out of the box, on whichever agent the user is signed in to. While Rusty is
on, a Rusty entry sits beside it in the Agent Panel.

Chad, 2026-10-08: "marley should come pre configured out of the box … Marley is the agent behind
the ide, rusty is the agent behind rusty enabled so both will be in the agent panel".

## Scope
### In
- `MarleyAssistantAgent::Auto`, the new default. Marley resolves it at start, and again when the
  setting turns to it:
  - Claude Code when `claude auth status` says signed in;
  - else Codex when `codex login status` does;
  - else Zed's agent when its default model's provider is signed in;
  - else nothing.
- `assets/settings/default.json`: `marley.assistant` is `{ "enabled": true, "agent": "auto" }`.
- The offer goes, since nothing is left to offer. A user's own `enabled: false`, including an
  earlier Not Now, still turns the agent off.
- A **Rusty** entry while `marley.rusty.enabled` is on, on the agent `marley.assistant.agent`
  resolves to, whether or not the Marley agent is on:
  - Claude Code and Codex: a `Rusty` entry running the same adapter as the Marley entry, with
    Rusty's instructions and Rusty's MCP server on its own sessions;
  - Zed's agent: a `rusty` profile.
- Text: the settings page, the guide's Marley agent section, and the settings' documentation.

### Out (explicitly deferred)
- Narrowing the Marley agent's Marley MCP tools to its eight on Claude Code and Codex. Today
  only Claude Code's own file and shell tools are blocked, and Zed passes every agent the whole
  `marley` server. This is a follow-up ticket.
- A separate agent choice for Rusty: `marley.assistant.agent` picks the agent for both.
- Scenarios 683 and 687, which test the offer, are left as they were. The 696 scenario covers
  what replaces the offer; they are not in the golden set.

## Reference (§20)
Upstream Zed, Agent Panel external agents. `agent_servers`' `CustomAgentServerSettings::Custom`
entries are listed in New Agent Thread, and `agent.profiles` profiles are offered to Zed's agent.
Marley adds entries and a profile to the settings' in-memory defaults, as #683 and #687 do, with
no Zed hunk. No Warp analog: Warp has no Agent Panel of per-agent entries.

### Prior art
- **The code we ship:** `assistant.rs` (#683, #687) already does all of this for one entry:
  - the in-memory defaults, the adapter's command resolved through `AgentServerStore`, and the
    session `_meta` keyed by entry (`marley.agent_session_meta`, the one hunk in
    `agent_servers`);
  - Codex's `CODEX_CONFIG`, and Zed's profile.

  This ticket makes it two entries and adds Auto. `rusty.rs` (#643) already finds Rusty's server:
  `find(search_path)` for `embedded`, the loopback URL for `service`.
- **The adapters:**
  - claude-acp merges `_meta.claudeCode.options.mcpServers` with the servers ACP passes
    (`dist/acp-agent.js` around line 6110: `mcpServers: { ...userProvidedOptions.mcpServers,
    ...mcpServers }`). So a server named in the meta reaches only that entry's sessions.
  - Codex's `mcp_servers` table, given through `CODEX_CONFIG`, is the same for Codex.
  - Claude Code's permission rules take `mcp__<server>` for all of a server's tools.
- **The behavior maps:** `docs/zed_architecture/` has nothing on custom agent entries beyond what
  the code shows.
- **Decisions this builds on:** AD-683, AD-687, AD-643 and AD-661 (Rusty off leaves no trace,
  which the Rusty entry follows).

## UI proof
`script/e2e/696-the-marley-and-rusty-agents-out-of-the-box.sh`, under `compositor sway`:
- a stand-in `claude` that says it is signed in, and a stand-in ACP adapter
  (`MARLEY_ASSISTANT_ADAPTER`) that records each session's `_meta`;
- the stand-in `rusty-mcp`;
- the run's settings with `marley.assistant` taken out, so the defaults apply, and Rusty on.

Shots:
- `696-01-no-offer`: the window after start, with no offer.
- `696-02-agents`: New Agent Thread lists Marley and Rusty.
- `696-03-rusty-thread`: a Rusty thread on the stand-in.
- `696-04-rusty-off`: New Agent Thread without Rusty, once Rusty is off.

## Locked-In Decisions
- **D1:** `marley.assistant` defaults to `{ enabled: true, agent: "auto" }`. `Auto` is the enum's
  first variant and its default. This is Chad's 2026-10-08 choice, reversing AD-683's "off,
  offered once".
- **D2:** Auto is resolved by the existing checks, in their order. Claude Code and Codex are
  checked at once; Zed's model waits a few seconds, as the offer did, for the providers to load.
  The result is kept in the `Assistant` global. Nothing is written to the user's settings.
- **D3:** the offer, its notification and its Turn On and Not Now go. `decided()` goes with them.
- **D4:** the Rusty entry follows `marley.rusty.enabled` and the resolved agent, not
  `marley.assistant.enabled`. Rusty is the agent behind Rusty, the Marley agent the one behind
  Marley.
- **D5:** Rusty's tools reach only the Rusty entry:
  - Claude Code: `_meta.claudeCode.options.mcpServers.rusty`;
  - Codex: `CODEX_CONFIG.mcp_servers.rusty`;
  - either one: a stdio `rusty-mcp` found as Marley's own connection finds it, or the service's
    loopback URL;
  - Zed's agent: the `rusty` context server, which Marley offers while the `rusty` profile is
    wanted, as `agent_tools` does. Zed's agent only reaches tools through a context server.
- **D6:** the Rusty entry's limits:
  - Claude Code: the Marley agent's disallowed tools, plus `mcp__marley`;
  - Codex: read-only;
  - the Zed profile: no built-in tools, and the context servers' tools. A profile names a
    server's tools one by one, and Rusty's are Rusty's to change; Zed asks before each call.

  Rusty works through its own tools.
- **D7:** the e2e runner's copy keeps `marley.assistant` off and Rusty off, so no other scenario
  meets either entry (PR-686).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts with no `marley.assistant` in the user's settings and Claude Code signed in, the system shall list Marley in New Agent Thread and show no offer. | Shots 696-01, 696-02 |
| REQ-002 | WHEN `marley.assistant.agent` is `auto`, the system shall run the Marley entry on Claude Code if it is signed in, else Codex if signed in, else Zed's agent if its model is ready, and shall log the choice. | The log line in the scenario; review of the diff |
| REQ-003 | WHILE `marley.rusty.enabled` is on and an agent resolves, the system shall list Rusty in New Agent Thread. | Shot 696-02 |
| REQ-004 | WHEN a Rusty thread starts on Claude Code, its session `_meta` shall carry Rusty's instructions, `mcpServers.rusty` naming Rusty's server, and disallowed tools that include `Bash`, `Edit`, `Write` and `mcp__marley`. | The stand-in's recorded meta; shot 696-03 |
| REQ-005 | WHEN `marley.rusty.enabled` turns off, the system shall take the Rusty entry out. | Shot 696-04 |
| REQ-006 | WHEN the resolved agent is Codex, the Rusty entry's environment shall carry a `CODEX_CONFIG` with Rusty's instructions, `mcp_servers.rusty` and the read-only sandbox; WHEN it is Zed's agent, the system shall add a `rusty` profile with no built-in tools and the context servers' tools, and offer the `rusty` context server. | Review of the diff |
| REQ-007 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan:** this spec, and the design in the notes.
- **P2 Code:** Auto and the defaults; the Rusty entry; the text; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test:** the 696 scenario, with every shot read.
- **P4 Complete:** the CHANGELOG, the guide and the architecture notes (§21), the ledger (§19),
  then close the ticket, archive and commit.
