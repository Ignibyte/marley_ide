# The Threads page as the board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-919-threads-page-as-the-board.md
- **Pipeline spec:** 919-threads-page-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, line 7 (The Threads page).
- **Classification / tier:** feature, M. Marley crate only (`marley_workbench`).
- **Recall (§18.3):**
  - PR-claude-701: the page's handlers that update its workspace are plain closures over a weak
    handle, never `cx.listener`. New Agent dispatches through the window, as the empty state does.
  - F-claude-865: a function called from a render never reads the view that renders. The rail's
    new accessor is a read of the rail's own snapshot and never reads the page.
  - F-claude-438-a: the rail builds its snapshot in `refresh`, not in `render`, so it is current
    while the rail is folded or closed. The page can trust it whatever the rail shows.
  - F-claude-644: never open or update the sidebar from inside the rail's update. The page only
    reads the rail, from its own render, as Home's `rail()` does (#903).
  - F-claude-903 and PR-claude-687: the stand-in `claude` goes first on the terminals' PATH
    through the scenario's `.bashrc`; nothing types into a thread before the stand-in is proved.
  - L-claude-508: a stand-in ACP agent in Python as a `custom` agent server; here it also
    declares modes. L-claude-836 (full-width buttons center their labels) applies to the WHO rows,
    now rows of their own rather than `Button`s. L-claude-836 (a cold start types the first
    palette late): settle after the first `ctrl-shift-p`.
  - AD-claude-737 and AD-claude-754: the page is over Zed's store and the CLIs' own files; #836
    kept the conversation out because the store holds titles, not messages.
- **Discovery:**
  - `crates/marley_workbench/src/threads_page.rs`: `ThreadsPage::new` (placeholder), `render`
    (header, search box), `render_chips` (WHO), `render_row` (card), `render_preview` (state line,
    facts), `sections` (counts per chip), `Row` (a state field).
  - `crates/marley_workbench/src/rail.rs`: `RowState` made `pub(crate)`; a new accessor beside
    `agents_at_work` that returns the states by thread key and by terminal view id, with the
    inbox's asks (from `snapshot.rail.inbox` and the `inbox` map's targets).
  - `crates/marley_workbench/src/home_page.rs`: `glyph_tile`, `state_tile`, `cli_glyph` shared.
  - Facts: `ThreadMetadata::created_at`; the action log through the loaded conversation's thread
    (`thread_tab::conversations_of`, `root_thread_view`), as `render_details` counts files; the
    session's mode through the conversation's connection (`session_modes`).
- **Decisions:** D1–D7 in the spec. The page reads the rail; it adds no second state reading.

### Design
- Approach: the page finds the window's rail as Home does (`MultiWorkspace::sidebar`, downcast
  to `Rail`, observed so a state change repaints the page). Before laying out, it asks the rail
  once for `ConversationStates`: a struct with named fields, `threads: HashMap<String,
  Conversation>` by thread key and `terminals: HashMap<u64, Conversation>` by view id, where
  `Conversation { state: RowState, ask: Option<SharedString> }`. `sections` fills each `Row`'s
  state: a thread by `Row::key`; a CLI session through `agent_events::view_running`, then the
  terminal map. It also counts rows per chip and the loaded ones for "M open".
