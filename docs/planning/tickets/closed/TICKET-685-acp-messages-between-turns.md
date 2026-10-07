# TICKET-685 — An ACP agent's messages and permission requests between turns

- **Ticket:** LOCAL #685 (spike, prong 2 C; harness ACP-003, Marley's half)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** ../../pipeline/completed/685-acp-messages-between-turns.spec.md
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 2; rustal-harness
  TICKET-111 (`rh acp`) waits on it
- **Status:** closed

## Summary
The harness's `rh acp` will show a manager's thread in Zed's Agent Panel, and a manager posts
reports and asks for confirmations on its own, between the person's prompts. Read on 2026-10-07,
Zed's client takes both with no running turn: `handle_session_notification`
(`crates/agent_servers/src/acp.rs:4805`) and `handle_request_permission` (`:4571`) look the
session up and update its thread without asking whether a turn runs. An `AgentMessageChunk` after
a turn joins the last assistant entry (`push_assistant_content_block_with_message_id`,
`crates/acp_thread/src/acp_thread.rs:3021`; a different `message_id` starts a new chunk in that
entry rather than merging the text), with no notification. A permission request adds a tool call
waiting for confirmation and notifies with "Waiting for tool confirmation"
(`crates/agent_ui/src/conversation_view.rs:1672`). This ticket proves it in Marley's build with a
small scripted ACP agent registered under `agent_servers`, and records what the panel shows.

## Acceptance
A scripted ACP agent that answers one prompt, then three seconds later sends an agent message and
a permission request: the Agent Panel shows the message, shows the request with its options and
notifies, and the agent receives the person's choice. The shots and the agent's log are sent to the
harness.
