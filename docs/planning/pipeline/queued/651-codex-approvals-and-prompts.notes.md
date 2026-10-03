# Codex's approvals answered from the inbox — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-651-codex-approvals-and-prompts.md
- **Pipeline spec:** 651-codex-approvals-and-prompts.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02: "also we need to brain storm integration into claude and codex
  using their tools instead of fighting them as well." On B1, "should Marley own Codex's App
  Server and join it as a second client?": "Yes". This batch (#648 to #653) is the brainstorm's
  build order; #651 is B1 part 2, on #650 (part 1). The brief named three parts: approvals in the
  inbox, prompts as `turn/start`, `turn/steer` or `thread/queue/add`, and resume after a restart,
  with the instruction to keep approvals here and defer the rest if it is more than one slice.
- **Classification / tier:** feature, prong 2. Marley crates only (`marley_rail`,
  `marley_workbench`); no Zed path changes, no new setting. Depends on #650 (the server, the
  client, the per-terminal state, the inbox's wait entry) and on #648's version table through
  #650. Size M.
- **The split.** Three slices, each with its own path and scenario: approvals (this ticket: the
  inbox and #650's client), prompts (the four senders and `terminal_drive`), resume (`resume.rs`).
  Prompts and resume are in the spec's Out with one-line scopes; what was found for them is under
  "For the deferred slices" below.

### Recall (§18.3)
- **Ledger.**
  - AD-claude-508-one-inbox-lists-every-agent-that-waits-on-the-user-001: answer in place only
    what needs nothing more, everything else opens where it waits; an entry leaves only on
    evidence; the inbox keeps nothing but when it first saw each entry and holds no view.
  - AD-claude-568-inbox-risk-chips-come-from-code-and-a-reading-only-adds-001 and
    AD-claude-570-the-inbox-marks-who-should-answer-and-answers-nothing-001: a Codex entry is
    marked by the same `mark`, and nothing answers on the user's behalf.
  - BF-claude-answer-race-no-question-identity-001 and
    PR-claude-async-answer-carries-question-identity-every-hop-001: the rendered question, an
    exact check at dispatch, the identity on the wire. D4.
  - PR-claude-an-answer-for-a-seat-names-the-session-prompt-and-stop-it-belongs-to-001: an answer
    names what it belongs to (here the connection, the thread and the request).
  - PR-claude-two-engines-answering-the-same-question-will-diverge-001: "a Codex thread waits"
    is one fact read by the clearing and by the sends' guard. D9.
  - L-claude-508-a-stand-in-acp-agent-asks-for-permission-001: a stand-in that sends a request and
    logs the answer is how a scenario reaches a permission prompt with no model; this ticket's
    stand-in server does the same over the App Server's JSON-RPC.
  - L-claude-568-a-seat-holds-one-wait-so-each-entry-needs-its-own-terminal-001: not needed here;
    a Codex thread can hold several requests at once and each is its own entry.
  - F-claude-594-an-enter-sent-with-a-paste-was-read-as-part-of-it-001 and
    PR-claude-keys-after-a-paste-go-in-a-later-write-001: the typed paths, and #650's hold on them
    while a Codex seat waits, stay as they are in this slice.
- **Completed pipelines.** #508 (the inbox, its D3 and D5, the pick guard), #568 (chips), #570
  (route), #534 (harness entries open only, the precedent for an entry kind with its own source),
  #540 (resume by the terminal's id, for the deferred slice), #594 (the paste wait).
- **The harness** (`/srv/stacks/rustal-harness`, read only): `docs/CODEX_TURNS.md` "Approvals":
  the exact wire `RequestId` kept, one-shot decisions, "`sent` does not prove server receipt",
  a resolution before the reply recorded as a cancellation, an observer cannot decide;
  `docs/CODEX_PROTOCOL.md`: experimental opt-in false, the generated schema inventory;
  `dependencies/codex.json`: the 0.158.0 method lists (server requests at `server_methods[4..9]`,
  `serverRequest/resolved` at `server_notifications[44]`, `thread/queue/add` only in the
  experimental inventory).
- **Brain:** a read-only search (`rusty-cli brain search codex app server approvals`) found
  nothing on this seam; no `.mcp.json` in this repo, so the brain-loop hooks do not apply here.

### Discovery
- **Marley, at `f9ef577284`.**
  - `crates/marley_workbench/src/rail.rs:685-701` (`Waiting`), `:704-721` (`InboxTarget`: Thread,
    Terminal, Click, Harness), `:727-737` (`observe_marks`, six globals), `:1544-1620`
    (`note_inbox`: first-seen times, order, the 30 s refresh), `:1647-1673` (`answer_thread`, out of
    the rail's update with `defer_in`), `:1680-1700` (`open_inbox_entry`), `:1759-1840`
    (`render_inbox_entry`: icon by kind, `AiClaude` for every terminal entry, two buttons from
    `answers: bool`), `:1948-1967` (`answer_inbox(key, allow: bool)`: the risk outcome
    `allowed`/`denied`, the route's `owner_seen`, the dispatch), `:6064-6113` (`inbox_entries`: a
    terminal's entry from its `AgentEvents` seat in `State::Waiting`), `:6205-6221` (`seat_entry`:
    agent "Claude Code" hard-coded), `:6304-6400` (`mark`), `:6405-6448` (`thread_waiting`),
    `:6452-6497` (`seat_waiting`).
  - `crates/marley_rail/src/marley_rail.rs:424-433` (`InboxKind`), `:437-461` (`InboxEntry`, with
    `answers: bool`).
  - `crates/marley_agent/src/risk.rs:19-56` (`ToolClass`; `base_level`).
  - `crates/marley_workbench/src/send_selection.rs:47-61` (`Target`: `waiting`, `ready` from the
    seat), `:331-347` (`target_of`), `:363-410` (`send_text`: the waiting toast at `:380-391`; send
    block and English go through it, `send_block.rs:102,113`, `english.rs:279,290`);
    `review_notes.rs:107-155` (`deliver`: refuses unless `target.ready`, then `paste_then`).
  - `crates/marley_workbench/src/agents.rs:117` (`AgentKind::Codex => IconName::AiOpenAi`).
  - `crates/marley_agent/src/marley_agent.rs:125-138` (Codex's full access is
    `--sandbox danger-full-access --ask-for-approval never`, under which no request comes);
    `settings_content/src/marley.rs:551-557` (`CodexPermissions::Ask` the default).
  - `crates/context_server/src/client.rs:272-311` (a request with no handler is dropped, never
    answered).
  - `crates/agent_servers/src/custom.rs:19` (`CODEX_ID = "codex-acp"`).
- **Codex.** Upstream Codex at tag `rust-v0.155.1`, the version installed here, shallow-cloned
  for this drafting at `scratchpad/repos/codex-0.155.1` (the coordinator's correction:
  `/srv/stacks/rustal-codex` was removed in a cleanup). Paths below are in that tree. The same
  functions were also read at `rust-v0.158.0`, the harness's pin and the top of #650's range, and
  the protocol was generated from both binaries (`codex app-server generate-json-schema`, with and
  without `--experimental`; no server started, no turn).
  - `codex-rs/app-server-protocol/src/protocol/common.rs:1756-1786` (the server requests:
    `item/commandExecution/requestApproval`, `item/fileChange/requestApproval`,
    `item/tool/requestUserInput`, `mcpServer/elicitation/request`,
    `item/permissions/requestApproval`), `:1967` (`serverRequest/resolved`), `:631`
    (`#[experimental("thread/queue/add")]`).
  - `codex-rs/app-server-protocol/src/protocol/v2/item.rs:64-83`
    (`CommandExecutionApprovalDecision`: `accept`, `acceptForSession`,
    `acceptWithExecpolicyAmendment`, `applyNetworkPolicyAmendment`, `decline`, `cancel`),
    `:113-123` (`FileChangeApprovalDecision`: the four), `:1534-1597`
    (`CommandExecutionRequestApprovalParams`; `availableDecisions` marked experimental at
    `:1592-1595`, as is `additionalPermissions`), `:1599-1604` (`strip_experimental_fields` drops
    `additionalPermissions` only), `:1617-1632` (`FileChangeRequestApprovalParams`: no paths, an
    `itemId` and an optional `grantRoot`), `:320-324` and `:1125-1129` (the `fileChange` item's
    `changes[].path`); `v2/permissions.rs:789-810` (`PermissionGrantScope` `turn`/`session`,
    `PermissionsRequestApprovalResponse { permissions, scope, strictAutoReview }`);
    `v2/mcp.rs:346` (`McpServerElicitationAction`: `accept`, `decline`, `cancel`);
    `v2/notification.rs:74` (`ServerRequestResolvedNotification { threadId, requestId }`);
    `v2/thread.rs:1659` (`ThreadActiveFlag`: `waitingOnApproval`, `waitingOnUserInput`).
  - `codex-rs/app-server/src/outgoing_message.rs:168-179` (a thread's request goes to its
    subscribed connections), `:334-448` (one id, written to each connection in turn; a broadcast
    when no thread), `:362` ("One app owns this ceremony": a user-verification elicitation goes to
    one eligible connection only), `:450-470` (replay to a joining connection), `:471-498` with
    `:565-584` (the first response removes the callback; a later one logs "could not find
    callback" and is dropped), `:602` (a thread's requests cancelled together).
  - `codex-rs/app-server/src/request_processors/thread_lifecycle.rs:795` (`thread/resume` on a
    running thread replays its unresolved requests to the new connection), `:851-873`
    (`resolve_pending_server_request`: `serverRequest/resolved` to every subscriber);
    `thread_state.rs:210` (resolution goes through the thread's listener).
  - `codex-rs/app-server/src/bespoke_event_handling.rs:645-647,674-679,829-831,876-882,941-943`
    (each request sets the thread's waiting flag before it is sent; `availableDecisions` always
    filled, `:805`), `:1745-1760` (a user-input request: a client error becomes an empty answer),
    `:1831-1900` (an elicitation: a client error becomes `decline`), `:1902-1917` with
    `:1968-2000` (permissions: a client error, or a response that does not parse, becomes an empty
    grant for the turn), `:2034-2060` (a file change: either becomes a denial), `:2076-2150` (a
    command: either becomes a denial and the item `failed`); each handler resolves through the
    listener (`:1754,1841,1917,2043,2090`), so a turn ending also sends `serverRequest/resolved`.
  - `codex-rs/app-server/src/transport.rs:182-200` (`filter_outgoing_message_for_connection`: a
    connection without `experimentalApi` gets the command request with `additionalPermissions`
    stripped and nothing else).
  - `codex-rs/tui/src/bottom_pane/approval_overlay.rs:829-916` (`exec_options`: the request's own
    list), `:1012-1029` (`patch_options`: Yes, Yes and don't ask again for these files, No and tell
    Codex what to do differently, the last being `cancel`), `:1032-1064` (`permissions_options`),
    `:1073-1105` (`elicitation_options`), `:406-426` (the permissions answer: the requested
    profile, or `Default::default()` for Deny; scope `session` only for the session grant);
    `codex-rs/tui/src/approval_events.rs:56-110` (the TUI's own fallback when a request has no
    list); `codex-rs/tui/src/app/app_server_events.rs:173-200` and
    `codex-rs/tui/src/chatwidget.rs:1036-1056` (on `serverRequest/resolved` the TUI dismisses the
    prompt, shown or deferred).
  - The two versions' generated schemas for the four requests, their responses,
    `serverRequest/resolved` and `thread/status/changed` compare byte-identical.
- **#650** (`650-codex-app-server-state.spec.md`, queued in parallel, read 2026-10-03):
  `marley.codex_app_server`, off by default (its D4); one `codex app-server --listen
  unix://<socket>` per Codex terminal Marley launches, through `process.rs`, living as long as the
  terminal; the TUI typed as `codex --remote unix://<socket>`; every thread that server loads is
  that terminal's. Its client, `marley_workbench::codex_server`, speaks JSON-RPC over a WebSocket on
  the socket, shaped as `marley_browser::cdp` but reading server requests "since #651 answers
  them" (its D2); `initialize` with no experimental API and `optOutNotificationMethods` for the item
  deltas it does not read; it subscribes to every loaded thread with `thread/resume { excludeTurns:
  true }` (its D3) and sends nothing that changes a thread. The pure fold is
  `marley_agent::codex_events`, into the terminal's `AgentEvents` seat (`agent: codex`, a `wait`
  label `approval` or `input`); a sub-agent's thread never moves the seat. Its inbox entry is a
  terminal entry named Codex, "Waits on an approval" or "Waits on an answer", opening the terminal
  (its D9); a waiting seat holds a paste (send selection, the Browser tab's Send); a server that
  stops makes the seat failed. Its stand-in `codex` (`MARLEY_CODEX`) serves the WebSocket and turns
  lines typed into its TUI into cues the server plays. Tested range 0.155.1 to 0.158.0 (its D5).
- **Not in #650, added here:** a table of unresolved server requests per thread with method, id
  and params, cleared by `serverRequest/resolved`, the thread's close and the connection's end; a
  connection generation; the `fileChange` items from `item/started` (#650 reads no items, its Out);
  the response path. #650's Out still names prompts as #651's; with this split they are a later
  ticket.

### Decisions
D1 to D10 in the spec. The reasons not written there:
- **Why a list from the request and not Codex's fallback.** The TUI computes its own list when a
  request has none (`approval_events.rs:56-110`); copying that would be a second engine of
  Codex's rule inside Marley (PR-claude-two-engines-...). Both tested versions send the list, so
  the fallback would only ever run on a version #648's table has not passed.
- **Why `decline` on a file change.** The protocol offers it; Codex's TUI leaves it out because its
  "No" asks the user to type what to do instead, which a user at the rail is not placed to do.
  Deny there means the agent goes on without the change, and Stop turn is the TUI's "No".
- **Why no in-place accept for an elicitation.** A form's accept carries `content` matching its
  `requestedSchema`, and a URL's means the user went to it; both are the TUI's to collect.
- **Rejected:** typing the TUI's shortcut keys from the rail (depends on its keymap and focus, as
  #508's D4 rejected for Claude Code); answering requests of threads Marley does not follow;
  opting into `experimentalApi` for `availableDecisions` (not needed at either version, and it
  would subscribe Marley to every experimental notification and field); a setting of its own.

### Design
- **The model** (`marley_rail`): `InboxKind::Codex`. `InboxEntry.answers: bool` becomes
  `answers: Vec<InboxAnswer>`, each with an id the rail maps back, its words, its tooltip and
  whether it is filled (Allow is). The Agent Panel's entries keep Deny and Allow, a held click's
  Refuse and Allow, so their rendering and the scenarios of #508 and #571 read the same. A pure
  `codex_answers` there takes what the client knows of a request (its method, the offered list,
  whether a grant root is set, whether the command, files or permissions are known) and returns
  the answers in order, so D2 and D3 live in one place.
- **The types** (`marley_agent::codex_events`, beside #650's read types): the four requests'
  params as Marley reads them, their decisions and responses, serialized in the protocol's names,
  and a pure step from a request, `item/started` or `serverRequest/resolved` to the table's change.
- **The client** (`marley_workbench::codex_server`, #650's): the table and the items, kept per
  terminal with the connection's generation; `respond(terminal, generation, thread, request id,
  rendered params, decision)` refuses unless the request is in the table on that generation with
  params equal to the rendered ones, writes the JSON-RPC response with the same id and a body
  serialized from the types (a response that does not parse is a denial at the server), and marks
  the request sent with the decision. It notifies the global the rail observes, so the entry
  redraws as "Sent: <decision>". It answers nothing else (D6).
- **The rail** (`marley_workbench::rail`):
  - `InboxTarget::Codex { terminal: u64, generation, thread: String, request: RequestId }`.
  - In `inbox_entries`, for a terminal `codex_server` follows, one entry per unresolved request of
    the four methods on any of its threads, keyed `codex:<terminal>:<generation>:<thread>:<request
    id>`; #650's seat entry only when none is listed. The ask: the command (with
    `additionalPermissions` when sent), "Edit N files: a, b" with the reason, "Permissions: write
    a, read b, network" with the reason, "<server>: <message>"; "Sub-agent: " before a sub-agent
    thread's.
  - `codex_waiting` builds the `Waiting` `mark` reads, with the classes the spec's In names;
    `tool_name` is the method's item kind (`commandExecution`, `fileChange`, `permissions`, the
    elicitation's server), `agent` "Codex in a terminal".
  - `render_inbox_entry` draws the entry's answers as buttons in a wrapping row, with tooltips, and
    `IconName::AiOpenAi` for `InboxKind::Codex`.
  - `answer_inbox(key, answer)` takes the answer's id instead of a bool; the risk outcome maps
    Allow and Allow session to `allowed`, the rest to `denied`, and records it as sent from the
    inbox; a Codex answer defers out of the rail's update (as `answer_thread` does) and calls the
    client's `respond` with the target the click captured.
  - `open_inbox_entry` shows the terminal for `InboxTarget::Codex`, as for `Terminal`.
  - `observe_marks` observes `codex_server`'s global, unless #650 already has the rail observe it.
- **The sends** stay on #650's hold (a waiting Codex seat holds a paste); no change here (D9).
- **File manifest.**
  - Marley: `crates/marley_rail/src/marley_rail.rs` (`InboxKind::Codex`, `InboxAnswer`,
    `codex_answers`); `crates/marley_agent/src/codex_events.rs` (#650's; the request types and the
    table's step); `crates/marley_workbench/src/codex_server.rs` (#650's; the table, the items,
    `respond`, the sent mark); `crates/marley_workbench/src/rail.rs` (entries, target, buttons,
    answer, open, observe); `script/e2e/651-codex-approvals-and-prompts.sh`; #650's stand-in
    `codex`, extended with the request cues.
  - Zed: none. `IconName::AiOpenAi`, `Button`, `Tooltip` and `h_flex().flex_wrap()` exist.
  - `docs/marley/zed-touchpoints.md`: no row. `.config/spawn-sites.txt`: no new spawn (the server
    process is #650's).

### Visual check plan
| REQ | What the scenario sets up and does | Shot |
|---|---|---|
| REQ-001 | Codex launched from the + with the switch on; `command rm -rf build` typed into the stand-in TUI (all four decisions) | `651-01-command` |
| REQ-002 | the inbox's risk use in `shadow` (#568's run) | `651-01-command`: `destroys` |
| REQ-003 | Allow clicked on that entry | `651-02-allowed`; log `1 marley accept` |
| REQ-004 | the stand-in sends `serverRequest/resolved` after each first answer | `651-02-allowed`, `651-03-answered-in-terminal` |
| REQ-005 | `command rm -rf dist`, then `n` typed in the terminal | `651-03-answered-in-terminal`; log `2 tui decline`, nothing from Marley for 2 |
| REQ-006 | `file README.md src/main.rs`: an `item/started` fileChange, then its request | `651-04-file-change` |
| REQ-007 | Deny clicked | `651-05-denied`; log `3 marley decline` |
| REQ-008 | none: a request with a `grantRoot`, or with an item Marley never saw, is read in the diff | review |
| REQ-009 | `perms write:/tmp/out network`; Allow clicked | `651-06-permissions`; log reads the requested profile with scope `turn` |
| REQ-010 | `elicit docs "Pick a branch"`; Dismiss clicked | `651-07-elicitation`; log `5 marley cancel` |
| REQ-011 | a second `elicit`; the entry's body clicked | `651-08-opened` |
| REQ-012 | none: the window between a resolution and a click is too short to aim at | review |
| REQ-013 | `input Which branch?`: a request the inbox does not list | `651-09-unlisted`: #650's "Waits on an answer" |
| REQ-014 | ten seconds after that cue, the log | `651-09-unlisted`; log: no response from Marley's connection |
| REQ-015 | `command rm -rf out`, then `crash` | `651-10-server-gone` |
| REQ-016 | the gate | `script/gates.sh --diff` |

What no scenario reaches: Codex's real TUI closing its prompt (it needs a real server; read in
`codex-rs/tui/src/chatwidget.rs:1036-1056` and its tests at `codex-rs/tui/src/app/tests.rs:541,
4932`); a real Codex sending these requests (it needs a model turn).

### Risks
- **#650's shape.** Read from its queued spec; if it changes at its promotion or in its Code
  phase, this ticket's names follow. Its `optOutNotificationMethods` must not drop `item/started`
  (only the deltas), or file changes lose their paths and with them their Allow (D3).
- **`excludeTurns: true`.** #650 subscribes without the live turn's items, so a file change whose
  `item/started` came before the subscription has no paths; with #650 subscribing at launch, before
  any turn, that happens only after a reconnect.
- **Two answers at once.** A click in Marley and a key in the TUI within the same moment both send;
  the server keeps the first and drops the other with a log line. Marley's risk outcome then says
  "sent from the inbox" for an answer that may have lost; nothing else depends on it. The entry
  still leaves on the resolution.
- **Request ids repeat across server restarts** (`next_server_request_id` starts again); the
  generation in the key keeps an old click from reaching a new request.
- **A malformed response is a denial**, not an error back to Marley; the response bodies come from
  typed values of the protocol, and the scenario's log shows each one as the server would parse it.
- **`availableDecisions` is experimental in the schema.** A later Codex could strip it from stable
  connections; command entries would then answer in the terminal only until #648's table and this
  rule are revisited.
- **Full access.** Codex started with Marley's full access asks nothing, so a user may never see a
  Codex entry; that is the setting working.
- **Auto review.** With `approvals_reviewer = "auto_review"` Codex's reviewer can answer first; an
  entry may show briefly and leave on the resolution.
- **Rail width.** Four buttons do not fit one row at the rail's default width; they wrap, and the
  shots check that the entry stays readable.

### For the deferred slices
- **Prompts (B1 part 3).** At 0.155.1 `turn/start` calls `start_or_steer_turn`
  (`codex-rs/app-server/src/request_processors/turn_processor.rs:646`): sent while a turn runs, it
  steers into that turn rather than failing, so "idle, then start" is advisory. `turn/steer`
  requires `expectedTurnId` and fails on a mismatch or with no active turn
  (`turn_processor.rs:1020-1145`), the precondition a running send wants. `thread/queue/add` is
  experimental (`app-server-protocol/src/protocol/common.rs:631`), needs the queue service the
  server may not have (`request_processors/thread_queue_processor.rs:72`), and Codex's TUI already
  runs with `experimentalApi` and its own queue (`codex-rs/tui/src/app_server_connection.rs:36`,
  `codex-rs/tui/src/session_queue_commands.rs:85`). Proposed: `turn/start` when idle,
  `turn/steer` with the last `turn/started` id while running, no queue and no opt-in. What the TUI
  shows: a live `item/completed` user message it did not send is drawn in its history once
  (`codex-rs/tui/src/chatwidget/tests/app_server.rs:685,723`), and `turn/started` sets its working
  status (`:760,841`): Marley's prompt appears in the TUI as a user message, as if typed.
  Confirmed from the TUI's source and tests; not seen live. Open for that ticket: send selection
  and send block paste without Enter today so the user can finish the prompt, and the App Server
  has no call that puts text in the TUI's composer, so they either submit a prompt of their own or
  stay typed.
- **Resume (B1 part 4, B5).** `codex resume --remote <ADDR> [SESSION_ID]` exists at 0.155.1
  (`codex resume --help`); a connection that resumes a running thread gets its unresolved requests
  replayed (`thread_lifecycle.rs:795`), so requests asked while Marley was away come back as
  entries. The key is the thread #650 maps to the terminal, saved under `MARLEY_TERMINAL_ID` in
  `resume.rs`'s table (#540) with the agent's kind beside it.

### Checklist (no TaskCreate in this harness)
- [x] Read the brief (both parts), CONSTITUTION §3, §7, §14, §18, §19, §20, the ticket and
      pipeline templates, and #633's pair for shape.
- [x] Read the design note in full: the fights table, the tools, B1 to B7, Chad's answers, the
      harness's side.
- [x] Read what this ticket changes and borders: the inbox in `rail.rs` (target, entries, buttons,
      answers, `mark`), `marley_rail`'s model, `risk.rs`, the four senders and their guard,
      `resume.rs`.
- [x] Codex: upstream at `rust-v0.155.1` (the installed version, the coordinator's clone) for every
      citation, the same functions at `rust-v0.158.0`, generated schemas of both versions; the
      harness's Codex docs and pin.
- [x] Recall: the ledgers (AD-508, AD-568, AD-570, BF and PR on answer identity, PR on two
      engines, PR on a seat's answers, L-508, L-568, F-594), the completed pipelines (#508, #534,
      #540, #568, #570, #594), a read-only brain search.
- [x] #650's ticket doc and queued spec read and its names taken (`codex_server`, `codex_events`,
      the seat and its inbox entry, the stand-in, the paste hold); what this ticket adds to its
      client listed.
- [x] Split into three slices; approvals drafted here, prompts and resume in Out with one-line
      scopes and their findings above.
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D10, sixteen EARS rows, phase plan.
- [x] Design: approach, file manifest by crate, no ledger row, the visual check plan, risks.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.
