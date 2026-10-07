# An ACP agent's messages and permission requests between turns — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-685-acp-messages-between-turns.md
- **Pipeline spec:** 685-acp-messages-between-turns.spec.md

## Phase 1 — Plan
- **Request:** the top of the queue, put there on 2026-10-07 because rustal-harness's TICKET-111
  (`rh acp`) waits on it (the harness's message: "Out-of-turn session/update display in Zed
  (acp.rs:4805) is part of ACP-003's check, and that half is yours. Please prove it in your build
  before I build on it"). Chad, 2026-10-07: "lets let you direct the harness and yourself and just
  have it go", so the plan runs on without a stop for review.
- **Classification / tier:** spike. No `.rs` change; the scenario is the deliverable.
- **Checklist (no task tool offered):** pick ✓ · pre-flight ✓ (no active pipeline, README marker
  present, cargo busy with #680's install, which the plan does not need) · recall ✓ · mint ✓ ·
  prior art ✓ · spec ✓ · design ✓.
- **Recall (§18.3):**
  - `F-claude-588-a-scenario-that-opened-nothing-restored-the-users-session-001`: a scenario must
    open its own folder, or the copied profile restores the user's threads and starts their agent
    over ACP. This one opens its scratch repository.
  - #605 and #501 run a stand-in ACP agent as a custom agent server added inside the copied
    settings' own `agent_servers` block, and reach it through the project's + and New Agent
    Thread's submenu (`new_thread`). The rail changed since (#674 to #679), so the + and the
    submenu's steps are found again from the first run's shots.
  - PR on protocol event streams (`prevention-rules.md:2282`): no `unwrap` on an event; nothing
    here is Rust.
  - The brain: not asked; this spike decides nothing, it reports what the build does, and #680's
    consultation covered the plan.
- **Discovery:** the three Zed paths in the spec's prior art; the schema at
  `~/.cargo/registry/src/*/agent-client-protocol-schema-1.7.0/src/v1/client.rs:611`
  (`ContentChunk`) and `:863` (`RequestPermissionRequest`); Marley's `Cargo.toml:550` builds
  `agent-client-protocol` 2.1.0 with `unstable`.
- **Decisions:** D1 to D3 in the spec.

### Design

**Approach.** The scenario writes its own scripted agent, #605's stand-in grown:

- `initialize` and `session/new` as #605 answers them.
- `session/prompt`: an `agent_message_chunk` "Noted: <prompt>" with `messageId` `reply-<n>`, then
  the turn's result, `end_turn`. After the first prompt only, a timer: three seconds later an
  `agent_message_chunk` "Report between turns: the build passed." with `messageId` `report-1`,
  and two seconds after that a `session/request_permission` request (id `permission-1`) for tool
  call `confirm-1`, title "Merge the branch into main?", kind `other`, status `pending`, options
  `allow` ("Allow", `allow_once`) and `deny` ("Deny", `reject_once`).
- Writes to stdout under a lock (the timer and the reading loop both send), and logs each message
  it reads and sends, one JSON line each, to `$E2E_WORK/agent.log`.

The scenario adds it to the copied settings' `agent_servers` as `Scripted`, opens a thread of it
from the project's +, and drives and shoots each step.

**File manifest.**

| File | Kind | Change |
|---|---|---|
| `script/e2e/685-acp-messages-between-turns.sh` | script | the scenario and its scripted agent |

No crate changes; no `zed-touchpoints.md` row.

### Visual check plan

| REQ | Setup and action | Shot and evidence |
|---|---|---|
| 001 | the thread prompted "start"; wait four seconds after the reply | `685-02-report`: "Report between turns: the build passed." in the thread |
| 002 | the same | `685-02-report`: "Noted: start" and the report on lines of their own |
| 003 | wait three more seconds | `685-03-permission`: the tool call "Merge the branch into main?" with Allow and Deny |
| 004 | click Allow | `685-04-allowed`: the call no longer waiting; `agent.log` holds the response with `"optionId": "allow"` |
| 005 | type "again", Enter | `685-05-second-turn`: "Noted: again" |

`685-01-replied` shows the thread right after the first answer, before the timer fires. Whether
Zed shows a desktop notification for the request depends on the window's focus in a headless sway;
the run records what it sees and does not count it.

### Risks
- **Coordinates.** The project's + and the submenu's steps moved with the rail's rework; the first
  run's shots set them, as #605's did.
- **The timer firing before the panel shows the reply.** Three seconds after the turn's end; a slow
  first draw could put the report in the first shot, which then shows REQ-001 early rather than
  failing it.

## Phase 2 — Code
- **Built:** `script/e2e/685-acp-messages-between-turns.sh`: setup adds `Scripted` to the copied
  settings' `agent_servers` (#605's insertion), with the agent's script and its log as arguments;
  `write_scripted_agent` writes the agent (sends under a lock, logs every message read and sent);
  `allowed_in_log` finds the response to `permission-1` and checks its `optionId`; `steps` opens a
  thread from the project's +, with two calibration shots of the menu and the submenu
  (`685-00-menu`, `685-00-submenu`) since the rail changed after #605's coordinates.
- **Deviations:** none. The two calibration shots are extra, kept so a moved menu shows itself.
- **Review:** the agent's protocol against the 1.7.0 v1 schema: `messageId` on the chunk,
  `toolCall` with `toolCallId`, `title`, `kind`, `status`, options with `allow_once` and
  `reject_once`. Driven by hand outside Marley (`scratchpad/scripted-agent-check.py`): the reply
  and `end_turn` at once, the report three seconds later with `messageId` `report-1`, the
  `session/request_permission` two seconds after that; a hand-sent response with
  `"optionId": "allow"` lands in the log. No deadlock between the timer threads and the reading
  loop (the log write happens outside the send lock).
- **Gate:** no `.rs` in the change, so `script/gates.sh --fast` (§3), run after #680's install
  released cargo: green, 16 of 16 (`scratchpad/685-gate-1.log`).

## Phase 3 — Test
- **Runs:** two, on #680's debug build (no `.rs` changed since).
  - Run 1 red at "the agent got the allow option": the click at the guessed (900, 500) missed
    Allow. Its shots placed Allow at (1020, 410) and the message editor at (1200, 845); the
    submenu's steps (two Downs to `Scripted`) and the project's + (224, 123) were right.
  - Run 2 green, 2 of 2 checks (`scratchpad/685-e2e-2.log`). The agent's log holds the response
    `{"outcome": {"outcome": "selected", "optionId": "allow"}}` and the second prompt "again".
- **Shots** (run 2, `scratchpad/shots-685/`, all Marley only):
  - `685-00-menu`, `685-00-submenu`: the project's + menu (New Terminal, New Browser Tab, New Agent
    Thread, the Agent CLIs) and New Agent Thread's submenu with Zed Agent, Claude Agent and
    Scripted, Scripted highlighted.
  - `685-01-replied`: the thread in the Agent Panel (right dock), prompt "start", the reply
    "Noted: start"; the rail's thread row "start · Scripted · idle".
  - `685-02-report` (REQ-001, REQ-002): "Report between turns: the build passed." under the reply
    as a paragraph of its own, sent three seconds after the turn ended with `messageId`
    `report-1`. The rail's row stays "idle", and nothing else marks the new text: no badge, no
    toast, no Needs-you entry.
  - `685-03-permission` (REQ-003): the tool call "Merge the branch into main?" with View Raw Input
    and the options Allow (Alt-Shift-A) and Deny (Alt-Shift-X). Marley's rail raises it too:
    "Needs you 1" with the Scripted row, "Merge the bra…", Deny and Allow, and the thread's row
    carries the warning mark. No turn was running.
  - `685-04-allowed` (REQ-004): after the click on Allow, the call is a plain line "Merge the
    branch into main?" with no options; Needs you is gone and the row is "idle" again. The agent
    sent no `tool_call_update` after the choice, so the line shows no done or failed state.
  - `685-05-second-turn` (REQ-005): the second prompt "again" and its answer "Noted: again" below
    the earlier entries, in the same thread.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it"; the run's sway stopped with it.
