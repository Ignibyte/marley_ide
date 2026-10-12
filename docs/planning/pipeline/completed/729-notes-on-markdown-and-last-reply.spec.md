---
pipeline_id: 2846b77e-40ff-4d5b-99f7-c5eb111782a3
ticket: docs/planning/tickets/open/TICKET-729-notes-on-markdown-and-last-reply.md
status: Phase 4 — Complete PASS
title: Notes on any Markdown file or the agent's last reply
type: feature
slice: the review batch after the finish (docs/planning/intake/plannotator-findings.md item 4; docs/planning/design-notes/finish-line-2026-10-11.md row 11); after #726 slice 1 and #732
references:
  - docs/planning/pipeline/completed/522-review-notes-to-the-agent.spec.md
  - docs/planning/pipeline/queued/726-plan-review-in-a-tab.spec.md
  - docs/planning/pipeline/queued/732-a-feedback-template-for-review-notes.spec.md
  - docs/planning/pipeline/completed/706-agent-panel-threads.spec.md
---

## Title
The plan tab's review (#726: a document drawn as Markdown, notes on selected passages and an
overall note) opens on any Markdown file and on an agent's last reply. A file's notes go to an agent
through #522's picker; a reply's notes go back to the agent that wrote it, in its terminal or its
thread. Chad, 2026-10-11, on the finish-line list: "11.) yes".

## Scope
### In
- **The tab** is #726's, with what it reviews widened from a plan to a file or a reply. For a file
  or a reply it offers **Send Notes** where a plan offers Approve and Send Feedback.
- **A Markdown file:** `marley: review markdown` from the palette, with a Markdown file's editor or
  Zed's Markdown preview active, opens the file's current text in the tab ("Review · <file
  name>"), in the pane of the editor or preview it came from. Each note knows its passage's lines
  in the file.
- **A file's notes:** Send Notes opens #522's picker (the agents whose folder holds the file, each
  with its readiness, and Copy notes). Each note goes as `File:`, `Line:` or `Lines:`,
  `Quote: "<passage>"` and `User comment: "<words>"`; the overall note as `File:` and
  `User comment:`; all inside #732's template.
- **A reply in a Zed agent thread:** `marley: review last reply` from the palette with the thread
  active (the Agent Panel or a thread tab), or Review Last Reply in the thread's rail row menu,
  opens the last turn's assistant text (thoughts left out).
- **A reply of Claude Code in a terminal:** the same command with the agent's terminal active, or
  Review Last Reply in its rail row menu, opens the last assistant text of its session, read from
  the transcript Claude Code names in its hook events. Local terminals only.
- **A reply's notes** go only to the agent that wrote it, led by "Notes on your last reply:", each
  as `Quote:` and `User comment:`, then the overall note, inside #732's template:
  - to a terminal by #522's delivery, and only while it is ready (idle); otherwise nothing is sent
    and a toast says why;
  - to a thread by the thread's own send (AD-706): refused while the user has a draft there; a
    running thread queues it.
- Sent notes stay in the tab marked Sent; Send Notes counts only unsent ones.

### Out (explicitly deferred)
- Codex's and other CLIs' replies in a terminal (Marley keeps no reply text for them), and remote
  terminals' transcripts.
- Question cards (#728) in files and replies.
- Saving a file's notes, or editing the file from the tab.
- A Review button in Zed's Markdown preview or the editor's tab menu (Zed crates); the palette and
  Marley's rail menus are the ways in.

## Reference (§20)
Upstream Zed's Markdown preview (`markdown_preview`) and agent thread view (`agent_ui`) are kept as
they are; the review opens beside them in Marley's tab. The behavior follows plannotator's document
and last-message annotation (published docs, `docs.plannotator.ai/open-source/workflows/documents`
and the agents pages: `/plannotator-annotate <file>` and `/plannotator-last`): the same review a
plan gets, on a file or on the agent's last message, whose feedback goes back to that agent. The
delivery is #522's, which follows Orca report 02 §2.9. No plannotator code is taken.

### Prior art
- **The code we ship:**
  - #726's tab (`plan_review.rs`): the drawn Markdown, notes on source ranges, highlights; this
    ticket widens its subject.
  - #522's picker and delivery (`crates/marley_workbench/src/review_notes.rs`: `open_picker` 38,
    `holds_every_file` 85, `readiness` 93, `deliver` 108) and `send_selection::{Target,
    TargetPicker, Row, Pick}`; `marley_agent::review_prompt` (`marley_agent.rs:371`).
  - Zed's Markdown preview (`crates/markdown_preview/src/markdown_preview_view.rs`):
    `resolve_active_item_as_markdown_editor` (344), `is_markdown_file` (536), the preview's source
    editor (`active_editor`, 87).
  - A thread's last reply: `AcpThread::entries` (`crates/acp_thread/src/acp_thread.rs:3842`),
    `AgentThreadEntry::AssistantMessage` (486), `AssistantMessage.chunks` (361),
    `AssistantMessageChunk::Message { block, .. }` (380), `MessageContent::to_markdown` (2000).
    `ThreadView::get_agent_message_content` (`crates/agent_ui/src/conversation_view/thread_view.rs:8325`)
    gathers a turn's assistant text but is private; Marley gathers the same through the public
    entries. The thread: `ConversationView::root_thread_view` (`conversation_view.rs:861`), the
    `thread` field (`thread_view.rs:573`); Marley's `thread_tab::conversations_of`
    (`thread_tab.rs:955`).
  - A thread's send: `thread_tools::post` (`crates/marley_workbench/src/thread_tools.rs:265`): the
    message editor's draft check, `insert_text`, `ThreadView::send`.
  - A terminal's reply: Claude Code's hook frames carry `transcript_path`
    (`claude_plugin/marley/hooks/event.py:37`), kept on the seat as a label for the lead agent
    (`crates/marley_agent/src/claude_events.rs:396`), reached through `AgentEvents::seat`
    (`crates/marley_workbench/src/agent_events.rs:70`). The Stop frame's `last_assistant_message`
    is cut to 300 characters (`event.py:18, 119`), too short; nothing reads a transcript yet.
  - Rail menus: the terminal row's `terminal_context_menu` (`crates/marley_workbench/src/rail.rs:5595`),
    the thread row's menu (5442).
- **Behavior maps:** Orca report 02 §2.9: notes carry `source` (diff or markdown) and go to a
  running agent; `docs/orca_architecture/02-worktrees-and-review.md`.
- **Published material:** plannotator's documents and last-message pages (above); Claude Code's
  transcript JSONL (one entry a line; assistant entries carry `message.content` with `text`
  blocks).

## UI proof
`script/e2e/729-notes-on-markdown-and-last-reply.sh`, under `compositor sway`: a scratch repository
with a `README.md`; in its terminal #522's stand-in `claude` (idle through Marley's plugin's
`SessionStart`, echoing each line it reads), which also sends a `Stop` frame whose
`transcript_path` names a scratch transcript with one assistant reply; and #739's scripted ACP agent
in a thread tab, which answers each prompt with "Noted: <prompt>".

