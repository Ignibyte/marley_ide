---
pipeline_id: 6456bd22-5889-4428-a396-504241de73fb
ticket: docs/planning/tickets/open/TICKET-914-thread-answers-as-the-board.md
status: Phase 4 — Complete PASS
title: An agent's answers as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/A-core.md, docs/marley/design-review-2026-10-23/C-threads.md]
---

## Title
In a thread Marley hosts, each answer opens with a header: the agent's tile, its name, its model
and effort, and, on the newest answer while the agent works, a live state at the right ("Working
· 12 min"). EDIT and RUN labels are filled in their colours; a RUN card carries a result chip and
an EDIT card its line counts. The reply chips are pills, the send button a filled accent square,
the text-width toggle a text button, and the user's message plain text in its bubble.

## Scope
### In
- `crates/agent_ui/src/conversation_view/thread_view.rs` (Zed crate, under the existing
  `MarleyThreadLook` and `MarleyThreadHost` hooks):
  - `render_entry`: the first top-level entry after a user message (an assistant message or a
    tool call) gets a header row above it, drawn by a new `MarleyThreadLook::answer_header`
    function from an `AnswerFacts` value: the agent's id and name, the model name, the effort
    when the thread reports one, and the state (`Working` with the turn's elapsed time from
    `turn_fields.turn_started_at`, `NeedsYou` while a tool call waits for permission, or none).
    Only the newest answer gets a state.
  - `marley_card_label`: a filled label; EDIT on the success colour's background, RUN on the
    warning colour's, the ink their status colour.
  - The terminal card's header: after the command, a result chip once the command ends ("passed"
    on success for exit 0, "exit N" on error), the elapsed time Zed already shows staying.
  - The edit card's header: "+N −M" from the call's diff (`DiffStats::single_file`), in the
    added and removed colours.
  - `render_marley_chips`: each chip a pill (full radius, `element_background` fill, a border).
  - `render_send_button`: while Marley hosts threads, a 34 px square in the accent fill with an
    up arrow in the background's ink; disabled, the same square in `element_background` with a
    muted arrow. Stop and the queue's icon keep Zed's look.
  - `render_marley_text_width`: a text button reading the current width, "Full width" or
    "Readable column"; a click flips it as today.
  - A user message's bubble: its editor drawn without its own darker field and in the UI font
    (the entry's message editor only; the composer is TICKET-915's).
- `crates/agent_ui/src/message_editor.rs` (Zed crate): a flag the entry's editor is made with, so
  its text style takes the UI font while `MarleyThreadHost` is set.
- `search_looks.rs` (Marley), where `MarleyThreadLook` is set today, names the new field; the
  drawing lives in a new `answer_header.rs` (Marley): `answer_header`, which draws a 32 px tile with the agent's glyph (CC for Claude Code, CX for
  Codex, M for Marley's own agent, Z for Zed's agent, else the name's first letter), the name in
  semibold, the model and effort muted, and the state at the right with a dot in the state colour.

