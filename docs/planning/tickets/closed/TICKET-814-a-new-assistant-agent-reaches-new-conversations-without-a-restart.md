# TICKET-814 — A new assistant agent reaches new conversations without a restart

- **Ticket:** LOCAL #814 (bug, Prong 2: the Marley and Rusty entries)
- **Owner:** 2f48853a-c455-455f-9b94-114158170228
- **Pipeline doc:** ../../pipeline/completed/814-a-new-assistant-agent-reaches-new-conversations.spec.md
- **Source ticket:** found by #813's visual check, 2026-10-18.
- **Status:** closed

## Summary
When `marley.assistant.agent` changes while Marley runs (Claude Code to Codex, say), Marley
rewrites the Marley and Rusty entries' command and environment, but a new Marley or Rusty
conversation still runs on the process the old agent started. Zed's `AgentConnectionStore`
(`agent_ui/src/agent_connection_store.rs`) caches a connection by agent id and drops it only when
the id leaves `agent_servers`, a registry agent's version changes, the connection fails, or the
user restarts it (L-claude-813). Since #687 the entries' old process keeps serving until Marley
restarts. Its session meta is already gone, so a Rusty conversation on Claude Code's old process
even loses Rusty's server and instructions. #813's scenario restarts Marley before its Codex step
to work around it.

## Acceptance
After `marley.assistant.agent` changes, the next Marley or Rusty conversation starts on the new
agent with that entry's instructions and limits, with no restart; open conversations on the old
agent keep working until closed.
