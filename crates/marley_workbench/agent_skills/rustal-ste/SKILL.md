---
name: rustal-ste
description: "Use for every message or instruction one agent sends another in the Rustal ecosystem (rustal-harness, Marley, Rusty, the Brain): messages, sends, reports, handoffs, assignments, questions, and tool descriptions or error text that agents read. Simplified Technical English (ASD-STE100) with Rustal's glossary, verbs, identifiers and message shapes. Also for: disambiguate, STE rewrite, rewrite so an agent cannot misread this. Not for prose a person reads for its voice."
version: 1.0.0
---

# Rustal STE: Simplified Technical English for Rustal's agents

An agent that reads another agent's message has nobody to ask what it meant. ASD-STE100, the
controlled English the aerospace industry wrote so maintenance instructions cannot be misread,
removes the two causes of misreading: words with more than one meaning, and sentences with more
than one structure. This skill applies its rules to messages between Rustal's agents, and adds
Rustal's own glossary: one word for each thing and each action across rustal-harness, Marley,
Rusty and the Brain.

Adapted from danyuchn/asd-ste100-skill (MIT); see `SOURCE.md`.

## When it applies

Chad, 2026-10-09: "for harness communication we should use this", then "adapt it and take it and
use it for rustal specific instructions across agents". Write in **Strict** mode, without being
asked, every message one agent sends another:

1. Messages between agents' sessions (an agent's message tool, such as Claude Code's SendMessage).
2. The harness's `message_send`, `session_send`, `thread_post` and reports.
3. What a manager, a foreman or a seat reports, assigns or hands off.
4. Tool descriptions and error text that agents read.

Apply it silently: no rule table, no mode note, no change summary. Text for a person (Chad, a doc,
a commit message) follows the `no-ai-slop` skill instead, with the glossary's words.

## Structural rules (Strict mode)

| Rule | Do | Don't |
|---|---|---|
| Active voice | "The agent deletes the file." | "The file is deleted (by the agent)." — unless the actor is genuinely unknown or irrelevant |
| No phrasal verbs (Rule 9.3) | "Remove the panel." / "Start the job." | "Take off the panel." / "Spin up the job." — a two-word verb has meanings the parts do not predict |
| One instruction per sentence | "Open the file. Read line 3." | "Open the file and read line 3, then check if it matches." |
| Sentence length | ≤20 words for instructions/procedures, ≤25 words for descriptions | Long compound/subordinate-clause sentences |
| No semicolons (Rule 8.1) | Split into separate sentences | Any semicolon at all — STE bans the mark outright, not only as a clause join. (Rule 8.1 permits every other standard punctuation mark. The em dash is *not* banned by STE, though it often signals a sentence that should be split.) |
| Noun clusters | ≤3 words stacked as a noun phrase ("fuel pump valve") | 4+ word noun stacks ("high pressure fuel pump inlet valve assembly") |
| No ellipsis | Keep the subject, verb, and article explicit even if it reads longer | Drop words to save space ("Files not backed up will be lost" → ambiguous which files) |
| Keep modality | "The request **may have** failed." stays "may have" | Promote a hedge to a fact ("The request failed.") or invent a certainty the source did not state |
| Paragraph limits | One topic per paragraph, ≤6 sentences | Multi-topic paragraphs |
| Lists for sequences | Use a numbered or bulleted list for 3+ steps or conditions | Bury a sequence inside one prose sentence |

**Simple tenses, with one exception.**

STE permits infinitive, imperative, simple present, simple past, simple future, and past participle as adjective. It excludes present perfect and other compound forms: "we received the report", not "we have received the report".

Aircraft manuals never need present perfect, so the exclusion costs the standard nothing. Other text is not always so lucky. "The job has completed" (and its output is available now) and "the job completed" (at some past point) are different statements, and status text frequently needs the first. **Where the compound form carries information the simple form cannot — current relevance, or a hedge as in "may have failed" — keep it and flag the departure.** Elsewhere, follow the rule.

## Rustal's words: one meaning each

