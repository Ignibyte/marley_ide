---
pipeline_id: 830e0d11-7b64-4bfc-aafb-e68fa4dab084
ticket: docs/planning/tickets/open/TICKET-547-claude-code-events-slice-2.md
status: Phase 4 — Complete PASS
title: "Claude Code's events, slice 2: the plugin update, fleet_snapshot, and stale rows"
type: feature
slice: prong 2, C1's first piece, second slice (after #519)
references: [docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.notes.md, docs/planning/pipeline/completed/482-claude-code-notifications-chip.spec.md, docs/orca_architecture/01-agents-and-sessions.md]
---

## Title
#519 put Claude Code's hook events into a `marley_fleet` seat per terminal and onto the rail.
Three parts of its design land here. An existing install of Marley's plugin (1.1.0 or older) never
gets the new hook until it is updated, so the agent bar offers the update. The MCP server's
`fleet_snapshot` has been empty since the fork; it now answers the same seats the rail reads, so
an agent (or the harness) can see every Claude Code session Marley hosts. And a working row whose
Claude Code has gone quiet for half an hour (an Escape fires no hook) says `no update in N m`
instead of claiming to work.

## Scope
### In
- **The update chip.** `ClaudePlugin` reads the installed version of `marley@marley` from
  `installed_plugins.json` beside whether it is installed. While that version is older than the
  version Marley ships (`plugin.json`, 1.2.0), the agent bar's chip reads "Update Marley's
  plugin". A click writes the marketplace again, runs `claude plugin marketplace update marley`
  and `claude plugin update marley@marley`, and shows a toast (restart a running Claude Code to
  use it) or the error. An install that fails leaves the chip as it was.
- **`fleet_snapshot` and `fleet://snapshot`.** The `McpServer` global keeps the transport's
  shared data. Each change of `AgentEvents` replaces the server's snapshot with the app's and
  calls `transport::signal_change`, so the tool answers the seats and a subscribed client gets
  `resources/updated`. A seat's id is its terminal's id as `terminal_list` gives it.
- **Seats end with their Claude Code.** When a terminal whose seat is live no longer has Claude
  Code in its foreground (it exited without a `SessionEnd` frame, or was killed), the rail's
  refresh ends the seat (`Ended`, so `done`). A closed terminal's seat goes, as in #519.
- **Stale rows.** A new setting, `marley.no_update_after_minutes` (default 30), in the settings
  schema, `default.json` and the Marley settings page's Agents section. A working seat whose
  last event is at least that old, with Claude Code still in the foreground, shows
  `no update in N m` where its row said `working`. While a working seat is listed the rail
  refreshes once a minute so the minutes move.
- The stand-in agent (`script/e2e/browser-fixture.sh`) gains a `fleet` command that calls
  `fleet_snapshot` and prints each seat.

