# The Marley agent stops and removes a harness seat — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-710-the-marley-agent-removes-a-seat.md
- **Pipeline spec:** 710-the-marley-agent-removes-a-seat.spec.md

## Phase 1 — Plan
- **Request:**
  - Chad, 2026-10-09, on the Manager: "we can communicate still via our harness to remove agents
    and so forth right?"
  - Then "lets do the small things".
  - Unblocked by harness TICKET-114 (`22d3157`).
- **Recall:**
  - #692 built `seat_add`: `writes_on`, `seat_command`, the `settings_change::ask_user` question,
    `run_seat` and `refusal_of`.
  - #698 keeps the Marley agent to `PROFILE_TOOLS` and an allowed list.
  - Harness 114's contract:
    - one JSON object on stdout;
    - a refusal as `rh: CODE: reason` on stderr, exit 1;
    - neither command takes `--request-id`.
- **Design:**
  - **`registry.rs`:** Seat `stop` and `remove` (Write, `harness.write`) in `seat_schemas(verb)`;
    `seat_add_schemas` stays for `add`; the count goes to 54.
  - **`harness_seat.rs`:** `answer_seat_end(call, Ending::{Stop, Remove}, cx)` mirrors
    `answer_seat_add`:
    - a name check (non-empty);
    - the question: "X wants to stop the harness seat NAME" / "… remove …", whose change line
      says what happens to the profile;
    - `run_seat` with `seat stop|remove NAME`;
    - the answer adds `result`.
  - **`mcp.rs`:** route `seat_stop` and `seat_remove`.
  - **`assistant.rs`:** `PROFILE_TOOLS` gains both (ten), and the instructions name them; the
    Claude entry's allowed list follows `PROFILE_TOOLS`, to be checked in Code.
  - **Docs:**
    - `dispatch.rs`'s server instructions name the new tools.
    - The guide and the HTML guide's tool rows; the Marley agent's "eight" becomes "ten".
- **File manifest:** Marley crates (`marley_mcp`, `marley_workbench`), docs, the scenario.
- **Visual check plan:** the spec's two shots and four replies.

## Phase 2 — Code
- **Built:**
  - **`registry.rs`:** Seat `stop` and `remove` (Write, `harness.write`), with descriptions in
    Strict STE using rustal-ste's verbs, and `seat_schemas(verb)` (the add schemas unchanged);
    count 54.
  - **`harness_seat.rs`:** `answer_seat_end`:
    - `writes_on` and the name check;
    - the question "<agent> wants to stop|remove the harness seat NAME", whose change line says
      whether the profile stays or is deleted;
    - `run_seat` with `seat stop|remove NAME`;
    - the harness's object plus `result`, or `refusal_of` its code.
  - **`mcp.rs`:** routes both tools.
  - **`assistant.rs`:** `PROFILE_TOOLS` has ten, and the instructions name the tools. The Claude
    entry's kept-out list is built from the registry minus `PROFILE_TOOLS`, so it follows.
  - **`dispatch.rs`:** the server's instructions name them.
  - **`guide.md` and the HTML guide:** the two rows; eight becomes ten.
- **Deviation:** none.
- **Review:**
  - Neither command takes `--request-id` (harness D187); Marley passes none.
  - A refusal's code passes through `refusal_of`, as `seat_add`'s does.
  - The question is the same `settings_change::ask_user`, so Deny and no answer refuse as they do
    for `seat_add`.
- **Gate:** `710-gate-1.log` GATE GREEN [diff] (run with `nice -n 19` beside the harness's gate).

## Phase 3 — Test
- **Scenario:** `script/e2e/710-the-marley-agent-removes-a-seat.sh`, under `compositor sway`, on
  #692's set-up (a real runtime, a stand-in `rh`), run with `nice -n 19` beside the harness's
  gate.
- **Run a (`shots-710a`):** 710-01 showed the question right, but the scripted click missed. Apply
  sits at (1172, 933) on this layout, not #692's (958, 901). The call refused `no_answer`, as it
  should without an answer.
- **Run b (`shots-710b`): every check passes.**
  - **710-01-stop-asked (REQ-001):** "Stand-in agent wants to stop the harness seat builder",
    "builder · stop its sessions; the profile stays, so it can start again", the command line,
    Apply and Decline.
  - **`seat_stop` after Apply (REQ-002):** `"result": "stopped"`, `stopped` with the session in
    `done`, and `supervision_ended`.
  - **710-02-remove-asked (REQ-001):** "… wants to remove the harness seat builder", "builder ·
    stop its sessions and delete its profile".
  - **`seat_remove` after Apply (REQ-002):** `"result": "removed"` with
    `"removed": "…/profiles/builder.json"`.
  - **The stand-in's log:** `seat stop builder`, then `seat remove builder`.
  - **`seat_stop ghost` (REQ-003):** `code: "seat_unknown"`.
- **Not covered:** the real `rh seat stop` against live sessions is harness TICKET-114's, proven by
  its gate `ticket-114-dev-2`. This scenario proves Marley's half.
- **REQ-004:** `PROFILE_TOOLS` lists both, and the instructions name them (Phase 2).
- Chad's Hyprland untouched.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added); `guide.md` and the HTML guide (Phase 2).
- **Knowledge:** none new. The click-coordinate miss is the known kind (L-707's family: measure
  from the first run's shot).
- **Ticket:** closed; the BACKLOG row went at promotion.
- **Gate:** `710-gate-2.log`, GATE GREEN [diff], on the tree committed.
