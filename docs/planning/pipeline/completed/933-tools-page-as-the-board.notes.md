# MCP Servers and Agent Activity as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-933-tools-page-as-the-board.md
- **Pipeline spec:** 933-tools-page-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, Tools).
- **Classification / tier:** feature, M. Marley crate only (`marley_workbench`). Depends on #905
  (`item_kind.rs`, `AgentMark`); #934 builds on this ticket's `Outcome` and row layout.
- **Recall (§18.3):**
  - AD-claude-703: the log keeps who, tool, a redacted one-line summary and the outcome; this
    ticket adds a redacted one-line result to the same row, nothing bigger.
  - F-claude-703 / PR-claude-703 (Home's page hid a tab just opened into Home): Agent Activity
    opens through `groups::in_group(GroupKind::Home, …)`; the scenario opens it from the palette
    and checks the tab is in front before the shot.
  - L-claude-799: a long page is checked best near its top; the activity rows are the newest
    first, so the two calls sit at the top.
  - #703's scenario: a scripted MCP client through the plugin's bridge makes real write-tier
    calls with no agent; reused here.
  - #846's scenario: stand-in `claude` and `codex` on the PATH, the run's own Claude Code config
    (#877). Here `codex` is left off the PATH so its card shows "missing".
  - No failure names `mcp_servers.rs` or `render_rows`.
- **Discovery:**
  - `crates/marley_workbench/src/mcp_servers.rs`: `McpServers::render` (subtitle, Marley's
    card, the Zed card without `marley`), `render_agent` (tile, name over path, pill, chips), a
    new `render_marley_server`, `zed_rows` (keeps returning `marley`'s row; the render splits it
    off by `marleys` and its name).
  - `crates/marley_workbench/src/agent_activity.rs`: `ActivityRow` (`agent`), `Outcome` (`result`,
    `reason`), `log` (takes the answer's line), new `result_line`, `render_rows` (`RowLayout`).
  - `crates/marley_workbench/src/click_pause.rs`: `Who` gains `mark: Option<AgentMark>`.
  - `crates/marley_workbench/src/home_page.rs`: `render_activity` passes `RowLayout::Line`.
  - `crates/marley_workbench/src/mcp.rs`: `enabled_tools` becomes `pub(crate)`.
- **Decisions:** D1–D7 in the spec.

### Design
- `RowLayout { Card, Line }` replaces a bool, as the style guide asks.
- `Outcome` stays internally tagged (`outcome`): `Done { #[serde(default)] result:
  Option<String> }`, `Refused { code: String, #[serde(default)] reason: Option<String> }`. An old
  line `{"outcome":"done"}` reads as `Done { result: None }`.
- `result_line(answer: &ToolAnswer) -> Option<String>`: in order, `structured.result`
  ("accepted" → "you accepted", "refused" → "you refused"), `structured.running == true` →
  "running", `structured.exit_code` (integer) → "exit N", `structured.saved == true` → "saved",
  else `answer.text`'s first non-empty line. The `on_answer` hook runs off the main thread, so
  `log` takes the redactor (`crate::mcp::model_redactor(cx)`) before it and moves it into the
  hook; the line is redacted and cut to 120 characters there.
- The refusal's reason: the text up to the first ". " or the end, cut to 120.
- `Who::of` sets `mark`: a terminal's Claude Code → `AgentMark::ClaudeCode`; `ZED_CLIENT` →
  `AgentMark::Other('Z')` (#905's "first letter" form); an outside client → its name's first
  letter; Claude Code's and Codex's adapter client names, confirmed in Code from a real
  `initialize` (or the adapters' published source), → `CC`, `CX`.
- The row card: `h_flex` with `glyph_tile`-like 28 px tile (`AgentMark::tint`/`ink`), then a
  `v_flex`: the first line `Label(tool).buffer_font(cx).color(Color::Accent)` and the time
  right-aligned; the second line the summary, truncated; the third line the result
  (`Color::Muted`) or the reason (`Color::Error`). Border `border_variant`, background
  `panel_background`, `rounded_lg`, as #903's cards.
- The agent card head: a 34 px tile; a `v_flex` of the name with version and the path in the
  buffer font; a pill (`rounded_full`, the state's colour at low opacity behind its text). The
  chips: `h_flex().flex_wrap().gap_1p5()` of bordered chips with an id per integration and
  `.tooltip(Tooltip::text(state + " · tested on " + versions))`.
- Marley's card: the status from the same per-project statuses `zed_rows` builds for the
  `marley` id; the pill's word is the first project's status word; the count
  `tools_list_for(&Principal::Marley, &enabled_tools(cx))["tools"].as_array().len()`.
- File manifest (all Marley crate): `mcp_servers.rs`, `agent_activity.rs`, `click_pause.rs`,
  `home_page.rs`, `mcp.rs`; the scenario `script/e2e/933-tools-page-as-the-board.sh`. No Zed
  path, so no `docs/marley/zed-touchpoints.md` row.

### Visual check plan
- REQ-001, REQ-002, REQ-004: open MCP Servers from the palette; `933-01-servers` shows the
  subtitle, Claude Code's card (tile, "Claude Code <version>" over the stand-in's path, "ready"),
  Codex's card ("not on Marley's PATH", "missing"), and Marley's card with the count and
  "running".
- REQ-003: the pointer on Claude Code's first chip; `933-02-chip` shows the tooltip.
- REQ-005 to REQ-007: run the scripted client's `terminal_run` of `echo hello` on the project's
  terminal, then a `terminal_run` naming terminal 999; open Agent Activity; `933-03-activity`
  shows two cards, "exit 0" muted and the refusal's reason in red.
- REQ-008: show Home; `933-04-home` shows the card's two lines.
- Marley Day: `933-05-day`, MCP Servers and Agent Activity.
- REQ-009: the review of the diff (serde defaults); the scenario's run also reads the day file
  the first calls wrote.

### Risks
- The redactor in a `Send` closure: if `model_redactor` hands a non-`Send` value, the hook takes
  the redacted line from a clone made on the main thread, or `log` sends the raw line over the
  existing channel and the main-thread receiver redacts it before the row is kept. Either keeps
  secrets out of the day file.
- `terminal_run`'s answer for a command that is still running has no exit code; the line says
  "running" and does not update later. A later "exit N" needs a second row or an update; not this
  slice.
- #905 not landed when this is promoted: promote #905 first (D1).
- The adapters' client names may differ by version; unknown names fall to the first letter, so a
  wrong guess draws a plain tile, never a wrong agent's.

## Phase 2 — Code
- **Built:** `mcp_servers.rs` (the subtitle; `pill`, `card_head`, `render_chip`; `render_agent`
  rewritten; `render_marley_server`; `render` partitions `zed_rows` by `mcp::CONTEXT_SERVER`),
  `agent_activity.rs` (`ActivityRow.agent`, `Outcome::Done { result }`, `Outcome::Refused {
  code, reason }`, `RowLayout`, `result_line`, `first_sentence`, `shortened`, `cut`, `minute`,
  `render_card`, `render_line`), `click_pause.rs` (`Who::mark`), `item_kind.rs`
  (`AgentMark` serde derive, `AgentMark::from_client`, `square_tile`), `home_page.rs`
  (`RowLayout::Line`), `mcp.rs` (`enabled_tools` `pub(crate)`).
- **Deviations:** the tile is a new `item_kind::square_tile` rather than `tile` (whose height
  follows its label), so the 34 px and 28 px tiles are square. The adapters' client names were
  not read from a live `initialize`: `claude-code` is the name Marley already maps for Claude
  Code's events (`agent_events.rs`), and any name starting `codex` maps to CX. An unknown name
  draws its first letter, so a wrong guess never draws another agent's tile. A terminal caller's
  mark comes from its seat's agent label (`agent_events::seat_agent`).
- **Review:** the `on_answer` hook is `Send`; `model_redactor` hands an `Arc<Redactor>`, which
  the hook takes. Old rows: `{"outcome":"done"}` reads as `Done { result: None }`; `agent`
  absent reads as `None`. New fields are skipped when empty.
- **Gate:** clippy found four (a name too close to another, a redundant closure, two
  `unwrap_or` calls); fixed with `minute`. `just gate-diff`: 17 passed, 0 failed.

## Phase 3 — Test
- **Scenario:** `script/e2e/933-tools-page-as-the-board.sh` (sway), run in full; every shot read.
- `933-01-servers`: the subtitle beside the title; Claude Code's card (CC tile, "Claude Code
  2.1.304" over the stand-in's path, green "ready", five chips with dots, Prompt tags green);
  Codex's (CX tile, "not on Marley's PATH", red "missing", the App Servers chip); Marley's card
  (M tile, "Zed's agents · running in repo", "68 tools: editor, terminal, …", green "running");
  the ZED card with no context servers. REQ-001 to REQ-004.
- `933-02-chip`: the tooltip "Prompt tags: on · tested on 2.1.283 and later 2.1 releases".
  REQ-003.
- `933-03-activity`: two cards, newest first: S tile, `terminal_run` in accent, "Stand-in
  agent", 22:02; "echo nowhere" with "no terminal has the id 99999" in red; "echo hello" with
  "exit 0" muted. The day file's two lines hold `"result":"exit 0"` and `"reason":"no terminal
  has the id 99999"`. REQ-005 to REQ-007, REQ-009.
- `933-04-home`: Home's AGENT ACTIVITY card, two lines: 22:05, `terminal_run`, the command.
  The first run shot Agent Activity, because the palette's `rusty: open home` left the group's
  active tab in front; the scenario clicks the Home tab now. REQ-008.
- `933-05-day`, `933-06-day-activity`: both pages in Marley Day, tiles and pills tinted for the
  light theme. REQ-001, REQ-005.
- **Pre-existing, not in scope:** Home's RECENT PROJECTS in a scenario lists the machine's real
  recent projects (the run's profile reads them); the shots stay out of the repository.

## Phase 4 — Complete
- CHANGELOG entry; `docs/marley_architecture/marley_workbench.md` (MCP Servers and Agent
  activity sections); the look-plan row. No Zed path, so no touchpoint row.
- Brain: the consultation closed with `brain decide` (the result line kept in the row).