- **What it means for `rh acp`** (sent to the harness): reports between turns display, each
  `messageId` its own paragraph, but silently, so a manager's report needs another way to catch
  the eye; a confirmation between turns shows in the panel and in Marley's Needs you with
  Allow and Deny on both, and the choice comes back as the request's result; after the choice
  the agent should send a `tool_call_update` with a final status so the line reads as settled.
- **No fix:** the spike changes no code; its findings are the result.

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Added, #685). The plan doc
  (`docs/planning/intake/marley-agent-manager-foreman.md`) records the proof's findings where it
  named the proof owed. No crate changed, so no architecture note or touchpoint row moves.
- **Knowledge (§19):** `L-claude-685-zeds-agent-panel-takes-acp-messages-between-turns-silently-001`
  (lessons), `AD-claude-685-the-manager-talks-to-the-person-in-the-agent-panel-over-acp-001`
  (decisions). No `F-` block: no bug; run 1's red was a guessed click position. Brain:
  consultation 0b4687ee23b24d409e9663ccde028b13 closed with `rusty-cli brain decide`
  (`decisions/the-manager-talks-to-the-person-in-zeds-agent-panel-over-acp`, follow-up by
  2026-10-28).
- **The harness:** the result sent to rustal-harness-91, clearing its TICKET-111.
- **Ticket:** TICKET-685 closed; it left `BACKLOG.md` at promotion.
- **Gate:** no `.rs`; `script/gates.sh --fast` green in Code, shellcheck clean after the
  coordinates changed.
