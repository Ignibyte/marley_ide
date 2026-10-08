# The Marley agent sets up a harness seat — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-692-the-marley-agent-sets-up-a-seat.md
- **Pipeline spec:** 692-the-marley-agent-sets-up-a-seat.spec.md

## Phase 1 — Plan
- **Request:** the plan's item 6: "The Marley agent gets the same as a tool".
- **Recall:**
  - #682 and #686's question: 25 seconds under the app call's 30 (L-682).
  - #691's commands.
  - The harness's Claude start waits up to 60 seconds.
  - L-689: invoke `/pipeline:test` first.
- **Design:**
  - `marley_mcp`:
    - `registry.rs`: `Family::Seat` (`seat`, served), the `seat_add` row and `seat_add_schemas`.
      The count test becomes 41, and the listed names gain `seat_add`.
    - `dispatch.rs`: the deferred arm and the instructions.
  - `marley_workbench`:
    - `mcp.rs`: the `harness.write` grant and the route.
    - `settings_change.rs`: `pub(crate) Question` and `pub(crate) async fn ask_user`.
    - `harness_seat.rs`: `answer_seat_add`, `refusal_of`, and the start-failure notification.
    - `assistant.rs`: the instructions and `PROFILE_TOOLS`.
- **Visual check plan:** #691's stand-in, with `MCP_CLIENT_NAME="Stand-in agent"`.
  - `mcp_agent tool seat_add {builder, codex, repo, manager}` runs in the background, and the
    shot `692-01-card` shows the card. Clicking Apply (as #686 does) gives the answer
    `starting`. The log holds add and start, and `692-02-started` shows the rail listing
    builder.
  - A second call with role foreman, applied, comes back `seat_role_reserved`.

## Phase 2 — Code
- **Built:**
  - `Family::Seat` and `seat_add`, with its schemas, its dispatch arm and the instructions.
  - `harness.write`, granted, and the route.
  - `settings_change::ask_user`, which `ask_then_write` now calls.
  - `harness_seat`:
    - `Seat` and `Seat::commands`, shared with #691's form.
    - `answer_seat_add` and `seat_of`.
    - `HARNESS_CODES` and `refusal_of`.
    - The `SeatStartFailed` notification.
  - The Marley agent's instructions and `PROFILE_TOOLS` gain `seat_add`.
- **Deviation:** the harness's codes are matched against `HARNESS_CODES`, since `Refusal`'s code
  is `&'static str`. An unknown one is `refused`, with all of what the harness said.
- **Review:**
  - The answer comes once, after `seat add`. `seat start` runs after it, and its failure is a
    notification (D1).
  - `tool_off` comes before any argument check.
- **Install note:** #691's release install ran while this code was being written, which L-682
  warns of. Its binary holds #691's strings ("A seat needs a name and a folder",
  `harness-seat-create`) and none of #692's ("Propose a seat on the harness", "wants to set up
  a harness seat"), so the install is #691 as committed.
- **Gate:** run 1 was red (`too_many_lines` in `answer_seat_add`); the checks moved into
  `seat_of`. Run 2 (`scratchpad/692-gate-2.log`): **GATE GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/692-the-marley-agent-sets-up-a-seat.sh`: #691's stand-in `rh`, with
  the stand-in MCP client. Run 1 passed 3 of 3 (`scratchpad/692-e2e-1.log`).
- **Shots:**
  - `692-01-card` (REQ-001): "Stand-in agent wants to set up a harness seat", then "builder ·
    Codex · in <repo> · role manager", then "through <stand-in rh> --state <root>", with Apply
    and Decline.
  - `692-02-started` (REQ-002): the rail's Harness section lists builder, working. The answer was
    `{"result": "starting", "seat": "builder", "profile": "<root>/profiles/builder.json", "kind":
    "codex"}`. The log holds `seat add builder --agent codex --cwd <repo> --role manager`, then
    `seat start builder`.
  - REQ-003, from the answer: `{"result": "refused", "code": "seat_role_reserved", …}`.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md` (#692).
  - The guide's MCP write-tool table, and its "A seat in one step".
  - `marley_mcp.md` (the `Seat` family) and `marley_workbench.md` (`harness_seat.rs`).
  - The plan's item 6, now done.
- **Knowledge:**
  `AD-claude-692-seat-add-answers-after-the-add-and-starts-in-the-background-001`.
- **Ticket:** TICKET-692 closed.
