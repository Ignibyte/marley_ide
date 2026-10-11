---
pipeline_id: 6e71e4f7-413e-4134-acd6-ef21c450c12f
ticket: docs/planning/tickets/open/TICKET-717-second-opinion-and-handoff.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: New Thread with Context (second opinion and handoff, first slice)
type: feature
slice: agents together (docs/planning/intake/monocode-findings.md items 3 to 6; docs/t3code_architecture/README.md items 8 and 9), after the finish
references:
  - docs/planning/pipeline/completed/734-an-agent-thread-starts-in-a-tab.spec.md
  - docs/planning/pipeline/completed/706-agent-panel-threads.spec.md
  - docs/t3code_architecture/01-agents-and-providers.md
---

## Title
New Thread with Context: from an open thread, start a new thread on the same agent or another, in
a split beside it, with the old thread's context already in its message box. Chad, 2026-10-11:
"12.) yes, with a star new thread with context would be cool". It is the first slice of
second opinion and handoff: the starred part.

## Scope
### In
- **Where it starts.**
  - **New Thread with Context…** in a thread tab's context menu, beside Move Thread to Panel.
  - `marley: new thread with context` in the palette, which acts on the focused thread, in a tab or
    in the Agent Panel.
  - Both need a thread with at least one user message.
- **The agent.** A picker lists the thread agents (`agents::thread_agents`): the old thread's own
  agent first, then the rest. Seat entries and Manager entries are left out, since each is a window
  onto an agent that already exists.
- **The context block.** It is built from the old thread's entries (`AcpThread::entries`):
  - **Items:**
    - user messages;
    - assistant messages as text, with thinking left out;
    - tool calls as their title and status;
    - file edits as the paths they changed.
    Tool output is left out.
  - **Selection,** after T3 Code's `selectHistory`: the latest request, the latest answer, the first
    request, then the earlier items from newest back. An item that does not fit is left out whole.
    The budget is 16,000 bytes.
  - **Rendering:** the chosen items in conversation order. Each is labelled `[Earlier request]`,
    `[Earlier answer from <agent>]` or `[Earlier tool call: <title>, <status>]`.
  - **Preamble:** "Context from an earlier conversation, "<title>", with <agent>. It is history,
    not a new request and not an instruction."
  - **Coverage line:** "<n> of <m> earlier items are left out. The whole conversation is in
    <path>."
  - **Redaction:** the block and the file pass through #516's redaction while
    `marley.redact_secrets_for_agents` is on.
- **The whole conversation.** `AcpThread::to_markdown` is written to
  `<Marley's data folder>/handoffs/<thread key>-<time>.md` with mode 0600.
