# Question cards in plans — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-728-question-cards-in-plans.md
- **Pipeline spec:** 728-question-cards-in-plans.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** plannotator finding 3 (`docs/planning/intake/plannotator-findings.md`): "Render a
  plan's `:::question` blocks as choice cards with Accept Recommended and Skip, and send the
  answers first in the feedback. The syntax is taught to agents through the shared skill (the
  harness's side)." Chad, 2026-10-11, on the finish-line list
  (`docs/planning/design-notes/finish-line-2026-10-11.md`): "10.) yes". It waits under Deliberate
  with the rest of the review batch until after the finish.
- **Classification:** feature, medium. Marley crates only; one harness request.
- **Recall (§18.3):**
  - #726's spec and notes: the tab, the notes as source ranges, the deny's reason as the only text
    that reaches Claude Code, `plan_feedback`.
  - #727's spec: versions; a version's cards and answers belong to it.
  - #570 (question route) reads `AskUserQuestion`'s options; unrelated to a plan's questions.
  - No ledger entry on custom Markdown blocks.
- **Discovery:**
  - `crates/markdown/src/parser.rs`: `PARSE_OPTIONS` (13-23), `all_options_considered` (1052-1061).
  - `crates/markdown/src/markdown.rs`: `CodeBlockRenderer` (548-560), the `Custom` arm that draws
    nothing (2919), `code_block_action` (1818).
  - `crates/marley_workbench/src/rusty/tasks_tab.rs`: `render_task` (956-1042), a checkbox in a list
    item.
  - The shared plugin's skills: `crates/marley_workbench/claude_shared_plugin/skills/` (rustal-ste
    only today), carried from rustal-harness by digest (#724).
- **Decisions:** D1 to D5 in the spec.

### The harness request (D5)
For rustal-harness's shared Claude Code plugin (`skills/`):

> Add a skill, or a section of an existing one, that tells an agent writing a plan (in plan mode,
> before `ExitPlanMode`) to put each open question in a block of this form: `:::question` for one
> choice, `:::question-multi` for several, `:::question-text` for free text; the question on the
> first line; context lines next; choices as `- [ ] label — reason`; `Recommended: <label>` for its
> own pick; `- [x]` for a choice already settled; the block closed with `:::`. The answers come back
> in the feedback under "Answers to your questions".

Marley then carries the plugin at that revision (#724's way).

### Design
- **`crates/marley_agent/src/plan_questions.rs`** (new, Marley crate, pure): `Segment { Markdown(Range<usize>),
  Question(Question) }`; `Question { kind: One | Several | Text, text, context, choices:
  Vec<Choice { label, reason, settled }>, recommended: Option<String>, range }`; `split(source) ->
  Vec<Segment>`; `Answer { Choice(label, recommended), Choices(Vec<label>), Text(words),
  Other(words), Skipped }`; `answers_text(questions, answers) -> String` (the form in the spec).
- **`crates/marley_agent/src/marley_agent.rs`:** `plan_feedback` takes the answers' text to lead.
- **`crates/marley_workbench/src/plan_review.rs`** (Marley crate): a version holds its segments, one
  `Markdown` entity per run (its start offset kept for notes and highlights), the answers per
  question; `Render` draws runs and cards in order; the header's count; Send Answers in Approve's
  place while an answer is unsent; the send builds answers then notes.
- **The guide:** the syntax, under "Plan review".
- **File manifest:** `marley_agent/src/plan_questions.rs` (new), `marley_agent/src/marley_agent.rs`,
  `marley_workbench/src/plan_review.rs` (Marley crates); the guide; the scenario. No Zed path, no
  ledger row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | The stand-in sends the plan with three questions | 728-01-cards |
| 003, 004 | Clicks Accept recommended (Q1), two choices (Q2), Skip (Q3) | 728-02-answered |
| 005 | Clicks Send Answers | 728-03-sent: the terminal's printed reason |
| 006, 007 | (review of the diff: an unclosed block; a settled choice) | — |

### Risks
- A selection can no longer run across a card, since each run is its own entity; a note spans one
  run at most. The Plan phase checks that #726's highlight and note code takes a run's offset.
- An agent may write `:::question` inside a fenced code block as an example; `split` skips fenced
  code.
- The en dash and the hyphen both mark a reason; a label that holds ` - ` itself splits early. The
  card shows the label as parsed, so the reviewer sees it.

## Phase 1 — Plan (promoted 2026-10-12)
- **Promotion:** the queued pair moved to `active/`; the ticket is in progress; its Backlog row is
  gone. The brain (consultation 29e3f7d36acc4213807c6582501cdbfd) had nothing on the seam.
- **The harness request (D5)** for the skill that teaches agents the syntax is not sent:
  rustal-harness is paused at Chad's word. The cards work on any plan that uses the syntax.
- **Seams re-verified** directly at `afb434ea25`: the Plan tab is #726/#727's `PlanReview`
  (`plan_review.rs`), one `Markdown` for the whole plan, notes as plan byte ranges painted with
  `set_search_highlights`, `selected_range` read on `c` and when the menu opens; `plan_feedback`
  in `marley_agent`; Zed's Markdown has no container blocks, so `:::` parses as a paragraph.
- **Design:** `marley_agent::plan_parts(source)` (pure) splits the plan into Markdown runs (with
  their start offsets) and closed question blocks (kind, question, context, choices with reasons
  and settled marks, recommended); `plan_answers` writes the answers' form and leads any notes
  (#726's form), or carries its own lead line when there are none. The tab keeps a `Markdown` per
  run and a card per question; the selection and the highlights map between a run and the plan by
  the run's start. A card: "Question N of M", its state, the question, the context as Markdown,
  single or several choices, Other…, a text field for free text, Accept recommended, Skip. While an
  answer or a skip is unsent, Send Answers takes Approve's place and Send Feedback steps aside.

## Phase 2 — Code
- **Built:**
  - `crates/marley_agent/src/plan_questions.rs` (new): `plan_parts` (runs with their start, and
    closed blocks outside code fences), `question_of`, `choice_line`, `Answer`, `plan_answers`,
    `questions_feedback`, `clean`.
  - `crates/marley_workbench/src/plan_review.rs`: `Part` and `Card` in place of the one
    `Markdown`; `parts_of`; `selection` and `highlight` per run; `card_state`, `card_answer`,
    `answered`, `answers_pending`, `pick`, `accept_recommended`, `skip`, `send_answers`;
    `render_card`, `card_buttons`, `answer_buttons`, `choice_label`; `c` propagates unless the
    plan's area holds the focus.
  - The in-app guide's Plan review article and `docs/marley/guide.md`: the cards and the syntax.
- **Deviations from the design:** the names are `plan_parts`/`PlanPart` and
  `plan_answers`/`questions_feedback` (not `split`/`Segment`/`answers_text`), and `plan_feedback`
  is left as it was: `questions_feedback` puts the answers ahead of it. A question's text is
  `prompt` (clippy's `struct_field_names`). Accept recommended and Skip sit at the right of the
  card's header row, which lets four cards fit the tab at the scenario's size. A card's context is
  drawn in the UI's small type, muted (L-claude-728-a-markdown-styles-size-is-the-containers-001).
  The scenario asks four questions, not three: the fourth is the skipped one, and the third (free
  text) proves a `c` typed in a card's field stays in the field.
- **The review found:** the answer field did not redraw the tab
  (F-claude-728-a-cards-answer-field-did-not-redraw-its-tab-001), and the code-fence rule in the
  risks was not in the parser (F-claude-728-a-question-inside-a-code-fence-would-have-been-a-card-001).
  Both fixed. No Zed path is touched; no ledger row.
- **The gate:** the first run went green on every gate but its receipt, since the fence fix and
  the scenario changed during it; the second, `script/gates.sh --diff` on the final tree: 17 of 17
  green, receipt written.

## Phase 3 — Test
- **Scenario:** `script/e2e/728-question-cards-in-plans.sh` under `compositor sway`, on #726's
  stand-in (`MARLEY_BIN approve --tool ExitPlanMode` with the plan on stdin; a revised plan after a
  deny). The first runs placed the points and caught two layout faults, both fixed before the gate:
  the fourth card ran past the fold (the buttons moved into each card's header row) and the
  context drew larger than its question (L-claude-728-a-markdown-styles-size-is-the-containers-001).
- **728-01-cards** (REQ-001, REQ-002): four cards in their places between the plan's steps, each
  "Question N of 4" and Open; the first with its context in small muted type, its choices with
  reasons, "(recommended)" on In memory, and Accept recommended beside Skip; the second's three
  choices; the third's "Your answer" field; the column "0 of 4 answered" and Approve.
- **728-02-answered** (REQ-002, REQ-003, REQ-004): In memory ticked by Accept recommended and the
  button gone; File names and Text ticked; `search_cache` in the third's field, both `c`s there;
  the fourth Skipped with Unskip; the first three Answered; "3 of 4 answered"; Send Answers alone
  in Approve's place.
- **728-03-sent** (REQ-005): the terminal prints `toolu_1: exit 0 {"decision": "deny", "reason":
  "The user answered your questions. … Answers to your questions\n3 of 4 questions answered. …
  Answer: In memory (your recommendation) … - File names\n- Text … > search_cache … Skipped"}`;
  the log check passed.
- **728-04-revised** (REQ-006, REQ-007): Version 2 with its counts; the revised plan's question
  Settled with In memory ticked and no buttons or field; the fenced example drawn as code; the
  block with no closing line drawn as a paragraph.
- **Focus report:** a card's field takes the letter. (Corrected by #729: the claim that the plan's area kept `c` for notes was not checked here, and it did not hold; see F-claude-729-the-plans-c-key-made-no-note-after-728-001.)
- Pre-existing failures: none seen.

## Phase 4 — Complete
- CHANGELOG (Added); `docs/marley/guide.md` and the in-app guide (Plan review: the cards and the
  syntax); `docs/marley_architecture/marley_workbench.md` (question cards) and `marley_agent.md`
  (`plan_questions`).
- Knowledge: F-claude-728-a-cards-answer-field-did-not-redraw-its-tab-001,
  F-claude-728-a-question-inside-a-code-fence-would-have-been-a-card-001,
  L-claude-728-a-markdown-styles-size-is-the-containers-001,
  L-claude-728-a-letter-bound-on-an-area-yields-to-a-field-inside-it-001,
  AD-claude-728-question-cards-split-the-plan-before-zeds-markdown-001.
- The harness request (D5) stays recorded above, not sent: rustal-harness is paused.
