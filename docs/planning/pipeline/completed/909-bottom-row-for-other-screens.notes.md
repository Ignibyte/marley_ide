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

- **Promoted 2026-10-25:** seams re-read after #907 and #908: `focus_row.rs` now has `Focus`
  with File, Terminal and Thread, `chip(text, kind, cx)` and `muted`; `ThreadsPage`
  (`threads_page.rs:204`), `conversations` :91; `BrowserHub::title`, `address`, `pick_count`,
  `console_entries`, `is_error` (private, :8496); `agent_versions::found`; `marley_mcp`'s
  `tools_list_for` filter; `Templates::broken` and `templates()`, `marley_components::catalog`;
  `TasksView` (`chosen`, `lists`, `counts` as open and done, `side.due`), `GraphView` (`scope`,
  `depth`, `shown.nodes`), `PageView::slug`, `BrainTab.page`, `favourites::is_page`
  (`pub(super)` in a private module), `rusty::is_connected`, the `Rusty` global's `semantic`.

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
- **Built:** `focus_screens.rs` (new: `Screen` with `of`, `observe`, `render`, one function per
  screen, `dot_chip`, `name_chip`, `capped`, `count_words`); `focus_row.rs` (`Focus::Screen`,
  `chip` and `muted` shared); accessors `TasksView::row_facts`, `GraphView::row_facts`,
  `rusty::is_favourite_page`, `rusty::semantic`, `browser::is_error`, and
  `marley_mcp::served_tool_count` (re-exported). `BrainTab::page` already existed.
- **Deviations:** a module of its own for the screens, so `focus_row.rs` keeps to files,
  terminals and threads. The Graph's scope and depth are a `&'static str` and an `Option<usize>`
  (no `bool` for the scope).
- **Review:** `set_active_pane_item` only downcasts the item and observes it (D1); the Threads
  page's open count reads the window's workspaces in `render`, outside a workspace update. No Zed
  path.
- **Gate:** `just gate-diff` 17/17 green after `option_if_let_else` (the downcasts as an
  `or_else` chain), `needless_pass_by_value` (`Severity` is `Copy`) and `missing_const_for_fn`.

## Phase 3 — Test
- **Scenario:** `script/e2e/909-bottom-row-for-other-screens.sh` (sway): Rusty's stand-in with
  today's note bookmarked, a decision due yesterday and a three-task list; a broken template;
  Codex's folder off Marley's PATH; a local page that logs a console error.
- `909-01-threads`: "Threads", "1 conversation", "0 open" (REQ-001).
- `909-02-browser`: `www` "Console fixture", the address in the buffer font, "2 console errors" in
  red (the page's own and the browser's favicon miss), "recording the last minute" (REQ-002).
- `909-08-day`: the Browser row in Marley Day (REQ-002, REQ-008).
- `909-03-mcp`: "Codex is not on the PATH" in pink, "marley · 68 tools" (REQ-003).
- `909-04-components`: "quota.json · The file is not a template…" in pink, capped; "15 components
  · 0 templates" (REQ-004).
- `909-05-tasks`: "Tasks · Home", "1 decision overdue", "3 open" (REQ-005).
- `909-06-graph`: "Graph · vault", "2 pages", "full-text only" (the stand-in has no embedding
  provider) (REQ-006).
- `909-07-brain-page`: "Brain", "daily/2026-10-25", "★ favourite", "Rusty connected" (REQ-007).
- Not reached: a pick in the Browser tray ("N picks ready" shows only above zero), a local graph's
  depth (the tab opened on the vault with no page in front), and the embedded count with a
  provider; each reads the state shown in the diff.

## Phase 4 — Complete
- CHANGELOG, `docs/marley_architecture/marley_workbench.md`. No Zed path, so no ledger row.
- Knowledge: nothing new; the run found no bug.
