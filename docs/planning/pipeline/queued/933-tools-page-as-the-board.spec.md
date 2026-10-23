---
pipeline_id: bf12e69f-ce30-49f1-8e9d-bd975c3c025f
ticket: docs/planning/tickets/open/TICKET-933-tools-page-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: MCP Servers and Agent Activity as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/F-tools.md, docs/marley/look-plan.md]
---

## Title
MCP Servers with its subtitle, agent cards with a lettered tile, a state pill and integration
chips, and Marley's own server as its own card; Agent Activity's rows as the Tools board lays
them out, each with the call's result or error, which Marley now records.

## Scope
### In
- **Page subtitle** (Marley crate, `mcp_servers.rs`, `McpServers::render`). After the headline,
  muted: "What each agent can reach, and what Marley needs of it".
- **Agent card head** (`mcp_servers.rs`, `render_agent`). A 34 px rounded tile with the agent's
  mark (`CC`, `CX`) in its tint and ink; beside it the name with the version found ("Claude Code
  2.1.304") over the program's path, or "not on Marley's PATH", in the buffer font; at the right a
  state pill: "ready" (success) when a version was read, "missing" (error) when the program is not
  on the PATH, "version unknown" (warning) when `--version` did not read, "checking" (muted)
  before the first check.
- **Integrations as chips** (`render_agent`). One chip per integration in a wrapping row: a dot
  and the integration's name. The dot is green while it is on, amber while the agent is older than
  it needs, grey otherwise. The chip's tooltip holds what the row says now: its state in words and
  "tested on …". "Run … to update it." stays under the chips while an integration needs a newer
  agent.
- **Marley's own server card** (`mcp_servers.rs`, new `render_marley_server`). Its own card above
  the Zed card: an `M` tile; "marley" and, muted, "Zed's agents · <status>" ("running in repo",
  from the same context server statuses `zed_rows` reads); the line "<N> tools: editor, terminal,
  browser, settings, threads. Every call that acts is listed in Agent Activity."; a pill with the
  status ("running" in success, "starting" muted, "error" in error). N is the count of tools
  Marley's server lists to its own agents now (`marley_mcp::tools_list_for(&Principal::Marley,
  &enabled_tools(cx))`). The Zed card keeps every other context server and drops the `marley` row.
- **Activity rows** (`agent_activity.rs`, `render_rows`). Each row a rounded card: a 28 px tile
  with the caller's mark; the tool in the buffer font and accent colour with the time (`HH:MM`) at
  the right; what it acted on, one line, truncated; then how it ended, muted, or in the error
  colour for a refusal. Home's AGENT ACTIVITY card keeps one compact line per call, as the Home
  board draws it: time, tool, what (`render_rows` takes a `RowLayout` of `Card` or `Line`).
- **A call's result and error recorded** (`agent_activity.rs`, `ActivityRow`, `Outcome`, `log`).
  - `Outcome::Done` carries `result: Option<String>`, a line taken from the answer by
    `result_line`: "you accepted" or "you refused" for a receipt's `result` (#367), "exit N" for an
    `exit_code` and "running" while `running` is true, "saved" for `saved`, else
    the first line of the answer's text; redacted as the summary is, cut to 120 characters.
  - `Outcome::Refused` carries `reason: Option<String>`, the refusal's reason cut to its first
    sentence and 120 characters; the row shows it, or "refused: <code>" when there is none.
  - Both new fields default when absent, so today's day files still read.
- **The caller's mark** (`agent_activity.rs`, `click_pause.rs` `Who`). `Who` gains the caller's
  `AgentMark` (#905's `item_kind.rs`): a terminal's Claude Code `CC`; Zed's agent `Z`; Claude
  Code's and Codex's adapters by the client names they send at `initialize` (`CC`, `CX`); any
  other client by its name's first letter. `ActivityRow` stores it (`agent: Option<AgentMark>`,
  default `None`, drawn as a neutral tile with the first letter of `who`).

### Out (explicitly deferred)
- Agent Activity as a column beside MCP Servers, with "Stop all": #846 Out ("Agent Activity …
  keep their tabs"). The tab stays its own.
- Rusty's calls in the activity list: #934.
- The bottom row on MCP Servers ("MCP Servers", the "Codex is not on the PATH" chip, "marley · 90
  tools"): the review lists it as missing; #909 (the bottom row for the other screens) owns it.
- A day label ("Oct 10") on rows of earlier days: the tab loads today's day file only.
- The header's single accent "Add Server": each card keeps its own Add Server, which knows where
  the server goes; the review names no gap there.

## Reference (§20)
N/A — Marley-specific: MCP Servers (#720, #846) and Agent Activity (#703) are Marley's own tabs,
drawn with Zed's `ui` components. The context server statuses come from Zed's
`project::context_server_store` as read today; no Zed behaviour changes.

### Prior art
- `crates/marley_workbench/src/mcp_servers.rs:358-439` `McpServers::render`: the headline with
  Refresh and Configure in Settings, no subtitle (`:376`). Gap holds at HEAD.
- `mcp_servers.rs:488-560` `render_agent`: `Label(display_name)` and one muted label with "version
  at path" (`:494-512`); one row per integration with its state word and "tested on …" inline
  (`:513-550`); the update line (`:551-559`). No tile, no pill. Gap holds.
- `mcp_servers.rs:430-437` the "ZED" card from `zed_rows` (`:653-722`): `marley` is a row like the
  others, flagged `marleys` (`:701-702`); `status_words` (`:737-751`) words Zed's
  `ContextServerStatus`. No tool count (grepped "tools:" in `mcp_servers.rs`: none).
- `marley_agent::versions` `Found::{Version, Unreadable, Missing}` and
  `crate::agent_versions::{found, program, wanted, is_on, outdated}`: everything the pill and the
  chips need is read already.
- `crates/marley_mcp/src/registry.rs:922-946` `tools_list_for` and
  `crates/marley_workbench/src/mcp.rs:232` `enabled_tools` (private today): the count an agent of
  Marley's sees.
- `crates/marley_workbench/src/agent_activity.rs:66-87` `ActivityRow` and `Outcome { Done,
  Refused { code } }`; `:164-187` `log` hooks `AppCall::on_answer`, which hands over the whole
  `Result<ToolAnswer, Refusal>` (`crates/marley_mcp/src/marley_mcp.rs:220`, `:236-250`), so the
  result and the reason are there to keep. `:363-407` `render_rows`: one table line. Gaps hold.
- `marley_mcp::ToolAnswer { structured, text, image }` (`marley_mcp.rs:256-263`); output schemas
  carry `exit_code` (`registry.rs:2709`, `:3365`), `running` (`:3364`), `saved` (`:3145`) and a
  receipt's `result` `accepted`/`refused` with its `reason` (`receipt_schema`, `:3445-3456`).
- `crates/marley_workbench/src/click_pause.rs:56-106` `Who::of`: sorts a caller into Claude Code
  in a terminal, Zed's agent, an outside client or an unknown agent, in words only.
- Queued #905 (`item_kind.rs`): `AgentMark` (`CC`, `CX`, `G`, `OC`, `M`, `R`, first letter
  otherwise) with `tint` and `ink` through the theme; its D1 says other screens reuse it. #903's
  `home_page.rs` `glyph_tile` and `card_with` for the card and tile shapes.
- `home_page.rs:294-310` `render_activity`: Home's card calls `render_rows(ACTIVITY_SHOWN, cx)`.
- The Tools and Home boards (`Tools.dc.html`, `Home.dc.html` in the design record).
- Specs: #703 (the log; AD-claude-703), #720 (the page), #846 (the cards; its Out keeps Agent
  Activity a tab).

## UI proof
Scenario `script/e2e/933-tools-page-as-the-board.sh` (sway). Setup: a scratch repository; a
stand-in `claude` first on the PATH that answers `--version` with a version and otherwise sleeps;
no `codex` on the PATH; the run's Claude Code config its own (#877); #703's scripted MCP client,
run through the plugin's bridge, makes the calls.
- `933-01-servers`: MCP Servers: the subtitle, Claude Code's card with its tile, path, "ready"
  pill and integration chips, Codex's card with "missing", Marley's card with the tool count and
  "running" (REQ-001 to REQ-004).
- `933-02-chip`: the pointer on an integration chip: its tooltip with the state and "tested on"
  (REQ-003).
- `933-03-activity`: Agent Activity after the client's `terminal_run` of `echo hello` and a
  `terminal_run` on a terminal that does not exist: two cards with the tile, tool and time, what,
  then "exit 0" and the refusal's reason in red (REQ-005, REQ-006, REQ-007).
- `933-04-home`: Home's AGENT ACTIVITY card: one line per call, time, tool, what (REQ-008).
- `933-05-day`: MCP Servers and then Agent Activity in Marley Day (REQ-001, REQ-005).

## Locked-In Decisions
- D1 — The tiles and their colours come from #905's `item_kind` (`AgentMark`), so the page, the
  rows, the tabs and the rail agree. #905 lands first.
- D2 — The state pill words what Marley found: ready, missing, version unknown, checking. An
  outdated integration stays the chip's amber dot and the update line, not a pill.
- D3 — Marley's own server leaves the Zed card for a card of its own, as the board draws it; the
  Zed card keeps every other context server, so nothing listed today goes away.
- D4 — The tool count is the live count Marley's server lists to its own agents, never a
  constant; the families named after it are the board's fixed words.
- D5 — A call's result is one line, read from fields the tools' output schemas already name;
  anything else is the answer's first text line. It is redacted and cut as the summary is. No
  image or whole answer is stored.
- D6 — Home's card stays compact (the Home board's lines); the tab draws the cards.
- D7 — Old day files keep reading: every new field has a serde default.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | MCP Servers shall show "What each agent can reach, and what Marley needs of it" beside its title. | `933-01-servers`, `933-05-day` |
| REQ-002 | Each agent's card shall show a lettered tile, the name with its version over its path, and a state pill: "ready" when a version was read, "missing" when the program is not on Marley's PATH. | `933-01-servers` |
| REQ-003 | Each agent's integrations shall be chips with a dot for their state, and a chip's tooltip shall hold its state and the versions it was tested on. | `933-01-servers`, `933-02-chip` |
| REQ-004 | Marley's own server shall have its own card with an `M` tile, its status in projects, the count of tools Marley's server lists, the line naming Agent Activity, and a status pill; the Zed card shall list the other context servers. | `933-01-servers` |
| REQ-005 | Each Agent Activity row shall show the caller's tile, the tool and the time, what it acted on, and how it ended. | `933-03-activity`, `933-05-day` |
| REQ-006 | WHEN a write-tier call is answered with a result, its row shall show that result's line ("exit 0", "you accepted", "saved", or the answer's first line). | `933-03-activity` |
| REQ-007 | WHEN a write-tier call is refused, its row shall show the refusal's reason in the error colour. | `933-03-activity` |
| REQ-008 | Home's AGENT ACTIVITY card shall show one line per call: time, tool and what it acted on. | `933-04-home` |
| REQ-009 | The day file shall store each call's result or reason, and rows written before this change shall still read. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `mcp_servers.rs`, `agent_activity.rs`, `click_pause.rs`, `home_page.rs`, `mcp.rs`
  (`enabled_tools` made `pub(crate)`); a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
