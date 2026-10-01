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

## Phase 1 — Plan (promoted 2026-10-01)
- **Recall (§18.3):**
  - AD-claude-611: #611's MCP fleet provider polls a stdio MCP server through Zed's client, each
    call bounded at 5 s, with a backoff; the harness adapter follows it.
  - BF-373: a reconnect must not rebuild an empty fleet; the rows stay, marked stale, until the
    re-seed.
  - AD-claude-614: the containers section sits outside the rail's model and its keys; the Harness
    section does the same in this slice.
  - AD-claude-508 and F-claude-508: the inbox lists every agent that waits; a new entry must reach
    the rail through a change the rail observes.
  - Brain (consultation ce2307479b51457a96b43e4cb8095f54): nothing on this seam.
- **Re-binding:** the Explore map's fourteen points (the spec's Prior art); the decisions moved
  to polling, a read-only Marley item, the section outside the rail's model, and #547's quiet rule.

### Design
- **Setting:** `harness: Option<ContextServerCommand>` in `MarleySettingsContent`;
  `MarleySettings::harness`.
- **`marley_workbench::harness`** (new): a global `Harness { connection: Connection, seats:
  FleetSnapshot, cursor, server: Option<Arc<ContextServer>>, run: Option<Task<()>>, command }`,
  `Connection::{Connecting, Connected, Down(reason)}`. `init` starts the run when the setting
  names a command and restarts it when the setting changes. The run: connect
  (`ContextServer::stdio` and `start`), `fleet_snapshot` (structured content
  `{instance_id, cursor, seats}`) folded into a fresh `FleetSnapshot` as an `Upsert` and a
  `QuestionRaised` per seat, then every second `fleet_events { after: cursor }` pages folded with
  `apply` until a page is empty; an error result starting `resync_required` re-seeds; any other
  failure or a 5 s timeout sets `Down(reason)` (the root cause's first line), drops the server,
  waits 1, 2, 4 … 60 s and connects again. The global changes only when the seats, the cursor or
  the connection do, so its observers redraw on a change.
- **`HarnessView`** (in the module): a workspace item titled with the session's title, its lines
  from `session_read { id, range: { mode: tail, lines: 500 } }` in the buffer font in a scrolled
  column; it reads them when it opens, when its seat's `last_event_ms` or state changes, and every
  2 s while its seat is not done. `open(workspace, id)` shows the open one for that id or adds one
  to the center.
- **Rail:** `render_harness` after `render_containers`: the header (`HARNESS` and the
  connection's state, a click folding the rows) and a row per seat (an icon and the state's word,
  the title, the question's prompt while waiting, `no update in N m` for a working seat past
  `no_update_after_minutes`; muted while the connection is not up); a click opens the view; hidden
  while filtering. The rail observes the `Harness` global. The inbox gains `InboxKind::Harness` and
  `InboxTarget::Harness(id)`: each waiting seat with a question is an entry, its ask the prompt
  and its options in parentheses, project `Harness`.

### File manifest
- Marley: `crates/marley_workbench/src/harness.rs` (new), `marley_workbench.rs`, `rail.rs`;
  `crates/marley_rail/src/marley_rail.rs` (`InboxKind::Harness`);
  `script/e2e/534-harness-sessions-in-the-rail.sh`.
- Zed: `crates/settings_content/src/marley.rs` (the field; its row widened first).

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, REQ-002, REQ-003 | the harness serving worker, asker and crasher | `534-01-group` |
| REQ-006 | a click on worker's row | `534-02-view` |
| REQ-008 | the inbox, while asker waits | `534-04-inbox` |
| REQ-004, REQ-007 | asker's view open; `rh actor send` | `534-03-answered` |
| REQ-005 | past a minute of quiet (`no_update_after_minutes: 1`) | `534-05-stale` |
| REQ-009 | `rh mcp` killed; then reconnected | `534-06-down`, `534-07-back` |
| REQ-010 | review: no setting, no section, no process | the diff |

## Phase 2 — Code (2026-10-01)
- **Built:** `marley.harness` (`settings_content`, `MarleySettings::harness`); `harness.rs`: the
  `Harness` global (the command, the connection, the fleet, the section's fold), the follow loop
  (start bounded at 5 s, `fleet_snapshot` deserialized into a `FleetSnapshot`, `fleet_events`
  every second folded with `apply`, a `resync_required` text re-seeding, any other failure
  marking the connection down with its root cause and starting again after 1, 2, 4 … 60 s),
  `HarnessView` (the session's last 500 lines from `session_read`, read again on an envelope
  change and every 2 s while the session runs) and `open`; in `rail.rs`, the observer, the Harness
  section (`render_harness`, `harness_row`), `harness_entries` for the inbox, `InboxTarget::Harness`
  and `open_harness`; `InboxKind::Harness` in `marley_rail`.
- **Deviations:**
  - The snapshot is deserialized, not folded from made-up events: the harness's conformance check
    (`harness-conformance`) asserts that its snapshot is Marley's own fold of its events, so the
    two are the same.
  - The fold state lives in the `Harness` global, not on `Rail`: clippy's bool limit for `Rail`,
    and its change redraws the rail through the observer it already has.
- **Review:** every call is bounded at 5 s, so a harness that stops answering is seen within
  about 6 s; the old run, and its server, go when the setting changes; the global changes only
  when events arrive, a minute passes while a session works, or the connection moves, so the rail
  is not rebuilt each second; the view's reads run one at a time.
- **Clippy found:** a binding too like another, an unneeded qualification, `Rail`'s bools,
  `Rail::new`'s length, four needlessly mutable parameters.
- **Gate:** GREEN, 17 PASS.


## Phase 3 — Test (2026-10-01)
- **Proved first by hand:** a 0700 root under the runtime folder, `rh serve` waited on for
  `"ready":true`, an actor, `rh fleet` (the worker `working`), `rh mcp` answering `initialize` and
  `session_read` (`one`, `two`, `three`), and the teardown (`rh list`, `rh stop`, `shutdown
  --stop-backend`).
- **Scenario:** `script/e2e/534-harness-sessions-in-the-rail.sh` (sway): the harness's built
  `bin/rh`, a root at `$XDG_RUNTIME_DIR/rh534-<pid>`, worker, asker and crasher,
  `marley.harness` and `no_update_after_minutes: 1`. Two runs.
- **First run (the group shot only, to place the clicks):** the section showed connected with
  the three sessions; but the harness adds `[wait base, generation …]` to an actor's prompt, which
  pushed the options out of the inbox's line, and the terminal ran the user's own shell files.
- **Fix:** `harness::shown_prompt` drops that routing note from the rows and the inbox (the
  question keeps it); the scenario gives the terminal a scratch HOME. Gate GREEN, 17 PASS.
- **Second run:**
  - `534-01-group` (REQ-001 to REQ-003): `HARNESS connected`; worker `working`, asker its
    question, crasher `error`, each with its state's dot.
  - `534-02-view` (REQ-006): a click on worker: a `worker` tab holding `one`, `two`, `three`.
  - `534-04-inbox` (REQ-008): `Needs you 1`: `asker · Harness`, `Which base branch? (main,
    release)`.
  - `534-03-answered` (REQ-004, REQ-007): asker's tab open, `rh actor send … main`: within five
    seconds asker's row reads `working`, the inbox is empty, and its tab shows `chose it`.
  - `534-05-stale` (REQ-005): past a minute, worker reads `no update in 1 m`.
  - `534-06-down` (REQ-009): `rh mcp` killed and its root refused: `not running: sending into a
    closed c…`, the rows kept, each `· stale` with a grey dot; the open tab shows its failed read
    in red above its kept lines.
  - `534-07-back` (REQ-009): the root allowed again: `connected`, the rows as before, the tab's
    error gone.
- **Review only:** REQ-010 (no setting, no section, no process): `follow_setting` starts nothing
  without a command, and `render_harness` draws nothing without a connection; every other
  scenario's rail shows no Harness section.

## Phase 4 — Complete (2026-10-01)
- **Documented:** `CHANGELOG.md`; the guide ("The harness's sessions"); `marley_workbench.md`
  (the harness module); `marley_rail.md` (`InboxKind::Harness`); the plan's C1 row; the ledger
  row of `settings_content/src/marley.rs`.
- **Knowledge:** F-claude-534-the-harnesss-routing-note-hid-the-options-001,
  L-claude-534-zeds-mcp-client-sees-no-server-exit-001,
  L-claude-534-a-harness-scenario-refuses-the-root-to-hold-a-down-state-001,
  AD-claude-534-the-harness-is-followed-by-polling-in-a-section-outside-the-rails-model-001.
- **Brain:** the consultation closed with `brain decide`.