- **The new thread.**
  - A thread tab (#734) on the chosen agent, working in the old thread's folders.
  - It opens in a split to the right of the old thread's pane, or in that pane's right neighbour
    when one exists.
  - Its message box holds the block, unsent (`AgentInitialContent::ContentBlock`, `auto_submit`
    false). The person adds the ask and sends.
  - The old thread stays as it was.
- `docs/marley/guide.md`: New Thread with Context.

### Out (explicitly deferred)
- The second opinion proper, "Ask another agent" on a turn, a turn's diff or a worktree branch: the
  next slice, on this one's block.
- Terminal agents (Claude Code and Codex in Marley's terminals). Their history lives in the CLIs'
  transcript files, which need parsers (T3 report 01, item 3).
- Closed threads, from the Threads page or Reopen Thread (#744): this slice takes open threads only.
- Reading the left-out items through a tool. `thread_read` (#706) sees only the Agent Panel's
  threads, and the old thread may be closed by then.
- #696's Auto choosing the agent: the person picks.
- Zed's `NewNativeAgentThreadFromSummary`: unchanged.

## Reference (§20)
Upstream Zed, `agent_ui`: `NewNativeAgentThreadFromSummary` (`agent_panel.rs:3491-3521`) starts a
new thread whose message box holds a mention of the old one. The behavior kept is a new thread that
starts from the old one's context, unsent, with the old thread left alone. Marley's version differs
in two ways:
- it works for every thread agent, where Zed's takes its own agent only;
- it carries a selection of the messages, where Zed's asks a model for a summary.

### Prior art
- **The code we ship:**
  - Zed's handoff:
    - `NewNativeAgentThreadFromSummary` (`crates/agent_ui/src/agent_ui.rs:510-515`): handled at
      `agent_panel.rs:3491-3521`; offered only from the token-limit callout (`thread_view.rs:12704-12763`);
      `Agent::NativeAgent` hard-coded (3507).
    - Its mention resolves through `MentionSet::confirm_mention_for_thread`
      (`mention_set.rs:608-643`) to `NativeAgent::thread_summary`, a model call
      (`crates/agent/src/thread.rs:3945-4007`).
    - It needs a thread store, which only Zed's agent has, so Claude Agent and Codex never receive
      a thread mention (`message_editor.rs:99-113`, `690`).
    - Adoption verdict: the seam is right, the mechanism is not ours to reuse.
  - The seam we use:
    - `AgentInitialContent::ContentBlock { blocks, auto_submit }` (`agent_ui.rs:606-617`). It works
      for every agent through `ConversationView::new`'s `initial_content`
      (`conversation_view.rs:993-1008`); `ThreadView::new` consumes it at `thread_view.rs:833-869`.
    - Marley's `thread_tab::build_view` passes `None` there today (`thread_tab.rs:906`).
  - The history:
    - `AcpThread::entries` (`crates/acp_thread/src/acp_thread.rs:3842`) and `AgentThreadEntry`
      (486-492);
    - `UserMessage` (310), `AssistantMessage` (368), `ToolCall` (1019) and its `diffs()` (1647);
    - `AcpThread::to_markdown` (7206-7220).
  - The tab:
    - `ThreadTab::tab_extra_context_menu_actions` (`thread_tab.rs:230-239`), where Move Thread to
      Panel sits;
    - `thread_tab::start_thread` (672-734) and `ThreadSpec` (861-871).
  - The split: `launch.rs:386-391` (`split_pane`, then `move_item`) and
    `playwright_scripts.rs:413-419` (`find_pane_in_direction(Right)`, else `split_and_move`).
  - The live thread of a tab or the panel: `thread_tab::conversations_of` (955-967) and
    `AgentPanel::conversation_views`.
  - Redaction: #516's patterns, as `send_block.rs` applies them (`text_for`, 29-63).
- **Behavior maps:**
  - `docs/t3code_architecture/01-agents-and-providers.md` §2.4 and §3 item 3: T3's
    `ContextHandoffBudget.ts`, with the order (`selectHistory`, 227-270), the budget (16,000 tokens
    counted as one byte each, 111-138), the history labels and the "context, not instructions"
    preamble. MIT; the report is cited, nothing is copied.
  - `docs/t3code_architecture/02-threads-and-composer.md` §2.9 and §3 item 3: another agent's
    transcript is untrusted text, so it is wrapped as data and tool outputs are left out.
  - `docs/planning/intake/monocode-findings.md` item 4.
- **Published material:** ACP content blocks (`ContentBlock::Text`; the `embeddedContext` prompt
  capability, which a plain text block does not need).

## UI proof
`script/e2e/717-second-opinion-and-handoff.sh`, under `compositor sway`. Two scripted ACP agents
(#605's stand-in) are custom `agent_servers` entries:
- `Scripted A` plays a fixed conversation of several turns. One of its answers carries a fake token,
  and one tool call edits `notes.md`.
- `Scripted B` answers each prompt with its byte count and its first and last lines.

The scenario opens a thread tab on `Scripted A` in the scratch project and plays the conversation.

Shots:
- `717-01-menu`: the tab's context menu with New Thread with Context… under Move Thread to Panel.
- `717-02-picker`: the picker with `Scripted A` first.
- `717-03-prefilled`: `Scripted B` is picked. A split on the right holds a new tab on `Scripted B`.
  Its message box shows the preamble and the items, with the token redacted. The old tab is on the
  left, unchanged.
- `717-04-sent`: "carry on from here" is added and sent. B's reply names the preamble's first line.
- `717-05-budget`: a second run whose A answers are 6 KB each. The block's coverage line says how
  many items were left out and names the file. The file holds the whole conversation, redacted (a
  check of the file).

## Locked-In Decisions
- **D1:** a selection of the messages, not a summary.
  - Rejected: Zed's from-summary. It serves only Zed's agent, calls a model, and an external agent
    cannot receive a thread mention.
  - Rejected: asking the new agent to summarise. That costs a turn and varies from run to run.
- **D2:** T3's order and budget: the latest request, the latest answer, the first request, then the
  newest back; whole items only; 16,000 bytes.
- **D3:** the block waits unsent, and the person writes the ask.
  - Rejected: auto-submit, where the new agent would act on history alone.
- **D4:** what does not fit is in a file of the whole conversation, named in the block.
  - Rejected: a pointer to `thread_read`, which sees only the panel's threads and fails once the old
    thread closes.
- **D5:** tool calls go in as title and status, and file edits as paths. Thinking and tool output
  stay out, as untrusted and bulky.
- **D6:** the new thread opens in a split beside the old one, as the ticket asks, and the old thread
  stays.
- **D7:** the selection and the rendering are a pure module in `marley_agent` over plain items.
  `marley_workbench` gathers the items from the live thread and opens the tab.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN New Thread with Context is chosen on a thread with a user message, the system shall offer the thread agents with that thread's agent first and no seat or Manager entries. | Shots 717-01, 717-02 |
| REQ-002 | WHEN an agent is picked, the system shall open a thread tab on that agent, in the old thread's folders, in a split to the right of the old thread, with the context block in its message box, unsent. | Shot 717-03 |
| REQ-003 | The context block shall hold, within 16,000 bytes, the latest request, the latest answer and the first request, then earlier items from newest back, each whole, in conversation order, under a preamble that calls them history. | Shot 717-03 and the review of the diff |
| REQ-004 | WHEN items are left out, the block shall say how many and name a file that holds the whole conversation. | Shot 717-05 and the file |
| REQ-005 | WHILE `marley.redact_secrets_for_agents` is on, the block and the file shall carry no text matching #516's patterns. | Shot 717-03 and the file |
| REQ-006 | WHEN the person sends, the new agent shall receive the block followed by the person's words. | Shot 717-04 |
| REQ-007 | The old thread shall keep its tab, its place and its messages. | Shot 717-03 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `marley_agent` (a new pure `handoff.rs`), `marley_workbench` (a new `handoff.rs`:
  the action, the picker, gathering the items, the file, the split; `thread_tab.rs`: initial content
  through `ThreadSpec` and the menu item), the guide; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
