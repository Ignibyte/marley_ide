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

## Promotion (2026-09-26, at `bfa88a1a58`)
- **Seams re-read:**
  - `marley_rail`: `ProjectSnapshot` (30), `TerminalSnapshot` (46), `Focus` (159), `Selection`
    (185), `Row` (258), `SwitcherRow::selection` (279), `selection` (295), `cycle_row` (456),
    `parent` (483), `rail_rows` (505) and `switcher_rows` (580). They are as the design assumes:
    the selection is exhaustive and threads follow terminals.
  - `rail.rs`: `activate_terminal` (576), `close_terminal` (611), `open_row` (842),
    `render_terminal_row` (1318), `Watched` (1416), `resubscribe` (1505), `terminal_snapshot`
    (1716) and `build_snapshot` (1804).
  - `browser.rs`: `BrowserEvent` (156), `PageState` (280), `HubHandle` (436), `pick_captured`
    (1270), the tray's `discard_pick` (1399), `annotations` (1477), `loading_changed` (1555),
    `agent_ended` (1676), `add_viewer` (1859), and the `Page.loadEventFired` arm (2347-2349).
  - `page.rs`: `isolated_context` (822, private: the favicon reader lives in the same crate).
  - `source_map.rs`: `load_resource` (356) and `read_stream` (395, text), which get a bytes form.
  - #510 (worktree agents) has not started, so `ProjectSnapshot` has one change in flight, this
    one.
- **What has landed since the draft:** #503's `links.rs` and the footer's offer; #574's
  `tab_workspaces`, `window_of` and the focus history (`set_focused`), which the row's click
  reuses to count as the user's focus; #579's link menus. Nothing else touches the rail.
- **Recall added:**
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: Browser rows come after
    terminal rows, so the first-terminal clicks in #503, #549, #561 and #579 do not move. A
    scenario that clicks a second project's rows after a Browser tab opens in the first could
    move, and the golden set shows it.
  - AD-claude-503 and #500: a tab opened from the rail or a link is a `BrowserView` in the
    workspace's pane, which is what a row lists.
- **Brain consultation 9bd9180ba9da46e4b86ff2382a103741:** nothing on this seam.

