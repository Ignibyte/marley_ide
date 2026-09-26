---
pipeline_id: 0c49c625-4cba-4377-9c12-5bcca1b1055e
ticket: docs/planning/tickets/open/TICKET-504-browser-tabs-in-the-rail.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Browser tabs as rows of their project in the rail, with favicon, loading, host, counts and the agent's mark"
type: feature
slice: prong 3 with the rail (after wave 2), item 4 (first half) of the list after the browser waves
references: [docs/orca_architecture/README.md, docs/orca_architecture/03-browser-and-design-mode.md, docs/planning/pipeline/completed/500-browser-from-the-rail.spec.md, docs/planning/pipeline/completed/493-browser-tabs-and-restore.spec.md]
---

## Title
Each Browser tab gets a row under its project in the rail, beside the project's terminals, so a
page an agent opened, or one still loading, or one holding picks and annotations, is one click
away and visible at a glance.

## Scope
### In
- **The model (`marley_rail`, pure).** `ProjectSnapshot.browsers`, one `BrowserSnapshot` per
  Browser tab of the project's workspaces (the view's entity id, title, host and port, loading,
  pick and annotation counts, the agent's mark, the filter's match); `Row::Browser`,
  `Selection::Browser`, `Focus.browser` (the displayed workspace's active item when it is a
  Browser tab). Rows under a project: terminals, then Browser tabs, then agent threads. The
  selection, the keyboard (`step`, `first_row`, `last_row`, `cycle_row`), `parent` and the filter
  include Browser rows.
- **The row (`marley_workbench::rail`).** Icon: the page's favicon, a spinning `LoadCircle` while
  the main frame loads, the `ToolWeb` globe when the page has no icon. Title: the page's title (the
  tab's text). Second line: host and port (`localhost:5173`), nothing for `about:blank`. At the
  end: the pick count and the annotation count, each an icon and a number when above zero, and
  the agent's `Sparkle` when an agent acted in the page since the user last looked; a close button
  on hover. A click shows the row's workspace and brings the tab forward with the focus.
- **The hub (`marley_workbench::browser`).** `BrowserHub::try_global`, which never starts the
  browser; `BrowserEvent::PageStatusChanged` for loading, favicon, counts and the agent's mark; per
  page the favicon and its origin, and `agent_unseen`.
- **The favicon (`marley_browser`).** After each main-frame load whose origin has no icon, the
  page's `link[rel~="icon"]` hrefs are read in an isolated world; the first http, https or
  `data:image/` one is taken, else `<origin>/favicon.ico`; it is loaded through the page's frame
  (`Network.loadNetworkResource`, 256 KiB at most) and its format read from its bytes (PNG, ICO,
  SVG, GIF, JPEG, WebP), for gpui's `Image::from_bytes`.

### Out (explicitly deferred)
- Rows for live ports with no tab, whose click opens one: the ports-per-project ticket, fed by
  #503's listener check (the survey's "later" rows).
- Browser tabs in the rail's switcher (`toggle_thread_switcher`): the switcher keeps terminals
  and threads.
- The favicon in the Browser tab's own tab and in Zed's tab switcher, which keep the globe.
- A Browser row lighting its folded project's attention dot; a right-click menu on the row;
  dragging rows.
- Worktree agents' own groups (#510, nested under their project by Chad's decision): Browser
  rows sit under whichever group holds their workspace, as terminal rows do.

## Reference (§20)
Upstream Zed: the rail stands in for Zed's Threads Sidebar through `workspace::Sidebar`, and a
Browser row activates its item as Zed's tab bar and tab switcher (`crates/tab_switcher`) do, with
the item's own title. Orca (report 03 §2.1 and item 8): browser tabs are not sidebar rows there,
only palette rows (`worktree-jump-palette-browser-simulator-rows.tsx`), but its favicon rules and
loading spinner (`describe-page/browser-favicon-url.ts`, `browser-favicon.tsx`) are the ones
Marley takes. Warp: the once-over's item 7 (PR state and diff stats on vertical-tab rows) is the
same row surface for terminals and a later ticket; the once-over lists browser use as Marley's
own, with nothing of Warp's to match.

