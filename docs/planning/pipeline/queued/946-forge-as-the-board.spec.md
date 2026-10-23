---
pipeline_id: 73b0e417-4d89-4de4-813b-1c1646340b31
ticket: docs/planning/tickets/open/TICKET-946-forge-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Forge's board and online agents as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/G-system.md, docs/marley/look-plan.md]
---

## Title
Forge as the ForgeBoard and ForgeAgents boards draw it. The sprint board gets coloured kind pills,
boxed phase chips, a status dot on each column and an agent tile on each card; its detail pane gets
a phase progress strip, a QA failed callout, comment authors and Open the agent. In the Fleet
panel, Forge's agents get their harness, ticket title, folder and "output N ago", hosts their agent
count, and the Forge source its connection: endpoint, token source, projects and latency.

## Scope
### In
- `crates/marley_workbench/src/forge_board.rs` (Marley crate):
  - `render_columns`: each column's header starts with a dot in its status colour (D2).
  - `render_card`: the kind as a pill in its kind's colour (D3), the phase as a boxed chip at the
    row's right, and the agent as a tile (D4) before its name.
  - `render_detail`: the kind and status as pills; a strip of four phase segments under the
    description (D5); while the ticket's status is `qa-failure`, a callout in the error tint, "QA
    failed · <age>" over its newest comment's body (D6); each comment led by its author's name in
    the author's runtime colour (D7); Open the agent as the primary action while an agent is on the
    ticket, with Start here in a worktree plain and Move to… quiet. Claim stays while no agent is
    on it.
  - `Comment` reads `author_id`; `Ticket` keeps its fields.
- `crates/marley_workbench/src/fleet.rs` (Marley crate): `agent_tile(runtime, state, cx)`, the
  runtime's two letters in its colour with a state dot at its corner, used by the board's cards and
  the Fleet rows; `render_agent` draws the name over the harness, the ticket's title over the
  folder, the state chip and "output N ago", each column only when its value is known;
  `render_source` gives each host's header its agent count, and a Forge source a connection card
  (Endpoint, Token, Projects, Status with the last read's latency).
- `crates/marley_workbench/src/forge.rs` (Marley crate): `summary` keeps the ticket number as the
  work item's key, the folder and the last output on their own fields, not one joined line.
- `crates/marley_sdk/src/work.rs` (Marley crate): `AgentSummary` gains `folder: Option<String>` and
  `last_output_ms: Option<u64>`, both `#[serde(default)]`, so the contract stays readable by older
  providers (AD-claude-607).
- `crates/marley_workbench/src/fleet_providers.rs` (Marley crate): a Forge client records where its
  token came from and how long its last `agent-list` round took; `Source` gains
  `forge_connection: Option<ForgeConnection>`.
- `crates/marley_workbench/src/forge_watch.rs` (Marley crate): its snapshot keeps the sprint's
  ticket titles by number, which the Fleet rows read for a Forge agent's ticket.

### Out (explicitly deferred)
- The project picker in the board's header: #842 Out.
- The By agent and List views: #842 Out.
- New Ticket: #842 Out.
- Attach and Give a ticket on an agent's row: #841 Out and #843 Out ("Forge has no tool for
  either").
- Searching forge's knowledge: #843 Out ("a later ticket with its own design").
- A host's role in its header: Forge's `agent-list` reports no role and Marley has none to show.
  A follow-up ticket, once Forge reports one or Marley's provider settings name it.
- The board header's accent tile and progress bar, the Agent and Sprint tiles side by side, the
  "Online agents" page title and note, the rail section's header and runtime-coloured rows, Home's
  filled chips, the bulletins' time and critical border, and the bottom row's Forge chip: look gaps
  the review lists, for a follow-up ticket.

## Reference (§20)
N/A — Marley-specific: Ignibyte Forge's MCP tools (`agent-list`, `sprint-current`, `ticket-list`,
`ticket-get` with comments) as #841–#843 read them, drawn in Marley's own sprint board and Fleet
panel. The fields used are Forge's DTOs (`/srv/stacks/ignibyte-forge/crates/forge-mcp/src/tools/`):
`TicketCommentDto.author_id`, `TicketDto.phase` and `status`, `AgentSessionDto.harness`, `folder`,
`current_ticket` and `last_output_at`.

### Prior art
- `crates/marley_workbench/src/forge_board.rs:100` `Comment` (body and `created_at` only); `:507`
  `render_columns` (name and count); `:547` `render_card` (kind as accent text, phase muted, agent
  as text); `:624` `render_detail` (kind and status as text, Agent and Sprint rows, comments with
  their age, then Claim, Move to…, Start here in a worktree); `:388` `agent_of`, the running
  session whose `current_ticket` names the ticket's number.
- Forge: `TicketCommentDto` has `author_id`, "typically the caller's agent session"
  (`tickets/ticket_comment.rs`); a ticket's `phase` is a free slug, the known ones `intake`,
  `explore`, `plan`, `generate`, `validate`, `refine`, `re-validate`, `complete`
  (`forge-domain/src/slug.rs`), and #842's stand-in uses `plan`, `implement`, `review`, `test`,
  `complete`; `AgentSessionDto` (`agents/mod.rs:26`) has no host role. No tool lists hosts.
- `crates/marley_workbench/src/forge.rs:73` `summary`: joins ticket, folder and "silent N" into the
  work item's key; `state_since_ms` is the last output.
