---
pipeline_id: f3275f6c-4451-4e81-8582-286809ed41f8
ticket: docs/planning/tickets/closed/TICKET-568-inbox-order-and-risk-chips.md
status: Phase 4 — Complete PASS
title: "The approvals inbox: needs-you order and risk chips"
type: feature
slice: prong 2 (attention); the Jev note's use 3, on #508's inbox and #565's layer
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/completed/565-system-one-layer.spec.md, docs/planning/pipeline/completed/508-approvals-inbox.spec.md, docs/planning/pipeline/completed/508-approvals-inbox.notes.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md, docs/planning/pipeline/completed/571-pause-before-a-consequential-click.spec.md]
---

## Title
#508's inbox gains chips and an order. Code classifies each waiting tool call from the tool and
what it acts on, against the project's folders (`destroys`, `credentials`, `rewrites history`,
`sends out`, `installs`, `outside project`, `claims approval`), each chip with a level, and a
click a Browser tab holds carries its #571 class as a chip (`pays`, `destroys`, `sends out`,
`changes account`). Entries order by level, then age. #565's layer is asked only about the tool
calls code found nothing on: nouls that may add a chip, never remove one, and an urgency score
that may raise such an entry's level, never lower it. The chips and the order approve nothing:
Allow and Deny stay the user's clicks, and Claude Code's own reviewer keeps the verdict. Off by
default: with the use off, the inbox is #508's, oldest first.

## Scope
### In
- **The classifier** (`crates/marley_agent/src/risk.rs`, pure): `classify(&Action) -> Vec<Chip>`
  over `Action { tool, line, paths, cwd, folders, home, secret }`, where `tool` is a
  `ToolClass` (read, write, execute, delete, question, other) and `secret` says the model
  redactor found a secret in the line. `Chip { kind, source }` takes its level from its kind;
  `ChipSource` is `Rules`, or `Model { permille }` for a reading. The patterns are whole words
  over each simple command of the line (after `;`, `&&`, `||`, `|` and `(`, with `sudo`, `env`,
  `nohup`, `time` and assignments skipped), with no regex and no shell parse, as #571's
  `consequence` matches words:
  - `destroys` (5): `rm` with a recursive or force flag or a glob operand, `git reset --hard`,
    `git clean -f…`, `git checkout -- .` or `git checkout .`, `git restore .`, `git branch -D`,
    `git stash drop` and `clear`, `drop table`, `drop database`, `truncate`, `mkfs…`, `dd …
    of=`, `shred`, `wipefs`, `find … -delete`; an Agent Panel tool call of kind `delete`;
  - `credentials` (5): a path or word naming `.env` or `.env.*`, `id_rsa`, `id_ed25519`,
    `id_ecdsa`, `.ssh/`, `.aws/`, `.gnupg`, `.config/gh`, `.netrc`, a `.pem` or `.key` file,
    `keychain`, `secret-tool`, `gpg`, the `pass` command; or `secret`;
  - `rewrites history` (5): `git rebase`, `git commit --amend`, `git push` with `--force`, `-f`
    or `--force-with-lease`, `git filter-branch`, `git filter-repo`, `git reflog expire`;
  - `sends out` (4): `curl` or `wget` with `-d`, `--data…`, `-F`, `--form`, `-T`,
    `--upload-file`, or a method other than GET (`-X`, `--request`); `scp`; `rsync` to a `host:`
    operand; `ssh`; `git push`; `gh pr create`, `gh release`, `gh issue create`; `npm publish`,
    `cargo publish`; `mail`, `sendmail`; `aws s3 cp` or `sync` to `s3://`;
  - `installs` (4): `npm i` or `install`, `pnpm add` or `install`, `yarn add`, `pip install`,
    `pip3 install`, `uv pip install`, `cargo install`, `gem install`, `go install`, `pacman -S…`,
    `yay -S…`, `apt install`, `apt-get install`, `brew install`, and `curl` or `wget` piped to
    `sh` or `bash`;
  - `outside project` (3): a write tool's path, or an absolute, `~` or `cd` path in a command,
    under no folder of the project, with `/dev/` and the temporary folder aside;
  - `claims approval` (+1, at most 5): the line or a question asserting an approval ("owner
    approved", "already approved", "pre-approved", "you allowed this", "you already allowed",
    "permission granted", "user approved"), the note's safety rule 6.
  A read tool with no chip is level 1; any other with no chip is level 2. A click's chip from
  #571's class: `pays` (5), `destroys` (5, from Deletes), `sends out` (4), `changes account` (4).
