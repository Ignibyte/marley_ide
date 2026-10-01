---
pipeline_id: 7ba012af-a806-4450-95ce-b9043c06749d
ticket: docs/planning/tickets/open/TICKET-534-harness-sessions-in-the-rail.md
status: Phase 4 — Complete PASS
title: "Harness sessions in the rail: Marley follows rh mcp's fleet, read side only"
type: feature
slice: prong 2, C1 (the harness's read side) with C3's observer view; deliberate until the harness's M9 exits; after #533 and #508
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/queued/533-harness-contract-alignment.spec.md, docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md]
---

## Title
Marley reads rustal-harness's fleet over `rh mcp`: every harness session is a row of a Harness
section in the rail, with its state, its question and how long it has been silent, a session opens
as a read-only view of its output, and a waiting question joins the approvals inbox.

## Scope
### In
- **The setting:** `marley.harness`, Zed's `ContextServerCommand` (`command`, `args`, `env`),
  for example `{"command": "/srv/stacks/rustal-harness/bin/rh", "args": ["--state", "<root>",
  "mcp"]}`. Unset: no group, no process.
- **The connection** (D1, D3): Marley starts the command with Zed's `ContextServer::stdio`,
  seeds its own `FleetSnapshot` from `fleet_snapshot` by folding an `Upsert` (and a
  `QuestionRaised` for a seat with a question) per seat, keeps the cursor, and asks `fleet_events`
  from the cursor every second, folding each event with `marley_fleet::apply`; a page whose error
  text starts `resync_required` re-seeds. A call that fails or takes more than five seconds marks
  the connection down with its reason, keeps the rows marked stale, and starts the command again
  after 1 s, doubling to at most 60 s. A change of the setting reconnects.
- **The rail's Harness section** (D4), after the projects and the containers, drawn as the
  containers are (#614): a header with the connection's state (connected, connecting, not running
  and why), which folds the section; one row per session in the harness's order, with its title,
  its state as a word and an icon, the question's prompt while it waits on one, and #547's
  `no update in N m` for a working session past `marley.no_update_after_minutes`.
- **The session view** (D5): a click on a row opens, or shows again, a tab in the displayed
  workspace's center, a read-only view titled with the session's title, holding the session's last
  500 lines from `session_read` in the buffer font; it reads them again when the session's
  envelope changes, and every two seconds while it is open and the session not done, since an
  actor's output does not change its envelope.
- **The approvals inbox** (D6): a session waiting on a question is an entry of #508's inbox, its
  ask the prompt with the options after it; opening the entry opens the session's view.
- **Read side only** (D7): Marley calls only `fleet_snapshot`, `fleet_events` and `session_read`.

### Out (explicitly deferred)
- Every write verb (C4).
- The keyboard, the filter and the switcher reaching the Harness rows: the rail's model has no
  group outside the projects (the containers' precedent stays outside it too).
- The byte stream of an actor's pane and managed input (C3's controller half), and colours in the
  view (`session_read` gives rendered lines).
- Marley starting the harness's runtime itself and shipping `rh` (D19's slice after this one).
- Placing a session under a Marley project; Rusty's sessions (C2); brain labels (C5); views that
  come back after a restart.
