---
pipeline_id: a4ac44d3-d939-42ea-8d8c-1caa9bfe9e4a
ticket: docs/planning/tickets/open/TICKET-825-agent-thread-with-panels.md
status: Phase 4 — Complete PASS
title: The agent thread with a details panel, bubbles and a composer card
type: feature
slice: The Marley look, W2 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
An agent thread in a Marley tab reads like a conversation. The user's messages are bubbles on the
right, the message box is a rounded card, and a details panel on the right of the tab shows the
thread's agent and model, how much of its context it has used, and the files it changed. A button
in the panel's header and `marley: toggle thread details` show or hide the panel. Today the user's
message is a full-width box like the agent's, the message box is a full-width strip, and none of
the thread's facts show beside it.

## Scope
### In
- The details panel, in `ThreadTab` (Marley): a 280 px column to the right of the conversation,
  drawn while `marley.thread_details` is on (default on). It holds the agent's icon and name, the
  model, the context use (`used of max` with a thin bar, when the agent reports it), and the files
  changed (path, `+added −removed`), each opening the file on a click. It follows the thread's
  events and its action log.
- `marley.thread_details: Option<bool>` (its own settings struct in `thread_tab.rs`, as
  #831's `ProjectLogos`), `default.json`, and the action `marley::ToggleThreadDetails`, also on the
  panel's header button and a small button at the tab's top right while the panel is hidden.
- The user's messages as bubbles, a `// Marley:` hunk in `thread_view.rs`: while Marley hosts
  threads (`MarleyThreadHost` set), a top-level user message is aligned right, at most 85 % of the
  width, rounded large, on the accent at 8 %. Subagent (indented) messages are left as they are.
- The composer as a card, a `// Marley:` hunk in the same file: while Marley hosts threads, the
  message editor's strip gets side and bottom margins, a full border and rounded corners in place
  of the top border.

### Out (explicitly deferred)
- The plan checklist, EDIT/RUN labels, reply chips, the terminal peek and the checks:
  TICKET-864.
- The Agent Panel's own dock (the panel goes in Marley's thread tabs only; the bubbles and card
  show wherever Marley hosts, which is the whole app).

## Reference (§20)
Upstream Zed (`agent_ui`, `conversation_view/thread_view.rs`): the thread's entries, the user
message box and the message editor are Zed's; Marley changes their containers only, behind its
existing `MarleyThreadHost` global, and keeps every behavior (editing a sent message, checkpoints,
the toggles). The details panel reads Zed's `AcpThread` (`token_usage`, `action_log`) and
`ThreadView`'s agent fields. The look is the design canvas's Main and MainDay boards.

### Prior art
- The code we ship: `ThreadTab` renders the `ConversationView` alone (`thread_tab.rs:607`), so a
  side column needs no agent_ui change; `ConversationView::root_thread_view`, `ThreadView {
  thread, agent_display_name, agent_icon, .. }`, `current_model_id`, `AcpThread::token_usage`
  (`TokenUsage { used_tokens, max_tokens }`), `humanize_token_count`, `AcpThread::action_log`
  and `ActionLog::changed_buffers` / `diff_stats`, and `render_edited_files` as Zed's own drawing
  of the same list. `MarleyThreadHost` (#734, #798) is the global agent_ui already reads for
  Marley; #822's `render_marley_text_width` is the pattern for a Marley-only look in the view.
- Ledger: F-claude-798-a / PR-claude-798 (read no pane bounds in render), PR-claude-687 (a scenario
  types into a thread only after checking the stand-in runs it), PR-claude-831 (no workspace read
  in a view's `new` when a hook makes it). #822's scenario (a scripted ACP agent as Marley's own).
  The brain had nothing on this seam.

## UI proof
Scenario `script/e2e/825-agent-thread-with-panels.sh` (sway), #822's set-up: Marley's own agent on
a scripted ACP agent that answers each prompt and, for one prompt, writes `notes.md` through the
client's `fs/write_text_file`.
- `825-01-night`: a thread tab after two prompts: the user's messages as right bubbles, the
  agent's answers, the message box as a rounded card, and the details panel with the agent, its
  model and `notes.md` under files changed (REQ-001, REQ-002, REQ-003, REQ-004).
- `825-02-hidden`: the panel's button clicked: the conversation takes the tab's width, the setting
  printed `false` (REQ-005).
- `825-03-day`: shown again, Marley Day (REQ-001 to REQ-004).

## Locked-In Decisions
- D1 — The panel lives in `ThreadTab`, beside the conversation, with no agent_ui change.
- D2 — The bubbles and the card are drawn only while `MarleyThreadHost` is set, so Zed's own build
  of agent_ui is unchanged.
- D3 — Files changed are the action log's changed buffers, the same list as Zed's edited-files
  bar: a file kept or rejected leaves it.
- D4 — Context use shows only when the agent reports token usage; otherwise the row says
  "not reported".

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a thread shows in a Marley tab with details on, a panel at its right shall show the thread's agent and model, in Marley Night and Marley Day. | `825-01-night`, `825-03-day` |
| REQ-002 | The panel shall list the files the thread changed with their added and removed lines, and its context use when reported. | `825-01-night` |
| REQ-003 | WHILE Marley hosts threads, the user's messages shall draw as bubbles on the right. | `825-01-night`, `825-03-day` |
| REQ-004 | WHILE Marley hosts threads, the message box shall draw as a rounded card. | `825-01-night`, `825-03-day` |
| REQ-005 | WHEN the user clicks the panel's button or runs `marley: toggle thread details`, the panel shall hide or show and `marley.thread_details` shall record it. | `825-02-hidden`; the printed setting |
| REQ-006 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the panel and its setting and action; the two agent_ui hunks with their rows; a
  review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