Use these terms with these meanings only; the definitions are the glossary's own. The full
glossary (70 terms, with their sources and the words not to use) is `references/glossary.md`.

| Term | One meaning |
|---|---|
| seat | A seat is a named agent position in a root that one profile defines. `rh seat` commands act on it. |
| session | A session is one agent entry in the fleet. A fleet id names it, and `session_*` tools act on it. |
| root | The root is the private directory that holds the state, profiles and journal of one harness runtime. |
| workspace | A workspace is one tmux session that the runtime owns, with a UUID, a name, windows and panes. |
| generation | A generation is one process lifetime of a session. A restart opens the next generation with the same fleet id. |
| delivery | A delivery is one carried send or message, with a UUID and a state that only moves forward. |
| message | A message is text that the harness carries to one recipient's inbox, with a delivery id. |
| send | A send is text that `session_send` delivers into the next turn of an idle session. |
| receipt | A receipt is the record of one result: the answer to a write, a received message or a passed gate. |
| manager | The manager is the one session that a root designates to assign work, answer questions and report to the user. |
| foreman | The foreman is the one session that a root designates to take `failure` and `hold` owner items before the manager. |
| actor | An actor is a scripted fixture agent that the runtime runs in a tmux terminal for offline tests. |
| Needs you | Needs you is the rail section that shows each wait on the user, the longest wait first. |
| Agent Activity | Agent Activity is Marley's log of each acting tool call that an agent makes, newest first. |
| kill switch | The kill switch, the Stop button, makes each acting Marley tool refuse each agent until the user resumes. |
| Rusty's brain | Rusty's brain is the user's markdown vault of pages. Rusty indexes it and keeps it in git. |
| the Brain | The Brain is the Rustal Brain server that assigns work, enforces the workflow and decides completion from evidence. |
| ticket | A ticket is the local file that records one work item and its acceptance criteria. |
| gate | A gate is a project's check script, such as `script/gates.sh` or `bin/gate.sh`, with the checks it runs. |

## Words that mean different things in different systems

Use the glossary term in the "Write" column. Quote UI labels, commands and field names
verbatim, whatever word they use.

