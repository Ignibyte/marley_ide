# Keep the Marley agent to its own tools on Claude Code and Codex — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-698-keep-the-marley-agent-to-its-own-tools.md
- **Pipeline spec:** 698-keep-the-marley-agent-to-its-own-tools.spec.md

## Phase 1 — Plan
- **Request:** queued at #696; Chad, 2026-10-09: "go ahead and work on the remaining items you
  have open as well". Run by a fork of the session.
- **Classification:** bug, the Marley agent, Marley crate only.
- **Recall (§18.3):**
  - #696 disallows the whole `mcp__marley` server for Rusty's entry.
  - L-claude-696: claude-acp merges the session `_meta`'s options into Claude Code's.
  - #687: codex-acp reads `CODEX_CONFIG` as a JSON object merged into the session config.
- **Discovery:**
  - `marley_mcp::registry::registry()` lists every tool, including families not yet served,
    which a client can still call by name.
  - codex-acp's `build_session_config` overwrites a same-named server entry (spec, Prior art).

### Design
- **`assistant.rs`:** `marley_tools_kept_out()`, the registry's names minus `PROFILE_TOOLS`, each
  as `mcp__{CONTEXT_SERVER}__{name}`. `session_meta` adds them for `Entry::Marley`; Rusty's keeps
  `mcp__marley`.
- **File manifest:** `crates/marley_workbench/src/assistant.rs` (Marley crate); the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | Opens a Marley thread from the project's + → New Agent Thread → Marley, on the scripted agent; checks the `_meta` of its `session/new` | 698-01-marley-thread |
| 003 | — | The gate |

- **Not reached:** whether real Claude Code honors `mcp__marley__<tool>`. PR-687 keeps runs off
  real accounts.

### Risks
- A tool a later ticket adds to the profile must leave the list; D1 ties both to the same
  constant.

## Phase 2 — Code
- **Built:** `assistant.rs`: `marley_tools_kept_out()` (`marley_mcp::registry()`'s names minus
  `PROFILE_TOOLS`, each `mcp__marley__<name>`); `session_meta` adds them to `disallowedTools` for
  `Entry::Marley`. Rusty's entry keeps `mcp__marley`, the whole server.
- **Deviations:** none. Codex gets nothing (D2).
- **Hook note:** the fork's phase hooks read the parent's transcript, as in #700; edits went through
  Bash.
- **Review of the diff:** the list is built once per `session_meta` call from a const table; no
  entity is read. A tool the registry gains is kept out until it joins `PROFILE_TOOLS`. The eight
  stay callable: the filter is the same constant the Zed profile uses.
- **Gate (`698-gate-1.log`):** GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/698-keep-the-marley-agent-to-its-own-tools.sh`, under `compositor sway`,
  on a profile with no database (so the rail is Home over the project, #700), Rusty off, and the
  scripted agent behind the Marley entry. Nothing is typed into the thread.
- **Run (`shots-698a`), first time, every check passes:**
  - **698-01-marley-thread:** the Agent Panel shows "New Marley Thread" with "Message Marley — @ to
    include context"; the rail lists Home, then repo with its terminal and "New Agent Thread ·
    Marley · idle".
  - **REQ-001, REQ-002 (the `_meta` check):** `disallowedTools` holds Bash, Edit, Write,
    NotebookEdit and MultiEdit and 33 Marley tools as `mcp__marley__<tool>`. Those are all the
    browser tools (`browser_click`, `browser_type`, `browser_navigate` and the rest),
    `editor_open` and `editor_wait`, `fleet_snapshot`, `ports_list`,
    `session_surface_to_human`, and the terminal tools (`terminal_run`, `terminal_type`,
    `terminal_read`…). It holds none of the eight, and no bare `mcp__marley`.
  - **REQ-003:** the gate.
- **Not reached:** whether real Claude Code refuses `mcp__marley__<tool>` (PR-687 keeps runs off
  real accounts), and Codex, which the source shows can't be limited this way (spec, Prior art).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Fixed); `docs/marley_architecture/marley_workbench.md` (the Marley
  agent's section, "Since #698"); the slice in `docs/planning/intake/marley-agent-manager-foreman.md`
  (item 5). No Zed path touched.
- **Knowledge appended:** L-claude-698-codex-acp-replaces-a-servers-config-with-the-one-acp-passes-001.
- **For the main session:** Codex's Marley entry is still unlimited for Marley's tools; a limit
  needs Marley's server to tell the Marley entry's calls apart (a ticket of its own, not minted).
- **Brain:** no `rusty` MCP server in this repository's sessions.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate at commit:** `698-gate-2.log`, GATE GREEN [diff].
