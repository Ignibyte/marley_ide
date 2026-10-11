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
