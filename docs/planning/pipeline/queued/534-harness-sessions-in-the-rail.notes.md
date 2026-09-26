# Harness sessions in the rail: Marley follows rh mcp's fleet, read side only — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-534-harness-sessions-in-the-rail.md
- **Pipeline spec:** 534-harness-sessions-in-the-rail.spec.md

## Phase 1 — Plan
- **Request:** the lead's brief for 2026-09-25: a new feature, deliberate until the harness's M9
  exits: Marley's adapter over `rh mcp` (its tools and the `fleet_session` feed, in the harness's
  `docs/MCP.md` and `docs/FLEET.md`): harness sessions as rail rows with their state and questions,
  their questions in the approvals inbox, and a harness session opened as a display-only Zed
  terminal fed by the harness (plan D10); the read side only. Chad's decision behind it, as the
  brief records it: rustal-harness will be embedded in Marley and also run standalone, and its
  `rh mcp` serves Marley's fleet contract.
- **Classification / tier:** feature, prong 2, C1 with C3's observer view, M to L. One Zed-crate
  file touched, `crates/settings_content/src/marley.rs`, which is Marley's own file there and has a
  row; the rest in `marley_rail` and `marley_workbench`. Depends on #533 (the contract's types and
  the plan's text) and on #508 (the inbox) for REQ-008; waits for the harness's M9 exit (D8).
- **Recall (§18.3):**
  - BF-373-reconnect-rebuilds-empty-fleetsync: a reconnect that rebuilt an empty snapshot and
    applied only a delta dropped every seat seen before the cursor. D3 keeps one snapshot for the
    connection's whole life and re-seeds from `fleet_snapshot`, never from nothing.
  - AD-claude-fleet-rail-quiet-no-transport-until-brain-001: the gpui era's fleet rail rendered a
    quiet header when it had no transport and no invented states; REQ-009 and REQ-010 follow it
    (not running says why; unset shows nothing).
  - AD-claude-491: Marley's own server; this ticket is the other direction, Marley as a client, and
    touches none of it.
  - L-claude-448-running-zeds-dylint-library-on-the-fork-001 and the fork-port memory: guards wait
    with `pgrep -x`, never `pgrep -f`, which matches the guarding shell; the scenario's kill of the
    MCP process (`534-06-down`) targets its pid, read from `/proc`, never a pattern.
  - Brain: no consultation run by this drafting agent; the Planner's `brain_ask` at promotion is
    owed.