## Phase 2 — Code
- **Built** (Marley crates only):
  - `marley_rail`:
    - `BrowserSnapshot` (id, title, host, loading, picks, annotations, `agent_unseen`, the
      icon's id, matched) on `ProjectSnapshot.browsers`, and `Focus.browser`;
    - `Selection::Browser`, `BrowserRow`, `Row::Browser` and `Shown::Browser`;
    - `walk` lists terminals, then Browser tabs, then threads; `selection` tries thread,
      terminal, Browser tab, then project; `cycle_row` and `parent` take Browser rows;
    - the switcher stays as it was;
    - the tests' helpers and matches gain the new fields and arms, so they build (not run, §7).
  - `marley_browser`:
    - `favicon` (new): `FAVICON_HREF` (the isolated-world script), `format` from the first bytes
      (PNG, ICO, GIF, JPEG, WebP, SVG), `data_url_bytes` and `MAX_BYTES` (256 KiB);
    - `Page::favicon_href`, and `Page::load_bytes(url, cap)`, split out of `load_resource`,
      whose stream reader now returns bytes.
  - `browser.rs`:
    - `BrowserHub::try_global`, which never makes the hub, and
      `BrowserEvent::PageStatusChanged`, emitted through `status_changed` for loading, picks
      staged or dropped, annotations added, removed or cleared, the icon's arrival or loss, and
      the agent's mark;
    - `PageState.favicon` (`Favicon { origin, image }`) and `agent_unseen: Option<Instant>`,
      set by `agent_ended` when no tab draws the page and cleared by `add_viewer`, which now
      takes `&mut Context` to emit;
    - `read_favicon` after each `Page.loadEventFired`, beside the recorder's snapshot, when
      the origin has no icon; the icon is dropped in `navigated` when the page leaves its
      origin;
    - accessors `favicon`, `agent_unseen` and `pick_count`, and `BrowserView::target`.
  - `rail.rs`:
    - `BrowserEntry`, and `Snapshot.browsers` and `favicons`; `Watched.browsers`;
    - `follow_browsers` (each tab's `ItemEvent`s, and the hub's page events once
      `try_global` finds it);
    - `member_browsers` and `browser_snapshot`; `host_and_port`; `active_rows` (the active
      terminal or Browser tab);
    - `render_browser_row` (the icon, spinner or globe, the title, the host, the counts, the
      mark, the close button on hover), `activate_browser`, `close_browser`, and `open_row`'s
      Browser arm;
    - `rail_tests.rs`' matches gain the arm.
- **Deviations from the plan, and why:**
  - `BrowserSnapshot.icon` (the image's id). The rail redraws only when the pure snapshot
    changes (`refresh`), and an icon arriving changed nothing in it, so the icon never showed.
  - `agent_unseen` is an `Option<Instant>` rather than a bool: `PageState` passed clippy's
    limit of three bools.
  - `sync_subscriptions` and `build_snapshot` passed clippy's 100 lines, so `follow_browsers`,
    `member_browsers` and `active_rows` are functions of their own.
- **Review:**
  - The rail reads the hub only through `try_global`, in `member_browsers` and
    `follow_browsers`, so nothing in it starts the browser (REQ-010).
  - No refresh per screencast frame: the rail follows the hub's page events, never its notify.
  - Re-entrancy: `add_viewer` emits inside the hub's own update, from the page element's paint,
    and the rail's callback runs once effects flush. The row's click and close use the
    workspace and the pane, not the tab being updated.
  - A favicon read that fails logs at debug and leaves the globe.
  - Checks: `cargo check` with all targets; fmt; `cargo clippy -p marley_rail -p marley_browser
    -p marley_workbench --all-targets -- -D warnings`, clean after the findings above; and
    `cargo dylint` on the same three crates, clean (#579's lesson).

## Phase 3 — Test
- **Scenario:** `script/e2e/504-browser-tabs-in-the-rail.sh` (`compositor sway`, offline
  Chromium, 1600x1000). The setup writes the site: `index.html` ("Checkout", a
  `<link rel="icon" href="icon.png">` to a solid red 32 px PNG made with `zlib` and `struct`, a
  Save button at 40,80 in the page), `plain.html` ("Plain page", no icon link; the fixture's
  server answers `/favicon.ico` with a 404) and the fixture's `/slow`. Fakes of `xdg-open`, `gio`,
  `google-chrome` and `firefox` come first on the PATH (checked with `command -v` before the
  launch) and log to `leak.log`. Tabs open through the palette's `marley: new browser tab` and a
  typed URL.
- **Checks (7):**
  - no browser unit before the first tab (`systemctl --user is-active` on the run's unit);
  - the icon's red (230,20,20) inside the 16 px square at the Checkout row's icon, and none at
    the Plain row's;
  - `browser_tabs` names Checkout focused after its row's click, and Plain page focused after
    Down, Down and Enter from the terminal's row;
  - the Slow page is gone from `browser_tabs` after its row's close button;
  - `leak.log` stays empty.
- **Deviations from the E2E plan:**
  - Tabs open through the palette rather than the project's +: #500 covers the +, and the
    palette opens each tab beside the one in front.
  - The pick comes from the palette's `marley: pick element`; the annotation from the stand-in
    agent (`mcp_agent annotate`), which D6 counts the same. It is made while the Checkout tab is
    in front, so shot 3 also shows D5's other half: an action the user watched sets no mark.
  - REQ-007 goes Down twice, to the Plain row. The Checkout tab was the last focused, so Enter
    on the first Browser row would have left `browser_tabs` unchanged and proved nothing.
- **Coordinates,** measured on a first run with its checks made soft (a copy, the `expect` call
  sites changed by sed): the terminal row at y 136, Browser rows at 182, 228 and 274 (46 px
  apart), the icon's 14 px square at x 32 to 45, the close button at x 233. Every check passed on
  the guessed values too; the measured ones replace them.
- **The shots** (`SHOT_DIR` in the scratchpad; each read, cropped at 2x where small):
  - `504-01-rows`, REQ-001, 002, 011: under `repo`, `repo — bash`, then `Checkout` with the red
    icon and `127.0.0.1:<port>`, then `Plain page` with the globe and the same host, selected
    because its tab is in front.
  - `504-02-loading`, REQ-003: a fourth row with the spinning `LoadCircle`. Its title is
    `about:blank` and it has no host line: until `/slow` answers, the tab's committed document
    is the blank page, and the row takes the tab's own title (D1).
  - `504-03-counts`, REQ-004: the Checkout row, selected, with `⊕ 1` and `✎ 1`; the tray holds
    `Pick 1 button "Save"`, the agent's box reads "The agent checked this"; no mark.
  - `504-04-agent-mark`, REQ-005: the Plain page in front; the Checkout row carries the blue
    `Sparkle` after the counts.
  - `504-05-shown`, REQ-005, 006, 011: the Checkout tab in front, its row selected, the mark gone.
  - `504-06-keyboard`, REQ-007: the Plain page in front and its row selected, after Ctrl+Alt+;,
    Down, Down and Enter from the terminal's row.
  - `504-07-filter`, REQ-008: `plain` in the filter; under `repo` only the Plain row, "Plain"
    in the accent color, the clear button in the field.
  - `504-08-closed`, REQ-009: the terminal in front, three rows left, and the tab bar without
    the Slow tab.
- **REQ-010:** the run's first check, `systemctl --user is-active` on the run's unit after the
  launch and before the first tab, passed. Marley's log does not record the unit's start, so that
  check is the evidence, not the log.
- **Red run:** the same scenario on the installed build (`E2E_BINARY=~/.local/bin/marley`, at
  `e83b5e943c`, without #504) fails its first rail check, "the page's icon shows on the Checkout
  row"; its shot shows only `repo — bash` under the project while the tab bar holds both tabs.
- **Focus:** under sway, `hyprland: 0 Marley windows before the run, 0 after; the run added no
  rule and did not reload it`. Nothing reached Chad's session.
- **The Marley log:** no panic; errors are the usual noise (wtype's `NoKeymap`, the Vulkan
  loader, the llama.cpp provider) and the browser's closing at teardown.
- **The golden set** (`just regress`, 25 with #504's added to `script/e2e/golden`): all 25
  passed on the debug build (`~/.local/state/marley/regress/20260926-202017`).
- **#574's click moved (L-claude-498).** Its checks passed, but its shots showed why they should
  not be trusted alone: repo-b's page now has a row under repo-b's terminal, which puts repo-a's
  rows 46 px lower, so the click at y 229 meant for repo-a's terminal row landed on repo-a's
  header. The header shows the project and the next click, in the terminal's pane, focused the
  terminal, so nothing failed. `RAIL_A_Y` is now 275: a copy with a shot straight after the click
  shows the pointer on repo-a's terminal row (its close button on hover), and the scenario itself
  passes 9 of 9 again. No other scenario clicks a row below a project that holds a Browser tab:
  #500 and #501 click the first header's +; #503, #549, #561 and #579 the first terminal row,
  which Browser rows, coming after terminals, never move; #576 a tab.
- **#501, rerun** as the Phase Plan asks (it is not in the golden set, and has no checks): the
  project's + opens its menu (`501-01-agents`: New Terminal, New Browser Tab, New Agent Thread with
  the stand-in), and after the stand-in's `browser_navigate` the rail lists under `repo` the
  terminal, then "Driven by an agent" (globe, `127.0.0.1:<port>`), then the thread (`501-02-driven`),
  D2's order. The row has no mark: the tab drew the page as it opened, which clears it (D5).
- **The gate:**
  - The first `script/gates.sh --diff` was red on gate:14 only (full log kept in the scratchpad,
    `gate-504.log`). `favicon.rs`'s module doc linked `FAVICON_HREF`, which is `pub(crate)`, from
    a public module (`private_intra_doc_links`), and ``[`format`]`` was ambiguous with the
    `format!` macro (`broken_intra_doc_links`). Fixed at the source: the constant named without a
    link, the function linked as ``[`format()`]``. The failure had stopped the doc build before
    `marley_workbench`, so `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` ran over the whole
    scope next, clean.
  - The second run: `GATE GREEN [diff]`, 16 passed, 0 failed; the receipt matches the tree
    (`16817861b036…`).
- **Pre-existing, not in scope:** none.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Documented (§21):**
  - `CHANGELOG.md`: Added, "Browser tabs in the rail".
  - `docs/marley/three-prong-plan.md`: slice B7a (#504), shipped.
  - `docs/marley_architecture/marley_rail.md`: the input's `BrowserSnapshot`s and
    `focus.browser`, the walk's order, the selection, `parent`, `cycle_row`, attention and the
    switcher.
  - `docs/marley_architecture/marley_workbench.md`: the rail's Browser rows; what the rail reads
    from the hub; a known limit (first icon per origin, no load event, not in the switcher, no
    attention dot).
  - `docs/marley_architecture/marley_browser.md`: page icons (`src/favicon.rs`), and
    `load_resource` over `load_bytes`.
  - No path outside the Marley-owned set changed, so no touchpoint row to check.
- **Knowledge appended (§19):**
  - F-claude-504-a-favicons-arrival-changed-nothing-the-rail-compares-001
  - PR-claude-a-view-that-redraws-on-a-changed-snapshot-keeps-what-it-draws-in-it-001
  - L-claude-504-cdp-has-no-favicon-event-so-read-the-icon-after-load-001
  - L-claude-504-a-click-that-passes-can-still-miss-its-target-001
  - L-claude-504-rustdoc-checks-what-clippy-and-dylint-do-not-001
  - AD-claude-504-browser-tabs-are-rows-of-their-project-in-the-rail-001
- **Brain:** consultation 9bd9180ba9da46e4b86ff2382a103741 closed with `brain decide`
  (`decisions/browser-tabs-are-rows-of-their-project-in-the-marley-rail-504`, follow-up by
  2026-10-26).
- **Ticket:** closed, moved to `tickets/closed/`, its pipeline link on `completed/`; no stale
  BACKLOG row.
- **Checklist (no task tool in this session):** document, capture knowledge, close the ticket,
  archive, commit.
