# Agents that start agents, tracked through the harness — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-716-delegation-tools.md
- **Pipeline spec:** 716-delegation-tools.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** MonoCode finding 3 (`docs/planning/intake/monocode-findings.md`):
  - start a thread or a terminal agent, in a worktree or a split beside the caller;
  - draft a message for the user to review;
  - report back to the caller when the work ends.
  The T3 survey's item 9 adds one prompt to several agents. Chad, 2026-10-11, answering item 13 of
  the findings review: "13.) yes as long as its tracking and not lost". The agents-together batch,
  after the finish (Deliberate).
- **Classification:** feature, L. Of the four parts, this slice takes the one Chad's condition is
  about: agents starting agents with tracking. The drafted message and the several-agent prompt are
  proposed as their own tickets (numbers not minted here).
- **Where each part belongs** (Chad's standing rule: agent hosting and session running are
  rustal-harness's; Marley draws):
  - **The harness:**
    - hosting and supervising the started agent (a seat);
    - delivering its first prompt once (`session_send` with a delivery id);
    - recording who started it and whether the caller was told (labels, requested);
    - saying when the delivered turn ended (requested).
  - **Marley:**
    - the MCP tools and their card;
    - the worktree (#510);
    - the view in a split;
    - the rows;
    - typing the pointer into its own terminal, the one step only Marley can do.
- **Recall (§18.3):**
  - AD-claude-691 and AD-claude-692: a seat is set up through the followed harness's own command;
    `seat_add` answers after the add and starts in the background. #692's tool is the shape to
    follow.
  - AD-claude-741: a remote agent is a harness seat with a terminal attached. Its view and naming
    are reused here.
  - #522 (D6, D7): ready means the seat is `Idle`, and delivery is a paste then `\r`.
  - #555: refuse only while the target waits on a question. The pointer is stricter and waits for
    `Idle`.
  - #575 (AD-claude-575): a restored terminal keeps its id, so a caller survives a Marley restart.
    #540 resumes its Claude Code.
  - #704 and #703: area modes, levels and the activity log.
  - #706's gap: `thread_tools::live` sees only panel threads (`thread_tools.rs:61-86`). That is one
    reason thread callers are out.
  - The T3 survey (`docs/t3code_architecture/README.md`, item 9 and open question 4): the default was
    "Marley's, never broader than the caller; nothing built until Chad agrees". Chad has agreed, with
    the condition. Report 06 §2.18 puts hosting with the harness and the typed pointer with Marley.
  - The harness, read on 2026-10-11:
    - agent mail reaches harness sessions only;
    - delegated work needs an operator offer and an accepting endpoint;
    - no MCP tool offers work or writes a profile;
    - `session_send` sends once per delivery id;
    - `delivery_status` reads a delivery under either grant;
    - its latest request to it is MREQ-009, and its next ticket is TICKET-117.
- **Discovery:**
  - The registry: `crates/marley_mcp/src/registry.rs` (`Family` 15-45; the seat tools 576-615; the
    name-list test 2565-2629, 54 tools).
  - Grants: `crates/marley_workbench/src/mcp.rs:106-116`; `permits` (519); the activity gate
    (526-532).
  - The caller: `Caller` (`marley_mcp.rs:159-168`); `caller_terminal` (`transport.rs:585-596`);
    the bridge sends `MARLEY_TERMINAL_ID` (`claude_plugin/marley/bin/marley-mcp-bridge:165-182`).
  - Seats: `harness_seat.rs` (`add_and_start` 615-643, `Seat::commands` 101-125); a `--label` per
    label is added to `seat add`'s words once the harness takes it.
  - The ready test: `send_selection.rs:331-347`. Delivery: `terminal_drive::paste_then`
    (`terminal_drive.rs:313`).
  - Worktrees: `worktree_agents.rs:508-607` (`create`), which calls `start_cli_with_prompt` at 584.
    `here` needs no worktree. `worktree` needs `create` without its CLI start, so that function
    splits in two.
  - The split: `launch.rs:386-391`.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **The design** is written at promotion (`/pipeline:plan`), once the harness has answered the
  request and every cited seam is checked against the code again. A sketch follows.

### Design
- **Approach:**
  - **`marley_mcp`:**
    - `Family::Agents` with `agent_start` (write, `agents.write`) and `agent_status` (read), their
      schemas and refusal codes (`caller_unknown`, `tool_off`, `agent_control_stopped`,
      `bad_argument`, the harness's own codes passed through);
    - the `INSTRUCTIONS` lines.
  - **`marley_workbench/src/delegation.rs`** (new):
    - `answer_start`: admit, the card, the worktree when asked, `add_and_start` with labels,
      `session_send {delivery: request_id, wait_ms}`, the view;
    - `answer_status`;
    - `follow(slot)`, run after each fleet pass: ended deliveries whose seat's `started_by` names a
      Marley terminal and whose `delegation.told` is unset. Each one goes to `deliver`, or to Needs
      you when its terminal is gone.
    - `deliver` waits on the caller's seat (`AgentEvents`) for `Idle`, pastes the line with Enter,
      then sets `delegation.told`.
  - **`agent_control.rs`:** `Area::Agents`.
  - **`settings_content`:** `MarleyAgentControlContent.agents` (Zed crate, ledger row).
  - **`rail.rs`:** the seat row's caller line; `InboxTarget::Delegation(slot, seat)`.
  - **`worktree_agents.rs`:** `create` returns the worktree before its CLI starts, so a delegation
    can use it.
- **File manifest:**
  - `crates/marley_mcp/src/registry.rs`, `dispatch.rs` (instructions): Marley crate;
  - `crates/marley_workbench/src/delegation.rs` (new), `mcp.rs`, `agent_control.rs`,
    `harness_seat.rs`, `harness.rs`, `rail.rs`, `worktree_agents.rs`, `marley_workbench.rs`:
    Marley crate;
  - `crates/settings_content/src/marley.rs`: Zed crate, a row in `docs/marley/zed-touchpoints.md`
    beside the existing `marley.rs` row;
  - `assets/settings/default.json` (the default `agents: "ask_first"`): its existing row extended;
  - the guide; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | The caller's stand-in calls `agent_start` (claude, here, terminal) | 716-01-card |
| 002, 003 | Allow | 716-02-started; the stand-in `rh`'s log holds `seat add … --label started_by=marley:<id>` |
| 004 | The caller is cued busy, the child cued to end, then the caller idle | 716-03-pointer |
| 005 | A second delegation; quit Marley; cue the child; start Marley | 716-04-restart |
| 006 | The caller's stand-in calls `agent_status` | 716-05-status |
| 007 | A third delegation; close the caller's terminal; cue the child | 716-06-closed |
| 008, 009 | — | The review of the diff; the gate |

- **Not reached:** real Claude Code and Codex children. The harness's own suites run its seats on
  the pinned binaries; the actor stands in for the child's turn.

### Risks
- **The harness request** may come back in another shape, such as delegations as a resource of
  their own rather than labels. D8 says the spec follows it. The pointer and the card do not
  change.
- **Spoofed callers:** any bearer holder can name any terminal (`transport.rs:585`), so a pointer
  could be steered into another terminal. Pointers carry ids only, and every start needs the
  person's Allow. A per-terminal token is the follow-up.
- **Codex children:** a Codex seat is an App Server session. Its view comes from `seat start`, but
  no thread view exists for it (`rh acp --seat` is Claude-only), so `view: thread` with `codex` is
  refused.
- **Two agents in one checkout** (`here`) can collide. The card says so, and `worktree` is offered
  first in the tool's description.

## Phase 1 — Plan (promoted 2026-10-12)
- **Promotion:** Chad, 2026-10-12, asked under the goal: "Unpause for these two" (the harness, for
  #716 and #719). The pair moved to `active/`, the ticket is in progress, its Backlog row is gone.
  Brain consultation df5e1629fc2a4073a22b905ca03aeb27.
- **The harness request** went to rustal-harness-a6 as MREQ-010 (2026-10-12). Its answer, accepted:
  `rh seat add --label KEY=VALUE` (repeatable, keys `[a-z][a-z0-9._-]{0,63}`, values 1 to 256
  bytes, at most 16, reserved keys refused as `seat_label_reserved`); the write verb
  `session_label {id, key, value?, request?}` (no `value` clears; accepted for an ended session);
  delivery state `ended` after `started` with `outcome` `finished`, `failed` or `stopped`, for every
  kind of session; the envelope labels `delivery`, `delivery.state` and `delivery.outcome` for the
  session's last send, changed by a `fleet_session` upsert; `delivery_status` the record. Marley
  adds `Ended` to `marley_fleet::DeliveryState`. It is harness TICKET-117, in Code at promotion.
- **Seams re-verified** by an Explore agent at `a72cbb7c87`. What changed the design:
  - `DeliveryState` (`marley_fleet/src/dispatch.rs:13-20`) has no `Ended`; nothing in Marley reads
    delivery labels yet.
  - A tool's wire name is `<family>_<verb>`, so the family is `agent` for `agent_start` and
    `agent_status` (`registry.rs:125-127`). A new family must enter `tool_schemas`, `tools_call`'s
    deferred arm and the registry test (69 tools now).
  - `AppCall` refuses with `call.answer(Err(Refusal::new(..)))`. `caller.terminal` is set only when
    the terminal's token checks (#755); `mcp::caller_terminal` maps it to the view.
  - Harness writes (`call_write`, `server_of`) are private to `harness.rs`, with a 5 s timeout, so no
    `wait_ms`. Every fleet update ends in `change_seats` (`harness.rs:295-305`), the place for the
    delegation pass; labels arrive whole on each upsert.
  - `Seat::commands` takes no labels; `add_and_start` (620-648) and `seat_stem` (652-674) are the add
    to reuse. `start_agent_on` opens a terminal view in the center; a view beside the caller needs
    a split (`thread_tab::open_beside`'s pattern) and a move.
  - `worktree_agents::create` (618-705) makes the worktree, then starts the CLI at 679-690; the two
    halves split.
  - `InboxTarget::Harness(Slot, String)` opens a seat; Needs you entries for a seat come from
    `harness_slot_entries` (7795-7830).
  - The e2e harnesses run the built `rh` at `/srv/stacks/rustal-harness/bin/rh`, which needs
    TICKET-117 built before this scenario runs.
- **Design:**
  - `marley_mcp`: `Family::Agent` (`agent`), `agent_start` (write, `agents.write`) and
    `agent_status` (read, `agents.read`), schemas, the deferred arm, `INSTRUCTIONS`, the test.
  - `marley_fleet`: `DeliveryState::Ended`.
  - `marley_workbench/src/delegation.rs` (new): `answer_start` (the caller from its token, the
    `harness_writes` and area gates, Sensitive admission, the worktree for `worktree`, the add with
    the labels through `harness_seat`, `session_send {id, text, delivery: request_id}`, the view
    beside the caller or in the worktree's window); `answer_status`; `follow`, run from
    `change_seats`: each primary seat whose `started_by` names `marley:<id>`, whose
    `delivery.state` is `ended` and whose `delegation.told` is unset gets its pointer typed into the
    caller once `target_of` reads it ready, then `session_label` sets `told`; a caller that no open
    terminal holds gets a Needs you entry that opens the seat. In-flight pointers are held in memory
    so a pass types each once.
  - `agent_control.rs`: `Area::Agents`; settings `agent_control.agents` (Zed crate, row extended),
    its default and its Settings dropdown.
  - `harness.rs`: crate-visible write and read calls on a slot; the pass hook.
  - `harness_seat.rs`: labels on the add.
  - `worktree_agents.rs`: `create` split, the worktree made without its CLI.
  - `rail.rs`: the seat row's caller line; the Needs you entry.

## Phase 2 — Code
- **Built:** `marley_mcp` (`Family::Agent`, `agent_start`, `agent_status`, `agent_schemas`, the
  deferred arm, `INSTRUCTIONS`, the registry test at 71); `marley_fleet` (`DeliveryState::Ended`);
  `marley_workbench`: `delegation.rs` (new), `mcp.rs` (routing, `agents.write`), `agent_control.rs`
  (`Area::Agents`), `harness.rs` (`write_tool`, `read_tool`, `read_lines` crate-visible, the pass
  in `change_seats`), `harness_seat.rs` (labels on the add, `add_and_start` returning the name, its
  helpers crate-visible), `worktree_agents.rs` (`make` and `make_for_seat` split from `create`),
  `rail.rs` (the caller line, `InboxTarget::Delegation`, the observer); `settings_content`,
  `settings_ui` and `default.json` (`agent_control.agents`, rows extended first); the guides.
- **Deviations from the spec:** `agent_start` answers `{delegation, result: "starting"}` once the
  user allows it, not `{delegation, seat, folder}` after the start: the server waits 30 seconds and
  a seat takes up to a minute (L-claude-716-an-mcp-answer-waits-30-seconds-so-a-long-start-answers-starting-001);
  the seat and its folder come from `agent_status`, and a failed start is a notification. The
  pointer names the session's title, which the rail shows. Opening the Needs you entry counts as
  told; there is no separate dismiss. The diff stat for a worktree in `agent_status` is left out.
- **The review:** the first prompt is refused until the seat is idle, so it is sent again with the
  same delivery id; the recheck loop was split from `follow` so it cannot start twice.
- **The gate:** the first run failed dylint (`shared_string_from_str_literal`) on three
  `make_for_seat` refusals; fixed with `SharedString::new_static`; the second, 17 of 17 green,
  receipt written.

## Phase 3 — Test
- **Scenario:** `script/e2e/716-delegation-tools.sh` under `compositor sway`, on a real `rh serve`
  of the harness's working tree with TICKET-117 (its `rh` built 08:03, the ticket in its own gate,
  not yet committed). The stand-in `rh` writes an actor profile with the labels it is given, and
  refuses an existing one as the real `seat add` does; the caller is #522's stand-in with a
  UUID session, so #540 resumes it after a restart.
- **The runs found:** the stand-in read the seat's name from the wrong argument (a scenario fault);
  the actor goes by the session's title, so `child_done` finds it in `rh actor list`; the rail's
  caller line found no terminal (F-claude-716-the-delegated-seats-caller-line-found-no-terminal-001),
  fixed in the pass; the pointer names the session's title, so its check matches that.
- **716-01-card** (REQ-001): "Claude Code wants to use agent_start", the agent, the folder and the
  prompt's first line, with Allow for This Session, Always for This Project and Deny.
- **716-02-started** (REQ-002, REQ-003): the seat's terminal in a split right of the caller,
  "working on the task" (the prompt delivered); the Harness section's `claude-repo-…` row with
  "for repo — python3 …"; `agent_start` answered `starting` and the seat log holds both labels.
- **716-03a-busy, 716-03-pointer** (REQ-004): the child ended while the caller was busy: no pointer;
  the caller idle: "Agent claude-repo-… ended: finished. Read the result: agent_status
  {"delegation": "2b0f6a4e-…"}." typed and echoed.
- **716-05-status** (REQ-006): `agent_status` answers the seat, `idle`, `delivery: ended`,
  `outcome: finished`, the two output lines and `told: true`.
- **716-04-restart** (REQ-005): a second delegation; Marley quit; the child ended; Marley started;
  the caller's session resumed in its restored terminal and got the second pointer once; the first
  was not typed again.
- **716-06-closed** (REQ-007): a third delegation whose caller's tab was closed: Needs you lists
  "claude-repo-3-… ended; its caller is closed".
- **Not reached:** REQ-008's refusals (no terminal, the area off, writes off), by review of the diff;
  real Claude Code and Codex children, which the harness's own suite runs on its pinned binaries.
- **Rerun on the committed harness:** TICKET-117 landed as harness `dec2014` (pushed, 2026-10-12,
  791 tests and 86 audits green). No harness crate source changed after the 08:03 `rh` build, so the
  scenario ran on the committed code: every check passed, and the seven shots show what the first
  run's did.

## Phase 4 — Complete
- **Documented:** CHANGELOG (Added); `docs/marley/guide.md` and the in-app guide (the article
  `agents-start-agents`, the tool rows); `docs/marley_architecture/marley_workbench.md`,
  `marley_mcp.md`, `marley_fleet.md`; the touchpoint rows for `settings_content`, `settings_ui`
  and `assets/settings/default.json`; the slice's open question 4 in
  `docs/t3code_architecture/README.md` and item 3 in `docs/planning/intake/monocode-findings.md`
  marked built.
- **Knowledge:** F-claude-716-the-delegated-seats-caller-line-found-no-terminal-001,
  L-claude-716-an-mcp-answer-waits-30-seconds-so-a-long-start-answers-starting-001,
  AD-claude-716-a-delegation-is-a-labelled-harness-seat-and-a-pointer-001.
- **Brain:** consultation df5e1629fc2a4073a22b905ca03aeb27 decided as
  `decisions/marleys-agents-start-agents-as-labelled-harness-seats`, follow-up 2026-11-12.
- **Closed:** TICKET-716; the pair archived to `completed/`.