- **Discovery (checked in the tree and the harness):**
  - `crates/context_server/src/context_server.rs`: `ContextServer::stdio(id, command,
    working_directory)` (49), `client` (106), `start` (120: a new client, then `initialize`),
    `stop` (168). `protocol.rs`: `request::<T>` (107), `wait_for_shutdown` (136),
    `on_notification(method, callback)` (140). `types.rs`: `CallTool` (40), `ResourcesSubscribe`
    (47 to 52), `ResourcesRead` (53 to 58), `ResourcesUpdated` (114 to 118),
    `CallToolResponse::structured_content` (726).
  - `crates/settings_content/src/project.rs:521`: `ContextServerCommand { path (as "command"), args,
    env, timeout }`, with a `Debug` that hides the environment's values (531).
  - `crates/settings_content/src/marley.rs`: `MarleySettingsContent { layout }` and its row in
    `docs/marley/zed-touchpoints.md` (55).
  - `crates/marley_workbench/src/marley_workbench.rs:165`: `MarleySettings { layout }`, `Copy`,
    so the harness setting gets a `Settings` type of its own rather than a field there.
  - `crates/marley_fleet/src/reducer.rs`: `SessionEvent` (17, `kind` tagged, the six v1 kinds),
    `FleetSnapshot` (122), `apply` (147, idempotent, auto-vivifying); `session.rs`: `State`,
    `Question`, `Session`; `attention.rs:41`, `is_stale`, which flags every state but `done`, so the
    rail applies its two-minute rule to `starting` and `working` only (D4).
  - `crates/marley_mcp/src/resource.rs:13`: Marley's own `FLEET_RESOURCE_URI` is `fleet://snapshot`,
    the harness's URI too.
  - `crates/terminal/src/terminal.rs`: `TerminalBuilder::new_display_only` (1048),
    `TerminalType::DisplayOnly` (1599), `write_output` (2132); display-only terminals already build
    in Marley's tests (`crates/marley_workbench/src/marley_workbench_tests.rs:45`).
  - `crates/marley_rail/src/marley_rail.rs`: `RailSnapshot` (171), `Selection` (182), `Row` (253),
    `walk` (347), `switcher_rows` (574); `crates/marley_workbench/src/rail.rs`: `build_snapshot`
    (1734), the header's disclosure and menu (1040 to 1110).
  - The plan: the cast table (146 to 152), D7 (168 to 171), D8 (173 to 178), D10 (187 to 192), C1
    and C3 (204, 206), the risks (212 to 217).
  - rustal-harness (read, not built): `docs/MCP.md` (running it; resources; tools; answering,
    sending, reading, opening; errors; not yet served); `docs/FLEET.md` (the envelope; states;
    events; reading the fleet; checked with Marley's code); `docs/ACTORS.md` (fixture scripts:
    `progress`, `publish` with a state, `wait`, `output`, `sleep`, `exit`; a choice question
    published before a message wait; `actor send`, `actor stop`); `docs/TERMINALS.md` (`serve`,
    `shutdown --stop-backend` after every session is stopped); `bin/rh` (runs `target/debug/rh`,
    present, built 2026-09-25 21:33); `crates/harness-runtime/src/mcp.rs` (`tools`, 245 to 296,
    read-grant tools first).
- **Decisions:** D1 to D8 in the spec.

