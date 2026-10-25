---
pipeline_id: ec4e1a13-fcdd-490e-bafb-bc2abf2b524e
ticket: docs/planning/tickets/open/TICKET-909-bottom-row-for-other-screens.md
status: Phase 4 — Complete PASS
title: The bottom row for the other screens
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/C-threads.md, docs/marley/design-review-2026-10-23/F-tools.md, docs/marley/design-review-2026-10-23/E-rusty.md]
---

## Title
The bottom row says what is in front for the Threads page, a Browser tab, MCP Servers, the
Component Library, and Rusty's Tasks, Graph and brain page tabs, with the chips each board draws.
Today the row knows only files, terminals and threads (`focus_row.rs:33-38`), so these screens
leave it empty.

## Scope
### In
All in `crates/marley_workbench/src/focus_row.rs` (Marley crate) unless named. Each case is a new
`Focus` variant, set in `FocusRow::set_active_pane_item` by downcasting the item, as the terminal
and thread cases are, and drawn by its own `render_*` function.
- **Threads page** (`ThreadsPage`): a "Threads" chip, "N conversations" (the count
  `threads_page::conversations("", cx)` lists), and "N open" (the thread tabs open in the window,
  from `thread_tab::tab_threads` over the window's workspaces).
- **Browser tab** (`BrowserView`): a chip with the `www` tile and the page's title; the address in
  the buffer font; a red chip with a dot, "N console errors", when the page logged errors; "N picks
  ready" when the tray holds picks (`BrowserHub::pick_count`); "recording the last minute" while
  the tab draws the page. Read from `BrowserHub` (`title`, `address`, `console_entries`,
  `pick_count`) for the tab's `target()`; the row observes the hub while a Browser tab is in front.
- **MCP Servers** (`McpServers`): an "MCP Servers" chip; a pink chip per agent CLI Marley found
  missing ("Codex is not on the PATH", from `agent_versions::found`); "marley · N tools", the count
  of tools Marley's server lists. `marley_mcp::registry` (Marley crate) gains a small
  `served_tool_count()` beside `tools_list()`, so the row does not build the JSON list.
- **Component Library** (`ComponentLibrary`): a "Component Library" chip; a pink chip for the
  first broken template, its file name and its error ("quota.json · line 7: …"), with "+N more"
  when several are broken; "N components · N templates". `component_library.rs` gains an accessor
  for its catalog's length.
- **Tasks** (`TasksView`): a chip "Tasks · <the chosen list's name>"; a pink chip "N decisions
  overdue" from the decisions due the tab last read; "N open" for the chosen list.
  `rusty/tasks_tab.rs` gains one accessor returning these three facts as a struct.
- **Graph** (`GraphView`): a chip "Graph · local" or "Graph · vault"; "depth N · N pages" (the
  local graph's depth, the shown nodes); the embedding as Rusty last reported it (D6).
  `rusty/graph_tab.rs` gains one accessor for scope, depth and the shown page count.
- **Brain page** (`PageView`, and `BrainTab` through the page it shows): a "Brain" chip; the
  page's slug in the buffer font; "★ favourite" when the page is one of Rusty's bookmarks
  (`favourites::is_page`); "Rusty connected", or "Rusty not connected" (`rusty::is_connected`).
  `rusty/brain_tab.rs` gains an accessor for its page.

### Out (explicitly deferred)
- **The Fleet's row** ("Fleet", "3 agents · 2 hosts", "review-1 waits", "Harness connected";
  F-tools.md, Fleet). The Fleet is a dock panel (`fleet.rs:35`), and the row follows the active
  pane item only. It needs a panel-focus case, or the Fleet as a centre tab (a separate look gap
  in F-tools.md). A follow-up ticket.
- **Agent Activity.** No board draws a bottom row for it; nothing to build.
- **The other tabs with no row on a board** (Decisions, Memory, Skills, Secrets, Rusty's Server,
  Sessions, Harness, Forge, System pages): they keep an empty focus part.
- **The board's right-aligned spacer.** The bar's right half belongs to Zed's items and the panel
  group (D3).
- **Rusty's vault total in "embedded N of M"**: Rusty's `brain_semantic_status` gives no total
  (D6).

## Reference (§20)
Upstream Zed (`workspace::status_bar`): the status bar's `StatusItemView` contract is kept. A left
item hears `set_active_pane_item` and draws for the active item, as Zed's own cursor position and
active buffer language items do. The rest is Marley-specific: the boards' row for Marley's own
screens, over the focus item #833 built.

### Prior art
- `focus_row.rs:33-38` `Focus` (None, File, Terminal, Thread); `set_active_pane_item` downcasts
  `TerminalView` and `ThreadTab` and observes the item's entity (`:335`); `render_waiting`
  (`:238`) draws the needs-you chip with Allow and Open; `chip` and `muted` (`:369`, `:383`). #907 and
  #908 extend `Focus` before this ticket and add the chip helpers this one reuses.
- `workspace/src/status_bar.rs:449` `update_active_pane_item` calls each item's
  `set_active_pane_item` from the status bar's update.
- `threads_page.rs:91` `conversations(query, cx)` (unarchived, not drafts, CLI sessions included);
  `thread_tab.rs:1888` `tab_threads`; `threads_page::open_thread` (`:751`) finds open tabs the
  same way.
- `browser.rs:1127-1167` `BrowserHub::title`, `url`, `address`, `is_loading`, `dialog`;
  `:2293` `pick_count`; `:2605` `console_entries`; `:8496` `is_error`; `:5234`
  `BrowserView::target`; `:905` `BrowserHub::try_global`. The recorder keeps a page's minute while
  a tab draws it (`:413`, `marley_browser/src/recorder.rs:1`).
- `mcp_servers.rs:488-497` `render_agent` reads `agent_versions::found` and words a missing CLI
  "not on Marley's PATH"; `marley_mcp/src/registry.rs:913` `tools_list`, derived from `REGISTRY`.
- `component_library.rs:48` `templates(cx)`; `marley_components.rs:892` `Templates::broken`
  (`BrokenTemplate { path, error }`), `:1619` `catalog()`.
- `rusty/tasks_tab.rs:271` `TasksView` (`chosen`, `lists`, `tasks`, `counts`, `side.due`);
  `:1094` counts the overdue decisions; the side is read at any width and drawn from 1300 px.
- `rusty/graph_tab.rs:179` `Scope { Local, Vault }`, `:254` `depth`, `shown.nodes`.
- `rusty/page.rs:407` `PageView::slug`, `:1326` the star from `favourites::is_page`;
  `rusty.rs:741` `is_connected`; `rusty.rs:206` the `Rusty` global's last
  `SemanticStatus` (`marley_rusty/src/settings.rs:171`: provider, pages, stale).
- `item_kind.rs` (#905, built before this ticket): the kind tints and state colours the chips
  reuse.
- The brain: `decisions/the-bottom-rows-focus-is-one-marley-status-item-after-debug` (#833: one
  Marley status item follows the active item). Nothing else on this seam.
- Warp: not consulted; these screens are Marley's own.

## UI proof
Scenario `script/e2e/909-bottom-row-for-other-screens.sh` (`COMPOSITOR=sway`, for the pick's
click). Setup: a scratch repo; stand-in `claude` first on the terminals' PATH through the
scenario's `.bashrc` and no `codex` anywhere on it; `browser-fixture.sh`'s `offline_chromium` and
`serve_site` with a page that calls `console.error` once; one broken template in the run's
`components/` folder; Rusty on through `crates/marley_rusty/stand_in/rusty-mcp` with a list of
three tasks, an overdue decision and a bookmarked page. Each screen is opened from the command
palette.
- `909-01-threads`: the Threads page in front (REQ-001).
- `909-02-browser`: the served page in a Browser tab after one pick (REQ-002).
- `909-03-mcp`: MCP Servers in front (REQ-003).
- `909-04-components`: the Component Library in front (REQ-004).
- `909-05-tasks`: Rusty's Tasks tab on the three-task list (REQ-005).
- `909-06-graph`: the local graph around the stand-in's page (REQ-006).
- `909-07-brain-page`: the bookmarked page open in the Brain tab (REQ-007).
- `909-08-day`: the Browser tab's row in Marley Day (REQ-002, REQ-008).

## Locked-In Decisions
- D1 — One `Focus` variant per screen, holding the item's entity. `set_active_pane_item` reads only
  the item and observes it; whatever needs the workspace or the window is read in `render`
  (PR-claude-831, F-claude-893).
- D2 — The board's chip grammar: a raised chip naming the screen, carrying the kind's tile from
  #905's shared palette, `crates/marley_workbench/src/item_kind.rs` (kind tints, agent glyphs,
  state colours; the rail's `RowKind` and `RowState` move there), where the screen has a kind (the Browser's `www`); facts as muted text; a pink
  (`status.conflict`) chip with a dot for what needs the user; a red (`status.error`) chip with a
  dot for console errors, as the Browser board draws it.
- D3 — The words the board right-aligns follow the chips inside the focus item. The item keeps
  `flex_none` and caps its long parts (a page title, an address, a broken template's error) with
  `max_w` and truncation (F-claude-833).
- D4 — The Threads page's "needs you" is the existing waiting chip (#833), not a second chip.
- D5 — The row makes no call of its own. Each fact comes from state the screen or a global already
  holds; a fact not read yet is left out.
- D6 — The embedding reads "N pages embedded", with "· N waiting" when Rusty reports stale pages,
  and "full-text only" when Rusty has no embedding provider. Rusty's status gives no vault total,
  so the board's "of M" is not drawn.
- D7 — The Brain tab draws its page's row, the same as a page in a tab of its own.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Threads page is in front, the bottom row shall show a "Threads" chip, the number of conversations and the number open in tabs. | `909-01-threads` |
| REQ-002 | WHILE a Browser tab is in front, the bottom row shall show the `www` chip with the page's title, the address, a red chip with the console error count when the page logged errors, the picks ready, and "recording the last minute". | `909-02-browser`, `909-08-day` |
| REQ-003 | WHILE MCP Servers is in front, the bottom row shall show an "MCP Servers" chip, a pink chip for each agent CLI not on the PATH, and the number of tools Marley's server lists. | `909-03-mcp` |
| REQ-004 | WHILE the Component Library is in front, the bottom row shall show a "Component Library" chip, a pink chip naming the first broken template's file and error, and the number of components and templates. | `909-04-components` |
| REQ-005 | WHILE Rusty's Tasks tab is in front, the bottom row shall show "Tasks · <list>", a pink chip with the overdue decisions when there are any, and the list's open count. | `909-05-tasks` |
| REQ-006 | WHILE Rusty's Graph tab is in front, the bottom row shall show the graph's scope, its depth and page count, and the embedding as Rusty last reported it. | `909-06-graph` |
| REQ-007 | WHILE a brain page is in front, in its own tab or in the Brain tab, the bottom row shall show a "Brain" chip, the page's slug, "★ favourite" for a bookmarked page, and whether Rusty is connected. | `909-07-brain-page` |
| REQ-008 | The rows shall draw in Marley Night and Marley Day from the theme's colours, with no colour of their own. | `909-08-day`, the diff review |
| REQ-009 | The row shall read each screen's facts from state already held, with no Rusty or browser call of its own, and shall not read the workspace in `set_active_pane_item`. | the diff review |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the seven `Focus` cases and their accessors; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