### Prior art
- **Behavior maps and reports.** `docs/orca_architecture/README.md` (#504 row), report 03 §2.1
  (favicons from `page-favicon-updated`: `data:,` means none, the first http, https or
  `data:image/` entry wins, the icon goes when the page leaves its origin and stays while the
  origin is unknown) and item 8 (read `link[rel~=icon]` after load and fall back to
  `/favicon.ico`, since headless Chromium sends no favicon event). Orca files read at
  `1c2cf120e3`: `src/renderer/src/components/browser-pane/describe-page/browser-favicon-url.ts`
  (`displayableFaviconUrl`, `pickDisplayableFaviconUrl`, `browserNavigationLeavesFaviconOrigin`),
  `src/renderer/src/components/browser-favicon.tsx`,
  `src/renderer/src/components/worktree-jump-palette-browser-simulator-rows.tsx`.
- **Published material.** HTML's `rel` keyword matching (`icon` is a token, so `shortcut icon`
  counts) and the `/favicon.ico` convention; CDP has no favicon event in the `Page` domain, and
  `Network.loadNetworkResource` loads a resource for a frame.
- **Code we already ship.** `marley_rail` (`ProjectSnapshot`, `walk`, `selection`, `rail_rows`,
  `cycle_row`, `parent`); `rail.rs` (`build_snapshot`, `terminal_snapshot`, `render_terminal_row`,
  `row_card`, the `LoadCircle` spinner with `with_rotate_animation(2)` for running threads,
  `Watched`, `resubscribe`); `browser.rs` (`BrowserView` as a workspace item, the hub's `title`,
  `url`, `is_loading`, `picks`, `annotations`, `agent_ended`, `add_viewer`, `BrowserEvent`, the
  `Page.loadEventFired` arm); `Page::isolated_context` (page.rs) for the icon read;
  `Page::load_resource` (source_map.rs), which reads text only, so a bytes reader is split out of
  it; gpui's `Image::from_bytes` and `img` (PNG, ICO, SVG, GIF, JPEG, WebP; the workspace's
  `image` crate has `ico`); the icons `Crosshair` (pick), `Pencil` (annotate), `Sparkle` (the Agent
  chip) and `ToolWeb` (the tab). Checked and not used: `ui::CountBadge` (an error-tinted badge for
  notification counts, wrong for a neutral count).

