# The bottom row for the other screens — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-909-bottom-row-for-other-screens.md
- **Pipeline spec:** 909-bottom-row-for-other-screens.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this". This is
  the design review's build order, item 2: the bottom row says what is in front, for every screen.
  Built after #906, #905, #907 and #908.
- **Classification / tier:** feature, M. Marley crates only (`marley_workbench`, plus one function
  in `marley_mcp`). No Zed crate changes.
- **Recall (§18.3):**
  - F-claude-833: the focus item shrank below its content and was drawn over. It keeps
    `flex_none`; long parts are capped with `max_w` and truncate.
  - PR-claude-831: a status item built from the workspace's construction reads the workspace only
    in `render` or a deferred step.
  - F-claude-893: a toolbar item read its workspace in `set_active_pane_item` and panicked. The
    status bar calls `set_active_pane_item` the same way (`status_bar.rs:449`), so this ticket
    reads only the item there.
  - F-claude-903: stand-ins go first on the terminals' PATH through the scenario's `.bashrc`.
  - F-claude-877 and F-claude-880: the harness isolates Claude Code's store and tmux's socket; the
    scenario starts no real agent CLI.
  - The brain: `decisions/the-bottom-rows-focus-is-one-marley-status-item-after-debug` (#833).
    Nothing else on this seam.
- **Discovery:**
  - `focus_row.rs`: `Focus` (`:33`), `render` (the match on `Focus`), `set_active_pane_item`
    (`:335`), `chip` (`:369`), `muted` (`:383`). #907 and #908 change the thread and terminal
    cases and add chip helpers first.
  - `threads_page.rs:91` `conversations`; `thread_tab.rs:1888` `tab_threads`.
  - `browser.rs`: `BrowserHub::try_global` (`:905`), `title`/`address` (`:1127-1145`),
    `pick_count` (`:2293`), `console_entries` (`:2605`), `is_error` (`:8496`),
    `BrowserView::target` (`:5234`).
  - `mcp_servers.rs:488` `render_agent`; `agent_versions::found` (`:440`);
    `marley_mcp/src/registry.rs:913` `tools_list`.
  - `component_library.rs:48` `templates`, `:114` `ComponentLibrary { shown, … }`;
    `marley_components.rs:826` `BrokenTemplate`.
  - `rusty/tasks_tab.rs:271` `TasksView`; `:1094` the overdue count.
  - `rusty/graph_tab.rs:179` `Scope`, `:249` `GraphView { scope, depth, shown, … }`.
  - `rusty/page.rs:407` `slug`, `:1326` the star; `rusty/brain_tab.rs:82` `BrainTab { page }`;
    `rusty.rs:206` `semantic`, `:741` `is_connected`.
  - The Fleet is `impl Panel` (`fleet.rs:35`), so `set_active_pane_item` never sees it: Out.
- **Decisions:** D1–D7 in the spec. Fleet and Agent Activity Out.

### Design
- `Focus` gains `Threads(Entity<ThreadsPage>)`, `Browser(Entity<BrowserView>)`,
  `McpServers(Entity<McpServers>)`, `Components(Entity<ComponentLibrary>)`,
  `Tasks(Entity<TasksView>)`, `Graph(Entity<GraphView>)` and `BrainPage(Entity<PageView>)`.
  `set_active_pane_item` downcasts each in turn and observes the entity in `item_events`. A
  Browser tab also observes the `BrowserHub`; the Rusty cases observe the `Rusty` global. For a
  `BrainTab` it takes the tab's page and observes the tab, so a new page redraws the row.
- One `render_*` per case. Each returns an `AnyElement` built from: `focus_chip(label, tile)`, a
  raised chip with an optional kind tile from `item_kind.rs` (#905); `muted(text)`;
  `alert_chip(text, Alert)`, where `enum Alert { NeedsYou, Error }` picks `status.conflict` or
  `status.error` for the dot and its background. If #907 or #908 already add an equal helper, it
  is reused.
- The accessors return small structs with named fields (the style guide: no tuples in
  signatures):
  - `TasksView::row_facts(&self) -> TasksFacts { list: Option<SharedString>, overdue: usize, open: usize }`.
  - `GraphView::row_facts(&self) -> GraphFacts { scope_label: &'static str, depth: Option<usize>, pages: usize }`
    (depth only for the local graph).
  - `ComponentLibrary::component_count(&self) -> usize`.
  - `BrainTab::page(&self) -> Option<&Entity<PageView>>`.
  - `marley_mcp::registry::served_tool_count() -> usize`, counted from `REGISTRY` as
    `tools_list` filters it.
- The Threads case counts open tabs in `render`, from `window.root::<MultiWorkspace>()`, over each
  member's `thread_tab::tab_threads`.
- The embedding words come from the `Rusty` global's last `SemanticStatus`. A `pub(crate)`
  accessor in `rusty.rs` returns it.
- File manifest (all Marley crates): `marley_workbench/src/focus_row.rs`, `threads_page.rs` (only if
  `conversations` needs widening), `browser.rs` (an `is_error` made `pub(crate)` if needed),
  `component_library.rs`, `rusty.rs`, `rusty/tasks_tab.rs`, `rusty/graph_tab.rs`,
  `rusty/brain_tab.rs`, `marley_mcp/src/registry.rs`.
- No `docs/marley/zed-touchpoints.md` rows: no Zed path changes.

### Visual check plan
- REQ-001: open the Threads page from the palette with two stand-in conversations, one open in a
  tab; `909-01-threads` shows "Threads", "2 conversations", "1 open".
- REQ-002: serve a page that logs one `console.error`, open it in a Browser tab, pick one element
  (a sway click); `909-02-browser` shows the `www` chip and title, the address, the red "1 console
  error" chip, "1 pick ready", "recording the last minute".
- REQ-003: open MCP Servers with no `codex` on the PATH; `909-03-mcp` shows the chip, the pink
  "Codex is not on the PATH" chip and "marley · N tools".
- REQ-004: a broken `quota.json` in the run's `components/`; open the Component Library;
  `909-04-components` shows the pink chip with the file and error, and the counts.
- REQ-005: Rusty's stand-in with a list of three open tasks and an overdue decision; open Tasks;
  `909-05-tasks` shows "Tasks · <list>", the pink overdue chip, "3 open".
- REQ-006: open the local graph around the stand-in's page; `909-06-graph` shows "Graph · local",
  "depth N · N pages" and the embedding words.
- REQ-007: open the bookmarked page in the Brain tab; `909-07-brain-page` shows "Brain", the slug,
  "★ favourite", "Rusty connected".
- REQ-008: switch to Marley Day with the Browser tab in front; `909-08-day`.
- REQ-009: the diff review checks that no case calls Rusty or the browser, and that
  `set_active_pane_item` reads no workspace.
- What no scenario reaches: none. The stand-in's `brain_semantic_status` answer decides which
  embedding wording the shot shows; the other two wordings are checked in the diff review.

### Risks
- The row grows long on a narrow window. Each case caps its long parts (D3); Zed's language
  server message still gives way first.
- Observing the `BrowserHub` redraws the row on every hub change while a Browser tab is in front.
  The row is cheap, so this is acceptable. If a probe run shows churn, follow only the tab's page
  events.
- Seven cases in one slice. If the Code phase runs long, the Graph and brain page cases move to a
  follow-up first, before Threads, Browser, MCP Servers or Tasks.
- `BrainTab`'s page can change while the tab stays in front. The row must observe the tab, not
  only the page it held when the tab came forward.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
