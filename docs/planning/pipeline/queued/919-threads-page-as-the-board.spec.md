---
pipeline_id: 2f997f81-278c-4226-982d-c99daec23f0d
ticket: docs/planning/tickets/open/TICKET-919-threads-page-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: The Threads page as the board draws it
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/C-threads.md, docs/planning/pipeline/completed/836-threads-with-a-preview.spec.md]
---

## Title
The Threads page as the Threads board draws it. Each conversation is a card with its agent's tile
and a state dot, its title and time on one line, and its agent and folder as pills. The header
counts the conversations and the open ones and offers New Agent. The search has a magnifier and
the board's words. Each WHO row has a tile and a count. The preview says the state under the
title and shows Folder, Files changed, Started and Mode in a grid.

## Scope
### In
All in `crates/marley_workbench` (Marley crate). No Zed crate changes.
- `threads_page.rs`, `ThreadsPage::render`: the header row. "Threads", then a muted count line
  "N conversations · M open", then an accent "New Agent" button at the right.
- `threads_page.rs`, `ThreadsPage::new` and `render`: the search placeholder "Search titles,
  folders, and what you asked", and a magnifier icon before the field inside its box.
- `threads_page.rs`, `render_chips`: each WHO row (All included) as a row with the agent's glyph
  tile, the name, and the count of listed conversations it keeps at the right. The row stays a
  toggle.
