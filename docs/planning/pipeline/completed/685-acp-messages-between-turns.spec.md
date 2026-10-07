---
pipeline_id: 6d2f0b78-9b2c-4262-a20d-c7d21b5dd233
ticket: docs/planning/tickets/open/TICKET-685-acp-messages-between-turns.md
status: Phase 4 — Complete PASS
title: An ACP agent's messages and permission requests between turns
type: spike
slice: prong 2 C; phase 2 of docs/planning/intake/marley-agent-manager-foreman.md (harness ACP-003, Marley's half)
references:
  - docs/planning/intake/marley-agent-manager-foreman.md
  - docs/planning/tickets/open/TICKET-685-acp-messages-between-turns.md
---

## Title
Prove in Marley's build that the Agent Panel shows a message an ACP agent sends after its turn
ended, and a permission request it makes while no turn runs, and that the person's choice
reaches the agent. rustal-harness's `rh acp` (its TICKET-111) shows a manager's thread this way,
and a manager posts reports and asks for confirmations on its own.

## Scope
### In
- A scripted ACP agent in the scenario (Python, #605's stand-in grown): it answers a prompt, ends
  its turn, then sends an `agent_message_chunk` with a `messageId` of its own, then a
  `session/request_permission` with Allow and Deny, and logs every message it gets.
- The scenario's shots of the panel at each step, and the agent's log of the person's choice.
- The result sent to the harness session, with the shots' findings and the code paths.

### Out (explicitly deferred)
- Any change to Zed's or Marley's code: the spike reports what the build does. A change it finds
  needed (a notification for a message between turns, say) becomes its own ticket.
- `rh acp` itself (rustal-harness TICKET-111) and the manager thread (its TICKET-106).
- `session/load` replay of a thread after a restart (Zed supports it, `acp.rs:1752`; the harness's
  ticket proves it with its own program).

## Reference (§20)
Upstream Zed (the `agent_servers`, `acp_thread` and `agent_ui` crates): how the Agent Panel takes
an ACP agent's `session/update` and `session/request_permission` is Zed's behavior, kept as it is.
The spike observes it in Marley's build; nothing is changed.

### Prior art
- **Behavior maps and our own record:** `docs/t3code_architecture/01-agents-and-providers.md`
  (T3 runs ACP agents as one of its adapters, `AcpAdapterV2.ts`); #605's and #501's scenarios run a
  stand-in ACP agent as a custom agent server (`script/e2e/605-archive-thread-rows.sh`,
  `write_stand_in_agent`), which this spike grows rather than rewriting.
- **Published material:** the ACP specification (agentclientprotocol.com, "Prompt Turn" and
  "Requesting Permission"): an agent streams `session/update` notifications and asks with
  `session/request_permission`; the spec ties both to a prompt turn but does not forbid either
  outside one. The schema Marley builds against, `agent-client-protocol-schema` 1.7.0 (v1):
  `ContentChunk.messageId` ("a change in `messageId` indicates a new message has started"),
  `RequestPermissionRequest { sessionId, toolCall, options }`, `PermissionOptionKind`
  `allow_once`, `allow_always`, `reject_once`, `reject_always`.
- **The code we ship:** `crates/agent_servers/src/acp.rs:4805` (`handle_session_notification`,
  no running-turn check) and `:4571` (`handle_request_permission`, none either);
  `crates/acp_thread/src/acp_thread.rs:3021` (`push_assistant_content_block_with_message_id`: a
  chunk after a turn joins the last assistant entry, a new chunk when its `messageId` differs,
  `can_merge_message_chunks` at `:384`) and `:3697` (`request_tool_call_authorization`);
  `crates/agent_ui/src/conversation_view.rs:1672` (a permission request notifies with sound,
  "Waiting for tool confirmation"; a message chunk notifies nothing). Zed owns the seam; Marley
  adds nothing.

## UI proof
`script/e2e/685-acp-messages-between-turns.sh` (`compositor sway`, for the project's + and the
Allow button). Fixtures: a scratch repository; the scripted agent, registered as a custom agent
server named `Scripted` in the run's settings. Steps and shots: a thread of the agent from the
project's +, prompted "start"; its reply (`685-01-replied`); three seconds after the turn ended,
the report (`685-02-report`); two seconds later, the permission request with its options
(`685-03-permission`); a click on Allow (`685-04-allowed`) and the agent's log naming the allow
option; a second prompt answered after it (`685-05-second-turn`).

## Locked-In Decisions
- D1 — The proof is Marley's real build driven by a scripted agent, not a test of Zed's crates:
  the harness asked for the behavior a person sees.
- D2 — The report carries its own `messageId`, as the harness was told to send each manager post.
- D3 — Nothing in the tree changes but the scenario. What the shots show is reported to the harness
  as it is, including what Zed does not do (no notification for a message between turns).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an ACP agent sends an `agent_message_chunk` after its turn ended, the Agent Panel shall show the message's text in the thread. | `685-02-report` shows "Report between turns" under the reply |
| REQ-002 | WHEN that chunk's `messageId` differs from the reply's, the panel shall keep the two texts apart rather than run them into one sentence. | `685-02-report`: the reply and the report on separate lines |
| REQ-003 | WHEN the agent sends `session/request_permission` while no turn runs, the panel shall show the tool call's title and its options. | `685-03-permission` shows "Merge the branch into main?" with Allow and Deny |
| REQ-004 | WHEN the person picks Allow, the agent shall receive a response that names the allow option. | The agent's log holds the response with `"optionId": "allow"`; `685-04-allowed` shows the call no longer waiting |
| REQ-005 | WHEN the person sends a second prompt after the request, the agent shall answer it in the same thread. | `685-05-second-turn` shows "Noted: again" |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the scripted agent and the scenario (no `.rs`); `script/gates.sh --fast` green.
- **P3 Test** — run the scenario, read every shot and the agent's log; send the result to the
  harness.
- **P4 Complete** — CHANGELOG, the plan doc and the architecture record (§21), ledger capture,
  close the ticket, archive, commit.
