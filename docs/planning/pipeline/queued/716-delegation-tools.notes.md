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