### Out (explicitly deferred)
- Reply chips that follow the turn ("Run the visual check", "Show the diff"): the fixed three
  stay; making them fit the answer is a follow-up ticket (the review's M).
- The plan inside the answer's flow: #864's Out (Zed's plan bar and the details panel keep it).
- A state on earlier answers ("Done · 4 min"): the thread keeps only the last turn's duration.
- The ASKS label on a waiting tool call, the Allow / Always / Deny wording and the width button's
  ⅓ ½ ⅔ wording (ThreadStates): TICKET-927.
- The thread's bottom row (model, mode, what it does, context, files): TICKET-907.
- The composer's context chips, Plan / Act and the model button, and the details panel:
  TICKET-915.
- Geist fonts (the review's "For Chad" item): the UI font is Marley's.

## Reference (§20)
Upstream Zed (`agent_ui`'s thread view): the conversation's entries, tool call cards, the turn
timer and the send, stop and queue buttons are Zed's and keep their behaviour. Marley changes
only how an answer opens and how labels, chips and buttons are drawn, behind its hooks.

### Prior art
- `thread_view.rs:570-596`: `MarleyThreadLook { extra_chip }` and `marley_card_label`, muted text
  in a 1 px frame (#864).
- `thread_view.rs:6906-7060` `render_entry`; `:6923` `marley_bubble`; `:7008` the 75 % bubble on
  the accent at 8 %; `:7136` the assistant message arm.
- `thread_view.rs:1480-1495` `initialize_turn`: `turn_started_at` and a one-second timer that
  notifies the view while a turn runs, so a live elapsed time needs no ticker of its own.
- `thread_view.rs:8160-8190` `render_generating`: Zed's own elapsed label from the same field.
- `thread_view.rs:8837-8846` the RUN slot; `TerminalToolHeader::failed(exit_code)` already marks a
  failure; `:11007` the EDIT label; `:3828` `DiffStats::single_file` used for a file's counts.
- `thread_view.rs:7770-7800` `render_marley_chips`; `:6126` `render_send_button`; `:6428`
  `render_marley_text_width`.
- `thread_view.rs:1282` `current_model_id`, `:12497` `current_model_name`, `:5841`
  `render_thinking_control` and `thread.thinking_effort()`: where the model and effort live.
- `message_editor.rs:2057-2080`: the editor's text style uses `agent_buffer_font_family`.
- `home_page.rs:925` `cli_glyph` (#903): the agents' letters.
- Ledger: F-claude-865-a-hook-called-from-a-render-read-its-own-view-001 (the hook gets facts,
  never the view).

## UI proof
Scenario `script/e2e/914-thread-answers-as-the-board.sh` (sway): #864's set-up, Marley's own agent
on the scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`) in a scratch crate. The scripted agent
reports a model, and for the prompt it waits five seconds, then writes an edit, runs a three-line
command through the client's terminal and answers.
- `914-01-working`: during the wait: the answer's header with the tile, name, model and "Working ·
  N s" (REQ-001).
- `914-02-answer`: the finished answer: the header with no state, the filled EDIT label with
  "+N −M", the filled RUN label with "passed" and its time, the pill chips, the bubble's text in
  the UI font, a word typed so the send button is filled, and "Full width" (REQ-001 to REQ-005).
- `914-03-column`: "Full width" clicked: the button reads "Readable column" and the text narrows
  (REQ-005).
- `914-04-day`: Marley Day, the finished answer (REQ-001 to REQ-005).

## Locked-In Decisions
- D1 — The header is drawn by Marley from facts Zed's view hands it (F-claude-865); the view never
  passes itself.
- D2 — Only the newest answer has a live state; finished answers carry the tile, name and model
  only.
- D3 — The result chip reads the exit status only ("passed", "exit N"); parsing a test count
  ("17 passed") from output is not done.
- D4 — The bubble's text takes the UI font and no inner field; editing a past message keeps Zed's
  editor, focus border and buttons.
- D5 — Every change shows only while Marley hosts threads (`MarleyThreadHost`, `MarleyThreadLook`);
  Zed's layout keeps Zed's thread.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Each answer shall open with the agent's tile, name, model and effort, and WHILE the agent works the newest answer's header shall show a live state with the elapsed time, in Marley Night and Marley Day. | `914-01-working`, `914-02-answer`, `914-04-day` |
| REQ-002 | EDIT and RUN labels shall be filled in their colours; an edit card's header shall show its added and removed line counts, and a finished command's header a result chip. | `914-02-answer`, `914-04-day` |
| REQ-003 | The reply chips shall be pills. | `914-02-answer` |
| REQ-004 | WHILE the message box has text, the send button shall be a filled accent square with an up arrow. | `914-02-answer` |
| REQ-005 | The text-width toggle shall be a text button naming the current width, and a click shall flip it. | `914-02-answer`, `914-03-column` |
| REQ-006 | A user's message shall read as text in the UI font inside its bubble, with no inner field. | `914-02-answer` |
| REQ-007 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the header hook, labels, chips, send and width buttons, the bubble's text, the
  ledger rows; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
