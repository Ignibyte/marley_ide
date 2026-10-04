---
pipeline_id: d3fc4afd-baba-48fc-9d4a-dba59ea933d6
ticket: docs/planning/tickets/closed/TICKET-651-codex-approvals-and-prompts.md
status: Phase 4 — Complete PASS
title: "Codex's approvals answered from the inbox"
type: feature
slice: prong 2, B1 part 2 (Codex on Marley's App Server, its approvals), on #650
references: [docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md, docs/planning/pipeline/completed/508-approvals-inbox.spec.md, docs/planning/pipeline/completed/568-inbox-order-and-risk-chips.spec.md, docs/planning/pipeline/completed/570-who-answers-a-question.spec.md, docs/planning/pipeline/completed/540-session-resume-after-restart.spec.md]
---

## Title
With Codex in a Marley terminal running on Marley's own App Server (#650), the rail's inbox lists
what Codex's server asks its clients to approve (a command, a file change, extra permissions, an
MCP server's elicitation) and answers it in place with the decisions each request offers. The
server sends each request to every client on the thread and takes the first answer: when Marley
answers, Codex's TUI closes its own prompt, and when the TUI answers first, the entry leaves. An
answer goes only to the request the user saw, by its id on the connection it came from. A wait
the inbox cannot answer keeps #650's entry, and Marley leaves its request to the TUI. The slug
keeps the brainstorm's name; prompts and resume are the next two slices (Out).

## Scope
### In
- **Codex entries.** For each terminal whose Codex runs on its own App Server (#650, with
  `marley.codex_app_server` on), one inbox entry per unresolved request of four methods on the
  lead thread Marley follows there, in place of #650's seat entry "Waits on an approval", in the
  order the rail first saw them:
  - `item/commandExecution/requestApproval`: the command on one line and its folder, and the
    `additionalPermissions` it asks for when the server sends them;
  - `item/fileChange/requestApproval`: "Edit N files" and their paths, from the `fileChange` item
    with the same `itemId`, which Marley reads from `item/started` (#650 reads no items), and the
    request's `reason`;
  - `item/permissions/requestApproval`: what it asks for (the paths to read or write, network)
    and its `reason`;
  - `mcpServer/elicitation/request`: the MCP server's name and its `message`.
  Each reads "Codex · <project>", with Codex's icon (`IconName::AiOpenAi`, as `agents.rs:117`
  draws it), and its age. A click on the entry shows its terminal.
- **Answers in place**, from the decisions of Codex's protocol (`app-server-protocol` v2 at
  0.155.1, `codex-rs/app-server-protocol/src/protocol/v2/item.rs:64-83,113-123`):
  - a command: those of the request's `availableDecisions` that are `accept`, `acceptForSession`,
    `decline` or `cancel`, in the request's order; a request without that list gets none, and its
    entry opens the terminal;
  - a file change: `accept`, `acceptForSession`, `decline` and `cancel`, without
    `acceptForSession` when the request carries a `grantRoot`;
  - permissions: the requested profile granted for the turn, the same granted for the session, or
    an empty grant, which is Codex's own deny (`approval_overlay.rs:406-426` in
    `codex-rs/tui/src/bottom_pane/`);
  - an elicitation: `decline` and `cancel`.
  The buttons read Allow, Allow session, Deny and Stop turn (`cancel` on an approval), and Deny and
  Dismiss on an elicitation, each with a tooltip saying what it does in Codex's terms (Stop turn:
  denied, and the turn is interrupted; Dismiss: the request is cancelled), and wrap under the entry
  when the rail is narrow. An allow decision shows only beside what it allows: an entry whose
  command, files or permissions Marley does not have offers Deny and Stop turn alone.
- **The one the user saw.** An entry's key is the terminal, Marley's connection to its server (a
  generation that changes when Marley connects again), the thread id and the JSON-RPC request
  id. A click carries the request as the entry rendered it. Marley sends the response only while
  that request is unresolved on that connection with the same params, then marks it sent; its
  entry shows "Sent: <decision>" without buttons until the server resolves it.
