---
pipeline_id: 7ba012af-a806-4450-95ce-b9043c06749d
ticket: docs/planning/tickets/open/TICKET-534-harness-sessions-in-the-rail.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Harness sessions in the rail: Marley follows rh mcp's fleet, read side only"
type: feature
slice: prong 2, C1 (the harness's read side) with C3's observer view; deliberate until the harness's M9 exits; after #533 and #508
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/queued/533-harness-contract-alignment.spec.md, docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md]
---

## Title
Marley reads rustal-harness's fleet over `rh mcp`: every harness session is a row of a Harness
group in the rail, with its state, its question and how long it has been silent, a session opens
as a read-only view of its output, and a waiting question joins the approvals inbox.

## Scope
### In
- **The setting:** `marley.harness`, a command in the shape of a context server's (`command`,
  `args`, `env`; Zed's `ContextServerCommand`), for example
  `{"command": "/srv/stacks/rustal-harness/bin/rh", "args": ["--state", "<root>", "mcp"]}`, or an
  `ssh` command for a harness on another host. Unset: no group, no process (D2).
- **The connection** (D1, D3): at startup, when the setting is there, Marley starts the command
  with Zed's `context_server::ContextServer::stdio` and keeps one connection for the app's life;
  `fleet_snapshot` seeds Marley's own `FleetSnapshot` with the harness's instance and cursor;
  `resources/subscribe` on `fleet://snapshot`; each `notifications/resources/updated` pages
  `fleet_events` from the cursor, folding each event with `marley_fleet::apply`; a
  `resync_required` error (`-32010`) or a changed instance re-reads `fleet_snapshot`. When the
  server exits or cannot start, the group says so with the reason, the last rows stay, marked
  stale, and Marley starts it again after 1 s, doubling to at most 60 s.
- **The rail's Harness group** (D4), after the projects: a header with the connection's state
  (connected, connecting, not running and why); one row per session in the feed's first-seen
  order, with its title, its state as a word and an icon (starting, working, idle, waiting, error,
  done), the question's prompt while it waits on one, and "no update in N m" for a starting or
  working session silent for two minutes or more. The header folds the group, as a project header
  does; the keyboard, the filter (by title) and the switcher reach the rows.
- **The session view** (D5): Enter or a click on a row opens, or shows again, a display-only
  terminal tab in the displayed workspace's center, titled with the session's title, holding the
  session's last 500 lines from `session_read`, read again whenever that session's envelope
  changes and redrawn in place. Keys typed in it reach nothing.
- **The approvals inbox** (D6): a session waiting on a question is an entry of #508's inbox, with
  its prompt, its options and its age; opening the entry opens the session's view.
- **Read grant only** (D7): Marley never passes `--grant write`.

### Out (explicitly deferred)
- Every write verb: `session_answer` from the question's options, `session_send`, `session_open`,
  `session_stop`, and the harness's surface verb (C4, after this ticket).
- The byte stream of an actor's pane and managed input (C3's controller half, over the harness's
  socket protocol), and colors in the view (`session_read` gives rendered lines without escapes).
- Marley starting the harness's runtime itself and shipping `rh` (the embedding slice after this
  one, plan D19 of #533).
- Marley's own `fleet_snapshot` tool serving the sessions Marley follows, and listing the `fleet`
  family.
- Placing a session under a Marley project: the envelope names no folder.
- Rusty's agent sessions (C2) and brain labels (C5).
- Session views that come back after a restart.

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