- `threads_page.rs`, `render_row`: the card in place of the `ListItem`. A rounded, bordered card;
  a 32 px glyph tile with the state dot at its corner; the title and the time on one line; the
  search snippet when there is one (#754); the agent and the folder as pills. Selected, the card
  is filled and its border shows. Click selects and double-click opens, as now.
- `threads_page.rs`, `Row`: a `state: Option<RowState>` field, filled from the window's rail;
  none without a rail. `rail.rs`'s `RowState` and its `dot` and `line` become `pub(crate)`.
- `rail.rs`, `Rail`: one accessor that hands out each listed thread's state by its key, each agent
  terminal's state by its view id, and the inbox's ask for each. It is read the way Home reads
  `agents_at_work`.
- `threads_page.rs`, `render_preview`: under the title, a state line in the state's color ("Needs
  you · <the ask>", "Working", "Done", "Idle"). It takes the place of "Last changed …". Then a
  2×2 facts grid: Folder, Files changed, Started, Mode.
- `home_page.rs`: `glyph_tile`, `state_tile` and `cli_glyph` become `pub(crate)` with a size
  argument, so the page and Home draw one tile.

### Out (explicitly deferred)
- The conversation excerpt in the preview (the user's bubble, the answer, the waiting banner):
  deferred by #836. Loading a thread to preview it would start its agent.
- A snippet line on every card ("You: …", "Done: …"): deferred by #836 for the same reason. The
  store holds titles, not messages. A card shows a snippet only when the search matched a prompt.
- Archive in a CLI session's preview: deferred by #836 D3.
- The columns' widths (filters 230, list flexing, preview 380, wrapping in a narrow pane): the
  build order's first item, its own S ticket ("The Threads page squeezes its list").
- The bottom row's Threads-page case ("Threads · 48 conversations · 1 needs you … 3 open"): the
  build order's second item, the bottom row ticket.
- Files changed and Mode for a conversation the window has not loaded: they read "—". Filling
  them would mean loading the thread (#836's reason).
- Started for a CLI session: it reads "—". The CLIs' files are read for titles and prompts, not
  for a start time (#754). A follow-up ticket can parse the first entry's time.

## Reference (§20)
N/A — Marley-specific. The Threads page is Marley's own (`threads_page.rs`, #737, #754, #836);
Zed's agent history is a picker in a sidebar the rail replaces (AD-claude-737). The layout is the
design canvas's Threads board. Its states are the rail's (#827).

### Prior art
- `ThreadsPage` (`threads_page.rs`): `render` (`:872`, headline at `:893`, filters `w(px(200.))`
  `:892`, preview `w(px(320.))` `:929`), `new` (placeholder `:315`), `render_chips` (`:422`,
  plain Subtle buttons, no tile, no count), `render_row` (`:650`, a `ListItem` with a muted icon
  start slot), `render_preview` (`:500`, "Last changed" `:538`, Archive only for a thread `:578`),
  `render_empty` (`:594`, New Agent dispatches `agent_ui::NewThread` at `:602`), `sections`
  (`:348`, the chips and folders). `Row` (`:228`) has no state.
- The rail's state for a row: `RowState` (`rail.rs:10994`, Working / NeedsYou / Done / Idle,
  with `dot` and `line` colors), `thread_row_state` (`rail.rs:11276`) from a `ThreadStatus` and
  the attention dot, `live_statuses` (`rail.rs:9367`) reading each loaded conversation through
  `marley_rail::thread_status`, `note_ended_runs` (`rail.rs:2860`) for attention. The snapshot's
  `ThreadSnapshot { key, status, attention }` (`marley_rail.rs:398`) keys a thread by
  `ThreadId::to_key_string`, the same key `Row::key` uses.
- An agent CLI's live state: the rail's terminal rows carry `TerminalAgent { status, mark }`
  (`marley_rail.rs:385`; `mark` is #532's bypass reading). `agent_events::view_running`
  (`agent_events.rs:114`) finds the terminal running a CLI session by its session id.
- The ask: `Rail::inbox` (`rail.rs:5419`) lists `InboxEntry { key, kind, ask, … }`
  (`marley_rail.rs:593`); the rail's `inbox` map (`rail.rs:779`) ties each key to its
  `InboxTarget::Thread { thread_key, … }` or terminal.
- Reading the rail from a page: `MarleyHome::rail` (`home_page.rs:317`) finds the window's rail
  through `MultiWorkspace::sidebar` and observes it; `agents_at_work` (`rail.rs:1555`) is the
  accessor shape.
- Tiles: `home_page.rs` `glyph_tile` (`:935`), `state_tile` (`:953`), `cli_glyph` (`:925`),
  `pill` (`:900`), from #903. `tab_tiles.rs` `glyph_of` (`:83`) names a tab's kind.
- The facts: `ThreadMetadata::created_at` (`agent_ui/src/thread_metadata_store.rs:319`) for
  Started; the details panel's changed files and +/− (`thread_tab.rs:728`, the action log's
  `changed_buffers` and `DiffStats::single_file`, #825); an ACP session's mode through
  `AgentConnection::session_modes` and `AgentSessionModes::current_mode` / `all_modes`
  (`acp_thread/src/connection.rs:284`, `:373`), as `ConversationView` builds its `ModeSelector`
  (`agent_ui/src/conversation_view.rs:1601`).
- New Agent: `marley::NewAgent` opens the New Agent picker (`agents.rs:865`, #735).
- The behaviour maps (`docs/zed_architecture/`) have nothing on this page; the brain was not
  consulted at drafting (Plan consults it).

## UI proof
Scenario `script/e2e/919-threads-page-as-the-board.sh` (sway). Setup: #754's and #836's fixtures
(two Claude Code CLI sessions in the run's `CLAUDE_CONFIG_DIR`, one in the repository changed
now, one in `sub` four days ago). Stand-in `claude` first on the terminals' PATH through the
scenario's `.bashrc` (F-claude-903). On `--resume` it prints #519's `SessionStart` for that
session, then raises a permission ask through Marley's hooks, as #827's stand-in does, so the
resumed session needs the user. A stand-in ACP agent (L-claude-508's shape) is a `custom` agent
server whose `session/new` declares two modes. One conversation with it is started, so a Zed
conversation is listed, loaded, with a Mode.
- `919-01-night`: `marley: open threads` after the resume. The header's count and New Agent,
  the search with its magnifier and words, WHO rows with tiles and counts, cards with tiles,
  state dots (the resumed one pink), title and time on one line, agent and folder pills
  (REQ-001 to REQ-005).
- `919-02-preview`: the waiting card clicked. Its preview's state line "Needs you · <ask>" and
  the facts grid (REQ-006, REQ-007).
- `919-03-preview-thread`: the stand-in agent's card clicked. Its state line, and Mode in the
  grid (REQ-006, REQ-007).
- `919-04-new-agent`: New Agent clicked. The New Agent picker is open (REQ-002).
- `919-05-day`: Marley Day, a card selected (REQ-001 to REQ-007).

## Locked-In Decisions
- D1 — The page's states are the rail's. A thread's state comes from the rail's snapshot by its
  key, through `thread_row_state`; a CLI session's from the terminal running it (`view_running`),
  through the rail's terminal row. One reading serves the rail, Home and the page. A
  conversation the window has not loaded is Idle. Without the rail (Zed's layout) a card has no
  dot.
- D2 — The count line reads "N conversations · M open". N counts the rows listed, not archived,
  across Zed's store and the CLI sessions. M counts those loaded in this window: in a tab, in a
  panel, or a CLI session running in a terminal.
- D3 — The header's New Agent dispatches `marley::NewAgent` (the picker, #735). The empty
  state's New Agent stays as #857 built it.
- D4 — The tiles are Home's (`glyph_tile`, `state_tile`, #903): one look for an agent's tile
  everywhere. A CLI's glyph is `cli_glyph`'s. A Zed agent's is the first letter of each word of
  its name, at most two (Marley "M", Rusty "R", Claude Agent "CA").
- D5 — The state line replaces "Last changed …"; the card keeps the time. While a conversation
  needs the user, the line ends with the inbox's ask for it (`InboxEntry::ask`).
- D6 — Folder and Started come from the store for every conversation. Files changed and Mode come
  only from a loaded conversation: the action log's count and +/− (as #825's details panel), and
  the ACP session's mode name, or #532's bypass mark for a CLI in a terminal. Otherwise each reads
  "—". The page never loads a thread to fill them (#836).
- D7 — The search keeps what it finds (titles, folders, CLI prompts). The board's placeholder
  says the same in fewer words.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Threads page header shall show "Threads", the line "N conversations · M open" and a New Agent button, in Marley Night and Marley Day. | `919-01-night`, `919-05-day` |
| REQ-002 | WHEN New Agent in the header is clicked, the system shall open the New Agent picker. | `919-04-new-agent` |
| REQ-003 | The search field shall show a magnifier and the placeholder "Search titles, folders, and what you asked". | `919-01-night` |
| REQ-004 | Each WHO row shall show its agent's glyph tile, its name and the count of conversations it keeps. | `919-01-night` |
| REQ-005 | Each conversation shall be a rounded card with a 32 px agent tile carrying a dot in its state's color, the title and time on one line, and the agent and folder as pills. | `919-01-night`, `919-05-day` |
| REQ-006 | WHEN a conversation is selected, the preview shall show its state under the title in the state's color, with the inbox's ask while it needs the user. | `919-02-preview`, `919-03-preview-thread` |
| REQ-007 | WHEN a conversation is selected, the preview shall show Folder, Files changed, Started and Mode in a 2×2 grid, with "—" for a fact the window does not have. | `919-02-preview`, `919-03-preview-thread` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the rail accessor, the page's header, search, WHO rows, cards and preview; Home's
  tiles shared; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