- **Leaving.** An entry leaves on `serverRequest/resolved` for its id, when its thread closes, and
  when Marley's connection to the terminal's server ends. No timer clears one.
- **Nothing else answered.** Marley answers a server request only when the user picks a decision
  on its entry. Every other request, of these four methods or any other, stays unanswered for the
  TUI, and Marley never answers with an error, which the server would take as the first answer.
- **The risk use and the route** (#568, #570): a Codex entry goes through `mark` as a terminal's
  does: a command as `ToolClass::Execute` with its line and folder, a file change as `Write` with
  its paths, permissions as `Other` with the paths they name, an elicitation as `Question`.
- **A wait the inbox does not answer.** A thread that waits only on a request the inbox does not
  list (Codex's question, `item/tool/requestUserInput`) keeps #650's seat entry, "Waits on an
  answer", which opens the terminal. The sends keep #650's hold on a paste while the seat waits;
  this ticket adds no guard of its own (D9).
- **The model**, in `marley_rail`: `InboxKind::Codex`, and an entry's answers as a list; the Agent
  Panel's Allow and Deny and a held click's Allow and Refuse stay as they are.
- `script/e2e/651-codex-approvals-and-prompts.sh`.

### Out (explicitly deferred)
- **Prompts through the App Server** (B1 part 3, its own ticket): send selection, send block,
  English lines and review notes go to the terminal's thread as `turn/start` when it is idle and
  `turn/steer` with the running turn's id while one runs, and are no longer typed into Codex's
  TUI; no `thread/queue/add`. The notes carry what was found for it.
- **Resume after a restart** (B1 part 4, B5): a restored terminal runs
  `codex resume --remote unix://<socket> <thread id>`, keyed by the thread #650 mapped to its
  `MARLEY_TERMINAL_ID`, in `resume.rs`'s table beside Claude Code's sessions.
- Accepting an elicitation (a form with fields, or a URL to open), the execpolicy and network
  policy amendments, and strict auto review: they are chosen in the TUI, as #508 leaves every
  choice past allow once and deny once to the thread.
- Codex's questions (`item/tool/requestUserInput`) as entries of their own, with their options;
  until then #650's entry for the wait stays for them and opens the terminal, where the TUI asks.
- Requests of a thread Marley does not follow, sub-agents' included (#650 lets those threads go),
  and a file change's paths when its `item/started` came before Marley subscribed (after a Marley
  restart: the resume slice).
- A notification per Codex wait (#538's path).
- Codex in the Agent Panel (`codex-acp`): #508's thread entries already answer its prompts.

## Reference (§20)
- **Warp:** a notification mailbox lists an agent's requests (command approvals, permission
  requests) and opens the session from an entry (docs.warp.dev/agent-platform/capabilities/
  agent-notifications/, the page #508 cites); Warp's agent asks before it acts at the autonomy
  level the request carries (`docs/warp_architecture/subsystems/04-agent-ai-mcp.md:102`). Marley's
  list sits in the rail and answers in place. No Warp code was read.
- **Upstream Zed:** no Zed crate speaks Codex's App Server. Zed runs Codex in the Agent Panel as
  the ACP agent `codex-acp` (`agent_servers/src/custom.rs:19`), whose permission prompts #508
  answers through `ThreadView::authorize_tool_call`; that path is kept as it is. The inbox
  (`marley_workbench::rail`, `marley_rail`) is Marley's own.
- **Codex's TUI** (Apache-2.0, upstream source at tag `rust-v0.155.1`, the version installed here)
  is the reference for the decisions offered and what they mean:
  `codex-rs/tui/src/bottom_pane/approval_overlay.rs:829-1105` (`exec_options`, `patch_options`,
  `permissions_options`, `elicitation_options`).

### Prior art
- **Behavior maps.** `docs/orca_architecture/04-remote-control-and-mobile.md` §2.5: Orca's phone
  answers a terminal agent by typing "1", "2" or Escape, and only sessions Orca runs itself get
  structured approvals; lines 181-186: a follow-up is refused while the agent sits at a
  permission prompt, the rule #650's paste hold keeps for Codex. #508's spec (D3: only the answers that need nothing
  more, in place), #568 (chips from code), #570 (the route answers nothing).
- **Published material.** Upstream Codex at tag `rust-v0.155.1`, the version installed on this
  box (`openai/codex`, Apache-2.0; paths below are under `codex-rs/app-server/src/`):
  `outgoing_message.rs:168-179,334-448` (a thread's request goes to every subscribed connection
  under one id), `outgoing_message.rs:471-498,565-584` (the first response takes the callback; a
  later one finds none and is dropped), `outgoing_message.rs:450-470` with
  `request_processors/thread_lifecycle.rs:795` (a connection that joins with `thread/resume` gets
  the thread's unresolved requests again), `thread_lifecycle.rs:851-873` (`serverRequest/resolved`
  to every subscriber), `bespoke_event_handling.rs:1760,1884,1968-2000,2055,2144` (a client's
  error, or a response that does not parse, is read as an empty answer, decline, an empty grant
  or a denial), `transport.rs:182-200` (a client without the experimental opt-in loses only
  `additionalPermissions`, so `availableDecisions`, filled at `bespoke_event_handling.rs:678-679,
  805`, reaches it), and `codex-rs/tui/src/chatwidget.rs:1036-1056` ("A remotely resolved request
  must not remain user-actionable"). The protocol generated from the installed binary with `codex
  app-server generate-json-schema` (stable and `--experimental`) and from the harness's pinned
  0.158.0: the four requests, their responses, `serverRequest/resolved` and
  `thread/status/changed` are byte-identical across the two, and the same functions were read at
  `rust-v0.158.0`. learn.chatgpt.com/docs/app-server, fetched 2026-10-03, does not describe
  several clients answering one request.
- **Code we already ship.** #650's `marley_workbench::codex_server`, shaped as
  `marley_browser::cdp` and reading server requests (its D2), is the client this ticket answers
  through. Zed's `context_server::Client::handle_input` (`context_server/src/client.rs:272-311`)
  leaves a request no handler claims unanswered, the behavior D6 asks of it; that client is stdio
  and HTTP only and `pub(crate)` (#650's prior art), so it is the model, not the code.
  `marley_fleet::AnswerRequest` (`verbs.rs:144-151`) carries the question's prompt as its
  stale-answer key, the idea D4 takes.
  `marley_agent::risk` (`ToolClass`, `classify`) and `marley_agent::route` mark the entries
  unchanged. The rustal-harness (Ignibyte's own, read and its ideas reused): `docs/CODEX_TURNS.md`
  "Approvals" keeps the exact wire request id, makes each decision one-shot, and says `sent` does
  not prove the server took it; its pin `dependencies/codex.json` lists the server's methods at
  0.158.0. Nothing else owns this seam: checked `agent_servers`, `acp_thread`, `context_server`,
  `lsp`'s JSON-RPC, and Cargo.lock (no Codex protocol crate).

## UI proof
`script/e2e/651-codex-approvals-and-prompts.sh` (`compositor sway`: the inbox's buttons and
entries are clicked). Never the user's Codex and never a model turn: #650's stand-in `codex`,
named by `MARLEY_CODEX`, with `marley.codex_app_server` set back on in the run's profile. Its
server (`app-server --listen unix://P`) gains a cue per request: typed into its TUI
(`--remote unix://P`), `command <line>`, `file <paths>`, `perms <spec>`, `elicit <server>
<message>` and `input <question>` each send, to every connection on the thread, the status change,
an `item/started` where the request has an item, and the request, in the 0.155.1 schema's shapes;
the server logs each response with the connection it came from, sends `serverRequest/resolved`
after the first, and `crash` makes it exit. The stand-in TUI prints each request it gets, answers
the last with `y` (`accept`) or `n` (`decline`) typed into it, and prints "closed: answered
elsewhere" on a resolution it did not answer, as Codex's TUI dismisses its prompt. The inbox's
risk use is on, as #568's `shadow` run sets it. Shots:
- `651-01-command`: the stand-in asks to run `rm -rf build` with all four decisions available:
  "Codex · repo", the command, its age, `destroys`, and Allow, Allow session, Deny, Stop turn; the
  terminal shows the stand-in TUI's prompt for the same request.
- `651-02-allowed`: Allow clicked: the entry is gone, the terminal reads "closed: answered
  elsewhere"; the log reads `1 marley accept`.
- `651-03-answered-in-terminal`: a second command, answered `n` in the terminal: its entry is gone;
  the log reads `2 tui decline` and nothing from Marley for 2.
- `651-04-file-change`: a file change of `README.md` and `src/main.rs`: "Edit 2 files", both paths,
  and the four buttons.
- `651-05-denied`: Deny on it: the entry is gone; the log reads `3 marley decline`.
- `651-06-permissions`: a permissions request (write `/tmp/out`, network): what it asks, Allow,
  Allow session, Deny; Allow clicked, the log reads the requested profile back with scope `turn`.
- `651-07-elicitation`: an elicitation from server `docs`: its message, Deny and Dismiss only;
  Dismiss clicked, the log reads `5 marley cancel`.
- `651-08-opened`: a second elicitation's entry clicked: its terminal in front.
- `651-09-unlisted`: `input Which branch?`: no Codex request entry, #650's "Waits on an answer"
  entry instead; ten seconds on, the log shows no response from Marley's connection.
- `651-10-server-gone`: `command rm -rf out`, then `crash`: no Codex entry; the row reads failed
  (#650).

## Locked-In Decisions
- D1: Codex entries come from #650's client, one per unresolved request of the four methods on the
  lead thread of the terminal's own server. They take the place of #650's seat entry, which stays
  only while the terminal waits on nothing the inbox lists (a question), so one wait is never
  listed twice. Sub-agents' requests are not listed: #650 lets their threads go (amended at
  promotion).
- D2: Only the four simple decisions go in place, and only those the request offers: a command's
  `availableDecisions` (Codex's own list, in its order); for a file change the protocol's four,
  `decline` kept although Codex's TUI shows three, since the user at the rail may not be at the
  terminal to say what to do differently; for permissions the turn, the session or nothing; for
  an elicitation decline and cancel. Amendments, strict auto review and accepting an elicitation
  open the terminal, as #508's D3 sends every other choice to the thread.
- D3: An allow decision shows only beside what it allows. A file change whose item Marley never
  saw, a command without its line, or a `grantRoot` (a whole folder for the session) gets no
  allow, or no session allow, in place.
- D4: An answer carries the request's identity at every hop
  (PR-claude-async-answer-carries-question-identity-every-hop-001): the key holds the connection
  generation, because the server's request ids start again when it restarts; the click carries
  the rendered request; the client sends only if the same request is still unresolved; a sent
  request shows no buttons, so one click sends one response.
- D5: An entry leaves on evidence (AD-claude-508): the server's `serverRequest/resolved`, the
  thread closing, or Marley's connection ending. A response Marley sent is not evidence: the TUI's
  may have come first.
- D6: Marley answers only what the user clicked. A request it does not list stays unanswered, and
  Marley never replies with an error: the server takes the first response, an error included, and
  reads an error or an unparsable response as decline, an empty grant or a denial.
- D7: #650's connection already asks for the experimental API (#650's D3 as shipped), so
  `availableDecisions` and `additionalPermissions` reach it; the command's entry shows the extra
  permissions it asks for. A command request without `availableDecisions` answers in the terminal
  only (amended at promotion).
- D8: No setting of its own. Only terminals on their own App Server have Codex entries, so
  #650's `marley.codex_app_server`, off by default, and its tested range govern them; the TUI
  keeps its prompt, and a second place to answer appears beside it.
- D9: Whether a Codex terminal waits stays #650's seat, from the server's own flags: each of these
  requests sets `waitingOnApproval` or `waitingOnUserInput` before it is sent
  (`bespoke_event_handling.rs:647,676,831,882,943` at 0.155.1), and the seat holds a paste as
  #650 has it. The entries answer a different question, which requests wait, from the requests
  and their resolutions; no second "waits" is computed
  (PR-claude-two-engines-answering-the-same-question-will-diverge-001).
- D10: Prompts and resume are separate slices: each changes a different path (the four senders,
  `resume.rs`) and needs its own scenario.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a Codex thread mapped to a Marley terminal holds an unresolved command approval, the inbox shall list it as "Codex · <project>" with the command, its age, and a button for each of `accept`, `acceptForSession`, `decline` and `cancel` the request's `availableDecisions` offers. | Shot `651-01-command` |
| REQ-002 | WHILE the inbox's risk use is on, a Codex entry shall carry the chips and level #568's rules give its command, folder or paths. | Shot `651-01-command` (`destroys`) |
| REQ-003 | WHEN a decision is clicked on a Codex entry, Marley shall answer that request id with that decision, once. | Shot `651-02-allowed`; the stand-in's log |
| REQ-004 | WHEN the server reports a Codex request resolved, its entry shall leave, whoever answered it. | Shots `651-02-allowed`, `651-03-answered-in-terminal` |
| REQ-005 | WHEN the TUI answers a request before Marley, Marley shall send no response for it. | Shot `651-03-answered-in-terminal`; the log |
| REQ-006 | WHILE a file change waits, its entry shall name the files from its item and offer Allow, Allow session, Deny and Stop turn. | Shot `651-04-file-change` |
| REQ-007 | WHEN Deny is clicked on a file change, Marley shall answer `decline`. | Shot `651-05-denied`; the log |
| REQ-008 | WHERE a request carries a `grantRoot`, or Marley lacks the command, files or permissions it would allow, its entry shall offer no allow decision for it in place. | Review of the diff |
| REQ-009 | WHEN Allow is clicked on a permissions request, Marley shall grant exactly the requested profile with scope `turn`, and Allow session the same with scope `session`. | Shot `651-06-permissions`; the log |
| REQ-010 | WHILE an elicitation waits, its entry shall offer Deny and Dismiss only, answering `decline` and `cancel`. | Shot `651-07-elicitation`; the log |
| REQ-011 | WHEN a Codex entry is clicked outside its buttons, Marley shall show its terminal. | Shot `651-08-opened` |
| REQ-012 | WHEN a click's request is no longer unresolved on the same connection with the params its entry rendered, Marley shall send nothing. | Review of the diff |
| REQ-013 | WHILE a Codex terminal waits only on a request the inbox does not list, the inbox shall show #650's entry for the wait in place of a request entry. | Shot `651-09-unlisted` |
| REQ-014 | Marley shall send no response to a server request its inbox does not list, and none with an error. | The log at `651-09-unlisted`; review |
| REQ-015 | WHEN Marley's connection to a terminal's App Server ends, every Codex entry of that terminal shall leave. | Shot `651-10-server-gone` |
| REQ-016 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec and the design in the notes; at promotion, align every name with #650 as
  shipped (its client, its per-terminal state, its connection generation, its inbox entry, its
  stand-in) and re-read the Codex range #650 registers with #648's table.
- **P2 Code:** the model in `marley_rail`; in `codex_server`, the request table, the `fileChange`
  items, the respond path and the sent mark; the entries, buttons, answer and open paths in the
  rail; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test:** the visual check: write and run the scenario, read every shot and the stand-in's log.
- **P4 Complete:** CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
