---
pipeline_id: 644da70e-4f54-431e-a5e7-52e34da0b7c7
ticket: docs/planning/tickets/closed/TICKET-508-approvals-inbox.md
status: Phase 4 — Complete PASS
title: "One approvals inbox in the rail"
type: feature
slice: prong 2 (attention), on #519's events
references: [docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/completed/571-pause-before-a-consequential-click.spec.md, docs/planning/pipeline/completed/549-selection-to-the-agent.spec.md, docs/orca_architecture/README.md, docs/orca_architecture/01-agents-and-sessions.md, docs/orca_architecture/04-remote-control-and-mobile.md, docs/orca_architecture/06-cli-automations-skills.md, docs/planning/pipeline/completed/501-zeds-agents-drive-the-browser.spec.md]
---

## Title
One "needs you" list at the top of the rail gathers every agent that waits on the user: Agent
Panel tool calls waiting for confirmation, answered in place with Allow or Deny; Claude Code in a
terminal waiting on a permission or a question (#519's events), whose entry opens the terminal;
and an agent's click a Browser tab holds (#571), answered in place with Allow or Refuse. Each entry
names the agent, its project, what it asks and how long it has waited, oldest first. A pick from
the Browser tab is no longer pasted into a terminal whose agent waits, where it would answer the
prompt.

## Scope
### In
- **The inbox**, a section at the top of the rail above the projects, shown while an entry waits:
  "Needs you" and the count, then the entries oldest first. An entry shows the agent's icon,
  "<agent> · <project>", what it asks (the tool call's label with its input preview, or the tool
  and preview from #519's labels), and how long ago Marley first saw the wait.
- **Agent Panel entries.** From each workspace's Agent Panel (`AgentPanel::conversation_views`),
  each conversation's first pending tool call (`ConversationView::pending_tool_call`), found in
  its thread (`ConversationView::thread_view`, then `AcpThread::tool_call`). Allow and Deny
  buttons show when the prompt offers an allow-once and a deny-once option
  (`PermissionOptions::allow_once_option_id`, `deny_once_option_id`), and answer through
  `ThreadView::authorize_tool_call` with `SelectedPermissionOutcome::new`: the panel's own path,
  which ends in `AcpThread::authorize_tool_call`. A prompt without that pair (Zed's action
  choices, such as Save or Discard) shows no buttons; a click on any Agent Panel entry opens its
  thread, where every other choice (always allow, patterns) is made.
- **Terminal entries.** #519's seats that wait (`State::Waiting`), on a permission ("Permission
  for Bash: …") or on a question (AskUserQuestion), with the seat's question as what it asks; a
  click shows the terminal (`Rail::activate_terminal`).
- **Paused clicks** (#571). Each Browser tab that holds an agent's click, with the card's sentence
  as what it asks, Allow and Refuse answering it as the card's buttons do
  (`BrowserHub::answer_pause`), and a click showing the tab with the focus on the card.
- **Clearing.** An entry leaves when its prompt is answered anywhere: the tool call leaves
  `WaitingForConfirmation` (in the panel or in the inbox), the terminal seat's wait ends by
  #519's rule (the tool it asked for finishes, or the turn ends), the seat ends or loses its
  terminal, or the paused click is answered, expires or its page goes.
- **Picks.** `BrowserView::send_pick` pastes nothing while the target terminal's seat waits on a
  permission or a question, says why in the tray, and keeps the pick and its caption unsent, as
  #549's selection already does for its own paste.
- **The model**, pure, in `marley_rail`: the entries, their order and the section's rows.

### Out (explicitly deferred)
- Answering a terminal agent's prompt from the rail: a PermissionRequest hook of type `mcp_tool`
  that waits on a Marley tool (the second slice, sketched in the notes).
- The harness's questions, once C1's harness adapter feeds `marley_fleet` (#534, which adds
  them to this inbox).
- MCP elicitations raised through an Agent Panel thread, and subagent threads' prompts beyond what
  `pending_tool_call` returns for the conversation.
- A notification per wait (report 01, item 4; #538); attention order in the rail (report 01,
  item 5; #542); keyboard reach into the inbox (the rail's selection keeps to rows).
- The phone (report 04).
- A guard on the terminal's own inputs (rich input #481, the agent bar's Attach File #477): the
  user acts in that terminal, in front of its prompt.

## Reference (§20)
- **Warp:** a notification mailbox behind a bell icon lists agents' requests (command approvals,
  permission requests, idle prompts), filters them, and opens the session from an entry
  (docs.warp.dev/agent-platform/capabilities/agent-notifications/). Marley's list sits in the rail
  instead, answers Agent Panel prompts in place, and opens the terminal for a CLI agent's. No
  Warp code was read.
- **Upstream Zed:** the Agent Panel's permission prompt for a tool call (`agent_ui`'s thread view
  and `acp_thread`'s `PermissionOptions`). Kept as the answer path: the inbox calls the panel's
  public `ThreadView::authorize_tool_call`, never a copy of it.
- **Orca:** the Needs You model (report 01, item 2), approvals from its phone (report 04 §2.5, the
  part not copied) and the survey's notes for #508 (report 06, item 3).

### Prior art
- **Reports.** Report 01 item 2: one "needs you" state (Orca's `AgentQuestionIcon`, the
  `--agent-question` token), entries that say what is asked (the AskUserQuestion JSON kept as
  `interactivePrompt`), oldest first, cleared on the session's next hook (narrowed here to the
  tool asked for, D5), and no paste into a waiting agent
  (`src/renderer/src/lib/running-agent-targets.ts`). Report 04 §2.5: Orca's phone
  answers a terminal agent by typing "1", "2" or Escape
  (`mobile/src/session/mobile-native-chat-permission.ts`), and Orca registers PermissionRequest
  for status only (`src/main/claude/hook-settings.ts:92`); §3.2 and §3.4 row 1 sketch the
  structured answer this ticket leaves to its second slice. Report 05 §2.5: AskUserQuestion is a
  question, not a permission.
- **Published material.** ACP's `session/request_permission`: the agent offers options of kind
  `allow_once`, `allow_always`, `reject_once` or `reject_always`, and the client answers with one.
  Claude Code's PermissionRequest decision (for the second slice). Warp's mailbox, above.
- **Code we already ship.** Zed's answer path is public on `ThreadView`, though the
  `Conversation` it runs through is `pub(crate)`, so the inbox answers exactly as the panel's
  buttons do. `ConversationView::pending_tool_call` already picks a conversation's first waiting
  request, subagents' included. The rail already knows a thread waits
  (`marley_rail::thread_status`) and already refreshes when a confirmation is asked or answered
  (`changes_the_row`). #519's seats carry the terminal side. `marley_fleet`'s `AnswerRequest` is
  the harness answer's shape for the third slice.

## UI proof
UI-AFFECTING: a new section of the rail, and the Browser tab's tray. The shot list below is the
draft's; the notes' "Changed at promotion" and E2E plan override it where they differ (the
fixtures, the paused click, the clearing step).
`script/e2e/508-approvals-inbox.sh` (`compositor sway`: the rail's + menu, the buttons and the
entries are clicked). Fixtures: #501's stand-in ACP agent, extended so a prompt makes it send a
pending tool call ("Edit README.md") and `session/request_permission` with `allow_once` "Allow"
and `reject_once` "Reject", log the answer and end the turn; #519's fake `claude`, whose script
holds a PermissionRequest (Bash, `rm -rf build`); an offline Chromium page for the pick. Shots:
- `508-01-thread-entry`: the stand-in's thread, started from the rail's + and prompted: the
  inbox shows "Stand-in · repo", "Edit README.md", its age, Allow and Deny.
- `508-02-two-entries`: the fake's PermissionRequest adds "Claude Code · repo",
  "Bash: rm -rf build", below the older entry.
- `508-03-allowed`: Allow clicked: the thread's entry is gone, and the thread shows the tool call
  running; the stand-in's log reads `allow_once`.
- `508-04-denied`: a second prompt, then Deny: the log reads `reject_once`.
- `508-05-terminal-opened`: the terminal entry clicked: its terminal in front.
- `508-06-pick-refused`: a pick sent from the Browser tab while the fake still waits: the tray
  says why, and the terminal shows nothing pasted.
- `508-07-cleared`: the fake's PostToolUse for that Bash: the inbox is gone.

## Locked-In Decisions
- D1: One list at the top of the rail, shown only while something waits. The rail is where the
  agents are, and a section that is always there would mostly show nothing.
- D2: Oldest first, with the age. The prompt that has blocked an agent longest comes first
  (Orca's order, report 01 item 2).
- D3: Agent Panel entries answer in place through `ThreadView::authorize_tool_call`, the panel's
  own path, and only with allow once and deny once, the two answers that need nothing more. Every
  other choice opens the thread, and so does a sandbox escalation, whose Allow the panel gates
  behind its confusables check (changed at promotion: the inbox would otherwise go around it).
- D4: Terminal entries only open the terminal in this slice. Answering from the rail needs a hook
  that waits on Marley; typing a digit into the TUI, as Orca's phone does, depends on the dialog's
  numbering and is not done.
- D5: An entry leaves on evidence that its prompt was answered: the tool call out of
  `WaitingForConfirmation`, the asked-for tool finishing or the turn ending in the seat's events,
  or the seat's end. Another tool finishing does not count, and no timer clears an entry.
- D6: Nothing is pasted into a terminal whose agent waits on a permission: the paste would answer
  the prompt (report 01, item 2).
- D7: The inbox reads #519's seats, the Agent Panel's live threads and the Browser hub's paused
  clicks, and keeps nothing of its own but when it first saw each entry.
- D8 (at promotion): a terminal agent's question (AskUserQuestion) is an entry too. It blocks the
  agent on the user as a permission does, and #519's seat keeps the two in one wait; the entry
  says what is asked, and its click opens the terminal.
- D9 (at promotion): #571's paused clicks join the inbox: one list for everything an agent waits
  on the user for.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE an Agent Panel tool call waits for confirmation, the rail's inbox shall list it with its agent, project, what it asks and its age. | Shot `508-01-thread-entry` |
| REQ-002 | WHILE a terminal's Claude Code waits on a permission, the inbox shall list it the same way, the oldest entry first. | Shot `508-02-two-entries` |
| REQ-003 | WHEN Allow or Deny is clicked on an Agent Panel entry, Marley shall answer the tool call with that option, and the entry shall leave. | Shots `508-03-allowed`, `508-04-denied`; the stand-in's log |
| REQ-004 | WHEN an Agent Panel prompt offers no allow-once and deny-once pair, its entry shall show no buttons, and a click shall open its thread. | Review: only Zed's native agent raises such a prompt, which the stand-in cannot |
| REQ-005 | WHEN a terminal entry is clicked, Marley shall show that terminal. | Shot `508-05-terminal-opened` |
| REQ-006 | WHEN the waiting agent moves on, its entry shall leave, and WHILE nothing waits the inbox shall not show. | Shot `508-07-cleared` |
| REQ-007 | WHEN a pick is sent to a terminal whose agent waits on a permission, Marley shall paste nothing and say why in the tray. | Shot `508-06-pick-refused` |
| REQ-008 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |
| REQ-009 | WHILE a Browser tab holds an agent's click (#571), the inbox shall list it, and Allow or Refuse on the entry shall answer it as the card does. | Shot `508-08-paused-click`; the stand-in's output |
| REQ-010 | WHEN an Agent Panel prompt is a sandbox escalation whose Allow the panel gates, its entry shall show no buttons. | Review of the diff: the entry's rule against `ToolCall.sandbox_authorization_details` |

## Phase Plan
- **P1 Plan:** this spec; at promotion (2026-09-28), #519 and #571 have shipped, and the Agent Panel
  seams were re-verified unchanged since the draft.
- **P2 Code:** the pure model in `marley_rail`; the section in the rail, its entries from the
  panels and from #519's seats, the answer path; the pick guard. fmt and clippy clean; a review
  of the diff.
- **P3 Test:** write and run the scenario, read every shot, `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
