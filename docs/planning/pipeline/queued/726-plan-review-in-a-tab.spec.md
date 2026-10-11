---
pipeline_id: 8a3f8df1-5c61-45a6-abb8-1d695b9f2836
ticket: docs/planning/tickets/open/TICKET-726-plan-review-in-a-tab.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Plan review in a center tab, slice 1 — Claude Code's plan from a terminal
type: feature
slice: the review batch after the finish (docs/planning/intake/plannotator-findings.md item 1; docs/planning/design-notes/finish-line-2026-10-11.md row 10); waits on a rustal-harness change to the shared plugin (D1)
references:
  - docs/planning/pipeline/completed/522-review-notes-to-the-agent.spec.md
  - docs/planning/pipeline/completed/652-shared-claude-plugin-marleys-half.spec.md
  - docs/planning/pipeline/completed/709-load-the-shared-claude-code-plugin.spec.md
  - docs/planning/pipeline/completed/724-the-shared-plugin-with-rustal-ste.spec.md
  - docs/planning/pipeline/completed/645-brain-page-tab.spec.md
---

## Title
When Claude Code in a Marley terminal finishes a plan and calls `ExitPlanMode`, the plan opens in a
center tab beside the terminal, drawn as Markdown. The user selects passages and comments on them,
adds an overall note, and answers Approve or Send Feedback; the answer goes back to the waiting
call. The plan also waits in Needs you. Off until turned on. Chad, 2026-10-11, on the finish-line
list: "10.) yes".

This is the first of four slices (the ticket lists them); #727 and #728 build on it.