- **The entries' inputs** (`rail.rs`, where #508 gathers the entries):
  - an Agent Panel tool call: its kind (`read`, `search`, `fetch` and `think` read; `edit` and
    `move` write; `delete`; `execute`), the raw input's `command` when it is a string and else
    the label with its Markdown escapes taken out, the call's locations and the raw input's
    `path`, and a terminal tool's `cd`;
  - a terminal seat's wait: `Permission for <Tool: preview>` split at its first `: ` into the
    tool and the preview, the tool's class by its name (Read, Grep, Glob, WebFetch and
    WebSearch read; Write, Edit, MultiEdit and NotebookEdit write; Bash execute), the preview as
    a write tool's path, and the seat's `cwd` label; a question (AskUserQuestion) is the question
    class, which only `claims approval` marks;
  - a held click: its class, which the pause now keeps (`PendingClick.class`).
- **The order** (`note_inbox`): with the use on, level descending, then first seen, oldest
  first; with it off, #508's order and no chips. `InboxEntry` gains `chips` and `level`.
- **The question set** `inbox_risk/1` (in `marley_system_one`, beside the other sets, on
  `jev-1.13.0`): a noul per tool chip class (`destroys`, `credentials`, `rewrites_history`,
  `sends_out`, `installs`, `outside_project`, `claims_approval`), each with `true` and `false`
  criteria written as situations, and `urgency`, a score of five levels ("a routine read inside
  the project", "an edit or a reversible command inside the project", "a reversible action that
  reaches outside the project", "an action that sends data out, installs software or changes an
  account", "an action that destroys data or cannot be undone"). The use `INBOX_RISK` (`inbox`,
  600 ms).
- **The state**: the facts `tool` (its name), `agent` (Claude Code in a terminal, or the thread's
  agent), `project` (its name) and `code found` (`nothing`); the text `ask` (the entry's line,
  masked and cut to 300). No path or working directory is a fact.
- **The ask** (`Rail::refresh`, the one place with a mutable context): once per tool entry code
  gave no chip, while the use is not `off`, with the entry's key as the subject. The answer is
  kept by key with the ask it answered, and dropped when the entry's ask differs or the entry
  leaves. Each tool entry code gave a chip is one `rules` row instead. A held click gets
  neither, since #571's use logged it.
- **The rows**: under the card, a line with the chips and #508's buttons at its end; code's chips
  plain; the model's in `suggest` with a question mark (`destroys?`) and in `act` with a dashed
  border, each with a tooltip giving its reading. In `act` a reading's chips count toward the
  level, and its urgency (the expected level, rounded, when the score reads at confidence 0.5 or
  more) may raise the level.
- **Outcomes**: when an entry with a row leaves, `system_one::outcome` names what cleared it
  (allowed, denied or refused from the inbox, or cleared elsewhere) and how long it waited.
- **The setting**: `marley.system_one.uses.inbox`, `off` by default, the Inbox Risk item on the
  Marley page's System One section.
- `script/e2e/568-inbox-order-and-risk-chips.sh` on the `replay` provider, over #508's fixtures.