- `rh mcp` over SSH (unqualified on the harness's side).

## Reference (§20)
Upstream Zed for the two seams: its MCP client (`crates/context_server`: a stdio transport,
`initialize`, requests, and notifications by method), which Marley uses as a client of the
harness rather than as an agent's tool server, and its display-only terminal
(`TerminalType::DisplayOnly` with `Terminal::write_output`), the seam the plan's D10 names for a
harness session's view. The model is the plan's (D7: `marley_fleet` is the envelope; C1). Warp:
N/A. Warp's managed agents run on Warp's servers, which the once-over ruled out; prong 2 is the
local counterpart, and nothing here copies a Warp behavior.

### Prior art
- **Behavior maps and reports.** The harness's contract, read at its state of 2026-09-25:
  `docs/MCP.md` (`rh mcp` on stdio, one client, exits when stdin closes; the resources
  `fleet://snapshot` and `fleet://events/{after}`; the tools `fleet_snapshot`, `fleet_events`,
  `session_read`; the errors, `-32010` with `resync_required`; remote clients through SSH under
  `ForceCommand`); `docs/FLEET.md` (the envelope's fields and labels, the states for Codex sessions
  and actors, heartbeats at most every 10 s while a session's activity advances, the durable
  `fleet_session` events); `docs/ROADMAP.md` (M9's exit, "Marley's fleet view can do the same once
  its adapter exists"). The Orca survey's report 07 item 5 (sessions that outlive the app belong to
  the harness client, with display-only terminals, D10) and report 06 §2.6 (Marley's verb types
  defined, no handlers). The gpui era's fleet rail (#369) and its ledger:
  AD-claude-fleet-rail-quiet-no-transport-until-brain-001 (show no invented state when no transport
  exists), BF-373-reconnect-rebuilds-empty-fleetsync (a reconnect must keep the accumulated
  snapshot).
- **Published material.** The MCP specification's resources (subscribe, `notifications/resources/
  updated`) and tools (`structuredContent`), protocol `2025-06-18`, which `rh mcp` answers.
- **Code we already ship.** `crates/context_server/src/context_server.rs` (`ContextServer::stdio`
  49, `client` 106, `start` 120, `stop` 168), `protocol.rs` (`request` 107, `on_notification` 140,
  `wait_for_shutdown` 136), `types.rs` (`CallTool` 40, `ResourcesSubscribe` and `ResourcesRead` 47
  to 58, `ResourcesUpdated` 114 to 118, `CallToolResponse::structured_content` 726);
  `crates/settings_content/src/project.rs:521` (`ContextServerCommand`) and `marley.rs` (the
  `marley` block); `crates/marley_fleet` (`FleetSnapshot`, `apply` at `reducer.rs:147`, `State`,
  `Question`, `is_stale` at `attention.rs:41`); `crates/marley_mcp/src/resource.rs:13`
  (`FLEET_RESOURCE_URI`, the same `fleet://snapshot`); `crates/terminal/src/terminal.rs`
  (`TerminalBuilder::new_display_only` 1048, `write_output` 2132, which turns LF into CRLF); the
  rail (`rail.rs`, `build_snapshot`) and `marley_rail`. The sweep's win: Zed's MCP client already
  does the transport the plan's D8 gave a crate of its own (`marley_harness`), so the adapter is a
  module over it.
- **Re-verified at promotion (2026-10-01)**, an Explore map of both repositories: M9 is complete
  (D126) and the read tools are as the draft read them, with `owner_inbox`, `delivery_status` and
  `--grant agent` added beside them. What the draft assumed and the code does not bear: an expired
  cursor comes back from `fleet_events` as an `isError` result whose text starts `resync_required`
  (the `-32010` is `resources/read`'s, and Zed's client drops error codes); Zed's client sees no
  server exit (`wait_for_shutdown` fires on a failed send only) and keeps the server's stderr to
  its own log; its typed `ResourcesSubscribe` fails on `rh`'s `{}`; `FleetSnapshot` has no seeding
  constructor and `Upsert` carries no question; the rail's model has no group outside the projects;
  `InboxEntry` has no options field; a display-only `TerminalView` would be listed as a terminal
  and restored as a shell. #611's MCP fleet provider (`fleet_providers.rs`: `connect`, `call` with a
  5 s timeout, `mcp_value`, a backoff) is the pattern this adapter follows. The scenario's fixtures
  come from the harness's own (`scripts/test_session_answer.py`, `test_actors.py`): a short 0700
  root, `serve` waited on for `"ready":true` without `TMUX`, actor scripts of `publish`, `output`
  (bytes), `sleep` (60 s at most), `wait` and `exit` steps, `actor send` with the incarnation from
  `actor inspect`, and every running workspace stopped before `shutdown --stop-backend`.

## UI proof
UI-AFFECTING. `script/e2e/534-harness-sessions-in-the-rail.sh` (`compositor sway`: it clicks
rows). Setup: a short scratch state root (mode 0700) with `rh --state <root> serve` started in the
background (the harness's `bin/rh`; the scenario stops with a message if its `target/debug/rh` is
missing and never builds the harness); three actors from scripts the scenario writes: `worker`
publishes `running`, writes three lines and sleeps; `asker` publishes the question "Which base
branch?" with `main` and `release`, waits, then writes `chose it` and publishes `running`;
`crasher` writes a line and exits 3. The run's settings gain `marley.harness` with that root and
`marley.no_update_after_minutes: 1`. Teardown stops each running workspace and runs
`rh shutdown --stop-backend`. Shots:
- `534-01-group`: the Harness section, connected: worker working, asker waiting with its question,
  crasher error;