## Scope
### In
- **The switch:** `marley.plan_review`, off by default, with a row in the Settings window's Agents
  section. On, Marley's new local terminals carry `MARLEY_APPROVE_TOOLS=ExitPlanMode`, which tells
  the shared Claude Code plugin (#709, loaded while `marley.claude_code_shared_plugin` is on) to
  hand that call to Marley. Off, nothing changes: Claude Code's own dialog decides.
- **`marley-agent approve`:** the verb the plugin runs, with `rh approve`'s arguments
  (`--source`, `--call`, `--tool`, the tool's input as JSON on stdin). It sends the request over
  Marley's agent socket and waits for the user's answer, then prints `{"decision": "approve"}` or
  `{"decision": "deny", "reason": "<feedback>"}` and exits 0. With no answer before its deadline
  (under the plugin's ten minutes), or with the request refused, it exits 1, and the plugin leaves
  the call to Claude Code's dialog (the harness request, D1).
- **The agent socket** answers an `approve` request when the user decides, not within its usual
  few seconds; the request is matched to its terminal as reports are (#652).
- **The tab** ("Plan", with the project's name): the plan drawn by Zed's `markdown` crate, HTML not
  parsed. It opens in the pane that holds the agent's terminal, beside it, and takes the focus only
  when that terminal has it.
- **Notes:** select a passage and choose Comment… from the right-click menu, or press `c`; type the
  comment and press Enter. Each note lists its quoted passage and its words in a column beside the
  plan, the passage stays highlighted, and an × removes a note. One overall note sits at the
  column's foot.
- **Approve** answers approve. **Send Feedback** (enabled once there is a note or an overall note)
  answers deny with the notes as the reason, in the form D6 gives.
- **After an answer** the tab stays and says Approved or Feedback sent, its buttons off. A later
  plan from the same Claude Code session, as after feedback, replaces the tab's plan and notes.
- **Needs you:** a plan waiting for review is an entry ("Plan to review", the agent and the
  project); a click brings the tab forward. It has no buttons in place (the plan must be read). It
  leaves when the plan is answered or withdrawn.
- **Withdrawn:** when the request goes away unanswered (the deadline, the agent ended, the terminal
  closed), the tab says the plan was left to the terminal's own dialog, its buttons off.
- `docs/marley/guide.md`: plan review, the switch, and that it needs the shared plugin.

### Out (explicitly deferred: the other slices and tickets)
- **Slice 2:** Delete and label marks on a passage, "Looks good", and editing a note.
- **Slice 3:** a plan in a Zed agent thread (Claude Code over ACP): the adapter shows
  `ExitPlanMode` as a permission request whose content is the plan; its answer carries no feedback
  text, so Send Feedback there is "No, keep planning" followed by a post into the thread (#706's
  path).
- **Slice 4:** Codex's plan. The App Server streams the plan (`item/plan/delta`,
  `turn/plan/updated`), but "Implement this plan?" is the TUI's own popup, not a server request, so
  Marley has nothing to answer; this slice starts with that question.
- Plan versions and their diff (#727); question cards (#728).
- A review longer than the plugin's ten minutes (Claude Code's newer flow that ends the turn and
  takes the decision later as a message); the permission mode after approval; remote terminals.

## Reference (§20)
N/A — Marley-specific: Zed shows an ACP agent's plan approval inline in its thread
(`agent_ui`'s permission buttons), not as a document to annotate, and has no Claude Code terminal
hook. The behavior follows plannotator (published docs, `docs.plannotator.ai`: the Claude Code
page, "Add Annotations and Send Feedback"): a `PermissionRequest` on `ExitPlanMode` holds Claude
while the plan is reviewed as rendered Markdown; selected text keeps its quote with the comment, a
global comment covers the whole plan; Approve lets Claude go on, Send Feedback returns the
annotations and Claude revises. Orca's chat surface shows "the plan for ExitPlanMode" on its
approval card (`docs/orca_architecture/01-agents-and-sessions.md`), and T3 marks a thread "Plan
Ready" (`docs/t3code_architecture/02-threads-and-composer.md`). No plannotator code is taken.

### Prior art
- **The code we ship:**
  - The shared plugin's `tool.check` (`crates/marley_workbench/claude_shared_plugin/hooks/register.js:256-276`):
    the harness path runs `rh approve --source --call --tool` with the input on stdin for up to ten
    minutes (`DECISION`, 33) and maps `{decision, reason}` (approve allows, deny denies with the
    reason, cancel also ends the turn); Marley's path leaves the dialog to decide and reports
    `waiting`. The files are carried unchanged by digest (`shared_plugin.rs:23, 130-156`,
    AD-claude-709-the-shared-plugin-is-carried-by-digest-and-off-by-default-001).
  - `marley-agent` (`crates/marley_workbench/bin/marley-agent`): `report` and `release` only,
    `DEADLINES` (22), one JSON line out and one in (113-130).
  - `marley_mcp::agent_socket` (`crates/marley_mcp/src/agent_socket.rs`): one connection per
    request on its own thread, a 64 KiB line (20), the answer within `ANSWER_TIMEOUT`, 3.5 s
    (26, 145-147); `AgentRequest::answer` takes any string (30-45).
  - `agent_reports.rs`: `take` answers on the main thread (180-194), `terminal_of` matches the
    process chain and `MARLEY_TERMINAL_ID` (198-214); `marley_agent::report::Request` (373-390).
  - `marley_terminal::identity::agent_environment` (`identity.rs:67-84`) sets `MARLEY_BIN` and the
    plugin folders for local terminals.
  - Zed's `markdown` crate: the selection is a source byte range (`markdown.rs:1438-1445`), read
    only as text (`selected_source`, 1107) — no public range; a left click outside the element
    clears it before a button's handler runs (2434-2438); `set_search_highlights` paints source
    ranges (1114). `rusty/page.rs` draws a brain page with it (338-351, 1545-1603;
    AD-claude-645-a-brain-page-is-drawn-by-zeds-markdown-after-marleys-wikilink-pass-001).
  - The inbox: `InboxKind`, `InboxAnswer`, `InboxEntry` (`crates/marley_rail/src/marley_rail.rs:457-546`);
    `inbox_entries`, `codex_entry`, `open_inbox_entry`, `answer_inbox`, `observe_marks`
    (`crates/marley_workbench/src/rail.rs:6688, 7085, 1948, 2255, 829`); Codex's requests kept in
    a global the rail observes (`codex_server.rs:117, 141`).
  - Center tabs: `threads_page::open` (`threads_page.rs:47-60`), `home_page::ensure` (40-61); every
    center tab gets a rail row (`rail.rs:8790-8846`,
    AD-claude-674-every-center-tab-has-a-rail-row-and-files-fold-under-files-001).
  - Zed's ACP path for comparison: the Claude adapter turns `ExitPlanMode` into a `switch_mode`
    tool call whose content is the plan, with "Yes, …" and "No, keep planning" options
    (`claude-agent-acp`, `tool-calls/reporters/interaction.js`, `permissions/options/tools.js`);
    `SelectedPermissionOutcome` carries no free text (`crates/acp_thread/src/acp_thread.rs:1762`).
- **Behavior maps:** `docs/orca_architecture/01-agents-and-sessions.md` (the approval card with the
  plan); `docs/t3code_architecture/02-threads-and-composer.md` (Plan Ready);
  `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (items and panes).
- **Published material:** Claude Code's hooks (`PermissionRequest`, `ExitPlanMode`'s `plan`
  input), plannotator's docs (above).

## UI proof
`script/e2e/726-plan-review-in-a-tab.sh`, under `compositor sway`, with `marley.plan_review` on.
The shared plugin is not loaded: in the project's terminal a stand-in `claude` plays the plugin's
part. It sends `"$MARLEY_BIN" report --source mod:claude-code --seq 1 idle` as the plugin does at
the session's start, prints `MARLEY_APPROVE_TOOLS`, then runs `"$MARLEY_BIN" approve --source
mod:claude-code --call toolu_1 --tool ExitPlanMode` with a three-step plan on stdin and prints the
answer and the exit code; on a deny it sends a revised plan as `toolu_2`.

Shots:
- `726-01-env`: the terminal: `MARLEY_APPROVE_TOOLS=ExitPlanMode`.
- `726-02-tab`: the Plan tab beside the terminal, the plan drawn, Approve and Send Feedback; Needs
  you lists "Plan to review".
- `726-03-note`: a passage dragged over, `c`, a comment, Enter: the note in the column with its
  quote, the passage highlighted; an overall note typed.
- `726-04-feedback`: Send Feedback: the terminal shows `"decision": "deny"` and the reason with the
  quote, the comment and the overall note; the Needs you entry gone.
- `726-05-revised`: the revised plan in the same tab, notes cleared, the buttons on again.
- `726-06-approved`: Approve: the terminal shows `"decision": "approve"` and exit 0; the tab says
  Approved.
- `726-07-off`: `marley.plan_review` turned off, a new terminal: `MARLEY_APPROVE_TOOLS` unset.

## Locked-In Decisions
- **D1:** the plan reaches Marley through the shared plugin's `tool.check` and a Marley verb with
  `rh approve`'s contract. The plugin is rustal-harness's, carried unchanged, so this slice names a
  **harness request**: *under Marley's host, a call whose tool `MARLEY_APPROVE_TOOLS` names
  (comma-separated) goes to `$MARLEY_BIN approve` with the arguments, stdin and answer `rh approve`
  takes, within the same ten minutes, and the session reads waiting meanwhile; unlike the harness
  path, a failure, a timeout or a non-zero exit returns Claude Code's own verdict, so its dialog
  decides.* When it ships, Marley carries the plugin at that revision as #724 did. Rejected: a
  blocking `PermissionRequest` hook in Marley's own plugin (a second hook path beside the one the
  shared plugin owns, AD-709, which would run twice once the plugins fold); the ACP permission
  request (slice 3: no feedback text).
- **D2:** off by default, behind `marley.plan_review`; on, the variable names `ExitPlanMode` only.
  Chad's rule: each CLI keeps its own prompts unless he opts in.
- **D3:** Marley answers within the plugin's ten minutes (`marley-agent` gives up first); a plan
  left longer goes back to Claude Code's dialog, which shows it in full, so nothing is lost.
  Rejected for now: the flow that ends Claude's turn and delivers the decision later as a prompt
  (it depends on Claude calling `ExitPlanMode` again after an approval).
- **D4:** the tab opens beside the agent's terminal without taking the focus unless the terminal
  has it; Needs you carries it otherwise. A Needs you entry opens the tab and answers nothing in
  place (AD-claude-508-one-inbox-lists-every-agent-that-waits-on-the-user-001: only what needs no
  further choice is answered in place, and a plan needs reading).
- **D5:** a note starts from the right-click menu or `c`, because a click outside the Markdown
  element clears its selection before a button sees it. One Zed touch in `crates/markdown`:
  `Markdown::selected_range()`, public, beside `selected_source()`; the highlights use the existing
  `set_search_highlights`.
- **D6:** the feedback, built in `marley_agent` (pure): "The user reviewed your plan and asks for
  changes. Address every note below, then call ExitPlanMode again with the revised plan.", then
  each note as `Note N`, `Quote: "<passage>"`, `Comment: "<words>"`, then `Overall: "<words>"`,
  escaped as `review_prompt` escapes.
- **D7:** an answer goes once, only to the request the tab shows (its terminal, its call id and its
  connection), and a withdrawn request takes no answer
  (PR-claude-async-answer-carries-question-identity-every-hop-001).
- **D8:** the plan is drawn with HTML not parsed, so nothing in it is hidden from the reviewer
  (F-claude-592-the-launch-approval-was-rendered-as-markdown-001).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.plan_review` is on, the system shall start new local terminals with `MARLEY_APPROVE_TOOLS=ExitPlanMode`. | Shot 726-01 |
| REQ-002 | WHILE `marley.plan_review` is off, the system shall start new terminals without `MARLEY_APPROVE_TOOLS`. | Shot 726-07 |
| REQ-003 | WHEN an agent in a Marley terminal runs `marley-agent approve` for `ExitPlanMode`, the system shall show the plan as Markdown in a tab beside that terminal, with Approve and Send Feedback. | Shot 726-02 |
| REQ-004 | WHILE a plan waits for an answer, Needs you shall list it, and a click on the entry shall bring its tab forward. | Shot 726-02; review of the diff (the click) |
| REQ-005 | WHEN the user selects a passage and chooses Comment or presses `c`, then enters words, the system shall list a note with the quoted passage and highlight the passage. | Shot 726-03 |
| REQ-006 | WHEN the user chooses Send Feedback, the system shall answer the waiting call with deny and a reason holding every note's quote and comment and the overall note, and the entry shall leave Needs you. | Shot 726-04 |
| REQ-007 | WHEN the same session sends a new plan, the system shall show it in the same tab with no notes. | Shot 726-05 |
| REQ-008 | WHEN the user chooses Approve, the system shall answer the waiting call with approve, and the tab shall say Approved. | Shot 726-06 |
| REQ-009 | WHEN a waiting request goes away unanswered, the system shall mark the tab as left to the terminal's dialog, turn its buttons off and remove the Needs you entry. | Review of the diff |
| REQ-010 | IF no answer comes before `marley-agent`'s deadline, THEN `marley-agent approve` shall exit 1 with nothing on stdout. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes; the harness request sent to
  rustal-harness (through Chad) before Code, and its state checked at promotion.
- **P2 Code** — the touchpoint rows first; `Markdown::selected_range`; the switch (settings content,
  `MarleySettings`, the Settings window row); `MARLEY_APPROVE_TOOLS` in `agent_environment`;
  `marley-agent approve`, `Request::Approve`, the socket's per-request deadline; a new
  `plan_review.rs` (the pending plans, the tab, the notes, the answers); the inbox entry; the
  feedback in `marley_agent`; the guide; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot. Once the harness
  ships its change, one run with the real Claude Code in plan mode, recorded in the notes (§7: what
  no scenario reaches is taken as far as it goes and said).
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