## UI proof
UI-AFFECTING. `script/e2e/534-harness-sessions-in-the-rail.sh` (`compositor sway`: it clicks
rows). Setup: a scratch harness state root (mode 0700) with `rh --state <root> serve` started in
the background, the harness's built `bin/rh` (the scenario stops with a message if
`target/debug/rh` is missing; it never builds the harness); three fake actors from scripts the
scenario writes (`rh actor new`): `worker` publishes `running`, writes three lines and sleeps;
`asker` publishes the choice question "Which base branch?" with `main` and `release` before a
message wait; `crasher` writes a line and exits 3. The profile copy's settings gain
`marley.harness` with that root. Teardown stops the actors and runs `rh shutdown --stop-backend`.
Shots: `534-01-group` (the Harness group: worker working, asker waiting with its question, crasher
error), `534-02-view` (a click on worker: its view with its three lines), `534-03-answered` (the
asker answered from a terminal with the harness's fixture command `rh actor send`: its row moves
on within five seconds and its open view shows its new line), `534-04-inbox` (the asker's question
in #508's inbox, taken before the answer), `534-05-stale` (after two quiet minutes: worker reads
"no update in 2 m"), `534-06-down` (the harness's MCP process killed: the header says not
running, the rows stay, marked stale; `534-07-back` when the retry has reconnected).

## Locked-In Decisions
- D1 — The adapter is a module in `marley_workbench` over Zed's MCP client
  (`ContextServer::stdio`), not a crate with a transport of its own: Zed's client already speaks
  stdio MCP, resources and notifications, and `marley_fleet` already holds the model. Plan D8's
  `marley_harness` shrinks accordingly (#534's P4 says so in the plan).
- D2 — `marley.harness` is a command, as a context server's is, and Marley starts nothing without
  it: locally `rh --state <root> mcp`, remotely the same over `ssh`, which is the harness's own
  path for remote clients. One setting serves the embedded and the standalone harness (#533's D5).
- D3 — Marley's `FleetSnapshot` is the truth the rail reads, built with Marley's reducer: seeded by
  `fleet_snapshot`, advanced by folding `fleet_events` pages from the cursor on each change
  notification, and re-seeded on `resync_required` or a new instance. A reconnect keeps the
  snapshot and its cursor, never an empty one (BF-373).
- D4 — The group shows declared state only: the envelope's `state`, its `question`, and silence as
  "no update in N m" for a starting or working session after two minutes, so an idle or waiting
  session's legitimate quiet never flags; `error` never reads as idle or done, which `State`
  already guarantees.
- D5 — A session's view is a display-only Zed terminal fed `session_read`'s last 500 lines, redrawn
  when that session's envelope changes: the MCP contract gives rendered lines, and D10's byte
  stream and input claims need the harness's socket protocol (C3's second half).
- D6 — Questions go to #508's inbox as entries of their own kind, shown and never answered here;
  answering is C4's.
- D7 — Read grant only: `rh mcp` without `--grant write` lists no write tool and refuses any.
- D8 — Deliberate until the harness's M9 exits: the contract is still growing (TICKET-056, the
  surface verb, was open on 2026-09-25), and the Planner pins the contract at the harness's state
  on promotion.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE `marley.harness` names a command, WHEN Marley starts, the system shall run it as an MCP server over stdio and show a Harness group in the rail with the connection's state. | Shot `534-01-group` |
| REQ-002 | WHILE connected, the Harness group shall list one row per harness session with its title and its state. | Shot `534-01-group` |
| REQ-003 | WHILE a harness session waits on a question, its row shall show the question's prompt. | Shot `534-01-group` |
| REQ-004 | WHEN the harness publishes a change to a session, the rail shall show it within five seconds. | Shot `534-03-answered` |
| REQ-005 | WHILE a starting or working session has had no event for two minutes, its row shall say how long it has been silent. | Shot `534-05-stale` |
| REQ-006 | WHEN the user opens a session's row, the system shall show a display-only terminal tab with the session's last 500 lines of output, and keys typed there shall reach nothing. | Shot `534-02-view` |
| REQ-007 | WHEN a shown session's envelope changes, its view shall show the session's output again. | Shot `534-03-answered` |
| REQ-008 | WHERE the approvals inbox (#508) is present, a session waiting on a question shall be an entry of it with its prompt and options. | Shot `534-04-inbox` |
| REQ-009 | IF the harness's MCP server cannot start or exits, THEN the group shall say it is not running with the reason, keep its rows marked stale, and start it again with a growing delay. | Shots `534-06-down`, `534-07-back` |
| REQ-010 | WHERE `marley.harness` is unset, the rail shall show no Harness group and Marley shall start no harness command. | Review; every other scenario's rail, whose profile has no such setting |

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