Shots:
- `729-01-file`: `README.md` open, `marley: review markdown`: the tab with the file drawn and Send
  Notes.
- `729-02-file-sent`: a note on a passage; Send Notes; Enter on the ready agent: the stand-in's echo
  shows `File: README.md`, `Lines:`, `Quote:` and `User comment:` inside the template.
- `729-03-terminal-reply`: Review Last Reply from the agent terminal's rail menu: the tab shows the
  transcript's reply.
- `729-04-terminal-sent`: a note; Send Notes: the stand-in's echo shows "Notes on your last reply:"
  and the note; the note marked Sent.
- `729-05-thread-reply`: in the thread tab after one prompt, `marley: review last reply`: the tab
  shows "Noted: …".
- `729-06-thread-sent`: a note; Send Notes: the thread shows the notes as the user's message and
  the scripted agent's answer.

## Locked-In Decisions
- **D1:** one review tab for plans, files and replies (#726's), so notes look and work the same.
  Rejected: notes inside Zed's Markdown preview (a Zed crate's view would draw Marley's notes, and
  the reply has no preview); review notes in a Markdown file's editor gutter (line notes on the raw
  text, not on what the reader sees, and nothing for a reply).
- **D2:** a file's notes go through #522's picker, as a diff's do; a reply's go only to its own
  agent, since only it holds the context.
- **D3:** a terminal's reply is read from Claude Code's transcript at the path its hook events name,
  the assistant text after the last user prompt. Rejected: the Stop frame's message (cut to 300
  characters); scraping the terminal's screen (wrapped, styled, partial).
- **D4:** a thread's notes go in through the thread's own send, with its draft check (AD-706).
- **D5:** the notes carry the quoted passage, since a reply has no lines and a file's lines move.
  `review_prompt`'s form gains an optional `Quote:` line; a diff note keeps its current form.
- **D6:** the ways in are the palette and Marley's rail menus; no Zed view gains a button.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user runs `marley: review markdown` with a Markdown file's editor or preview active, the system shall open the file's text in the review tab with Send Notes. | Shot 729-01 |
| REQ-002 | WHEN the user sends a file's notes and picks a ready agent, the system shall paste them with `File:`, the lines, `Quote:` and `User comment:` inside the template, press Enter and show the terminal. | Shot 729-02 |
| REQ-003 | WHEN the user asks to review the last reply of Claude Code in a local terminal, the system shall open that reply's text, read from its session's transcript. | Shot 729-03 |
| REQ-004 | WHEN the user sends a terminal reply's notes while that agent is ready, the system shall paste them into that terminal and mark them Sent. | Shot 729-04 |
| REQ-005 | WHEN the user asks to review the last reply of a Zed agent thread, the system shall open the last turn's assistant text without its thoughts. | Shot 729-05 |
| REQ-006 | WHEN the user sends a thread reply's notes, the system shall send them as the user's message in that thread. | Shot 729-06 |
| REQ-007 | IF the reply's agent is not ready, or its thread holds an unsent draft, THEN the system shall send nothing and say why. | Review of the diff |
| REQ-008 | IF the terminal's agent has no transcript Marley can read, THEN the system shall say so and open nothing. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `plan_review.rs`'s subject widened (file, thread reply, terminal reply) and Send
  Notes; a new `last_reply.rs` (the thread's text, the transcript's); the commands; the rail menus;
  `review_notes.rs` taking quoted notes and a fixed target; `thread_tools::post`'s send shared; the
  `Quote:` line in `marley_agent`; the guide; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
