# Codex push and answers — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-752-codex-push-and-answers.md
- **Pipeline spec:** 752-codex-push-and-answers.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, answering the T3 Code survey's item 5 ("Codex gets what Claude
  Code gets", `docs/t3code_architecture/README.md`): "3.) yes". This ticket is that item's first
  half: push for Codex and Zed's agent threads with per-kind switches, and Codex's own questions
  and app-access requests answered from Needs you. #753 is the second half (turn checkpoints).
  Queued after the tabs batch, #743 to #749.
- **Classification:** feature, prong 2. Marley crates (`marley_agent`, `marley_workbench`,
  `marley_rail`) and four Zed paths that already carry Marley rows (`agent_ui.rs`,
  `conversation_view.rs`, `settings_content`'s `marley.rs`, `settings_ui`'s `marley_page.rs`).
  Size M, at the top of M: two paths (push, answers) with one scenario. If the builder wants it
  smaller, the split is push (REQ-001 to REQ-010) first, then the answers (REQ-011 to REQ-018).
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.

### Recall (§18.3)
- **Ledger.**
  - AD-claude-535-agent-events-reach-the-phone-as-one-line-through-ntfy-on-the-box-001: one line,
    nothing the agent wrote, the seat change as the trigger, `looking_at` as the gate, 5-second
    cooldown, loopback URL, 0600 token. Kept whole; this ticket widens the sources and adds kinds.
  - AD-claude-651-codex-approvals-answer-in-the-inbox-only-when-clicked-001: the table, the
    generation, the identity check, `serverRequest/resolved` as the only evidence, never an error.
    The question and elicitation entries ride the same table.
  - AD-claude-650-marley-joins-a-codex-app-server-of-its-own-per-terminal-001 and
    L-claude-650-a-codex-thread-resume-renames-its-client-001: the lead only; nothing sent that
    changes a thread except an answer the user clicked.
  - PR-claude-two-engines-answering-the-same-question-will-diverge-001: D1 (one `of_change` for
    both agents) and D2 (Zed's notify path, not a Marley reading of `AcpThreadEvent`).
  - PR-claude-async-answer-carries-question-identity-every-hop-001 and
    BF-claude-answer-race-no-question-identity-001: an option click carries the rendered request.
  - L-claude-535-a-setup-wait-loop-under-set-e-exits-without-a-word-001: the fake ntfy's port wait
    in `setup` takes `|| true`.
  - L-claude-508-a-stand-in-acp-agent-asks-for-permission-001: the Zed thread's permission comes
    from a stand-in ACP agent.
- **Completed pipelines.** #535 (push), #538 (the banner shares `event_line`), #650 (the App Server
  and its stand-in), #651 (approvals, its D2 on elicitations, its Out naming questions and the
  prompt slice), #552 (Codex's own OSC 9 notices), #734 (`MarleyThreadHost`), #508 (the inbox).
- **Brain:** no `rusty` MCP server in this repository's sessions; the promotion runs `brain_ask`.

