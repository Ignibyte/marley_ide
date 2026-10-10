# rustal-ste message shapes

The fixed shapes for messages between agents in the Rustal ecosystem. Each shape is an ordered
list of labeled lines. The words in each value follow ASD-STE100 and the `rustal-ste` glossary
(`glossary.md`). Paths, SHAs and UUIDs in the examples are samples.

## Rules for every shape

1. Use one shape in each message. The first line starts with the shape's label.
2. Write the labels in the order shown, one label on each line. Leave out an optional label
   that has no value.
3. Write each value in Strict STE: one fact or one instruction in each sentence, at most 20
   words, active voice, with the glossary's terms and verbs.
4. Write ids, codes, paths, commands and the user's words as the glossary's Identifiers
   section says.
5. To answer a message, set `reply_to` to its delivery
   (rustal-harness: docs/AGENT_MESSAGES.md#guarantees).
6. A receipt is not completion. Send a Done message when the work is done
   (rustal-harness: docs/MESSAGES.md#delivery-receipt-and-recovery).
7. A message gives no authority. Only the Brain's assignment or the user's word authorizes
   work (rustal-harness: docs/MESSAGES.md#bounds-storage-and-authority; rustal-brain:
   docs/architecture/IDENTITY-ORCHESTRATION.md#orchestration-rules, rule 8).
8. Send a question for the user to the user: through the manager, or as a `waiting` report
   that Needs you shows. Ask one question in each message.
9. Send through a message tool. Never type into another agent's terminal; Marley's
   `terminal_type` refuses it (marley_ide: docs/marley/guide.md#the-tools).
10. Put no key, token, password or secret value in a message.

## Channels

| Channel | What it carries | Limits | Source |
|---|---|---|---|
| `message_send` (harness, agent grant) | A message to another session's fleet id or to `manager` | Text of 1 to 4,096 bytes | rustal-harness: docs/AGENT_MESSAGES.md#the-tools |
| `session_send` (harness, write grant or the manager) | A send into the next turn of an idle session | 1 to 8,192 bytes to a Codex or Claude session; 1 to 512 to an actor | rustal-harness: docs/MCP.md#sending |
| `thread_post` (harness, the manager only) | A `message`, `report` or `confirmation` in the manager thread | Text up to 8,192 bytes; an `action` up to 512 bytes; one answer within 60 seconds | rustal-harness: docs/MANAGER.md#the-thread |
| `agent_report`, `rh report`, `MARLEY_BIN report` | State, progress and a question, as fields | Activity: one line of at most 120 characters. Question: a prompt of 1 to 2,048 bytes and 1 to 16 options of at most 128 characters | rustal-harness: docs/AGENT_SEATS.md#reporting; marley_ide: docs/marley/guide.md#an-agents-own-reports-652 |
| `workflow_report` | Ticket, phase, outcome and gate, as fields | Each value 1 to 64 bytes; outcome `started`, `passed` or `failed` | rustal-harness: docs/PHASES.md#reporting-a-phase |
| Send Review to Agent (Marley) | Review notes, pasted as one prompt into an idle Claude Code | Only to a terminal that reads `ready` | marley_ide: docs/marley/guide.md#review-notes-to-the-agent |
| An agent's own message tool, such as Claude Code's SendMessage | Any shape between peer agents | None in the docs | (none) |

When a tool takes a fact as a field, use the field, and use the text shape only for what
the field cannot hold:

| Shape | Structured form |
|---|---|
| Status | `workflow_report` (`ticket`, `phase`, `outcome`, `gate`) and `agent_report` (`state`, `progress.activity`) |
| Question | `agent_report` with the state `waiting`, `question.prompt` and `question.options`; `session_answer` answers it |
| Receipt | `message_receive` with a `reply`, which records the receipt and lands the reply |
| Blocker | The spec's `status: Blocked`, with the reason and the next action in the notes (rustal-harness: docs/WORKFLOW.md#phases-and-records) |
| Report, Confirmation | `thread_post` with the kind `report` or `confirmation`; `thread_status` gives the answer |

## The shapes

### 1. Assignment

Gives one bounded piece of work to one agent. Sources: rustal-harness docs/MANAGER.md (the
manager gives out work); rustal-brain docs/architecture/IDENTITY-ORCHESTRATION.md#root-runs-and-assignments
(paths, phases, expected artifacts, expiry).

```text
Assignment: <ticket> — <title>
To: <seat name, fleet id or agent>
Do: <the work, as instructions>
Done when: <the observable criteria>
Limits: <paths, rules and budget>
Report to: <the recipient, and the shape to send>
```

```text
Assignment: harness TICKET-114 — stop and remove a seat
To: the seat `lead`
Do: Add `rh seat stop NAME` and `rh seat remove NAME`.
Done when: Each command prints one JSON object. Each refusal has a code, such as `seat_unknown`.
Limits: Do not change Marley. Run one cargo command at a time.
Report to: `manager`, with a Done message.
```

### 2. Status

Tells where the work is. Sources: rustal-harness docs/PHASES.md (ticket, phase, outcome,
gate); docs/AGENT_SEATS.md#reporting (state, activity); docs/WORKFLOW.md (the status line).
`Evidence` is optional.

```text
Status: <ticket>
Phase: <the spec's status line, verbatim>
State: <starting, working, idle, waiting, error or done>
Done: <what works now>
Next: <the next step>
Evidence: <paths>
```

```text
Status: harness TICKET-114
Phase: Phase 3 — Test
State: working
Done: `rh seat stop` and `rh seat remove` pass the focused checks.
Next: The full gate runs. I read `result.json` when it ends.
Evidence: `.artifacts/gates/ticket-114-full-1/`
```

### 3. Question

Asks for one choice. Sources: rustal-harness docs/AGENT_SEATS.md#reporting (a prompt and its
options); docs/ACTORS.md (a choice question); docs/MCP.md#answering-a-question. `Default` is
optional.

```text
Question: <ticket>
For: <the user, the manager or an agent>
Ask: <one question>
Options: <the options, at most 16>
Default: <the option that you take if no answer comes>
Blocks: <the step that waits>
```

```text
Question: harness TICKET-114
For: the manager
Ask: Which base branch do I use for the fix?
Options: `main` or `release`
Default: `main`
Blocks: The first commit of the fix.
```

### 4. Answer

Gives the choice for a Question or a Decision request. Send it as a reply. A question in a
session's envelope takes `session_answer`, not this shape. `Why` is optional.

```text
Answer: <ticket>
Re: <the delivery id of the question>
Choice: <one option, verbatim>
Why: <one sentence>
```

```text
Answer: harness TICKET-114
Re: 9b2f0c4e-1d7a-4c55-8f3b-2a6e7d1c0b9a
Choice: `main`
Why: The fix goes into the next release from `main`.
```

### 5. Blocker

Says that a required criterion cannot pass now, and what continues. Sources: rustal-harness
docs/WORKFLOW.md#phases-and-records ("a concrete reason and next action");
.agents/skills/harness-workflow/SKILL.md ("record it and leave the pipeline resumable");
docs/planning/CORE_REQUESTS.md (request template, "Independent work that can continue").

```text
Blocker: <ticket>
Blocked by: <the cause, with its id>
Tried: <what you did, and the result>
Needs: <what removes the block, and from whom>
Next: <the independent work that continues>
```

```text
Blocker: harness TICKET-097
Blocked by: CR-006. The Brain does not publish the hub interface yet.
Tried: A bridge against a fixture. The real interface can change all of it.
Needs: The Brain's addressed messages and assignments, each with a delivery id.
Next: I park TICKET-097 and start TICKET-114.
```

### 6. Decision request

Asks the decider to choose, with the facts that the choice rests on. Sources: Rusty skills
store ask-decide-follow-up/SKILL.md (consultation, choice, rationale, alternatives);
rustal-harness docs/DECISIONS.md#open-decisions.

```text
Decision request: <ticket>
Decider: <the user, the manager or an agent>
Decide: <one question>
Options: <the options>
Recommend: <one option and why, or None>
Rests on: <decision ids, consultation id, pages, paths>
```

```text
Decision request: harness TICKET-113
Decider: the user
Decide: Do you keep the foreman seat, or use supervision alone?
Options: foreman seat, or supervision alone
Recommend: None. The foreman runs now as the plan's default, not as your choice.
Rests on: D179, D180; `rustal-harness: docs/MANAGER.md#the-foreman`
```

### 7. Done / Handoff

Closes the work, or gives it to the next agent. Use `Done:` when the ticket is complete and
`Handoff:` when the work continues. Sources: rustal-harness
.agents/skills/harness-workflow/SKILL.md ("what works, current phase, actual checks and
evidence paths, limitations, and the next task"); docs/CLI_HANDOFF.md; marley_ide
.claude/commands/pipeline/complete.md ("the gate result, the commit SHA and the branch").

```text
Done: <ticket>
Phase: <the spec's status line, verbatim>
Works: <what works now>
Evidence: <the gate result path, the commit SHA, the branch>
Limits: <what does not work, or what you did not check>
Next: <the next step, and who takes it>
```

```text
Done: harness TICKET-109
Phase: Complete
Works: `rh seat add lead` writes the profile. `rh seat start lead` starts the seat.
Evidence: Full gate passed: `.artifacts/gates/ticket-109-full-1/result.json`. Commit `a1b2c3d` on `main`.
Limits: A Codex seat has no interface in a harness terminal.
Next: TICKET-110 lets Claude Code's interface act as the manager.
```

### 8. Relay

Passes the user's words to another agent, verbatim. Sources: rustal-harness
docs/DECISIONS.md, D176 to D186 ("The Marley session relayed Chad's word"); marley_ide
docs/planning/tickets/open/TICKET-710-the-marley-agent-removes-a-seat.md (Source ticket).

```text
Relay: <ticket or topic>
From: the user, <date>
Words: "<the user's words, verbatim>"
Applies to: <tickets or decisions>
Action: <what the receiving agent does now>
```

```text
Relay: Marley #710
From: Chad, 2026-10-09
Words: "lets do the small things"
Applies to: Marley #710 and harness TICKET-114
Action: Do the small tickets first.
```

### 9. Interface request

Asks another repository for a command, tool or field that it owns. Sources: rustal-harness
docs/planning/CORE_REQUESTS.md (request template); docs/planning/MARLEY_REQUESTS.md (date,
source, difference, evidence, requested behavior).

```text
Request: to <repository>, for <ticket>
Need: <the command, tool or field, and its behavior>
Why: <what fails without it>
Blocks: <the ticket or criterion that waits>
Evidence: <path or id>
```

```text
Request: to rustal-harness, for Marley #710
Need: `rh seat stop NAME` and `rh seat remove NAME`. Each prints one JSON object and refuses by name.
Why: `rh stop WORKSPACE` keeps the profile, so `seat add` refuses `seat_exists`.
Blocks: Marley #710, the `seat_remove` tool.
Evidence: `marley_ide: docs/planning/tickets/open/TICKET-710-the-marley-agent-removes-a-seat.md`
```

### 10. Receipt

Says that you received a message, and what is not done yet. Source: rustal-harness
docs/MESSAGES.md#operator-commands (the reply "Received; review is still pending");
docs/MESSAGES.md#delivery-receipt-and-recovery.

```text
Received: <delivery id>
Re: <the first line of the message>
State: <received, and what is not done>
Next: <the shape that you send when the work is done>
```

```text
Received: 533f7d9e-7c26-4b9a-b815-e649ac41ff0f
Re: Review the latest local gate report.
State: Received. The review is not done.
Next: I send a Done message after the review.
```

### 11. Review note

Marley's own shape for the user's notes on a diff. Marley writes it; keep the labels
verbatim. Source: marley_ide docs/marley/guide.md#review-notes-to-the-agent. Use `Line:`
for one line and `Lines:` for a range.

```text
File: <path relative to the agent's folder>
Lines: <first>-<last>
User comment: "<the user's note, verbatim>"
```

```text
File: crates/marley_workbench/src/agent_control.rs
Lines: 40-52
User comment: "Refuse this when the kill switch is on."
```

### 12. Report

The manager's report to the user in the manager thread. Marley shows the first line in a
desktop notice and in Needs you, so the first line must stand alone. Sources: rustal-harness
docs/MANAGER.md#the-thread; marley_ide docs/marley/guide.md#the-harnesss-sessions.

```text
Report: <the result, in one line>
Ticket: <ticket>
Evidence: <path or commit>
Needs: <what the user must do, or Nothing>
```

```text
Report: harness TICKET-109 is complete. A seat now starts with two commands.
Ticket: harness TICKET-109
Evidence: Commit `a1b2c3d`. The full gate passed.
Needs: Nothing.
```

### 13. Confirmation

The manager reads back one exact action before it does it. The user accepts or rejects it
within 60 seconds. The manager acts only on `accepted`. Source: rustal-harness
docs/MANAGER.md#the-thread.

```text
Confirm: <the exact action, at most 512 bytes>
Ticket: <ticket>
Why: <one sentence>
If rejected: <what you do instead>
```

```text
Confirm: merge the branch
Ticket: harness TICKET-114
Why: The full gate passed on the branch head.
If rejected: I keep the branch and ask what to change.
```