- The card is a `div` with `rounded(px(12.))`, `border_1`, a filled background while selected;
  `state_tile` at 32 px with the agent's glyph; an `h_flex` of the title (truncating) and the
  time; the snippet when present; two pills (Home's `pill` shape, not pressable). Its handlers stay
  plain closures over weak handles.
- The preview's facts grid is two `h_flex` rows of two cells each, a muted XSmall label over the
  value. Files changed reads "N · +a −r" from the action log; Mode the current mode's name; for a
  CLI running in a terminal, "bypass" when #532's mark says so.
- File manifest: `threads_page.rs` (Marley crate), `rail.rs` (Marley crate), `home_page.rs`
  (Marley crate). No Zed crate, so no row in docs/marley/zed-touchpoints.md.

### Visual check plan
- REQ-001 (header): `919-01-night` shows "Threads", the count line and New Agent; `919-05-day`
  the same in Marley Day.
- REQ-002 (New Agent): `919-04-new-agent`, the button clicked, the New Agent picker open.
- REQ-003 (search): `919-01-night`, the magnifier and the placeholder in the empty field.
- REQ-004 (WHO rows): `919-01-night`, All, the Claude Code CLI row and the stand-in agent's row,
  each with tile and count.
- REQ-005 (cards): `919-01-night` and `919-05-day`, the cards with tiles and dots (the resumed
  session pink, the others grey), the title and time on one line, the two pills.
- REQ-006 (state line): `919-02-preview` reads "Needs you · " and the stand-in's ask;
  `919-03-preview-thread` the stand-in agent's state.
- REQ-007 (facts): `919-02-preview` shows Folder, Files changed "—" or the CLI's, Started "—"
  for a CLI session, Mode "—" or bypass; `919-03-preview-thread` shows Started and the mode's
  name.
- Not reachable: a Zed conversation with changed files needs an agent that edits. The stand-in
  does not, so Files changed shows "0" or "—" in the run; the count's wiring is read in the diff.

### Risks
- The rail's inbox key for a CLI ask is `terminal:{id}:{ask}`; the accessor maps targets, not
  key strings, so a key format change does not break it.
- A Claude Code session resumed in a terminal is matched by its session id label. If the stand-in's
  `SessionStart` does not carry the fixture's id, the card stays Idle. #754's scenario proved the
  stand-in prints it.
- The stand-in ACP agent's modes: ACP v2 offers modes through config options; if Zed's
  `session_modes` stays `None` for it, Mode reads "—" and `919-03-preview-thread` proves only the
  state line. Plan checks the stand-in's answer shape against `acp_thread`'s connection first.
- Cards are taller than rows. In a half-width pane the list is narrow until the columns ticket
  lands; the scenario runs the page in a whole-width pane.

## Phase 2 — Code
- **Built:** `rail.rs` `ConversationState`, `ConversationStates`, `Rail::conversation_states`;
  `threads_page.rs` the header, search, WHO rows with counts (`Sections::offer`/`count`), cards,
  the preview's state line and facts grid, `Live`, `Facts`, `loaded_conversations`.
- **Deviations:**
  - The tiles are `item_kind`'s palette (#905: `tile` for the WHO rows, `ItemKind::tint`/`ink`
    for the card's 32 px tile), not Home's #903 `glyph_tile`/`state_tile`: #905 made the
    palette the one look for an agent's tile, so Home's helpers stay private. D4's aim holds.
  - `RowState` became #905's `ItemState`, already `pub(crate)`; nothing in `rail.rs` needed
    opening but the accessor.
  - The header's New Agent is #903's primary button (`prompts::render_button`), not a `ui::Button`.
  - The count line reads the rows listed after the search and filters ("N conversations").
- **Review:** the page reads the rail and the window's workspaces in its render; neither is being
  updated then (Home does the same). Handlers stay plain closures over weak handles.
- **Gate:** `just gate-diff` 17/17 green after the dylint fix (`SharedString::new_static`).

## Phase 3 — Test
- **Scenario:** `script/e2e/919-threads-page-as-the-board.sh`, sway, 5 shots, each read.
  - `919-01-night` (first probe run): "Threads", "3 conversations · 1 open", New Agent; the
    magnifier and "Search titles, folders, and what you asked"; WHO: All 3, Marley 1, Claude Code
    CLI 2, each with its tile; cards with tiles and dots, title and time on one line, pills.
    REQ-001, REQ-003, REQ-004, REQ-005 met.
  - `919-02-preview`: after the resume, "2 open"; the tokenizer card's dot pink; "Needs you ·
    Permission for Bash: make" in pink; Folder, Files changed "—", Started "—", Mode "—".
    REQ-006, REQ-007 met.
  - `919-03-preview-thread` / `919-05-day`: the Marley conversation: "Working", Folder, "0 · +0
    −0", Started "Oct 25, 14:55", Mode "Default"; Day legible.
  - `919-04-new-agent`: the New Agent picker over the page. REQ-002 met.
- **Fixes in Test:** the probe's clicks hit the wrong card; coordinates set from its shot.
- Pre-existing, not in scope: the bottom row's Threads case still counts its own "1 open"
  (the bottom-row ticket in the build order).

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md`, the look plan's W2.3 row. No Zed crate.
- Knowledge: nothing new; the page follows L-claude-713's rail-reading shape.
- Brain: decision recorded.
