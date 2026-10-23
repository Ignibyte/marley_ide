# Forge's board and online agents as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-946-forge-as-the-board.md
- **Pipeline spec:** 946-forge-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11: Forge).
- **Classification / tier:** feature, M. Marley crates only (`marley_workbench`, `marley_sdk`).
- **Recall (§18.3):**
  - L-claude-841-a-stand-in-mcp-server-over-http-001: the stand-in Forge is a Python
    `ThreadingHTTPServer` answering JSON-RPC; its token comes from a variable `setup` exports, which
    keeps the run off the user's keyring.
  - AD-claude-607-the-fleet-contract-lives-in-marley-sdk-001: `AgentSummary` is the contract; the
    two new fields are optional with serde defaults, so an older provider's answer still reads.
  - AD-claude-608-the-fleet-reads-full-detail-only-for-selected-agents-001: the rows' new columns
    come from the summary, not a detail read per agent.
  - AD-claude-609-the-agent-tab-is-read-only-and-its-samples-live-in-memory-001: Open the agent goes
    through `agent_tab::open_later`, which defers, so the board's click never reads a tab being
    updated.
  - F-claude-607-polling-followed-set-active-not-what-shows-001: the Fleet reads while it shows;
    the scenario shows the Fleet tab before its shot and waits a poll.
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: `agent_tile` takes the runtime
    and state and reads no view.
  - The brain: to be consulted when the spec is promoted.
- **Discovery:** `forge_board.rs` (`Comment`, `render_columns`, `render_card`, `render_detail`,
  `render_comments`, `render_actions`, `agent_of`), `forge.rs` (`ForgeAgent`, `summary`),
  `fleet.rs` (`render_source`, `render_agent`, `runtime_glyph`, `state_chip`, `Source`),
  `fleet_providers.rs` (`connect`, `poll_forge`, `keyring_token`, `bearer`), `forge_watch.rs`
  (`ForgeSnapshot`, `read`), `marley_sdk/src/work.rs` (`AgentSummary`), `agent_tab.rs`.
- **Decisions:** D1–D8 in the spec.

### Design
- **Contract:** `AgentSummary { …, folder: Option<String>, last_output_ms: Option<u64> }`, each
  `#[serde(default, skip_serializing_if = "Option::is_none")]` and documented; every constructor
  in the tree fills them (`None` where unknown).
- **Forge summary:** `work_item.key` is `#<n>` from `current_ticket`, `title` the same until the
  Fleet row finds the title; `folder` and `last_output_ms` filled from the DTO; the "silent N"
  wording stays in the chip's reason, as `agent_chip` gives it.
- **Titles:** `ForgeSnapshot` gains `ticket_titles: BTreeMap<i64, SharedString>` from the sprint's
  `ticket-list`; `forge_watch::ticket_title(number, cx) -> Option<SharedString>`. The Fleet row
  shows "#828 A project's mark from its logo.png", or "#828" while no title is known.
- **Connection:** `pub(crate) struct ForgeConnection { endpoint: String, token_from: TokenSource,
  projects: Vec<String>, last_round_ms: Option<u64> }` and `pub(crate) enum TokenSource { Keyring,
  Variable(String), Missing }` in `fleet_providers.rs`; `connect` reports which source answered;
  `poll_forge` times its calls with `Instant`. `Source.forge_connection` carries it to the panel.
- **Tile:** `pub(crate) fn agent_tile(runtime: &str, state: State, cx: &App) -> Div` in
  `fleet.rs`: `size_6`, `rounded_md`, the runtime colour's 15% tint behind its letters, a `size_2`
  dot absolutely placed at the bottom right with a ring in the panel's background.
- **Board:** `column_color(index)`, `kind_color(kind)` and `pill(text, color)` beside the
  renderers; `phase_strip(ticket, cx)` per D5 as four `flex_1` bars with their names under them;
  `qa_callout(detail, cx)` per D6; `render_comments` looks up `author_id` among `self.agents`;
  `render_actions` takes the ticket's agent and puts Open the agent first through
  `agent_tab::open_later(workspace, Selected { source, agent }, window, cx)`, the source being
  `fleet_providers::name_of(&forge_entry)`.
- **Fleet rows:** `render_agent` lays out tile | name over harness | ticket over folder (the
  flexible column, truncating) | chip | "output N ago" (`how_long`), dropping a column with no
  value; `render_source` adds "N agents" at the host header's right, before the resource line,
  and for a Forge source a card of four label and value rows under the header.
- **File manifest:** `crates/marley_workbench/src/forge_board.rs`, `forge.rs`, `fleet.rs`,
  `fleet_providers.rs`, `forge_watch.rs` (Marley crate), `crates/marley_sdk/src/work.rs` (Marley
  crate), `script/e2e/946-forge-as-the-board.sh` (Marley-owned).
- **Ledger rows:** none; no Zed path changes.

### Visual check plan
- REQ-001, REQ-002: `marley: open forge board`; `946-01-board`.
- REQ-003 to REQ-006: click #828's card; `946-02-detail`; click Open the agent; `946-03-agent`.
- REQ-007 to REQ-009: `marley: toggle fleet`, as #848's scenario does; in the Marley layout the
  Fleet opens as a center tab (#747); wait one poll; `946-04-fleet`.
- Day: `profile_setting theme '"Marley Day"'`, the board with #828 selected; `946-05-day`.

### Risks
- The Fleet in Zed's layout sits in the narrow right dock; six columns truncate there. The
  flexible column is the ticket's, so the name, state and age stay readable; the shot uses the
  Marley layout's center tab.
- A comment whose `author_id` is a person's, not a session's, shows no author (D7); the stand-in
  covers both.
- The contract's new fields must be filled by the pseudo, MCP, HTTP and hosts providers too, or
  their rows lose nothing but the new columns; the review of the diff checks each constructor.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