### Design
- **Approach.**
  - *`crates/settings_content/src/marley.rs`* (Marley's file inside a Zed crate):
    `MarleySettingsContent.harness: Option<ContextServerCommand>` with a doc (what it runs, local
    and remote examples, unset starts nothing).
  - *`marley_workbench/src/harness.rs` (new):* `HarnessSettings` (a `Settings` reading
    `content.marley.harness`); a global `Entity<Harness>` made at `init` holding the connection
    state (`Unset`, `Connecting`, `Connected`, `NotRunning { reason, retry_at }`), the
    `FleetSnapshot`, the instance, the cursor and the open views. The connect task:
    `ContextServer::stdio`, `start`, then `CallTool` `fleet_snapshot` (the seats parse as
    `FleetSnapshot`, the `instance_id` and `cursor` beside them), `ResourcesSubscribe`
    `fleet://snapshot`, `on_notification("notifications/resources/updated", ...)` sending on a
    channel that a task drains: `CallTool` `fleet_events { after: cursor }` until a page comes back
    empty, each event folded with `marley_fleet::apply`, the cursor advanced to `next_cursor`, then
    `cx.notify()`. An `isError` result naming `resync_required`, or a new `instance_id`, re-seeds.
    `wait_for_shutdown` ends the connection; the state becomes `NotRunning` with the stderr's last
    line when there is one, and a timer restarts it (1, 2, 4 ... 60 s). A settings change restarts
    the connection.
  - *The session view* (in `harness.rs`): a `TerminalView` over
    `TerminalBuilder::new_display_only`, titled with the session's title, added to the displayed
    workspace's active pane, or shown again when it is open; on open and on every change to that
    session's envelope while the view lives, `CallTool` `session_read { id, range: { mode: "tail",
    lines: 500 } }`, then `write_output` of `ESC[H ESC[2J ESC[3J` and the lines. The view's input
    goes nowhere (a display-only terminal has no PTY).
  - *`marley_rail`:* `RailSnapshot.harness: Option<HarnessGroup { state, expanded, seats:
    Vec<SeatSnapshot { id, title, state, question, silent_minutes, stale, matched }> }>`;
    `Selection::HarnessHeader` and `Selection::Seat(String)`; `Row::HarnessHeader` and `Row::Seat`;
    the walk puts the group after the projects; `parent` of a seat is the group's header; the filter
    matches a seat's title; `switcher_rows` includes seats; Next and Previous Project reach the
    header.
  - *`marley_workbench/src/rail.rs`:* the rail observes the `Harness` entity, builds the group from
    its snapshot with the clock (`silent_minutes` from `last_event_ms` for `starting` and `working`),
    draws the header and the rows, and opens a session's view on click or Enter.
  - *The inbox:* a source of entries for #508's inbox from every seat whose state is `waiting` with
    a question, in that ticket's entry shape; P1 at promotion reads #508's shipped design and names
    the seam here.
  - *`marley_workbench/Cargo.toml`:* `context_server`, `marley_fleet`.
- **File manifest.** Zed crate (a Marley file): `crates/settings_content/src/marley.rs`. Marley
  crates: `crates/marley_rail/src/marley_rail.rs`, `crates/marley_workbench/src/harness.rs` (new),
  `crates/marley_workbench/src/rail.rs`, `crates/marley_workbench/src/marley_workbench.rs` (the
  module and `init`), the inbox module #508 ships, `crates/marley_workbench/Cargo.toml`. Test
  phase: `script/e2e/534-harness-sessions-in-the-rail.sh`.
- **Ledger row (`docs/marley/zed-touchpoints.md`):** the existing
  `crates/settings_content/src/marley.rs` row gains "`harness`, an optional
  `ContextServerCommand`: the command whose stdio speaks the harness's fleet contract (#534)".

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | setup: the state root, `rh serve` in the background, the setting in the profile copy (the `python3` edit of #501's scenario); steps: settle, then the rail's Harness header reads connected | `534-01-group` |
| REQ-002 | setup: the three actors (`worker`, `asker`, `crasher`); the rail lists them with working, waiting and error | `534-01-group` |
| REQ-003 | the asker's row shows "Which base branch?" | `534-01-group` |
| REQ-004 | steps: open the asker's view; in a terminal, `rh actor inspect asker` for its incarnation, then `rh actor send asker --incarnation <id> --key base --delivery <new uuid> main`; settle 5; the asker's row has moved on | `534-03-answered` |
| REQ-005 | steps: settle 130 with worker asleep | `534-05-stale` |
| REQ-006 | steps: click worker's row; then type `x` in the view | `534-02-view` (its lines; no `x`) |
| REQ-007 | the asker's open view at `534-03-answered` shows the line it wrote after the answer | `534-03-answered` |
| REQ-008 | steps: before the answer, the inbox with the asker's entry | `534-04-inbox` |
| REQ-009 | steps: kill the `rh mcp` child by the pid its `/proc/<pid>/cmdline` names; settle 3; then settle through the first retry | `534-06-down`, `534-07-back` |
| REQ-010 | none in this scenario (it sets the setting); every other scenario's profile has none, and their rails show no group | review |

What no scenario can reach: a harness on another host over `ssh` (the setting is the same command
with `ssh` in front; the scenario runs the local one), and a real Codex session in the feed (paid,
and the fixtures cover every state the rail draws). Both go in the Phase 3 entry.

### Risks
- The harness's contract moves daily (TICKET-048 to TICKET-056 in two days). The promotion re-reads
  `MCP.md` and `FLEET.md`; Marley folds only the six v1 event kinds its reducer knows, and an
  envelope that fails to parse is logged and skipped, never fatal.
- A working session heartbeats every 10 s, and each change to a shown session re-reads 500 lines.
  The view re-reads only while it is visible and at most once per second; a hidden view re-reads
  when it is shown.
- `rh mcp` checks that the state root is owned by the user with mode 0700 and refuses otherwise;
  the group shows its stderr line as the reason, which names the check.
- Retention can renumber `session_read`'s lines, which is why the view redraws its tail instead of
  appending by line number.
- Many sessions make a long group; the header folds it.