### Out (explicitly deferred)
- Approving or denying anything from a reading: no mode of this use answers a prompt (D5); Jev
  decides approvals last and only for agents with no reviewer, in a later ticket after the
  planted never-allow set exists (the note's safety rules 3 and 4).
- Facts for Claude Code's own classifier through a PostToolUse hook's `classifierContext`
  (the note); a second opinion shown beside auto mode's verdict.
- Routing a question to an owner or the manager (the note's use 5, #534's harness).
- Project rules such as "one cargo at a time" as a chip (Marley's `CLAUDE.md`): it needs the
  box's process table, #521's business.
- Chips on rows outside the inbox (a working row's tool line).
- Fitted thresholds; the 0.65 noul line and the 0.5 confidence floor stand until labels exist.
- Other shells' syntax (fish, PowerShell): the patterns read POSIX-like command lines.
- Chips on the Browser tab's card: the card already says why the click waits.

## Reference (§20)
- **Warp:** its notification mailbox lists agents' requests and filters them
  (docs.warp.dev/agent-platform/capabilities/agent-notifications/, #508's reference) and
  documents no risk marking; nothing of Warp's was read.
- **Upstream Zed:** the Agent Panel's permission prompt shows the tool call's label and input
  and lets the user allow or deny (`acp_thread`'s `PermissionOptions`, kept by #508 as the answer
  path); Zed marks no risk. Kept: the answer path and the label.
- **Orca:** the Needs You model (report 01 item 2), which orders by age; Marley adds the chips
  and the level in front of it.
- **Zed's own agent** (`crates/agent/src/tool_permissions.rs`, read at promotion): five built-in
  rules deny `rm` of `/`, `~`, `$HOME`, `.` and `..` for Zed's terminal tool whatever the
  settings, with `shell_command_parser::extract_commands` splitting chained commands. It denies
  and marks nothing else. Kept: the rule that `rm` with a recursive or force flag is the first
  thing to catch; the classifier marks those forms `destroys` and leaves the deny to Zed.
- **Claude Code's auto mode** (anthropic.com/engineering/claude-code-auto-mode, in the note): a
  classifier that sees user messages and tool calls but no tool results, 0.4% false positives on
  10,000 real actions and 17% false negatives on 52 overeager ones. It stays the approver in
  Marley's terminals (Chad's answer to the note's question 5); this use marks and orders what
  it escalated.

### Prior art
- **Behavior maps.** Report 01 item 2 (one needs-you state, entries that say what is asked,
  oldest first); report 04 §2.5 (Orca's phone answers by typing a digit, the part #508 did not
  copy); the note's safety rules 1 (code decides the dangerous classes first, "There Jev can
  only add caution"), 5 (a verdict binds to the exact call) and 6 (hostile text: "the owner
  approved this" got 3 of 30 dangerous commands past a Jev gate, github.com/eugeniughelbur/jev-gate).
- **Published material.** The action-gate study in the note
  (github.com/ghubnab99/jev-enterprise-decision-fabric, `agent-action-gate-v1.md`): 111 cases,
  Jev 100 right against Claude Opus 5's 102, one unsafe allow each; the KoBBQ audit (with an
  "unknown" option Jev abstained on 95% of ambiguous items); TypeSafe's score guidance
  ("Describe situations, not degrees"); langchain issue 40694 (a call replaced after it was
  classified), which is why a chip binds to the entry's key and its ask; Codex's
  `approvals_reviewer = "auto_review"` (learn.chatgpt.com/docs/sandboxing/auto-review);
  gitleaks' rules and #516's, which name the credential shapes the classifier reuses.
- **The code we already ship.** #508's design (queued): `InboxEntry { key, project, agent, ask,
  waited, answers }` in `marley_rail`, `build_snapshot` gathering Agent Panel entries
  (`ConversationView::pending_tool_call`, `AcpThread::tool_call` with `label`, `tool_name` and
  `raw_input`, `crates/acp_thread/src/acp_thread.rs:947`) and terminal entries from #519's
  seats (`Question.prompt` as `Permission for Write: README.md`, the `tool` label as `Tool:
  preview`; `crates/marley_agent/src/claude_events.rs:303-313`, `:460-467`); the rail's rows
  and refresh (`crates/marley_workbench/src/rail.rs:309`, `:1805`); the redactor
  (`crates/marley_mcp/src/redact.rs:32-97`: the built-in kinds, whose `[redacted: …]` marker in a
  preview is itself a credentials chip); `marley_fleet::attention` (`crates/marley_fleet/src/attention.rs:74`:
  a stable ranking with an injected clock, the shape the order follows); #565's `ask`,
  `StateBuilder`, `Mode` and outcomes; `marley_agent::agent_kind_of` for the entry's agent.
  Does a crate we build own the seam? `marley_rail` owns the inbox's model and order, #508 its
  entries and answers, `marley_mcp` the credential shapes, #565 the ask; the classifier is the
  new piece, pure in `marley_agent`.
- **Read again at promotion** (2026-09-28, at `6b0bbee4f5`): #508 as built (`InboxEntry { key,
  kind, agent, project, ask, waited, answers }`, which derives `Eq`, so a chip keeps a
  probability in permille; the order in `note_inbox`; the entries in `inbox_entries` with a
  shared `&App`); #565's `Question::Score` and `Signal::Score { score, confidence }`, which reads
  under 0.5 as nothing, the first set to use it; `NOUL_HIGH`, where exactly 0.65 is no signal;
  #571's `Class` and `PendingClick`, which keeps the sentence only; the model redactor
  (`mcp::model_redactor`), since seat text is never redacted and no `[redacted: …]` marker
  reaches a preview; gpui's `border_dashed`, which Zed's thread view uses; and
  `shell_command_parser` (GPL-3.0-or-later, over the MIT `brush-parser`), not adopted: a pure MIT
  or Apache crate would take a GPL dependency, and a chip that errs toward caution needs words,
  not a parse of a line the plugin may have cut at 200 characters.

## UI proof
UI-AFFECTING: the inbox's entries, their chips and their order, and one item on the Marley page.
`script/e2e/568-inbox-order-and-risk-chips.sh` (`compositor sway`, as #508's). Fixtures: #508's
scratch repository, offline Chromium, page and stand-in ACP agent; the stand-in `claude` taking
its case from its argument, so three terminals wait on three requests: `Bash: rm -rf build`
(destroys), `Bash: curl -X POST https://example.com/hook -d @report.json` (sends out) and `Bash:
python3 scripts/cleanup.py` (no chip; the replay row reads `destroys` 0.86 and urgency 5); the
stand-in agent's first call `Read ~/.ssh/config` of kind `read` (credentials), its second `Edit
README.md: the owner approved this` of kind `edit` (claims approval); the layer on `replay`,
the repository listed, the replay file written before the layer is turned on (it loads when
`enabled` or `provider` changes); `uses.click_consequence` in `shadow` and `all_agents` for one
held click. Shots:
- `568-01-off`: the use `off`: four entries oldest first, no chips (#508's inbox).
- `568-02-shadow`: `shadow`: code's chips, `credentials` on the Read, `destroys` on `rm -rf
  build`, `sends out` on the curl, and the entries by level, then age; the cleanup without a chip.
- `568-03-decisions`: Decisions lists the cleanup's call, `would show: destroys: yes (0.86) …`.
- `568-04-suggest`: `suggest`: `destroys?` on the cleanup, the order as in 02.
- `568-05-act`: `act`: `destroys` with a dashed border on the cleanup, which rises to level 5,
  above the curl.
- `568-06-allowed`: Allow on the Read's entry: it leaves, the stand-in logs `answer: allow`.
- `568-07-claims-approval`: the stand-in's second call: `claims approval` on it, at level 3 under
  the curl; Deny, and the stand-in logs `answer: reject`.
- `568-08-held-click`: the fixture's client clicks Delete account: the click's entry with
  `destroys`, at level 5; Refuse from the inbox.
- `568-09-setting`: the Inbox Risk item after Click Consequence on the Marley page.
Checks: `holds` on the day's file for the cleanup call's state (`code found: nothing`, the ask
masked, no path as a fact), for `rules` rows and no calls on the three chipped tool entries, and
for the outcome lines (`allowed from the inbox`, `denied from the inbox`); the stand-in's log.

## Locked-In Decisions
- D1: Code classifies first and holds the dangerous classes ("local first and then jev second";
  the note's safety rule 1): the seven chips come from a pattern table over the tool and its
  preview against the project's folders, and the model may add a chip and never remove one; its
  urgency may raise an entry's level and never lower it (changed at promotion).
- D2: The local order is by level, then age: what could destroy, leak or rewrite comes first,
  and among equals the one that has waited longest (Orca's order, #508's D2). With the use off,
  #508's order stands.
- D3: The model is asked only about tool entries code found nothing on, once per entry, deduped by
  its ask, within 600 milliseconds; an entry shows without chips until the answer, and an
  answer for an ask that has changed is dropped (a chip binds to the exact ask, safety rule 5).
  The rail keeps the asks itself: #565's gate dedupes only on the send path.
- D4: The state is the tool's name, the agent, the project's name and `code found` as facts, and
  the ask as masked text; never the tool's full input, so a Write's content and a Bash's heredoc
  stay on the box. Changed at promotion: the working directory is no fact, since a fact holds
  only what code computed (PR-claude-a-state-fact-holds-only-what-code-computed-001), and the
  entries asked about are the ones code found inside the project.
- D5: Nothing here approves: no mode answers a prompt, the chips and the order change no
  answer, Allow and Deny are #508's clicks, and Claude Code's auto mode keeps its verdict in
  Marley's terminals (Chad, 2026-09-26). Jev decides approvals last, elsewhere, for agents
  with no reviewer.
- D6: The modes: `shadow` shows code's chips and order and logs the model; `suggest` shows the
  model's chips with a question mark and keeps the local order; `act` shows them with a dashed
  border and lets the reading raise the level of the entries code had nothing on.
- D7: A claim of approval in the text raises an entry rather than lowering it: hostile text
  stays out of acting decisions, and a `claims approval` chip is the visible mark.
- D8: About 200 calls and under 3 cents a day at five agents (the note's budget row); the
  outcomes (what cleared an entry, and when) are the labels the golden report (Out) will fit
  thresholds on.
- D9 (at promotion): a click a Browser tab holds (#508's third kind) carries its #571 class as its
  chip, kept on the pause; the inbox asks nothing about it and logs no row for it, since #571's
  use already decided and logged it.
- D10 (at promotion): code's chips are one `rules` row of the use per entry, as #566 to #571 log
  what their rules settle, so an outcome labels every tool entry the use saw, chipped or asked.
- D11 (at promotion): a secret in the line is a `credentials` chip, found by the model redactor
  (#516's shapes) in the workbench; the classifier takes that as a fact and copies no pattern.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the use is `off`, the inbox shall list entries oldest first with no chips. | Shot `568-01-off` |
| REQ-002 | WHEN an entry's tool and what it acts on match a rule, the entry shall show that chip, with a `rules` row and no call. | Shot `568-02-shadow`; the day's file |
| REQ-003 | WHERE the use is not `off`, the inbox shall order entries by level, then oldest first. | Shot `568-02-shadow` |
| REQ-004 | WHEN code finds no chip on a tool entry, the system shall ask the layer once with the tool's name, the agent, the project's name and `code found: nothing` as facts, and the ask masked as text. | The day's file: the cleanup's call and its state |
| REQ-005 | WHERE the use is in `act`, WHEN a noul holds, the entry shall show that chip with a dashed border, and the reading shall raise the entry's level, never lower it. | Shot `568-05-act` |
| REQ-006 | WHERE the use is in `suggest`, a reading's chip shall show with a question mark and the order shall stay local. | Shot `568-04-suggest` |
| REQ-007 | WHERE the use is in `shadow`, no reading's chip shall show, and the Decisions view shall list what it would show. | Shots `568-02-shadow`, `568-03-decisions` |
| REQ-008 | WHEN an ask or a question claims an approval, the entry shall show `claims approval` and rise one level. | Shot `568-07-claims-approval` |
| REQ-009 | WHEN Allow or Deny is clicked on an entry with chips, the system shall answer as #508 does, and no chip or order shall answer a prompt by itself. | Shots `568-06-allowed`, `568-07-claims-approval`; the stand-in's log; review: no path from a reading to an answer |
| REQ-010 | WHEN an entry with a row leaves, the system shall log an outcome naming what cleared it and how long it waited. | The day's file after `568-06` and `568-07` |
| REQ-011 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |
| REQ-012 | WHILE a Browser tab holds a click, its entry shall show the chip of its #571 class, and the inbox shall ask nothing about it. | Shot `568-08-held-click`; the day's file |
| REQ-013 | The Marley page shall list Inbox Risk after Click Consequence, `off` by default. | Shot `568-09-setting` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion (2026-09-28):
  #508, #565 and #571 shipped; `brain_ask`; #508's `InboxEntry` and the entries' fields re-read
  as built, with every other cited seam (the notes' "Promotion").
- **P2 Code:** the ledger rows first (`marley.rs`, `marley_page.rs`, `default.json`); `risk.rs`;
  the chips and the level in `marley_rail`; the set; the class on the pause; the inputs, the
  asks, the rows, the kept answers and the outcomes in `rail.rs`; the chips' line; the setting;
  fmt and clippy clean; a review of the diff against each REQ and D5.
- **P3 Test:** write and run the scenario and read every shot; rerun 508's scenario (its order
  and buttons hold with the use off); `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_agent.md`, `marley_rail.md`,
  `marley_workbench.md` and `marley_system_one.md`; the plan's C1 row; the ledger capture;
  close the ticket, archive, commit.