## UI proof
UI-AFFECTING (new rows in the rail). `script/e2e/504-browser-tabs-in-the-rail.sh` (`compositor
sway`, offline Chromium): a site with a page that declares a PNG icon, a page with none (and no
`/favicon.ico`), and the fixture's `/slow` page. Shots: `504-01-rows` (two Browser rows under the
project, one with the page's icon and one with the globe, each with host and port), `504-02-loading`
(the spinner on the `/slow` row), `504-03-counts` (one pick, one annotation), `504-04-agent-mark`
(the stand-in agent clicked in the tab behind), `504-05-shown` (the row clicked: the tab in front,
its row selected, the mark gone), `504-06-keyboard` (Down from the terminal row selects a Browser
row, Enter shows its tab), `504-07-filter` (the filter shows the matching Browser row),
`504-08-closed` (a row's close button: the row and the tab gone). The run log shows no browser
unit before the first Browser tab opens.

## Locked-In Decisions
- D1 — A row is a tab, not a page: one row per `BrowserView` item in the project's workspaces,
  keyed by the view's entity id as terminal rows are, with the tab's own title.
- D2 — Order under a project: terminals, Browser tabs in the order the workspace lists its items,
  then agent threads. A Browser row sits under the group of the workspace that holds its tab.
- D3 — The rail never starts Chromium: it reads the hub through `BrowserHub::try_global`, because
  `BrowserHub::global` creates the hub and starts the browser. Without a hub there are no Browser
  tabs to list.
- D4 — Favicons follow Orca's rules, read the only way headless Chromium allows: the page's
  `link[rel~="icon"]` hrefs in an isolated world after `Page.loadEventFired` when the origin has
  no icon yet, the first http, https or `data:image/` href (`data:,` means none), else
  `<origin>/favicon.ico`; loaded through the page's frame, 256 KiB at most; dropped when the page
  moves to another origin, kept while the origin is unknown. A load that fails leaves the globe.
- D5 — The agent's mark means an agent acted here since the user last looked: it is set when an
  agent's action ends in a page that no tab draws, and cleared when a tab starts drawing it
  (`add_viewer`). An action the user watched sets nothing.
- D6 — Counts are the tab's picks still in its tray (sent or not, not dismissed) and all the
  page's annotations, the user's and the agents'; each shows only above zero.
- D7 — The hub tells the rail through events, never through its notify: a new
  `BrowserEvent::PageStatusChanged { target }` for loading, the favicon, the counts and the mark,
  beside `PageInfoChanged` for the title and URL. The rail refreshes on those and on the tabs'
  `ItemEvent`s, not on every screencast frame.
- D8 — A click shows the tab with the focus, as a terminal row does; the close button closes the
  tab through its pane, which closes the page (#493). No right-click menu in this slice.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a project has Browser tabs, the rail shall list each as a row under the project, after its terminals, with the page's title and its host and port. | Shot `504-01-rows` |
| REQ-002 | WHERE a page declares an icon or serves `/favicon.ico`, its row shall show that icon; where it has none, the globe. | Shot `504-01-rows` |
| REQ-003 | WHILE a page's main frame loads, its row shall show a spinner in place of its icon. | Shot `504-02-loading` |
| REQ-004 | WHEN a tab holds picks or its page annotations, its row shall show how many of each. | Shot `504-03-counts` |
| REQ-005 | WHEN an agent acts in a page no tab is drawing, its row shall carry the agent's mark until a tab draws the page again. | Shots `504-04-agent-mark`, `504-05-shown` |
| REQ-006 | WHEN the user clicks a Browser row, Marley shall show its project and bring the tab forward with the focus. | Shot `504-05-shown` |
| REQ-007 | WHILE the rail has the focus, Up and Down shall reach Browser rows, and Enter on one shall show its tab. | Shot `504-06-keyboard` |
| REQ-008 | WHEN the rail's filter holds text, a Browser row shall show when its title matches, with the matched characters highlighted. | Shot `504-07-filter` |
| REQ-009 | WHEN a Browser tab closes, from its row's close button or anywhere else, its row shall go. | Shot `504-08-closed` |
| REQ-010 | WHILE no Browser tab is open, the rail shall start no browser. | The run log: no browser unit before the first Browser tab |
| REQ-011 | WHILE a Browser tab is the displayed project's active item, its row shall be the rail's selected row. | Shot `504-05-shown` |

## Phase Plan
- **P1 Plan** — promote this pair, re-verify the seams against HEAD, and check where #510's nested
  worktree groups have got to, since both change `ProjectSnapshot`.
- **P2 Code** — the model, the rows, the hub's event and `try_global`, the favicon reader; the
  crate's test helpers and `rail_tests.rs` gain the new fields so the old tests keep building
  (§7: built, not run); fmt and clippy clean; a review of the diff (no hub created from the rail,
  no refresh per frame, gpui re-entrancy in the row's handlers).
- **P3 Test** — write and run the scenario and read every shot; rerun #500's and #501's scenarios,
  whose clicks in the rail the new rows could move (L-claude-498); `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, the prong's slice status and `docs/marley_architecture/
  marley_rail.md` and `marley_workbench.md` (§21), ledger capture (§19), close, archive, commit.