### Out (explicitly deferred)
- The approvals inbox (#508) and per-turn diffs (#509), which read these seats.
- Rail attention order and summaries (#542); notifications that say what happened (#538).
- A seat for Codex, Gemini or OpenCode (their hooks), and seats for remote hosts.
- Dropping the fold's own labels (`lead_tool:…`, `waiting_on`) from what `fleet_snapshot`
  answers: labels are opaque chips by the fleet's contract, and the tools in flight are honest
  data for a reader.

## Reference (§20)
- **Orca:** a working agent's status decays after 30 minutes without a hook
  (`src/shared/agent-status-freshness.ts`, report 01 §2.3 and item 1); Marley shows the decay
  as `no update in N m` and never as idle or done, as #519's D7 locked.
- **Upstream Zed:** the Settings window renders a `u64` setting with its editable number field
  (`settings_ui`, `add_basic_renderer::<u64>(render_editable_number_field)`); the Marley page's
  new item uses it, as Zed's own numeric settings do.
- **Warp:** none. Warp's Claude Code support ships as its own notification plugin, and Warp's
  docs do not describe updating it; nothing of Warp's was read.

### Prior art
- **Code we ship.** `marley_mcp` already serves a `FleetSnapshot` through `fleet_snapshot`
  (`tools::fleet_snapshot_result`, `Family::Fleet` in `dispatch.rs`) and the `fleet://snapshot`
  resource with subscriptions, and `transport::signal_change` wakes the SSE threads after the app
  replaces `ServerData.snapshot` (`crates/marley_mcp/src/transport.rs:34`, `:369`); nothing has
  fed it since the forge client was removed (AD-claude-fleet-rail-quiet-no-transport-until-brain-001).
  `marley_fleet::is_stale` (`src/attention.rs:41`) flags a seat whose last event is older than a
  threshold. `claude_plugin::install` and `crate::run_program` already run `claude plugin`
  commands off the main thread. `semver` is a workspace dependency. Zed's `settings_ui` renders
  `u64` settings.
- **Published material.** Claude Code 2.1.283's CLI: `claude plugin update [options] <plugin>`
  ("restart required to apply") and `claude plugin marketplace update [name]` ("updates
  marketplace(s) from their source"), read from `--help` on the box. `installed_plugins.json`
  (format 2) lists each plugin's entries with `scope` and `version`; a local marketplace's plugin
  carries its `plugin.json` version, other plugins may carry a commit hash. MCP's resources
  subscription (`resources/subscribe`, `notifications/resources/updated`).
- **Behavior maps.** Orca's 30-minute decay (above).

## UI proof
UI-AFFECTING: the agent bar's chip and the rail's rows.
`script/e2e/547-claude-code-events-slice-2.sh` (`compositor sway`, for the chip's click). The
stand-in `claude` of #519, first on Marley's PATH and the terminal's, now also answers
`plugin …` by logging its arguments and, for `plugin update marley@marley`, writing 1.2.0 into
the scratch `CLAUDE_CONFIG_DIR`'s `installed_plugins.json`, which setup seeds with
`marley@marley` at 1.1.0. The profile's settings set `marley.no_update_after_minutes` to 1.
- `547-01-update-chip`: the stand-in at its prompt; the agent bar reads "Update Marley's plugin".
- `547-02-updated`: after the click, the toast and no chip; the log holds the two commands in
  order.
- `547-03-no-update`: a prompt and a PreToolUse, then 70 seconds with no event: the row reads
  `no update in 1 m`.
- Checks through the stand-in agent: `fleet_snapshot` lists the terminal's seat while it works
  (`working`, the prompt), after Stop (`idle`, the last message), and after the stand-in exits
  (`done`); after the terminal closes, it lists none. (A failed seat stays `error` when Claude
  Code leaves: the reducer keeps a failed seat failed.)

## Locked-In Decisions
- D1: The chip compares versions with `semver` and offers the update only while the installed one
  is older than the shipped one. An unparseable or newer installed version gets no chip: two
  Marley builds of different ages must not keep offering each other's plugin.
- D2: The shipped version is read from the embedded `plugin.json` at run time, not a constant
  kept beside it, so the two cannot drift.
- D3: The update rewrites the marketplace first, then runs Claude Code's own commands, as the
  install does (AD-claude-482); Marley never edits Claude Code's files itself.
- D4: The snapshot is published by an observer of `AgentEvents` in `mcp.rs`, so `agent_events`
  knows nothing of the server, and a server that did not start makes publishing a no-op.
- D5: The published snapshot is the app's as it is, the fold's labels included (Out, above).
- D6: A seat ends when the rail's refresh sees its terminal without Claude Code in the
  foreground. Ending it is idempotent, so several windows' rails may all do it.
- D7: The threshold is a setting because a scenario cannot wait 30 minutes and Chad may want a
  shorter one; the rail reads it on each refresh. The stale form replaces the state word only;
  the status stays `working` for everything else that reads it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `installed_plugins.json` lists `marley@marley` at a version older than the one Marley ships, a Claude Code terminal's agent bar shall offer "Update Marley's plugin". | Shot `547-01-update-chip` |
| REQ-002 | WHEN the user clicks the update, Marley shall rewrite its marketplace, run `claude plugin marketplace update marley` and then `claude plugin update marley@marley`, and on success say so and stop offering it. | Shot `547-02-updated`; the stand-in's plugin log |
| REQ-003 | WHEN a terminal's Claude Code has sent events, `fleet_snapshot` shall list a seat whose id is the terminal's id, with the seat's state and labels, and follow each change. | The run log: `fleet` after a prompt (`working`) and after Stop (`idle`, the last message) |
| REQ-004 | WHEN Claude Code leaves a terminal's foreground, its seat shall read `done`; WHEN the terminal closes, `fleet_snapshot` shall no longer list it. | The run log: `fleet` after the stand-in exits, and after the terminal closes |
| REQ-005 | WHILE a working seat has had no event for `marley.no_update_after_minutes` and Claude Code is still the foreground, its row shall read `no update in N m` in place of `working`. | Shot `547-03-no-update` (the setting at 1) |
| REQ-006 | The Marley settings page's Agents section shall show `No Update After Minutes`, and `default.json` shall set it to 30. | The 515 scenario's page shot; review of `default.json` |
| REQ-007 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec, the design and the e2e plan in the notes.
- **P2 Code:** the ledger rows first (`settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`); the setting; the chip and the update; the publishing; the
  seat's end; the stale form and the rail's minute timer; the stand-in's `fleet`. fmt and
  clippy clean; a review of the diff.
- **P3 Test:** write and run the scenario, read every shot, the golden set with 547 in it,
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
