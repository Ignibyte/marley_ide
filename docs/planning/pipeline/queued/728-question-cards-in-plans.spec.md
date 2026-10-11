---
pipeline_id: dd11a0a0-c7b7-4409-b15e-2e92827f02af
ticket: docs/planning/tickets/open/TICKET-728-question-cards-in-plans.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Question cards in plans
type: feature
slice: the review batch after the finish (docs/planning/intake/plannotator-findings.md item 3; docs/planning/design-notes/finish-line-2026-10-11.md row 10); after #726 slice 1; the syntax reaches agents through a rustal-harness skill (D5)
references:
  - docs/planning/pipeline/queued/726-plan-review-in-a-tab.spec.md
  - docs/planning/pipeline/queued/727-plan-versions-and-diff.spec.md
---

## Title
A plan's open questions, written as `:::question` blocks, are drawn in the plan tab as cards with
choices, Accept recommended and Skip. The answers go back first in the feedback, so the agent
revises the plan with them. Chad, 2026-10-11, on the finish-line list: "10.) yes".

## Scope
### In
- **The syntax** (plannotator's, so a plan written for it reads the same here): a block opens with
  `:::question` (one choice), `:::question-multi` (any number) or `:::question-text` (free text; a
  block with no choices is free text too) and closes with `:::`. Its first line is the question;
  other lines before the choices are context; choices are `- [ ] label`, a reason after ` — ` or
  ` - `; `Recommended: <label>` names the agent's pick; `- [x]` marks a choice already settled.
  A block with no closing `:::` stays plain text.
- **The cards**, in the plan where the blocks stand: "Question N of M", the state (Open, Answered,
  Skipped, Settled), the question and its context as Markdown, the choices (one or several), an
  Other… field for one's own answer, a text field for `question-text`, **Accept recommended**
  while the question is open and has a pick, and **Skip**. The header counts "K of M answered".
- **Sending:** while any answer or skip is unsent, the main button reads **Send Answers** and
  Approve is off (an approval carries no text, so the answers would be lost). Send Answers answers
  the call with deny and a reason that starts with the answers, then any notes (#726's form).
- **The answers' form:** `Answers to your questions`, `K of M questions answered.`, then per
  question `### Question N: <question>` and `Answer: <choice>` (with `(your recommendation)` when
  accepted), a bullet per choice for several, the text as a quote, `Other: <words>`, or `Skipped`;
  then `Unanswered:` and the questions left open. With answers only, the lead line says: "The user
  answered your questions. Revise the plan with these answers, then call ExitPlanMode again."
- Notes on the plan's text work as in #726, on the text around the cards.

### Out (explicitly deferred)
- A note attached to an answer, undo of an answer, and a jump to the next open question.
- Images in a question's context.
- Teaching the syntax to Codex and Zed's agent (#725's routes); this ticket asks for the shared
  plugin's skill only (D5).
- Question cards in Markdown files and replies (#729).

## Reference (§20)
N/A — Marley-specific: the plan tab is Marley's (#726). The syntax and the cards follow plannotator's
"Answer Your Agent's Questions Asynchronously" (published docs,
`docs.plannotator.ai/open-source/workflows/questions`): the three block kinds, task-list choices,
`Recommended:`, settled choices, the card's states, Accept recommended, Skip, Send answers in place
of Approve, and the answers first under "Answers to your questions". The syntax is a published
format; no plannotator code is taken.

### Prior art
- **The code we ship:**
  - Zed's `markdown` parser has no directive or container blocks (`crates/markdown/src/parser.rs:13-23`,
    pulldown-cmark 0.13.4; `all_options_considered` at 1052-1061 guards the list), so a `:::` block
    parses as a paragraph. `CodeBlockRenderer::Custom` exists but draws nothing in this tree
    (`crates/markdown/src/markdown.rs:548-560, 2919`), and there is no hook for a custom block.
    Marley therefore splits the plan's source itself and draws each run of Markdown with its own
    `Markdown` entity, the cards between them.
  - #726's tab (`plan_review.rs`): the notes are source byte ranges of the plan; with several
    entities each run keeps its start offset, so notes stay in plan coordinates.
  - #726's feedback in `marley_agent` (`plan_feedback`), which the answers lead.
  - `ui` controls for the cards: `Checkbox`, `ToggleButton` groups, `Button`, as the tasks tab uses
    them (`crates/marley_workbench/src/rusty/tasks_tab.rs:956-1042`).
  - Marley already reads `AskUserQuestion`'s options for the inbox (`claude_plugin/marley/hooks/event.py:81`,
    #570): a live question, not a plan's; it stays as it is.
- **Behavior maps:** Orca's and T3's question cards answer a live prompt
  (`docs/orca_architecture/01-agents-and-sessions.md`, `docs/t3code_architecture/01-agents-and-providers.md`),
  not questions inside a plan.
- **Published material:** plannotator's questions page (above); CommonMark (no `:::` blocks, which
  is why plannotator's syntax reads as plain text on GitHub).

## UI proof
`script/e2e/728-question-cards-in-plans.sh`, under `compositor sway`, on #726's set-up: the stand-in
`claude` sends a plan holding three questions: one choice with `Recommended:`, several choices,
and free text.

Shots:
- `728-01-cards`: the three cards in the plan with "Question N of 3", Open, Accept recommended on
  the first; the header "0 of 3 answered".
- `728-02-answered`: Accept recommended on the first, two choices ticked on the second, Skip on the
  third: their states, "2 of 3 answered", Send Answers in Approve's place.
- `728-03-sent`: Send Answers: the terminal shows the reason starting `Answers to your questions`,
  `Answer: … (your recommendation)`, the two choices as bullets, and `Skipped`.

## Locked-In Decisions
- **D1:** plannotator's syntax as published, so agents and plans written for it need no change.
  Rejected: a fenced ```` ```question ```` block (a second syntax for the same thing).
- **D2:** Marley splits the plan's source into Markdown runs and question blocks (a pure function in
  `marley_agent`), and the tab draws each run with its own `Markdown` entity and each block as a
  card. Rejected: a custom block in Zed's `markdown` crate (a parser option and a render hook in a
  Zed crate for one Marley use); cards in a separate panel (the questions lose their place in the
  plan).
- **D3:** answers go back as the deny's reason, first; while any is unsent, Approve is off.
- **D4:** the answers' form is plannotator's, in Marley's words, built in `marley_agent`.
- **D5:** agents learn the syntax from the shared Claude Code plugin's skills, which rustal-harness
  owns: a **harness request** for a skill, or a section of one, that tells an agent writing a plan
  to put each open question in a `:::question` block of this syntax. Until it ships, a plan's
  author can be told in its own instructions; the cards work either way.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a plan holds a closed `:::question`, `:::question-multi` or `:::question-text` block, the plan tab shall draw it as a card in its place, with its question, context and choices. | Shot 728-01 |
| REQ-002 | WHERE a question names a recommended choice and is open, the card shall offer Accept recommended, which selects that choice. | Shots 728-01, 728-02 |
| REQ-003 | WHEN the user selects choices, writes an answer, or skips, the card shall show Answered or Skipped and the header shall count the answered questions. | Shot 728-02 |
| REQ-004 | WHILE an answer or a skip is unsent, the tab shall offer Send Answers in Approve's place. | Shot 728-02 |
| REQ-005 | WHEN the user sends, the system shall answer the waiting call with deny and a reason that begins with the answers in the stated form, followed by any notes. | Shot 728-03 |
| REQ-006 | IF a block has no closing `:::`, THEN the tab shall draw it as plain Markdown. | Review of the diff |
| REQ-007 | WHERE a choice is marked `- [x]`, the card shall show the question Settled with that choice. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes; the harness request (D5) sent
  through Chad.
- **P2 Code** — the parser and the answers' form in `marley_agent`; the runs, the cards and the
  send in `plan_review.rs`; the guide (the syntax, for anyone writing a plan by hand); a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
