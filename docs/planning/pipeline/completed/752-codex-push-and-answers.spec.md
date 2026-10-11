---
pipeline_id: 80736367-3f05-4429-a1b2-9f04258d8ffd
ticket: docs/planning/tickets/open/TICKET-752-codex-push-and-answers.md
status: Phase 4 — Complete PASS
title: Codex push and answers
type: feature
slice: prong 2; T3 Code survey item 5 ("Codex gets what Claude Code gets"), first half
references:
  - docs/t3code_architecture/README.md
  - docs/t3code_architecture/04-remote-and-mobile.md
  - docs/t3code_architecture/01-agents-and-providers.md
  - docs/planning/pipeline/completed/535-phone-push-notifications.spec.md
  - docs/planning/pipeline/completed/650-codex-app-server-state.spec.md
  - docs/planning/pipeline/completed/651-codex-approvals-and-prompts.spec.md
  - docs/planning/pipeline/completed/734-an-agent-thread-starts-in-a-tab.spec.md
---

## Title
Codex gets the phone push and the in-place answers Claude Code and Codex's approvals already have.
A Codex on its own App Server (#650) and a Zed agent thread push a line to the phone when they
need an approval, need input, finish or fail, each kind behind its own switch after T3 Code's.
Codex's own questions and its app-access requests are answered from Needs you through the App
Server, as #651 answers its approvals. Chad, 2026-10-11: "3.) yes".

## Scope
### In
- **Push for Codex.** A Codex seat fed by its App Server (#650) pushes on the same seat changes a
  Claude Code seat pushes on (`TurnEvent::of_change`): into waiting, a turn's end, a failure.
  `fold_codex` hands back the seat before and after, as `apply_report` does, and the follow calls
  `push::on_change` for the terminal's view in its window. The line names `Codex` and the project
  from the thread's `cwd`.
- **Push for Zed agent threads.** A thread in the Agent Panel or in a thread tab (#734) pushes when
  Zed's own `ConversationView` decides the user should hear of it (`notify_with_sound`): a tool's
  permission, an elicitation, a finished turn, an error, a refusal, a stop at the output limit. A
  Marley hook on `MarleyThreadHost` receives the kind, whether the thread is visible, the agent's
  name and the thread's folders. A subagent's finish or error pushes nothing, as Zed notifies
  nothing for them; its permission request does, as Zed's banner does.
- **Four kinds, four switches**, after T3 Code's `RelayAgentAwarenessPreferences`
  (`packages/contracts/src/relay.ts:33-40`): `approval`, `input`, `finish`, `failure`, four
  booleans in `marley.push`, each on by default. A Claude Code permission is an approval and its
  `AskUserQuestion` an input; a Codex wait reads its `wait` label (`approval` or `input`); a Zed
  thread's tool permission is an approval and its elicitation an input. The Push section of the
  Marley settings page shows the four toggles.
- **The line**, still #535's D2: `<project>: <agent> <event>`, with the words `needs approval`,
  `needs input`, `finished` and `failed`. `<agent>` is `Claude`, `Codex`, or a Zed thread's agent
  name. Nothing the agent wrote. #538's desktop banner title, which uses the same `event_line`,
  says `needs approval` for a permission too.
- **Codex's questions in Needs you.** `item/tool/requestUserInput` on the lead thread becomes an
  entry, "Codex · <project>", in place of #650's "Waits on an answer":
  - one question with options and not secret: its header and question, and one button per option,
    in the request's order. A click answers `{"answers": {"<question id>": {"answers": ["<option
    label>"]}}}`, as Codex's TUI does (`tui/src/bottom_pane/request_user_input/mod.rs:853-886`).
  - anything else (several questions, no options, a secret question): the first question's text,
    no buttons; a click shows the terminal, where the TUI asks.
- **Codex's app-access requests in Needs you.** An `mcpServer/elicitation/request` whose form has
  no fields (`requestedSchema` empty or null, mode `form`), as Codex's TUI reads it
  (`tui/src/bottom_pane/mcp_server_elicitation.rs:240-330`):
  - a tool approval (`_meta.codex_approval_kind` `mcp_tool_call`): Allow, Allow session and
    Always allow as `_meta.persist` offers them, and Cancel;
  - any other: the same allows, Deny and Dismiss.
  Allow answers `{"action": "accept"}`; Allow session and Always allow add `"_meta": {"persist":
  "session"}` or `"always"`. An elicitation with fields, a URL or a user verification keeps #651's
  Deny and Dismiss.
- **#651's rules hold** for the new entries: the key holds the connection's generation and the
  request id, a click carries the request as rendered and is sent only while it is unresolved with
  the same params, once; the entry leaves on `serverRequest/resolved`, the thread's close or the
  connection's end; Marley never answers unasked and never with an error.
- `script/e2e/752-codex-push-and-answers.sh`; the stand-in Codex (`codex-fixture.sh`) gains cues
  for the question and the elicitation shapes.
- The guide: the push kinds and the Codex entries.

### Out (explicitly deferred)
- Grouping events from several projects into one push ("2 agents need you: marley_ide, rusty",
  report 04 item 1): a later slice. The 5-second cooldown per project stays.
- Codex's asynchronous questions (an `agentMessage` with `delivery: "async"` and `questions`):
  their answer is a user message, `turn/steer` or `turn/start`, and Marley sends neither yet.
  #715 brings `turn/start`.
- A free-text answer typed in the rail, and a form with fields: the TUI collects them.
- Codex in a terminal without its App Server: it has no seat, so nothing pushes. Its desktop
  notices stay Codex's own OSC 9 (#552).
- A desktop banner from a Codex seat: Codex prints its own (#552).
- A push's `Click` URL, answering from the phone, and taking a push back: #535's later slices.
- Harness sessions: the harness owns their notifications.
- A limited agent (#715).

## Reference (§20)
- **T3 Code** (MIT, read from its checkout at `17c0878941`, report 04 §2.12 and report 01 §2.9):
  a toggle per kind of alert (approval, input, completion, failure,
  `RelayAgentAwarenessPreferences`); Codex's blocking question answered with the chosen labels
  (`CodexAdapterV2.ts:5085-5140`, `toCodexUserInputAnswers` at `:591`); app-access elicitations
  offered once, for the session or always (`provider/CodexMcpElicitation.ts`). Marley keeps the
  kinds and the answers and pushes through ntfy on the box (#535), not a relay.
- **Upstream Zed**, `agent_ui::ConversationView`: which thread events notify the user and when a
  thread counts as seen (`notify_with_sound`, `agent_status_visible`), kept as they are; Marley's
  push follows that decision.
- **Codex's TUI** (Apache-2.0, upstream source at `rust-v0.155.1`): what a question and an
  elicitation offer and what each answer sends.
- **Warp:** N/A. Its phone path needs Warp's cloud, which the once-over ruled out
  (`docs/planning/design-notes/warp-once-over-2026-09-25.md`), as #535 records.

### Prior art
- **Behavior maps.** T3 report 04 §2.12 and §3 item 1 (the per-kind switches, the grouping left
  Out here, "keep text out"); report 01 §2.9 and §3 item 5 (Codex's two question forms, app-access
  persistence, "clear the entry when Codex's next turn starts" for async ones); Orca report 04 §2.6
  via #535 (the event kinds and the cooldown).
- **Published material.** Codex's App Server protocol generated from the installed 0.162.0 with
  `codex app-server generate-json-schema` (stable and `--experimental`, no server started):
  `ToolRequestUserInputParams` (`questions[]` with `id`, `header`, `question`, `isOther`,
  `isSecret`, `options[]` of `label` and `description`; `isBlocking`), its response `answers` map,
  `McpServerElicitationRequestParams` (modes `form`, `url`, `openai/userVerification`, `_meta`)
  and its response (`action`, `content`, `_meta`). The same names at 0.155.1:
  `app-server-protocol/src/protocol/common.rs:1769`, `v2/item.rs:1706-1770`,
  `protocol/src/mcp_approval_meta.rs` (`codex_approval_kind`, `mcp_tool_call`, `persist`,
  `session`, `always`). ntfy's publish API as #535 read it.
- **Code we already ship.**
  - `crates/marley_workbench/src/push.rs:53-103` (`on_change`, gated by the seat change, the
    setting, `looking_at` and the cooldown), `:89` (the hard-coded `AgentKind::Claude`),
    `:158-162` (priority and tag per event); `notifications.rs:116` (the only caller, Claude's
    frames), `:128` (`looking_at`).
  - `crates/marley_agent/src/marley_agent.rs:623-666` (`TurnEvent`, `of_change`, `words`,
    `event_line`), one function shared by the push and #538's banner.
  - `crates/marley_workbench/src/agent_events.rs:131-145` (`fold_codex`, which returns nothing
    today), `:151-225` (`apply_report`, which returns the change).
  - `crates/marley_workbench/src/codex_server.rs:856-866` (`Following::fold`), `:1198-1235`
    (`requested`: an undecoded method is left to the TUI, `:1202`), `:1237-1264` (`answered`),
    `:791` (`respond`).
  - `crates/marley_agent/src/codex_events.rs:599-660` (`RequestKind`, `Request`), `:663-718`
    (`decode`: four methods), `:722-760` (`decisions`: an elicitation offers decline and cancel),
    `:761-795` (`response`), `:503-528` (the wait label and "Waits on an answer").
  - `crates/marley_workbench/src/rail.rs:6688` (`inbox_entries`), `:6706-6728` (Codex request
    entries), `:7085` (`codex_entry`), `:7132-7143` (`inbox_answer`, `codex_decision`), `:2065`
    and `:2109-2131` (`render_inbox_entry`, the buttons), `:2255` (`answer_inbox`);
    `crates/marley_rail/src/marley_rail.rs:457`, `:472`, `:521` (`InboxKind`, `InboxAnswer`, a
    closed set of six, `InboxEntry`).
  - `crates/agent_ui/src/conversation_view.rs:1887-1892` (tool permission, elicitation),
    `:1952`, `:1969`, `:2000`, `:2024`, `:2042` (output limit, refusal, finished, refusal, error),
    `:3110-3121` (`notify_with_sound`), `:3122-3166` (`is_visible`, `is_visible_in_agent_panel`
    with Marley's `MarleyThreadHost::shows`, `agent_status_visible`);
    `crates/agent_ui/src/agent_ui.rs:517-527` (`MarleyThreadHost`).
  - `crates/acp_thread/src/acp_thread.rs:3531-3562` (`AcpThreadEvent`); the rail's own
    subscription (`rail.rs:2471-2481`) only redraws.
  - `crates/settings_content/src/marley.rs:54`, `:332-339` (`push`,
    `MarleyPushSettingsContent`); `crates/settings_ui/src/marley_page.rs:1116-1190` (the Push
    section).
  Nothing else owns this seam: no crate in `Cargo.lock` speaks Codex's protocol or ntfy, and Zed's
  notification path is a gpui popup with no hook for a second channel.

## UI proof
`script/e2e/752-codex-push-and-answers.sh`, under `compositor sway` (the entries' buttons are
clicked). Fakes only: #535's fake ntfy on a loopback port that logs each request, #650's stand-in
`codex` (`codex-fixture.sh`, `MARLEY_CODEX`, `marley.codex_app_server` on in the run's copy) with
new cues `ask <header>|<question>|<option>,<option>`, `ask2` (two questions), `askfree` (no
options), `toolok <server> <message> <persist>` (a tool approval with no fields) and `form <server>
<message>` (an elicitation with one field); #535's stand-in `claude`; #508's stand-in ACP agent
that asks for a tool's permission, in a thread tab. Marley's window moves off the active
workspace for each event that should push, and comes back for its shot. Shots:
- `752-01-codex-approval`: the Codex row waiting on `command rm -rf build`; the log's
  `repo: Codex needs approval`.
- `752-02-codex-input`: the row waiting on `ask`; the log's `repo: Codex needs input`.
- `752-03-codex-ends`: the row idle after `done`, then failed after `fail`; the log's `repo: Codex
  finished` and `repo: Codex failed`.
- `752-04-thread-approval`: a thread tab waiting on the stand-in agent's tool permission; the
  log's `repo: <agent> needs approval`.
- `752-05-claude-kinds`: the stand-in `claude` asks a permission, then an `AskUserQuestion`; the
  log's `repo: Claude needs approval`, then `repo: Claude needs input`.
- `752-06-switches`: the Push section with its four toggles, `finish` off; then a Codex `done`
  with the window away, and no request in the log.
- `752-07-question`: Needs you holds "Codex · repo", the question and two option buttons; the
  terminal shows the stand-in TUI's prompt.
- `752-08-answered`: the second option clicked: the entry gone, the TUI's "closed: answered
  elsewhere"; the server log's answer with that label.
- `752-09-question-in-terminal`: an `ask2` request: its first question, no buttons; the entry
  clicked, the terminal in front.
- `752-10-tool-approval`: a `toolok docs "Run search?" session,always` entry with Allow, Allow
  session, Always allow and Cancel.
- `752-11-allowed-for-session`: Allow session clicked; the server log's `accept` with `persist`
  `session`.
- `752-12-form`: a `form` request: Deny and Dismiss only.

## Locked-In Decisions
- **D1:** Codex pushes from its seat with the same `TurnEvent::of_change` Claude Code uses, so
  one function says what a seat change means for both
  (PR-claude-two-engines-answering-the-same-question-will-diverge-001). `fold_codex` returns the
  change as `apply_report` does. Rejected: a push decided in `codex_server` from the thread's
  notifications (a second reading of the same states).
- **D2:** A Zed thread pushes when Zed's `ConversationView` notifies, through one Marley hook in
  `notify_with_sound`: Zed already decides which events matter, skips subagents and a queued send,
  and knows whether the thread is visible in the panel or a tab. Rejected: Marley subscribing to
  every `AcpThread` and judging the events itself (a second engine of Zed's rules); the hook in
  `show_notification` (it returns early when a banner is up or `notify_when_agent_waiting` is
  `never`, which should not silence the phone).
- **D3:** Four kinds, after T3 Code's: approval, input, finish, failure, each a boolean in
  `marley.push`, on by default, so Claude Code's pushes go on as they do. Push as a whole stays
  off until `url` and `topic` are set (#535), which keeps it off outside the box by default.
  Rejected: a list setting (`notify_on`), which the settings page cannot draw as toggles.
- **D4:** The line keeps #535's D2: the project, the agent's short name and the event's words, no
  agent text. A permission's line becomes `needs approval`; a question's stays `needs input`. A Zed
  thread's agent is named by the name Zed shows for its agent server, which is configuration, not
  the agent's words.
- **D5:** A Codex seat's project is the lead thread's `cwd` (a required field of Codex's
  `Thread`), carried on the seat as `cwd`, as Claude Code's is. Without it the line would read
  `A terminal`.
- **D6:** A Codex question is answered in place only when it is one question with options and is
  not secret, by an option's label, as the TUI answers. Everything else shows its text and opens
  the terminal. Rejected: a text field in the rail (the TUI collects text, #651's D2); answering
  several questions from one row of buttons.
- **D7:** An elicitation is accepted in place only when it has nothing to fill in, the TUI's own
  approval mode, and only with the persistence `_meta.persist` offers. Always allow is the user's
  click on a choice Codex offered; Marley never picks it. #651's D2 still holds for a form with
  fields, a URL and a user verification.
- **D8:** No switch of its own for the answers: `marley.codex_app_server`, off by default, and
  #648's tested range govern them, as #651's D8.
- **D9:** The new entries reuse #651's table, generation, identity check and leaving rules
  (AD-claude-651-codex-approvals-answer-in-the-inbox-only-when-clicked-001); one table answers
  every Codex request.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Codex on its App Server, in a terminal the user is not looking at, starts waiting on an approval, the system shall post one push whose line is `<project>: Codex needs approval`. | Shot `752-01-codex-approval`; the fake's log |
| REQ-002 | WHEN such a Codex starts waiting on a question, the system shall post one push whose line is `<project>: Codex needs input`. | Shot `752-02-codex-input`; the log |
| REQ-003 | WHEN such a Codex's turn ends, the system shall post one push whose line is `<project>: Codex finished`. | Shot `752-03-codex-ends`; the log |
| REQ-004 | WHEN such a Codex's turn fails, the system shall post one push whose line is `<project>: Codex failed`. | Shot `752-03-codex-ends`; the log |
| REQ-005 | WHEN a Zed agent thread the user cannot see asks for a tool's permission, the system shall post one push whose line is `<project>: <agent> needs approval`. | Shot `752-04-thread-approval`; the log |
| REQ-006 | WHEN Claude Code asks for a permission, the system shall push `needs approval`, and WHEN it asks a question, `needs input`. | Shot `752-05-claude-kinds`; the log |
| REQ-007 | WHERE a kind's switch is off, the system shall post nothing for an event of that kind. | Shot `752-06-switches`; the log |
| REQ-008 | The Push section of the Marley settings page shall show a toggle for each of approval, input, finish and failure, each on by default. | Shot `752-06-switches`; review |
| REQ-009 | WHILE the user is looking at a Codex terminal or a Zed thread, the system shall post nothing for its events. | Review of the diff (the gates `looking_at` and `agent_status_visible`) |
| REQ-010 | Every push request shall carry the title, the one line, a priority and a tag, and nothing the agent wrote. | The log: every request in full |
| REQ-011 | WHILE a Codex question with options waits, Needs you shall list it with its question and one button per option. | Shot `752-07-question` |
| REQ-012 | WHEN an option is clicked, Marley shall answer that request once, with that option's label for that question. | Shot `752-08-answered`; the server log |
| REQ-013 | WHILE a Codex question Marley cannot answer in place waits, its entry shall show the question with no buttons, and a click shall show its terminal. | Shot `752-09-question-in-terminal` |
| REQ-014 | WHILE a Codex tool approval with no fields waits, its entry shall offer Allow, the persistence choices its `_meta.persist` offers, and Cancel. | Shot `752-10-tool-approval` |
| REQ-015 | WHEN Allow session or Always allow is clicked, Marley shall answer `accept` with `_meta.persist` `session` or `always`. | Shot `752-11-allowed-for-session`; the server log |
| REQ-016 | WHILE an elicitation with fields, a URL or a user verification waits, its entry shall offer Deny and Dismiss only. | Shot `752-12-form` |
| REQ-017 | WHEN the server reports a question or an elicitation resolved, its entry shall leave, whoever answered. | Shots `752-08-answered`, `752-11-allowed-for-session` |
| REQ-018 | Marley shall send no response to a request the user did not answer in Needs you, and none with an error. | The server log; review |
| REQ-019 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes: re-read #650's tested range
  against the installed Codex, the stand-in's cues, and every seam cited.
- **P2 Code** — `marley_agent` (`TurnEvent`'s four kinds, `event_line`'s agent name, the question
  and elicitation shapes in `codex_events`); `marley_workbench` (`push.rs`, `agent_events.rs`,
  `codex_server.rs`, `thread_tab.rs` for the host's hook, `rail.rs`); `marley_rail` (the answers);
  Zed paths with their ledger rows (`agent_ui.rs`, `conversation_view.rs`, `settings_content`'s
  `marley.rs`, `settings_ui`'s `marley_page.rs`); the stand-in's cues; the guide; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot and both logs.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