- `534-02-view`: a click on worker: its tab with its three lines;
- `534-04-inbox`: the asker's question in the inbox, with its options;
- `534-03-answered`: after `rh actor send` answers the asker: its row moves on within five seconds,
  and its open view shows `chose it`;
- `534-05-stale`: past a minute of quiet, worker reads `no update in 1 m`;
- `534-06-down`: the `rh mcp` process killed: the header says not running, the rows stay, marked
  stale; `534-07-back` once the retry has reconnected.

## Locked-In Decisions
- D1 — The adapter is a module in `marley_workbench` over Zed's MCP client, as #611's MCP fleet
  provider is (`fleet_providers.rs`), not a crate with a transport of its own.
- D2 — `marley.harness` is a command, as a context server's is, and Marley starts nothing without
  it. One setting serves the embedded and the standalone harness (#533's D5).
- D3 — Marley's `FleetSnapshot` is the truth the section reads, built with Marley's reducer:
  seeded from `fleet_snapshot` by folding events, advanced by polling `fleet_events` each second.
  Polling, not `resources/subscribe`: Zed's client does not see the server exit, and its typed
  subscribe expects a `null` answer where `rh` gives `{}`; a poll that fails is the exit's signal.
  A reconnect re-seeds and keeps the rows until then (BF-373).
- D4 — The section shows declared state only, and silence by the rail's own rule (#547): a working
  session past `marley.no_update_after_minutes`. It sits outside the rail's model, as the
  containers do, so the keys and the filter are a later slice.
- D5 — A session's view is a read-only Marley item over `session_read`'s rendered lines, not a
  display-only terminal: the rail would list a terminal under its project, and Zed would restore
  one after a restart as a shell with the session's title.
- D6 — Questions go to #508's inbox as entries of their own kind, shown and never answered here.
- D7 — Read side only: Marley never calls a write verb.
- D8 — The contract is pinned at the harness's state on 2026-10-01 (M9 complete; the read tools
  unchanged since #533).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE `marley.harness` names a command, WHEN Marley starts, the system shall run it as an MCP server over stdio and show a Harness section in the rail with the connection's state. | Shot `534-01-group` |
| REQ-002 | WHILE connected, the section shall list one row per harness session with its title and its state. | Shot `534-01-group` |
| REQ-003 | WHILE a harness session waits on a question, its row shall show the question's prompt. | Shot `534-01-group` |
| REQ-004 | WHEN the harness publishes a change to a session, the rail shall show it within five seconds. | Shot `534-03-answered` |
| REQ-005 | WHILE a working session has had no event for `marley.no_update_after_minutes`, its row shall say how long it has been silent. | Shot `534-05-stale` |
| REQ-006 | WHEN the user clicks a session's row, the system shall show a read-only tab with the session's last 500 lines of output. | Shot `534-02-view` |
| REQ-007 | WHILE a session's view is open, it shall show the session's new output. | Shot `534-03-answered` |
| REQ-008 | WHILE a session waits on a question, the approvals inbox shall hold an entry with its prompt and options. | Shot `534-04-inbox` |
| REQ-009 | IF the harness's MCP server cannot start or stops answering, THEN the section shall say it is not running with the reason, keep its rows marked stale, and start it again with a growing delay. | Shots `534-06-down`, `534-07-back` |
| REQ-010 | WHERE `marley.harness` is unset, the rail shall show no Harness section and Marley shall start no harness command. | Review; every other scenario's rail |

## Phase Plan
- **P1 Plan** — this spec; the design and the E2E plan in the notes. At promotion: re-read the
  harness's `MCP.md`, `FLEET.md` and `STATUS.md`, and confirm #533 and #508 have shipped.
- **P2 Code** — the touchpoint row's update, then the `marley.harness` setting; the adapter module;
  `marley_rail`'s Harness group; the rail's rows; the session view; the inbox source; fmt and
  clippy clean; a review of the diff (§18.1).
- **P3 Test** — write and run `script/e2e/534-harness-sessions-in-the-rail.sh`, read every shot;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md` and `marley_rail.md`,
  the plan's prong 2 (C1 shipped, D8's crate folded into the module), the touchpoint row checked,
  ledger capture, close, archive, commit.
