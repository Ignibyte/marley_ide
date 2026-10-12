---
name: plan-questions
description: "Use in plan mode before you call ExitPlanMode, when the plan has a question that the user must answer. Write each open question in a :::question, :::question-multi or :::question-text block. Marley shows each block as a card. The answers come back in the feedback under \"Answers to your questions\"."
version: 1.0.0
---

# Questions in a plan

Marley shows a plan from plan mode in a tab. Marley shows each question block in the
plan as a card with choices. The user answers the cards. Marley sends the answers to you
in the feedback on the plan.

Use this skill in plan mode, before you call `ExitPlanMode`. Without Marley, the user
reads the blocks as plain text.

## When to write a block

1. Write a block for each decision that the user must make before you can do the work.
2. Do not write a block for a fact that the code or the docs give. Find the fact.
3. Ask each question one time. Put its block where the plan needs the answer.

## The block

1. Write the first line of the block. Use one of these lines:
   - `:::question` for a question with one choice.
   - `:::question-multi` for a question with one or more choices.
   - `:::question-text` for a question that the user answers in words.
2. Write the question on the next line.
3. Write the context on the lines after the question. The context is optional.
4. Write each choice on its own line: `- [ ] LABEL — REASON`.
5. Write `Recommended: LABEL` on its own line for the choice that you recommend.
6. Use the label of the choice exactly in the `Recommended:` line.
7. Mark a choice that the user made before with `- [x]`. Marley shows that card as Settled.
8. Write `:::` alone on the last line of the block.

A `:::question-text` block has no choices. Marley shows a block without its last line
`:::` as plain text. Marley does not show a block in a fenced code block as a card.

## Example

```text
:::question
Which store keeps the run records?
The runtime keeps its journal in SQLite now.
- [ ] SQLite — the runtime has it already
- [ ] Postgres — each host needs a server for it
Recommended: SQLite
:::

:::question-multi
Which suites run on each change?
- [ ] the fast suite — it takes one minute
- [ ] the browser journeys — they take twenty minutes
- [x] the format check — the user asked for it before
:::

:::question-text
What name do you want for the new command?
:::
```

## The answers

When the user sends the answers, `ExitPlanMode` refuses the call. The feedback holds the
answers. It starts with the line `Answers to your questions`. Then it gives these parts:

- The count of answered questions.
- A heading `### Question N:` with the question, for each question.
- The answer under each heading: `Answer:` with the choice, the choices as a list, the
  user's words as a quote, `Other:` with the user's words, or `Skipped`.
- `(your recommendation)` after a choice that is your recommendation.
- `Unanswered:` with the questions that have no answer.
- The user's notes on the plan, after the answers.

Do these steps:

1. Read each answer and each note.
2. Change the plan so that it agrees with each answer and each note.
3. In the block of each answered choice question, mark each chosen choice `- [x]`.
4. Write the answer to a text question in the plan. Then remove its block.
5. Keep the block of a skipped question when you still need the answer.
6. Use your recommendation for a skipped question that you do not need. Write your choice in the plan.
7. Call `ExitPlanMode` again with the changed plan.
