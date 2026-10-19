---
pipeline_id: 5a9bb764-2e4d-43ee-816d-77cac0868dfd
ticket: docs/planning/tickets/closed/TICKET-814-a-new-assistant-agent-reaches-new-conversations-without-a-restart.md
status: Phase 4 — Complete PASS
title: A changed assistant agent reaches new conversations without a restart
type: bug
slice: Prong 2, the Marley and Rusty entries (#683, #687, #696, #813)
references: []
---

## Title
When `marley.assistant.agent` changes while Marley runs, Marley rewrites the Marley and Rusty
entries' command and environment under the same ids, but Zed's `AgentConnectionStore` keeps the
process it started for an id until the id leaves `agent_servers`. A new conversation therefore
runs on the old agent's process, and since the old session meta is gone, a Rusty conversation
there even loses Rusty's instructions and server. A new conversation should start on the new agent.

## Scope
### In
- `agent_ui`'s `AgentConnectionStore` gains `forget_connection(key)`: the store drops its entry
  for that agent, so the next request connects again; conversations already open keep the
  connection they hold.
- Marley's `assistant::set_entry` calls it for the entry's id in every workspace's Agent Panel
  when the entry's server settings change.

### Out (explicitly deferred)
- A user's own hand edit of an agent's command in `agent_servers`: Zed's behavior, unchanged.
- Ending the old process at once: it ends when its last conversation closes.

## Reference (§20)
Upstream Zed (`agent_ui::agent_connection_store`): `restart_connection` drops a cached entry and
connects again at once; `handle_agent_servers_updated` drops entries whose id left the store. The
new method is the first half of `restart_connection` without the new connection, so nothing
starts until a conversation asks.

### Prior art
- `crates/agent_ui/src/agent_connection_store.rs`: `entries: HashMap<Agent, Entity<..>>`,
  `request_connection` returns a cached entry by key, `restart_connection` removes then requests,
  `handle_agent_servers_updated` keeps every id still in `external_agents`.
- `crates/agent_ui/src/agent_panel.rs`: `AgentPanel::connection_store()` (public), one store per
  panel, so per workspace.
- Only `Agent::NativeAgent`'s status is read from the store (`agent_panel.rs`); a Custom entry's
  conversations hold their connection, so dropping the entry disturbs no open thread.
- `crates/marley_workbench/src/mcp.rs` `all_workspaces` (PR-claude-596: from outside a window's
  update).
- L-claude-813-zed-keeps-an-agents-process-by-its-id-001 named the cause.

## UI proof
`script/e2e/814-a-new-assistant-agent-reaches-new-conversations.sh` (#813's set-up and scripted
agent, sourced), under sway:
- `814-01-rusty-claude`: a Rusty conversation on Claude Code; its answer says `sandbox: no
  CODEX_CONFIG` (a Claude Code process).
- `814-02-rusty-codex`: `marley.assistant.agent` set to `codex` with no restart; a new Rusty
  conversation answers `sandbox: none`, a Codex process (REQ-001).
- `814-03-marley-codex`: a new Marley conversation answers `INITIAL_AGENT_MODE: read-only`
  (REQ-002).
- `814-04-still-claude`: the first conversation answers again, still `sandbox: no CODEX_CONFIG`
  (REQ-003).

## Locked-In Decisions
- D1 — The trigger is Marley's: `set_entry` is where an entry's command and environment change, so
  it says when the cached process is stale. A general rule in Zed (drop on any settings change of
  an agent) would also restart agents on favorites and default-mode writes.
- D2 — Forget, not restart: no process starts until a conversation asks, and each workspace's
  panel is told.
- D3 — Only when the entry's server settings differ from the ones it replaces.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `marley.assistant.agent` changes while Marley runs, the next Rusty conversation shall start on a process of the new agent with Rusty's instructions. | Shot `814-02-rusty-codex`; `expect` on the scripted agent's answer |
| REQ-002 | WHEN `marley.assistant.agent` changes while Marley runs, the next Marley conversation shall start on a process of the new agent with the Marley entry's limits. | Shot `814-03-marley-codex`; `expect` |
| REQ-003 | WHILE a conversation started before the change is open, it shall keep answering on the process it started with. | Shot `814-04-still-claude`; `expect` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the store's method, `set_entry`'s call; a review of the diff; `script/gates.sh
  --diff` green.
- **P3 Test** — the scenario on the debug build, every shot read; once on a build without the fix.
- **P4 Complete** — CHANGELOG, the crate note, the touchpoint row (§21), the ledger, close,
  archive, commit; #813's scenario loses its restart workaround.