### Discovery
- **Push today.** `push.rs:53-103`: `on_change(view, before, seat, window, cx)` maps
  `TurnEvent::of_change`, reads `MarleySettings::push`, gates on `notifications::looking_at`,
  checks the URL, keys the cooldown on `project_name(seat)` (the seat's `cwd` label, `:136-144`),
  and builds the line with `AgentKind::Claude` (`:89`). Its one caller is the `marley-event` frame
  branch in `notifications.rs:105-120`.
- **Codex seats.** `codex_server.rs:856-866` folds each input through
  `agent_events::fold_codex` (`agent_events.rs:131-145`), which applies the fleet events and
  returns nothing. The follow runs in an `AsyncApp` with the view's `EntityId`
  (`view_of`, `codex_server.rs:879-886`), not its window. Codex seats carry `thread`, `wait`,
  `tokens`, `sandbox` and `approval` labels (`codex_events.rs:25-45`), no `cwd`; Codex's `Thread`
  carries `cwd` as a required field (0.162.0 schema), which Marley's `Thread` does not read
  (`codex_events.rs:50-68`).
- **Kinds.** Claude Code's seat waits with `tool` = `AskUserQuestion` for a question
  (`claude_events.rs:511`) and after `PermissionRequest` otherwise (`:442`). Codex's seat has
  `wait` = `approval` or `input` (`codex_events.rs:503-504`). `TurnEvent` has `NeedsInput`,
  `Finished`, `Failed` (`marley_agent.rs:623-655`).
- **Zed threads.** `ConversationView::handle_thread_event` calls `notify_with_sound` for a tool
  permission (`conversation_view.rs:1888`), an elicitation (`:1892`), a stop at the output limit
  (`:1952`), a refusal stop (`:1969`), a finished turn unless a queued message was sent (`:2000`),
  a refusal (`:2024`) and an error (`:2042`), skipping a subagent's stops and errors.
  `notify_with_sound` (`:3110-3121`) plays the sound and calls `show_notification`, which returns
  early when a banner is up or the setting is `never` (`:3184-3215`). `agent_status_visible`
  (`:3154-3166`) is the visibility, and already asks `MarleyThreadHost::shows` (`:3133-3141`).
  The thread's folders are `root_thread.work_dirs()` (`:3208`).
- **Inbox.** `rail.rs:6706-6728` lists the published Codex requests; `codex_entry` (`:7085`)
  draws the ask and the buttons from `Request::decisions`; `inbox_answer` and `codex_decision`
  (`:7132-7143`) map the six `InboxAnswer`s (`marley_rail.rs:472`) to Codex decisions;
  `answer_inbox` (`:2255`) sends through `codex_server::answer` (`codex_server.rs:149`).
  `Request::decode` (`codex_events.rs:663-718`) knows four methods; `requestUserInput` is left to
  the TUI (`codex_server.rs:1202`), and the seat reads "Waits on an answer" (`codex_events.rs:528`).
- **Codex's shapes** (generated from the installed 0.162.0, and read at `rust-v0.155.1`):
  - `item/tool/requestUserInput`: `{threadId, turnId, itemId, questions[{id, header, question,
    isOther, isSecret, options[{label, description}]}], isBlocking}`; the answer
    `{answers: {<id>: {answers: [<label>, "user_note: …"]}}}`
    (`tui/src/bottom_pane/request_user_input/mod.rs:853-886`).
  - `mcpServer/elicitation/request`: `{threadId, turnId?, serverName}` plus one of `form`
    (`message`, `requestedSchema`, `_meta`), `openai/form`, `openaiForm`, `url` (`url`,
    `elicitationId`) or `openai/userVerification` (`challenge`). The TUI's approval mode is a
    message-only schema (`mcp_server_elicitation.rs:249-262`); its options are Allow, Allow for
    this session and Always allow as `_meta.persist` lists `session` and `always` (`:280-307`,
    `:413-429`), then Cancel for a tool approval or Deny and Cancel otherwise (`:308-330`); the
    answer is `accept` with `_meta: {persist: …}` for the persistent ones (`:1172-1195`).
  - Both methods carry the same names at 0.155.1 (`app-server-protocol/src/protocol/common.rs:1769`,
    `v2/item.rs:1706-1770`, `protocol/src/mcp_approval_meta.rs`).
- **Settings.** `MarleyPushSettingsContent { url, topic, token_file }`
  (`settings_content/src/marley.rs:332-339`); `PushSettings` in `MarleySettings`; the Push section
  (`marley_page.rs:1116-1190`) has three text fields.
- **The box.** Codex 0.162.0 and Claude Code 2.1.296 are installed. #650's tested range for the
  App Server is 0.155.1 up to 0.158.1 (`marley_agent/src/versions.rs:103-114`), so on this box the
  App Server stays off unless the range is widened or allowed; `marley.codex_app_server` and
  `marley.push` are both unset in Chad's settings.
- **Line numbers** in `rail.rs` are at `19dda499a2`; #743's Code phase was editing `rail.rs` while
  this was drafted, so the promotion reads them again.

### Design
- **`marley_agent`:**
  - `TurnEvent`: `NeedsApproval`, `NeedsInput`, `Finished`, `Failed`; `of_change(before, after,
    waits_on)` where `waits_on` says approval or input; `kind()` gives the switch's name.
    `event_line(project, agent: &str, event)` takes the agent's short name, so a Zed thread's name
    fits; `AgentKind::short_name` stays for the two CLIs. #538's banner calls it unchanged in shape.
  - `codex_events`: `Thread.cwd`; the `Lead` input sets the seat's `cwd` label
    (`claude_events::CWD_LABEL`). `RequestKind::Question { item, questions }` and
    `RequestKind::Elicitation` gains `form` (fields or not), `tool_approval`, and `persist` (the
    offered `session`, `always`). `Request::decisions` and `response` grow the new answers:
    `Decision::Option(index)`, `Decision::AcceptPersist(session | always)`.
- **`marley_rail`:** `InboxAnswer` gains `Option(SharedString)` (an option's label) and
  `AlwaysAllow`; the entry's buttons wrap as today.
- **`marley_workbench`:**
  - `agent_events::fold_codex` returns `Option<(State, Session)>`, as `apply_report` does.
  - `codex_server`: after a fold that moved the seat, the follow finds the window that holds the
    view (the windows' `MultiWorkspace`s, as `mcp::terminals` walks them) and, in it, calls
    `push::on_change` with the view. A view with no window pushes as one the user is not looking at.
    `requested` decodes the two new shapes into the same table.
  - `push.rs`: the kind from `TurnEvent::of_change` and the seat (Claude's `tool`, Codex's
    `wait`); a kind switched off posts nothing; `on_change` takes the agent from the seat
    (`agent_events::seat_agent`). A new `on_thread(kind, visible, agent, folders, window, cx)` for
    Zed threads goes through the same gate, cooldown and post.
  - `thread_tab.rs`: sets the host's new hook to `push::on_thread`.
  - `rail.rs`: `codex_entry` draws a question's header, question and options, or its text alone;
    an elicitation's new buttons; `inbox_answer` and `codex_decision` map them.
  - The guide: the push kinds; Codex's questions and tool approvals in Needs you.
- **Zed paths** (each with its `docs/marley/zed-touchpoints.md` row widened first):
  - `crates/agent_ui/src/agent_ui.rs`: `MarleyThreadHost` gains `attention: fn(MarleyAttention,
    bool, &str, &[PathBuf], &mut Window, &mut App)` and the four-variant `MarleyAttention` enum
    (approval, input, finish, failure).
  - `crates/agent_ui/src/conversation_view.rs`: `notify_with_sound` takes the attention kind and,
    before the sound, calls the host's `attention` with `agent_status_visible` and the root
    thread's agent name and folders (`// Marley: the phone push, #752`). Its seven callers pass
    their kind. About fifteen lines.
  - `crates/settings_content/src/marley.rs`: `approval`, `input`, `finish`, `failure:
    Option<bool>` on `MarleyPushSettingsContent`.
  - `crates/settings_ui/src/marley_page.rs`: four toggles in the Push section.
- **File manifest:** `marley_agent/src/marley_agent.rs`, `marley_agent/src/codex_events.rs`
  (Marley); `marley_rail/src/marley_rail.rs` (Marley); `marley_workbench/src/push.rs`,
  `agent_events.rs`, `codex_server.rs`, `notifications.rs`, `thread_tab.rs`, `rail.rs`,
  `marley_workbench.rs` (`PushSettings`) (Marley); the four Zed paths above;
  `script/e2e/codex-fixture.sh` and `script/e2e/752-codex-push-and-answers.sh`; `docs/marley/guide.md`.
- **Ledger rows:** the four Zed paths' rows in `docs/marley/zed-touchpoints.md`, widened. No new
  spawn site.

### Visual check plan
| REQ | The scenario sets up and does | The shot |
|---|---|---|
| 001 | Codex from the picker on the stand-in; window away; `command rm -rf build` | `752-01-codex-approval`; log `repo: Codex needs approval` |
| 002 | `ask Branch\|Which branch?\|main,dev`, window away | `752-02-codex-input`; log `repo: Codex needs input` |
| 003, 004 | `done`, then `fail`, window away | `752-03-codex-ends`; two log lines |
| 005 | A thread tab on the stand-in ACP agent; window away; a prompt that asks a tool's permission | `752-04-thread-approval`; log `repo: <agent> needs approval` |
| 006 | The stand-in `claude`: a permission, then an `AskUserQuestion`, window away | `752-05-claude-kinds`; two log lines |
| 007, 008 | The Push section opened; `finish` off in the run's settings; Codex `done` with the window away | `752-06-switches`; no new request |
| 009 | none: the gates are read in the diff | review |
| 010 | every request above | the log in full |
| 011, 012 | `ask …` with two options; the second clicked | `752-07-question`, `752-08-answered`; the server log |
| 013 | `ask2`; the entry clicked | `752-09-question-in-terminal` |
| 014, 015 | `toolok docs "Run search?" session,always`; Allow session clicked | `752-10-tool-approval`, `752-11-allowed-for-session`; the server log |
| 016 | `form docs "Your name?"` | `752-12-form` |
| 017 | the resolutions after 012 and 015 | `752-08`, `752-11` |
| 018 | ten seconds after `askfree`, the server log | no response from Marley |
| 019 | the gate | `script/gates.sh --diff` |

What no scenario reaches: the phone (ntfy on the box still waits on Chad's yes, #535's hand check);
a real Codex asking a question or a tool approval (it needs a model turn).

### Risks
- **The installed Codex is 0.162.0**, past #650's tested range (before 0.158.1). The promotion
  re-runs #648's check: the shapes this ticket reads were generated from 0.162.0 for this draft
  and match 0.155.1's source, but the range and the stand-in must be brought up to date before the
  feature reaches Chad's Codex.
- **The questions' fields are experimental** in Codex's schema; Marley's connection already asks
  for the experimental API (#650's D3), as Codex's TUI does.
- **`conversation_view.rs` is a busy upstream file.** The hook is one call and one parameter;
  a merge that moves `notify_with_sound` moves the hook with it.
- **The line changes for Claude Code's permissions** from `needs input` to `needs approval`, on the
  phone and in #538's banner title.
- **No window for the view.** A Codex terminal in a window that has closed has no `looking_at`;
  it pushes, as a terminal the user is not looking at.
- **Two pushes for one wait.** A Codex request raises the seat's wait and the inbox entry; only the
  seat change pushes, so one wait makes one push.
- **Always allow persists in Codex's own config.** It is offered only when Codex offers it and sent
  only on the user's click (D7).
- **#757 changes `push.rs` too** (a Claude Code session with Remote Control on pushes nothing
  through Marley, queued in the same batch). Whichever lands second keeps both rules: the skip
  applies to Claude Code seats, the kinds to every source.

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress. Brain:
  consultation 34b08565a45b4512ac611569a3a77418, nothing on this seam.
- **Seams checked again** at `9e5428bf75` (an Explore pass over all thirteen). Unchanged since the
  draft: `push.rs`, `notifications.rs`, `agent_events.rs`, `agent_reports.rs`, `claude_events.rs`,
  `marley_rail.rs`, `conversation_view.rs`, `agent_ui.rs`. Moved: `marley_agent.rs` `TurnEvent`
  655-664, `of_change` 670-678, `words` 682-688, `event_line` 694-700; `codex_server.rs`
  `Following` 813-829 (now with `folder`), `fold` 873-882, `view_of` 896-901, `requested`
  1220-1254 (unknown methods left to the TUI at 1223), `answered` 1259-1285; `rail.rs`
  `inbox_entries` 7185, the Codex request entries 7203-7225, `codex_entry` 7582-7625,
  `inbox_answer` 7629, `codex_decision` 7640, `render_inbox_entry` 2394-2480 (buttons 2438-2464),
  `answer_inbox` 2584-2621; `marley_page.rs` `push_section` 1138-1218 (four items);
  `MarleyPushSettingsContent` 335-345; `versions.rs` `CODEX_APP_SERVER` tested 0.155.1 before
  0.162.1 (#771), so the box's 0.162.0 is in range and the draft's first risk is gone.
- **What differs from the draft, and where the design takes it:**
  - **Two callers of `push::on_change`**, not one: `agent_reports::announce` (`agent_reports.rs:337`)
    pushes for a reporting agent, which can be Codex, and both it and #538's banner
    (`notifications.rs:155`) hard-code `AgentKind::Claude`. The agent's name comes from the seat
    (`agent_events::seat_agent`) at both sites, so a reporting Codex says Codex too.
  - **The follow keeps no window:** `Following` caches the view's id. After a fold that moved the
    seat, the follow finds the workspace and view through `mcp::terminals`, the window through
    `browser::window_of`, and pushes in it, as `agent_reports::announce` does.
  - **`InboxAnswer` and Codex's `Decision` are closed sets.** `InboxAnswer` gains
    `Option(SharedString)` and `AlwaysAllow`; Codex's `Decision` gains `Option(String)` and
    `AcceptFor(Persist)`, so one `Answer` path carries every reply.
  - **A Claude permission is told from a question** by its wait's prompt prefix, `Permission for `
    (`claude_events.rs:668`, private), as `banner_body` and the rail already do; a public
    `claude_events::waits_on_permission(seat)` says it once.
  - **The stand-in's `input` cue is unreachable** (`codex-fixture.sh:247` sends every `input` to
    `play_request`); the new `ask`, `ask2`, `askfree`, `toolok` and `form` cues go through
    `play_request` and `ask()`. #651's scenario expected the input question to stay unlisted
    (`651:153-157`); its expectation changes with this ticket.
  - **#508's way to a stand-in thread is stale** (#744 removed the New Agent Thread submenu): the
    scenario binds a key to `marley::NewAgentThread` on a custom agent server, as #734's does, with
    #508's stand-in ACP agent that asks a tool's permission.

## Phase 2 — Code
- **Built:**
  - `marley_agent`: `TurnEvent::NeedsApproval`, `of_seat`, `kind`; `event_line(project, agent:
    &str, event)`; `claude_events::waits_on_permission`, `banner_body` for an approval;
    `codex_events`: `Thread.cwd` into the seat's `cwd` label, `Decision::{AcceptAlways,
    Option(u8)}`, `Asked`, `Persist`, `Approval`, `RequestKind::Question`, the elicitation's
    `approval` (`approval_of`), `asked_of`, `option_label`, `decisions` and `response` for both.
  - `marley_rail`: `InboxAnswer::{AlwaysAllow, Option(u8)}`, their words, tooltip and `allows`.
  - `marley_workbench`: `push.rs` (`on_change` from the seat, `on_thread`, a shared `push`,
    `agent_name`, the approval's tag); `notifications.rs` (the banner's agent from the seat);
    `agent_events::fold_codex` returns the change; `codex_server::push_change`; `thread_tab` sets
    the host's `attention`; `rail.rs` (`inbox_answer`, `codex_decision`, `codex_answer_words`, an
    option's button label from the request, the question in the risk reading);
    `PushSettings::kinds` and `pushes`, `EnglishHint`'s neighbours untouched.
  - Zed paths: `agent_ui.rs` (`MarleyAttention`, `MarleyThreadHost::attention`),
    `conversation_view.rs` (`notify_with_sound`'s kind, `marley_attention`), the push switches in
    `settings_content`, `default.json` and the Push section (eight items).
  - Fixtures: the stand-in Codex's cues `ask`, `ask2`, `askfree`, `toolok`, `form`, and its TUI
    reading `tui.fifo`; #651's scenario's label for the question step.
  - Guides: the in-app guide (push article, Needs you, settings row), `docs/marley/guide.md`
    (Codex's entries), `docs/marley/walkthrough.md` 5.22.
- **Deviations from the plan:**
  - A Codex view in no window pushes nothing (the draft said it would push as one not looked
    at): `on_change` needs the window to know whether the user looks.
  - A tool approval's cancel is the inbox's existing Dismiss, as #651 named an elicitation's
    cancel; the spec's REQ-014 says "Cancel".
  - Four booleans would trip clippy's `struct_excessive_bools` on `PushSettings`; it keeps the set
    of kinds left on.
  - `push` takes the names by reference (clippy's `needless_pass_by_value`).
  - The option's button reads its label from the request the entry targets, so `InboxEntry` takes
    no new field.
- **Review of the diff:**
  - Found (F-claude-752-a-reporting-codex-pushed-as-claude-001): the report path pushed every
    agent as Claude; both lines now name the seat's agent.
  - REQ-009: a Codex push goes through `on_change`'s `looking_at`; a thread's through Zed's
    `agent_status_visible`, which `on_thread` refuses on.
  - REQ-018: an option, an allow or a persistence goes only from a click, through #651's
    `answer`, which checks the request's generation, id and params; a question with no answer in
    place gets no buttons.
  - Re-entrancy: `marley_attention` runs inside `ConversationView`'s update and the push touches
    no `ConversationView`; `push_change` runs from the follow's `cx.update`, outside any entity.
  - Provenance: Codex's protocol (Apache-2.0) and T3's report (MIT) as research; nothing from
    Warp.
- **Gate:** `just gate-diff` RED on rustfmt alone (one `match` in `push.rs`), 15 passed; after
  `rustfmt`, GREEN, 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/752-codex-push-and-answers.sh` (`compositor sway`). Two calibration
  runs: New Agent's folder question needed a second Return, and the TUI's FIFO had to exist before
  the first write; then the `main`/`dev` buttons (231,151), Allow session (122,267) and the entry
  (110,230) from the shots. The last run passed every check.
- **Shots:**
  - `752-01-codex-approval` (REQ-001): Needs you "Codex · repo", `rm -rf build · in …`, Allow,
    Allow session, Deny, Stop turn; the row `waiting on approval`; the push `repo: Codex needs
    approval`, priority 4, tag `lock`.
  - `752-02-codex-input`, `752-07-question` (REQ-002, REQ-011): "Codex · repo", `Branch: Which
    branch?`, buttons `main` and `dev`; the row `waiting on input`; the push `repo: Codex needs
    input`.
  - `752-08-answered` (REQ-012, REQ-017): `dev` clicked; the entry gone; the TUI `closed 102:
    answered elsewhere`; the server's `answered 102 by marley: {"answers": {"choice": {"answers":
    ["dev"]}}}`.
  - `752-03-codex-ends` (REQ-003, REQ-004): the row `failed · 12k tokens`, `stream disconnected
    before co…`; the pushes `repo: Codex finished` (3, `white_check_mark`) and `repo: Codex
    failed` (4, `x`).
  - `752-05-claude-kinds` (REQ-006): the stand-in's steps 1 and 2; Needs you "Claude Code · repo,
    Which base branch?"; the pushes `repo: Claude needs approval` then `repo: Claude needs input`.
  - `752-04-thread-approval` (REQ-005): the "Edit the README" tab waiting on Allow or Reject; Needs
    you "Stand-in · repo" with Deny and Allow; the push `repo: Stand-in needs approval`.
  - `752-06-switches` (REQ-007, REQ-008): the Push section with Push Approvals, Push Questions,
    Push Finished Turns (off) and Push Failures; then a Codex `done` with the window away and
    `pushes so far: 7`, unchanged.
  - `752-10-tool-approval` (REQ-014): "Codex · repo", `docs: Run search?`, Allow, Allow session,
    Always allow, Dismiss.
  - `752-11-allowed-for-session` (REQ-015, REQ-017): the entry gone; the server's `{"_meta":
    {"persist": "session"}, "action": "accept"}`; the TUI `closed 103`.
  - `752-09-question-in-terminal` (REQ-013): `Name: What is the branch called… · and 1 more`, no
    buttons; clicked, the Codex terminal in front with `asks 104`.
  - `752-12-form` (REQ-016): `docs: Your name?`, Deny and Dismiss only.
- **Checks:** every push's line, priority and tag as above; no push carries a header beyond the
  known ones (REQ-010); `request 106` (an `askfree`) got nothing from Marley in ten seconds
  (REQ-018).
- REQ-009 by the review.
- **Not reached by a scenario:** the phone (ntfy on the box, #535's hand check); a real Codex's
  question or tool approval (a model turn).

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added); the in-app guide (the push article, Needs you, the settings
  row); `docs/marley/guide.md` (Codex's entries); `docs/marley/walkthrough.md` 5.22;
  `docs/marley_architecture/marley_workbench.md` (Pushes, four kinds and more sources) and
  `marley_agent.md` (the two shapes, `NeedsApproval`); `docs/marley/zed-touchpoints.md`, the rows
  of `agent_ui.rs`, `conversation_view.rs`, `settings_content/src/marley.rs`,
  `settings_ui/src/marley_page.rs` and `default.json` widened.
- **Ledgers:** F-claude-752-a-reporting-codex-pushed-as-claude-001,
  L-claude-752-a-stand-in-cued-through-a-fifo-plays-while-the-window-is-away-001,
  L-claude-750-new-agent-asks-where-when-the-front-terminal-is-elsewhere-001 (widened),
  AD-claude-752-a-push-follows-each-sources-own-decision-and-four-kinds-switch-it-001.
- **Brain:** consultation 34b08565a45b4512ac611569a3a77418, `brain decide`.
- Ticket closed, pipeline archived, committed.
