# Browser tabs as rows of their project in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-504-browser-tabs-in-the-rail.md
- **Pipeline spec:** 504-browser-tabs-in-the-rail.spec.md

## Phase 1 — Plan (drafted overnight, 2026-09-25)
- **Request:** Chad, 2026-09-25: "lets do 1 through 8", item 4 (first half): Browser tabs as rows
  of their project in the rail. The Orca survey of the same day (report 03 item 8, the README's
  #504 row) adds a favicon read from the page, a spinner while loading, host and port, counts of
  picks and annotations, and a mark when an agent acted on the page; its "later" port rows stay
  out.
- **Classification / tier:** feature, prong 3 with the rail. Marley crates only: `marley_rail`
  (pure model), `marley_workbench` (rail, hub), `marley_browser` (the icon read). No Zed path.
- **Recall (§18.3):**
  - AD-claude-493-one-browser-tab-per-page-001: a tab is a Zed item, one per page; an agent's
    page never takes the focus. So a row per `BrowserView`, and the agent's mark covers exactly
    the pages opened or driven behind the user's back.
  - L-claude-488-chromium-reports-a-new-url-not-the-documents-title-001: titles come from
    `Target.getTargetInfo` after load events; the row reads the hub's title, which that already
    keeps current, and a title an SPA changes later stays stale here too.
  - L-claude-493-a-move-between-panes-is-a-remove-then-an-add-001: a tab moved between panes
    passes through `on_removed`; the rail rebuilds from the workspaces' items, so a moved tab keeps
    its row.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: new rows move whatever an
    older scenario clicks below them; #500's and #501's scenarios click the rail and rerun.
  - L-claude-499-a-screencast-sends-frames-only-when-the-page-changes-001: the hub is busy while
    pages paint; D7 keeps the rail off the per-frame notify.
  - Brain: not consulted in this drafting run (read-only brief); promotion runs `brain_ask`.
- **Discovery:**
  - `crates/marley_rail/src/marley_rail.rs`: `ProjectSnapshot` (line 30), `TerminalSnapshot`
    (46), `Focus` (156), `RailSnapshot` (171), `Selection` (182), `Row` (253), `SwitcherRow` (264),
    `selection` (290, the order thread, terminal, project), `Shown` (314), `walk` (347, terminals
    then threads under each shown project), `row_shows` (373), `cycle_row` (451), `parent` (478),
    `rail_rows` (500), `window_row` (548), `switcher_rows` (574); the tests module (670) builds
    snapshots through helpers such as `project` (685).
  - `crates/marley_workbench/src/rail.rs`: `TerminalEntry` (154) and `Snapshot` (173) hold the
    entities behind rows; `refresh` (299) rebuilds and notifies only when the pure snapshot
    changed; `sync_subscriptions` (389) resubscribes per entity; `activate_terminal` (526) and
    `close_terminal` (561) are the handlers a Browser row mirrors; `open_row` (792) opens a row
    for Enter; `render_terminal_row` (1268) is the row a Browser row copies (icon, label, second
    line, close on hover); `Watched` (1366, `in_window` 1376) lists the entities followed;
    `terminal_snapshot` (1666) and `build_snapshot` (1734) read each member workspace's
    `items_of_type::<TerminalView>`; `row_card` (1910); `thread_status_mark` (1959) holds the
    `LoadCircle` spinner; `impl Render for Rail` (2139) maps `Row`s to elements.
  - `crates/marley_workbench/src/browser.rs`: `BrowserHub::global` (448) creates the hub and
    calls `start`, which starts Chromium, so the rail cannot call it; `HubHandle` (432) is the
    global it keeps; `BrowserEvent` (153); `PageState` (277, with `loading`, `annotations`,
    `agent`, `viewers`); `title`, `url`, `is_loading` (559 to 586); `picks` (1316); `annotations`
    (1427); `agent_ended` (1626, where the mark is set); `loading_changed` (1505, which notifies
    but emits nothing); `add_viewer` (1809, where the mark clears); the `Page.loadEventFired` arm
    (2296, where the icon read starts); `BrowserView` (2988, its `target` and `workspace` private
    to the module); `Item for BrowserView` (4788: `tab_content_text` gives the title,
    `tab_icon` the `ToolWeb` globe).
  - `crates/marley_browser/src/page.rs`: `isolated_context` (822), the world the icon read runs
    in. `crates/marley_browser/src/source_map.rs`: `Page::load_resource` (356) and `read_stream`
    (395), which decode to UTF-8 and cap at 32 MiB; the icon needs the bytes.
  - gpui: `Image::from_bytes` (platform.rs 2953) and `ImageFormat` (2827, `Ico` among them); the
    workspace's `image` crate has `ico`, `png`, `gif`, `jpeg`, `webp`.
  - `crates/tab_switcher/src/tab_switcher.rs`: items listed by `tab_icon` and
    `tab_content_text`, activated with `pane.activate_item`.
- **Decisions:** D1 to D8 in the spec.

### Design
- **Approach.**
  1. *Model (`marley_rail`).* `BrowserSnapshot { id: u64, title: String, host: Option<String>,
     loading: bool, picks: usize, annotations: usize, agent_unseen: bool, matched: Option<Vec<
     usize>> }` on `ProjectSnapshot.browsers`; `Focus.browser: Option<u64>`; `Selection::Browser(
     u64)`; `BrowserRow` with the snapshot's fields plus `project`, `selected` and `highlight`;
     `Row::Browser(BrowserRow)`; `Shown::Browser`. `walk` yields terminals, then Browser tabs, then
     threads; `selection` tries thread, terminal, browser, project; `parent` finds a Browser row's
     project; `cycle_row` takes Browser rows as it takes terminals; `switcher_rows` stays as it is.
     Every `match` on `Selection` stays exhaustive, so the compiler finds each place to extend
     (the switcher's entries map `Selection::Browser` to nothing).
  2. *Hub (`browser.rs`).* `BrowserHub::try_global(cx: &App) -> Option<Entity<Self>>` reads
     `HubHandle` without creating it. `BrowserEvent::PageStatusChanged { target }`, emitted from
     `loading_changed`, `agent_ended`, `add_viewer` (when it clears the mark), the favicon's
     arrival, and each change to a page's picks or annotations (`pick_captured`, `discard_pick`,
     `add_annotation`, `clear_agent_annotations` and the user's delete). `PageState` gains
     `favicon: Option<Favicon { origin: String, image: Arc<gpui::Image> }>` and `agent_unseen:
     bool`; `agent_ended` sets `agent_unseen` when `viewers == 0`; `add_viewer` clears it.
     Accessors for the rail: `favicon(target)`, `agent_unseen(target)`, `pick_count(target)`,
     `annotations(target).len()`. `BrowserView` gains `pub(crate) fn target(&self)`.
  3. *Icon (`marley_browser`).* `Page::favicon_href(&self) -> Result<Option<String>, CdpError>`
     runs, in `isolated_context`, a function that lists `document.querySelectorAll('link[rel~=
     "icon" i]')` hrefs and returns the first http, https or `data:image/` one, else
     `new URL('/favicon.ico', location.origin)` for an http or https page, else nothing.
     `Page::load_bytes(url, cap)` is `load_resource` without the UTF-8 step, `read_stream`
     returning bytes and each caller deciding; `favicon::format(bytes) -> Option<ImageFormat>`
     reads the magic bytes (`\x89PNG`, `\0\0\1\0`, `GIF8`, `\xFF\xD8`, `RIFF....WEBP`, `<svg` or
     `<?xml` after whitespace). A `data:image/` href is decoded in place. After
     `Page.loadEventFired` the hub compares the page's origin with its favicon's and reads a new
     one only when they differ; a navigation to another origin drops the old icon at once.
  4. *Rail (`rail.rs`).* `BrowserEntry { workspace, view }` and `Snapshot.browsers:
     HashMap<u64, BrowserEntry>`, `Snapshot.favicons: HashMap<u64, Arc<Image>>` (gpui data stays
     out of the pure snapshot). `build_snapshot` reads each member's
     `items_of_type::<BrowserView>` beside its terminals, through `browser_snapshot(view, hub,
     cx)`: title from `tab_content_text`, host and port from the hub's URL (`url::Url::host_str`
     and `port`, http and https only), the counts, `loading`, `agent_unseen`. `Focus.browser` is
     the displayed workspace's active item when it downcasts to `BrowserView`. `Watched` gains
     the Browser views (their `ItemEvent`s) and the hub when `try_global` finds one (its
     `PageInfoChanged`, `PageStatusChanged`, `PageOpened`, `PageClosed`). `render_browser_row`
     copies `render_terminal_row`: the icon (the favicon through `img`, the spinner while
     loading, else `ToolWeb`), the title, the host as the second line, then the counts
     (`Crosshair` and a number, `Pencil` and a number) and the `Sparkle` mark in accent, with the
     close button on hover. `activate_browser` activates the workspace and the item with the
     focus; `close_browser` closes it through its pane, as `close_terminal` does. `open_row`
     handles `Selection::Browser`.
- **File manifest.** Marley crates only: `crates/marley_rail/src/marley_rail.rs` (the model, and
  its test helpers kept building); `crates/marley_workbench/src/rail.rs`, `rail_switcher.rs` (the
  exhaustive match), `rail_tests.rs` (fields, kept building), `browser.rs`;
  `crates/marley_browser/src/page.rs`, `source_map.rs` (the byte reader), a `favicon` module in
  `crates/marley_browser` (new, pure: the href rule's constants and `format`). The scenario
  `script/e2e/504-browser-tabs-in-the-rail.sh` (Test); the fixture's `serve.py` in
  `script/e2e/browser-fixture.sh` answers a 404 for a missing `/favicon.ico` already
  (`SimpleHTTPRequestHandler`). Docs at Complete: `CHANGELOG.md`,
  `docs/marley/three-prong-plan.md`, `docs/marley_architecture/marley_rail.md`,
  `marley_workbench.md`, `marley_browser.md`.
- **Ledger rows.** None in `docs/marley/zed-touchpoints.md`: no Zed path changes. Knowledge at
  Complete (expected): an AD for Browser rows (a row per tab, the mark's meaning, events not
  notify) and a lesson on reading favicons from headless Chromium.

### E2E plan
Setup: `offline_chromium`; a site with `index.html` (title "Checkout", `<link rel="icon"
href="icon.png">`, a 32 px PNG the setup writes with Python's `zlib` and `struct`, a Save button),
`plain.html` (title "Plain page", no icon link; the server has no `/favicon.ico`), and `/slow`;
the scratch repository opened; `systemctl --user is-active "$(browser_unit)"` logged before
anything opens a tab.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-010 | before the first Browser tab, the unit's state | the run log: `inactive` (no unit) |
| REQ-001, REQ-002 | the rail's +, New Browser Tab, `index.html`; the same for `plain.html`; settle | `504-01-rows`: two rows after the terminal, the Checkout row with the page's icon, the Plain row with the globe, each with `127.0.0.1:<port>` |
| REQ-003 | New Browser Tab, `/slow`, shot within a second | `504-02-loading`: the spinner on that row; after three seconds its title and globe (log line from the next shot) |
| REQ-004 | in the Checkout tab: Ctrl+Shift+C and a click on Save (a pick); annotate mode and a drag with a note (an annotation) | `504-03-counts`: the Checkout row shows 1 pick and 1 annotation |
| REQ-005 | show the Plain tab in the same pane with the focus in it; `mcp_agent --tab <Checkout's id> click-on button Save` | `504-04-agent-mark`: the Checkout row carries the mark; the Plain page still in front |
| REQ-005, REQ-006, REQ-011 | click the Checkout row | `504-05-shown`: the Checkout tab in front and focused, its row selected, the mark gone |
| REQ-007 | Ctrl+Alt+; (`multi_workspace::FocusWorkspaceSidebar` in Zed's Linux keymap) puts the focus in the rail; Down from the terminal row, then Enter | `504-06-keyboard`: the first Browser row selected, then its tab shown |
| REQ-008 | type `plain` in the rail's filter | `504-07-filter`: only the Plain row under the project, "Plain" highlighted |
| REQ-009 | clear the filter; hover the Slow row and click its close button | `504-08-closed`: the row and the tab gone |

Not reachable by a scenario: an icon larger than the 256 KiB cap and an SVG icon (the review
checks the format sniffing); an agent opening a page in another window (the rows are per window,
as terminal rows are).

### Risks
- #510 nests worktree agents under their project and changes `ProjectSnapshot` too; whichever
  lands second merges the two shapes. D2 keeps Browser rows with their workspace's group either
  way.
- Reading the icon costs a script call and a resource load per new origin per page; a page that
  changes its icon without a navigation (an unread-count favicon) keeps its first one. Accepted.
- `loadEventFired` never comes for a page that never finishes loading; its row keeps the globe.
- Picks and annotations change in several hub methods; one that forgets `PageStatusChanged`
  leaves a stale count until the next refresh. The review lists every mutation of `picks` and
  `annotations`.
- The rail rebuilds on each `PageStatusChanged`; a page that toggles loading rapidly (a long poll
  in an iframe does not, only the main frame counts) would rebuild often. `refresh` notifies only
  on a changed snapshot, so the cost is the rebuild itself.