| Word | Meanings in the docs | Write |
|---|---|---|
| session | A fleet entry (harness). Claude Code's own session, `session_id` and `--resume`. Marley's Allow for This Session. `rusty session start` (Rusty's service). An MCP connection. | "session" for a fleet entry only. "conversation" for Claude Code's or Codex's transcript, as rustal-harness docs/CLAUDE_CODE.md#stop-and-restart does. |
| seat | A named position from a profile (harness `rh seat`). An entry in Marley's fleet `seats` array. `rw`'s seat label (Brain). | "seat" for the named position. "session" for a fleet entry. A seat label is metadata, not identity (rustal-brain docs/architecture/SYSTEM.md#agent-plane). |
| workspace | A tmux session (harness). A leased worktree (Brain: "isolated workspace"). Zed's window and its projects. | "workspace" for the harness meaning. "isolated worktree" for the Brain's. "window" for Zed's. |
| root | The harness state root. A project's top folder (Marley: "the project root"). The Brain's root run. | "root" for the harness meaning. "project folder". "root run". |
| thread | An Agent Panel thread (Marley). The manager thread (harness). A Codex or Claude `thread` label. Both Marley and the harness serve a tool named `thread_post`. | "thread" for the Agent Panel. "manager thread". Quote the label. Name the server with the tool: "the harness's `thread_post`". |
| brain | Rusty's vault. The Rustal Brain. | "Rusty's brain" or "the Brain". Never "the brain" alone. |
| inbox | The owner inbox. An agent's message inbox. Needs you ("the inbox" in Marley's guide). Rusty's inbox page. | Always qualify: "owner inbox", "message inbox", "Needs you", "Rusty's inbox page". |
| stop | Stop a seat or a session (harness). The kill switch's Stop button. Claude Code's Stop hook at a turn's end. A port row's Stop. | "stop" for a seat, session or workspace. "kill switch" for agent control. "The turn ends" for a turn. |
| decision | A Rusty decision page. A harness decision register entry. A Marley `AD-` ledger entry. An approval's `approve`, `deny` or `cancel`. | "decision" for the Rusty page. Cite `D186` or `AD-…` by id. An approval gets an answer. |
| profile | The harness profile file. Zed's agent profile (Write, the Marley profile). | "profile" for the harness file. "Zed agent profile" for Zed's. |
| pane | A tmux pane (harness). Zed's center pane. | "pane" for tmux. "the center" or "a tab" for Zed. |
| report | `agent_report`. `workflow_report`. A manager thread `report`. A gate's `result.json`. | "report" for an agent's statement. "gate result" for `result.json`. |
| PR | A pull request. A `PR-` prevention rule. | "pull request" in full. `PR-001` only for the rule. |
| park folder | rustal-harness parks to `docs/planning/pipeline/paused/`. Marley_ide has `docs/planning/pipeline/parked/`. | The verb is "park". Name the folder the repository uses. |
| the person | MANAGER.md says "the person" for the human in the manager thread. | "the user". Quote the field value `person` verbatim. |

## Rustal's verbs: one verb for each action

One verb for each action. Tool names, commands and UI labels stay verbatim, even when their
word is in the "Don't use" column.

| Action | Approved verb | Don't use | Example sentence |
|---|---|---|---|
| End the live generation of a seat, session or workspace on purpose | stop | kill, shut down, terminate, take down | Stop the seat `scratch`. Its profile stays in the root. |
| Stop a seat and delete its profile | remove | destroy, tear down, purge, drop | Remove the seat `scratch` when the test ends. |
| Run a seat's profile and wait for its first report | start | launch, spin up, boot | Start the seat `lead` with `rh seat start lead`. |
| Make a new session from a profile | open | spawn, create, instantiate | Open a session from the profile `reviewer` with `session_open`. |
| Open the next generation of a failed session with the same fleet id | restart | respawn, relaunch, reboot, revive | Supervision restarts the seat from the resume argv that its agent reported. |
| End a running turn and keep the session | interrupt | abort, cancel, kill | Interrupt the turn with `rh interrupt agent/<workspace uuid>`. |
| Give text to a session or to another agent | send | ping, DM, nudge | Send the gate result to `manager` with `message_send`. |
| Carry a send or a message to its recipient | deliver | route, hand off | The runtime delivers the send when the seat is idle. |
| Take a message and record a receipt | receive | ack, acknowledge, read | Receive the message before you act on it. A read is not a receipt. |
| Send a message that answers an earlier message | reply | respond, write back | Reply with `reply_to` set to the delivery of the message that you answer. |
| Give the choice for a question or an approval | answer | resolve, handle, approve | Answer the approval with `deny`. Give the reason in the manager thread. |
| Tell the host your state, phase or result | report | announce, signal, notify | Report `waiting` with the prompt and the options. |
| End your own authority on your seat, as done | release | sign off, exit, quit | Release the seat when your session ends. |
| Close a view and keep its process | detach | leave, hide | Detach from the workspace with Ctrl+b, then d. The shell continues. |
| Name a session as the root's manager or foreman | designate | appoint, elect, crown | Designate `agent/<workspace uuid>` as the manager with `rh manager`. |
| Declare a restart policy for a session | supervise | watch, babysit, monitor | Supervise the seat with two attempts and backoffs of 0 and 5000 milliseconds. |
| Raise a failure for the manager when no restart is left | escalate | bubble up, punt, kick up | Supervision escalates the generation after two failed restarts. |
| Pass an owner item that you hold to the manager now | forward | pass on, hand up | Forward the failure to the manager with `owner_forward`. |
| Pass the user's words, verbatim, to another agent | relay | paraphrase, summarize, convey | Relay the user's words in quotes, with the date. |
| Give one bounded piece of work to one agent | assign | dispatch, delegate, hand out, allocate | Assign harness TICKET-114 to the seat `lead`. |
| Take input control of a session or a pane | claim | take over, grab, seize | Claim control with Ctrl-b c before you type in the view. |
| Consult Rusty's brain before a decision | ask | search, look up, query | Ask Rusty's brain with `brain_ask` before you choose the crate. |
| Record a choice with its rationale | decide | settle, conclude, rule | Decide with `brain_decide`. Set `follow_up_by` one week from today. |
| Record how a decision went | follow up | revisit, check back, review | Follow up the decision with the status `kept`. |
| Add one line to today's daily note or to the inbox page | capture | jot, note down | Capture the port number to today's daily note. |
| Keep a fact or a preference in Rusty | store | save, memorize, remember | Store the preference as a memory with the importance `high`. |
| Move a ticket's spec and notes out of the active pipeline as `Blocked`, ticket still open | park | shelve, defer, pause, freeze | Park harness TICKET-097 until the Brain publishes CR-006. |
| Continue a parked ticket or a conversation from its recorded point | resume | unpark, reopen, continue | Resume TICKET-097 at `Phase 2 — Code`. |
| Move a ticket from the backlog into the active pipeline at Plan | promote | pick up, pull, activate | Promote the top Queue row of `BACKLOG.md` with `/pipeline:plan`. |
| Move a finished ticket to `tickets/closed/` | close | finish, wrap up | Close #710 when each criterion has evidence. |
| Keep a thing, but move it out of the active list | archive | stash, retire | Archive the spec and notes files into `pipeline/completed/`. |
| Destroy a thing for good | delete | wipe, nuke, erase | Delete the thread only when the user tells you to. |
| Record staged changes in git | commit | check in, snapshot | Commit with the ticket id in the message body. |
| Send commits to a remote | push | ship, upload, sync | Push only when the user tells you to. Scan the range for private content first. |
| Execute a gate or a check | run | kick off, fire, trigger | Run `script/gates.sh --diff` at the end of Code. |
| Meet each check of a gate or a phase | pass | go green, clear, succeed | The phase passes when each criterion has evidence. |
| Not meet a check of a gate or a phase | fail | go red, break, flunk | The gate fails on `clippy`. Fix the cause. Do not add a suppression. |
| Decline a call and change nothing | refuse | reject, bounce, turn down | The runtime refuses the send because the seat is not idle. |
| Offer a change that waits for the user's Apply | propose | suggest, request | Propose the seat with `seat_add`. The user applies it. |
| Write a durable fact in a file or a store | record | log, jot, write down | Record the refusal code in the notes. |

## Identifiers

Write each id as the line shows. Put ids, codes, paths and commands in backticks.

- **Harness ticket:** `TICKET-114`, as rustal-harness writes it; cite it as "harness TICKET-114" outside that repository.
- **Marley ticket:** `#710` inside Marley; the file is `TICKET-710-<slug>.md`; cite it as "Marley #710" outside Marley. UNSURE: see item 7.
- **Rusty ticket:** "Rusty TICKET-053"; Rusty's record is private, so give the ticket id only.
- **Commit SHA:** 7 or more lowercase hex characters, `0b049fe`; all 40 when you pin a commit, `0ba2872d8f53e666d7a8596fa63afe353cb5800d`.
- **Branch:** the full name, `marley/workbench-shell`.
- **Fleet id (a session's id):** `agent/<workspace uuid>`, `claude/<session uuid>`, `codex/<session uuid>`, `actor/<actor uuid>` or `herdr/<server>/<pane>`; an agent seat's id is `agent/<workspace uuid>`.
- **Seat name:** the profile name, lowercase, as `rh seat add` takes it: `lead`.
- **Generation:** its UUID; a question's prompt shows its first 8 characters: `[generation 1a2b3c4d]`.
- **Delivery id:** a canonical UUID that you keep for a retry: `533f7d9e-7c26-4b9a-b815-e649ac41ff0f`.
- **Harness decision register entry:** `D186`.
- **Requests between repositories:** `CR-006` (harness to Rustal, the Brain or workd), `MREQ-009` (harness to Marley).
- **Requirements and criteria:** `R13-02` (a harness milestone requirement), `SS-001` (one ticket's criterion).
- **Ledger entries:** `F-…`, `PR-…`, `L-…`, `AD-…` in Marley; `PR-…`, `BF-…`, `AD-…` in Rusty. Write "pull request" in full, never "PR".
- **Rusty page:** its slug, `projects/orbit`; a decision, `decisions/<slug>`; a consultation, the id that `brain_ask` returns.
- **Refusal and error codes:** verbatim, never paraphrased: `seat_exists`, `agent_report_stale`, `not_granted`.
- **Tool names:** the underscore form, with the server when two servers serve the name: `session_send`, "the harness's `thread_post`".
- **Event names:** verbatim: `agent_seat_report`.
- **State, option and outcome values:** verbatim, lowercase: `waiting`, `approve`, `passed`.
- **Phase status line:** verbatim: `Phase 2 — Code PASS; ready for Phase 3 — Test`.
- **Setting keys:** the full dotted path: `marley.harness_writes`.
- **Commands:** exact, with the docs' capital placeholders: `rh --state ROOT seat stop NAME`.
- **File paths:** relative to the repository root, with the repository named when it is not the reader's: `rustal-harness: docs/AGENT_SEATS.md`; a line as `path:line`. Write `~` for a home folder, never an account name.
- **UI labels:** the exact text and case that the UI shows, without backticks: Apply, Stop, Allow for This Session.
- **The user's words:** verbatim, in double quotes, with the name and the date; keep the spelling: Chad, 2026-10-09: "lets do the small things".
- **Dates and durations:** dates as `YYYY-MM-DD`; durations as a number and a unit word: 590 seconds.

## Message shapes

**Rules for every shape.**

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

The thirteen shapes follow. Each has a worked example, and the channels and their limits are in
`references/message-shapes.md`.

**1. Assignment.** Gives one bounded piece of work to one agent

```text
Assignment: <ticket> — <title>
To: <seat name, fleet id or agent>
Do: <the work, as instructions>
Done when: <the observable criteria>
Limits: <paths, rules and budget>
Report to: <the recipient, and the shape to send>
```

**2. Status.** Tells where the work is

```text
Status: <ticket>
Phase: <the spec's status line, verbatim>
State: <starting, working, idle, waiting, error or done>
Done: <what works now>
Next: <the next step>
Evidence: <paths>
```

**3. Question.** Asks for one choice

```text
Question: <ticket>
For: <the user, the manager or an agent>
Ask: <one question>
Options: <the options, at most 16>
Default: <the option that you take if no answer comes>
Blocks: <the step that waits>
```

**4. Answer.** Gives the choice for a Question or a Decision request. Send it as a reply. A question in a
session's envelope takes `session_answer`, not this shape. `Why` is optional.

```text
Answer: <ticket>
Re: <the delivery id of the question>
Choice: <one option, verbatim>
Why: <one sentence>
```

**5. Blocker.** Says that a required criterion cannot pass now, and what continues

```text
Blocker: <ticket>
Blocked by: <the cause, with its id>
Tried: <what you did, and the result>
Needs: <what removes the block, and from whom>
Next: <the independent work that continues>
```

**6. Decision request.** Asks the decider to choose, with the facts that the choice rests on

```text
Decision request: <ticket>
Decider: <the user, the manager or an agent>
Decide: <one question>
Options: <the options>
Recommend: <one option and why, or None>
Rests on: <decision ids, consultation id, pages, paths>
```

**7. Done / Handoff.** Closes the work, or gives it to the next agent. Use `Done:` when the ticket is complete and
`Handoff:` when the work continues

```text
Done: <ticket>
Phase: <the spec's status line, verbatim>
Works: <what works now>
Evidence: <the gate result path, the commit SHA, the branch>
Limits: <what does not work, or what you did not check>
Next: <the next step, and who takes it>
```

**8. Relay.** Passes the user's words to another agent, verbatim

```text
Relay: <ticket or topic>
From: the user, <date>
Words: "<the user's words, verbatim>"
Applies to: <tickets or decisions>
Action: <what the receiving agent does now>
```

**9. Interface request.** Asks another repository for a command, tool or field that it owns

```text
Request: to <repository>, for <ticket>
Need: <the command, tool or field, and its behavior>
Why: <what fails without it>
Blocks: <the ticket or criterion that waits>
Evidence: <path or id>
```

**10. Receipt.** Says that you received a message, and what is not done yet. Source: rustal-harness
docs/MESSAGES.md#operator-commands (the reply "Received; review is still pending");
docs/MESSAGES.md#delivery-receipt-and-recovery.

```text
Received: <delivery id>
Re: <the first line of the message>
State: <received, and what is not done>
Next: <the shape that you send when the work is done>
```

**11. Review note.** Marley's own shape for the user's notes on a diff. Marley writes it; keep the labels
verbatim. Source: marley_ide docs/marley/guide.md#review-notes-to-the-agent. Use `Line:`
for one line and `Lines:` for a range.

```text
File: <path relative to the agent's folder>
Lines: <first>-<last>
User comment: "<the user's note, verbatim>"
```

**12. Report.** The manager's report to the user in the manager thread. Marley shows the first line in a
desktop notice and in Needs you, so the first line must stand alone

```text
Report: <the result, in one line>
Ticket: <ticket>
Evidence: <path or commit>
Needs: <what the user must do, or Nothing>
```

**13. Confirmation.** The manager reads back one exact action before it does it. The user accepts or rejects it
within 60 seconds. The manager acts only on `accepted`. Source: rustal-harness
docs/MANAGER.md#the-thread.

```text
Confirm: <the exact action, at most 512 bytes>
Ticket: <ticket>
Why: <one sentence>
If rejected: <what you do instead>
```

## Scan checklist

These six habits cover most of what makes machine-written English hard to parse. Each one is mechanical: you can point at the exact word or punctuation mark that breaks the rule, with no judgment call. Scan for all six before you rewrite anything.

1. **Synonym rotation** — the same thing gets several names in one document ("the user", "the customer", "the client"). The reader cannot tell whether they are one thing or three. Fix: pick one name, use it every time.
2. **Hedge stacking** — helper verbs and qualifiers pile up until the sentence asserts nothing ("it is important to note that this may potentially help to improve"). Fix: state the claim, or delete it.
3. **Nominalization** — an action frozen into a noun ("perform an analysis of", "provides assistance to"). Fix: use the verb ("analyze", "helps").
4. **Marketing adjectives** — words that claim quality instead of showing it: seamless, robust, powerful, cutting-edge, effortless, blazing-fast. Fix: delete, or replace with the measurement that earns the claim.
5. **Run-on sentences** — several ideas joined by semicolons or em dashes. Fix: one idea per sentence.
6. **Soft phrasal verbs** — spin up, reach out, dive into, kick off. Fix: use the single plain verb (start, contact, read, begin).

## Process

1. Pick the shape the message needs.
2. Write each value in Strict mode, with the glossary's terms and verbs.
3. Keep every fact, condition, number, hedge and id from what you mean to say. Add no claim.
4. Check the text with the linter when it is long or will be reused:
   `python3 -I <this skill>/scripts/ste-lint.py FILE`. It checks the structural rules and flags
   the glossary's drift words (`rustal-term`, advisory). `--no-glossary` checks STE alone.
5. Send it through a message tool, never by typing into another agent's terminal.

## Boundaries

- STE fixes the form of a message, not its substance. A message with nothing to say stays empty.
- Never upgrade a hedge to a fact: "may have failed" stays "may have failed".
- Keep tool names, commands, UI labels and the user's quoted words verbatim, even when they hold a
  word the glossary avoids.
- The glossary's open choices have defaults in force until Chad decides:
  `references/glossary.md`, "Defaults until Chad decides".
- This is not certified STE. ASD's ~900-word dictionary is not reproduced; the plain-word rule
  stands in for it.
