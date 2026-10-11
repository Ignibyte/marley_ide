---
pipeline_id: a954a9e2-7924-49f7-901e-fc76e110e308
ticket: docs/planning/tickets/open/TICKET-716-delegation-tools.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Agents that start agents, tracked through the harness
type: feature
slice: agents together (docs/planning/intake/monocode-findings.md items 3 to 6; docs/t3code_architecture/README.md item 9), after the finish
references:
  - docs/planning/pipeline/completed/692-the-marley-agent-sets-up-a-seat.spec.md
  - docs/planning/pipeline/completed/741-a-new-agent-on-a-remote-host.spec.md
  - docs/planning/pipeline/completed/742-a-thread-on-a-remote-seat.spec.md
  - docs/planning/pipeline/completed/704-editors-read-and-open.spec.md
  - docs/planning/pipeline/completed/522-review-notes-to-the-agent.spec.md
  - docs/t3code_architecture/06-orchestration-mcp-and-automations.md
---

## Title
An agent in a Marley terminal starts another agent through Marley's MCP server. The new agent is a
harness seat that the harness hosts, supervises and records. Marley asks the person, opens the
view, draws the rows, and tells the caller when the work ends. Chad, 2026-10-11: "13.) yes as long
as its tracking and not lost".

## Scope
### In
- **`agent_start`** (a new `agents` family, write, grant class `agents.write`).
  - **Arguments:**
    - `agent`: `claude` or `codex`;
    - `prompt`: 1 to 8,192 bytes, the harness's limit;
    - `place`: `here` (the caller's folder) or `worktree`, a new worktree and branch made from the
      caller's repository through #510's path;
    - `view`: `terminal` or `thread` (a thread for Claude Code only, #742);
    - `request_id`: a UUID the caller keeps.
  - **The caller** is the terminal the `Marley-Terminal` header names. A call with no terminal is
    refused as `caller_unknown`.
  - **The steps:**
    1. Marley asks the person on a card: the caller, the agent, the place and the prompt's first
       lines.
    2. On Allow, it adds and starts a seat on the first harness through #741's
       `add_and_start`. The seat is named `<agent>-<folder>`, and the add sets the labels
       `started_by = marley:<terminal id>` and `delegation = <request_id>`.
    3. It sends the prompt with `session_send`, using `request_id` as the delivery id, so a repeat
       delivers nothing twice.
    4. It answers `{delegation, seat, folder}`.
  - **The view** is the one `seat start` answers with (#741 D4):
    - with `here`, in a split to the right of the caller's pane;
    - with `worktree`, in the worktree's group, as #510 opens its agents.
- **`agent_status {delegation}`** (read). It answers:
  - the seat's state and its delivery's state;
  - the last 40 lines of its output (`session_read`), redacted;
  - for a worktree, its diff stat against its base.
  Reading an ended delegation's status counts as the caller being told (below).
- **Tracking.** The harness's record is the only record. A seat whose labels hold `started_by`
  shows its caller's row title under its name in the Harness section. Marley keeps no store of its
  own.
- **The pointer.** When a delegation's delivery reads `ended` and its label `delegation.told` is
  unset, Marley types one line into the caller and presses Enter:
  `Agent <seat> ended: <outcome>. Read the result: agent_status {"delegation": "<id>"}.` The line
  is written in Rustal STE (#725).
  - It types only while the caller is Claude Code or Codex at its prompt (#522's ready test), and
    waits otherwise.
  - Then it sets `delegation.told`.
  - The pointer carries ids only, never the child's words.
  - At each connect, Marley reads every ended delegation without `told` whose caller is one of
    its terminals, so a pointer missed during a restart is typed once the restored caller (#575,
    #540) is idle.
  - When the caller's terminal is gone, a Needs you entry ("<seat> ended; its caller is closed")
    opens the seat's view, and dismissing it sets `told`.
- **Who may.** `agent_start` is a Sensitive call of a new agent-control area, `agents`
  (`marley.agent_control.agents`, `ask_first` by default; asked every time as Sensitive unless
  `allow`), logged in Agent Activity (#703). It is refused as `tool_off` while
  `marley.harness_writes` is off.
- **Never broader than the caller.** The seat runs its CLI with the CLI's own permission prompts,
  which reach Needs you as harness questions (#534). No delegation passes a bypass.
- **Instructions:** the server's `INSTRUCTIONS` name both tools and say to end the turn after
  `agent_start`, since the pointer will come.
- `docs/marley/guide.md`: agents that start agents.

### Out (explicitly deferred)
- **A drafted message for the user to review.** Proposed as its own ticket: Marley-only, small.
- **One prompt to several agents** (T3 item 9, #510's prompt with agent checkboxes). Proposed as its
  own ticket: Marley-only, small to medium.
- Callers in the Agent Panel or a thread tab. The server cannot tell which thread calls
  (`mcp.rs:289-305` gives Zed's agents an empty terminal id), and #706's thread tools do not see tab
  threads (`thread_tools.rs:61-86`).
- Seats on other hosts (#740's harnesses). This slice uses the first harness, as `seat_add` does.
- A per-terminal token that proves the caller (T3's per-session credential; `transport.rs:585`
  takes any UUID). The person's Allow on every start is the guard for now.
- Stopping a delegated seat from the caller (`agent_stop`).

## Reference (§20)
N/A — Marley-specific. It draws and serves tools over rustal-harness's seats (TICKET-109),
`session_send` with a client delivery id (its D120), labels and delivery states, and the request
this ticket makes to it. No Zed or Warp behavior covers one agent starting another.

### Prior art
- **The code we ship:**
  - `marley_mcp`:
    - `registry.rs`: `Family` (15-45), `is_served` (72-87), and the seat family's `seat_add` with
      grant `harness.write` (576-615), the shape `agent_start` follows;
    - `marley_mcp.rs:159-168`: `Caller { terminal, project, cwd, client }`;
    - `transport.rs:585-596`: the `Marley-Terminal` header.
  - `marley_workbench`:
    - `harness_seat.rs`: `answer_seat_add` (151) and its card (163-180); `add_and_start` (615-643)
      and `seat_stem` (647-669); `start_agent_on` (527-611) and `attach_line` (673-684).
    - `harness.rs`: `seat_command_of` (227-239); the fleet's labels on each seat; `session_send` as
      the session tab sends it.
    - `agent_control.rs`: `Area` (35-42), `Level` (85-92), `admit` (161-227), and the three-button
      card `ask` (259).
    - `agent_activity.rs`: `gate` (147-161) and `log` (165-188).
    - `send_selection.rs`: `agent_targets` (268-301), `target_of` (331-347) and the ready test
      (343).
    - `terminal_drive.rs:313` (`paste_then`), as #522 delivers.
    - `worktree_agents.rs`: `create` (508-607), for `worktree`.
    - The split: `launch.rs:386-391` and `playwright_scripts.rs:413-419`.
    - `terminal_ids.rs` (#575): a restored terminal keeps its id.
  - `settings_content/src/marley.rs:413-451`: `MarleyAgentControlContent`, which gains `agents`.
- **The harness:**
  - `docs/AGENT_SEATS.md:178-230`: `seat add` and `seat start`, run as the operator through the
    shell.
  - `docs/MCP.md:155-192`: `session_send` with a client `delivery` UUID sends once, a retry reports
    the state, and `wait_ms` waits for `started`. `delivery_status` reads it under either grant.
  - Agent mail (`docs/AGENT_MESSAGES.md:29-39`) reaches harness sessions only, and nothing pushes
    it into a turn.
  - Delegated work (`docs/WORK.md:22-63`) is offered by the operator and accepted by its assignee
    through its own endpoint, a step a stock CLI seat does not take; no MCP tool offers work.
    Neither fits a caller outside the harness, hence the request.
- **Behavior maps:** `docs/t3code_architecture/06-orchestration-mcp-and-automations.md`:
  - §2.5 to §2.7 and §3 items 3 and 4: `t3_thread_launch`'s arguments, the privilege ceiling, the
    completion pointer that names the task and is typed only into an idle caller, and the "end your
    turn" instruction;
  - §2.18: agent hosting and delivery between harness sessions are the harness's; a pointer typed
    into a Marley terminal is Marley's.
- **Published material:** MCP (tools, `initialize` instructions).

## UI proof
`script/e2e/716-delegation-tools.sh`, under `compositor sway`.
- **The harness:** the first harness is a scratch root of the built `rh` with the request's labels
  and `ended`, followed with `--grant write`. `marley.harness_writes` is on.
- **The child:** a stand-in `rh` answers `seat add` by writing an actor profile with the given
  labels, as #742's does, then runs the real `seat start`. The actor takes a message, prints
  "working on: <text>", waits on a cue file, and goes idle.
- **The caller:** #522's stand-in Claude Code in the scratch project's terminal (Marley's plugin
  makes it idle; it echoes each line). On a cue it calls `agent_start` through the e2e MCP client
  from inside its terminal, so the header names it.

Shots:
- `716-01-card`: the ask card names the caller's terminal, Claude Code, `here` and the prompt.
- `716-02-started`: after Allow, the seat's view is in a split right of the caller. The Harness
  section lists the seat with the caller's title under it. The client's reply holds the delegation.
- `716-03-pointer`: the caller is made busy and the child is cued to end. The caller shows no
  pointer while busy. Once it is idle, it echoes the pointer line.
- `716-04-restart`: a second delegation; Marley quits; the child ends; Marley starts again. The
  restored caller echoes the pointer, once.
- `716-05-status`: `agent_status` answers the state `ended`, the output lines and the delegation's
  `told` mark (the client's reply).
- `716-06-closed`: a third delegation whose caller's terminal is closed before the child ends. Needs
  you lists "<seat> ended; its caller is closed".

## Locked-In Decisions
- **D1:** a delegated agent is a harness seat. The harness hosts it, supervises it and keeps its
  record. Marley serves the tools, draws, and types into its own terminals. This follows Chad's
  2026-10-02 rule and the T3 survey's §2.18.
  - Rejected: Marley starting a plain CLI in a terminal and keeping a store of its own. It would
    not outlive Marley, and it would host agents in Marley.
- **D2:** the tracking lives in the harness, as labels on the seat: `started_by`, `delegation` and
  `delegation.told`. Marley keeps nothing on disk. It needs the request below.
  - Rejected: the harness's delegated work. Only the operator offers roots, the assignee must
    accept through its own endpoint, and no MCP tool offers work (WORK.md).
  - Rejected: agent mail, which reaches harness sessions only.
- **D3:** the first prompt goes through `session_send` with the caller's `request_id` as the
  delivery id. A retry after a lost answer or a restart delivers nothing twice.
- **D4:** the pointer is ids only, typed and entered only while the caller is at its prompt, after
  T3's completion pointer. Rejected: the child's words in the pointer, an instruction channel from
  one agent into another.
- **D5:** at least once. A pointer typed just before a crash, with `told` not yet set, is typed
  again. It names its delegation, and `agent_status` marks it told.
- **D6:** Sensitive, asked every time unless the area is `allow`; the seat keeps the CLI's own
  prompts. The person stays the gate, and no child runs broader than the narrowest mode.
- **D7:** terminal callers only, since a thread caller cannot be identified yet.
- **D8:** this ticket waits for the harness request, as #742 waited for TICKET-116. If the harness
  answers in another shape, this spec follows it.

**The harness request** (the next after MREQ-009, filed by the session that promotes this ticket,
through rustal-harness's `docs/planning/MARLEY_REQUESTS.md`):
- labels on a seat:
  - `seat add --label KEY=VALUE`, repeatable, kept in the profile and published on its sessions'
    envelopes;
  - a write verb that sets or clears one label on a live session, as a receipted mutation;
- a delivery state `ended`, with the turn's outcome (`finished`, `failed`, `stopped`), for an agent
  seat's delivery, readable through `delivery_status` and raised as a fleet event.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent in a Marley terminal calls `agent_start`, the system shall ask the person with the caller, agent, place and prompt before anything starts. | Shot 716-01 |
| REQ-002 | WHEN the person allows it, the system shall add and start a seat labelled with its caller and delegation, send the prompt once, and open the seat's view beside the caller. | Shot 716-02 and the stand-in's log |
| REQ-003 | The Harness section shall show, on each seat labelled `started_by`, its caller's row title. | Shot 716-02 |
| REQ-004 | WHEN a delegation's delivery ends while its caller is busy, the system shall wait, then type the pointer once the caller is at its prompt. | Shot 716-03 |
| REQ-005 | WHEN Marley starts and a delegation ended without its caller told, the system shall type the pointer into the restored caller. | Shot 716-04 |
| REQ-006 | WHEN `agent_status` is called, the system shall answer the seat's state, its delivery's state and its last output lines, redacted, and mark the delegation told. | Shot 716-05 and the client's reply |
| REQ-007 | IF a delegation ends after its caller's terminal closed, THEN the system shall list a Needs you entry that opens the seat's view. | Shot 716-06 |
| REQ-008 | IF the caller names no terminal, the area is off, or `marley.harness_writes` is off, THEN `agent_start` shall be refused with its code and start nothing. | The review of the diff |
| REQ-009 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes; the harness request answered.
- **P2 Code** — `marley_mcp` (the `agents` family, schemas, instructions), `settings_content`
  (`agent_control.agents`, a Zed-crate touch with its ledger row), `marley_workbench` (a new
  `delegation.rs`; `harness_seat.rs` labels on the add; `agent_control.rs` the area; `rail.rs` the
  seat line and the Needs you entry; `mcp.rs` routing), the guide; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