- `crates/marley_sdk/src/work.rs:101` `AgentSummary`; `:149` `WorkItem` (`summary` is "a line or two
  on what it asks for", not a folder).
- `crates/marley_workbench/src/fleet.rs:707` `render_source` (host cards with a server icon, the
  name, a resource line); `:801` `render_agent` (tile with an accent glyph, name over one line,
  chip); `:1350` `state_chip`; `:1367` `runtime_glyph`; `crates/marley_workbench/src/search_looks.rs`
  `branch_mark`'s lilac (`players().color_for_participant(2)`), the colour Marley gives a second
  kind of thing.
- `crates/marley_workbench/src/fleet_providers.rs:397` `poll_forge`; `:419` `connect`, the token
  from the keyring (`keyring_token`) or the variable `bearer_env` names (`bearer`).
- `crates/marley_workbench/src/forge_watch.rs:49` `ForgeSnapshot` (counts and QA-failed tickets,
  no titles by number).
- `crates/marley_workbench/src/agent_tab.rs` `open_later`, which opens an agent's tab from a
  provider name and an agent id (AD-claude-609).
- Scenarios to reuse: `script/e2e/842-forge-board.sh` and `script/e2e/841-forge-provider.sh`, a
  stand-in Forge over HTTP (L-claude-841) with four sessions on three hosts and the sprint's
  tickets.

## UI proof
Scenario `script/e2e/946-forge-as-the-board.sh` (sway). Setup: #842's stand-in Forge, its token
from an environment variable the setup exports; its comments carry `author_id`s of its sessions;
#828 in `qa-failure` with a comment giving the reason; a `forge` provider in the run's settings.
- `946-01-board`: `marley: open forge board`: column dots, kind pills, phase chips and agent tiles
  (REQ-001, REQ-002).
- `946-02-detail`: #828 selected: kind and status pills, the phase strip, the QA failed callout,
  comments with authors, Open the agent first (REQ-003, REQ-004, REQ-005, REQ-006).
- `946-03-agent`: Open the agent clicked: code-2's Agent tab (REQ-006).
- `946-04-fleet`: `marley: toggle fleet`, which opens the Fleet as a center tab in the Marley
  layout (#747): Forge's connection card, hosts with their counts,
  each row with tile, name over harness, ticket title over folder, state and output age (REQ-007,
  REQ-008, REQ-009).
- `946-05-day`: Marley Day, the board with #828's detail (REQ-001, REQ-003).

## Locked-In Decisions
- D1 — Everything shown comes from Forge's answers or Marley's own connection; nothing is invented
  for a field Forge does not send (hence the host role in Out).
- D2 — Column dots: Open muted, In progress warning, In review the accent, QA the error colour,
  Done success.
- D3 — Kind pills: `feature` the accent, `bug` the error colour, `spike` the lilac, `chore` and
  `docs` muted, any other kind muted. A pill is the colour's text on its 15% tint.
- D4 — An agent tile is the runtime's two letters (`runtime_glyph`) in the runtime's colour
  (Claude Code the warning yellow, Codex the lilac, others muted), with a dot at its corner in the
  state chip's colour. "not claimed" keeps a dash tile.
- D5 — The strip's four segments are the board's: plan, implement, test, complete. Forge's phases
  map onto them: intake, explore, plan → plan; generate, implement, refine → implement; validate,
  re-validate, review, test → test; complete → complete. Segments before the ticket's are done
  (success), its own is the accent, or the error colour while its status is `qa-failure`, the rest
  to do (muted). A done or closed ticket is all done; an unknown phase draws no current segment
  and names the phase in text.
- D6 — The QA failed callout's age is the time since the ticket's `updated_at`; its reason is the
  newest comment's body. With no comment it says "QA failed" and the age only.
- D7 — A comment's author is the session in `agent-list` whose id is its `author_id`, named and
  coloured as its tile; an author Marley cannot match shows no name.
- D8 — The connection card's Token reads "from the keyring" or "from $NAME" (the variable's name,
  never its value) or "none"; Status reads "connected · N ms", N the last `agent-list` round, or the
  source's failure.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The sprint board shall draw each column's header with its status dot and each card's kind as a pill in its kind's colour, with the phase as a chip, in Marley Night and Marley Day. | `946-01-board`, `946-05-day` |
| REQ-002 | The sprint board shall lead each card's agent with a tile in its runtime's colour carrying a dot in its state's colour. | `946-01-board` |
| REQ-003 | WHEN a ticket is selected, its detail shall show its kind and status as pills and a strip of four phase segments marked done, current or failed, and to do, in Marley Night and Marley Day. | `946-02-detail`, `946-05-day` |
| REQ-004 | WHILE the selected ticket's status is `qa-failure`, its detail shall show a callout with "QA failed", how long ago, and the newest comment's text. | `946-02-detail` |
| REQ-005 | The detail's comments shall start with their author's name where `agent-list` names the author. | `946-02-detail` |
| REQ-006 | WHILE an agent is on the selected ticket, the detail shall offer Open the agent first, which opens that agent's tab. | `946-02-detail`, `946-03-agent` |
| REQ-007 | The Fleet panel shall show each Forge agent's harness, its ticket's title, its folder, its state and how long since its last output. | `946-04-fleet` |
| REQ-008 | The Fleet panel shall show each host's agent count in its header. | `946-04-fleet` |
| REQ-009 | The Fleet panel shall show a Forge source's endpoint, token source, projects and status with the last read's latency. | `946-04-fleet` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the contract's two fields, Forge's summary and connection, the board's pills,
  dots, tiles and detail, the Fleet rows, hosts and connection card; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
