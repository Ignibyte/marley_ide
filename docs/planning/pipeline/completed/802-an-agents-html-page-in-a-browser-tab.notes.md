# An agent's HTML page in a Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-802-an-agents-html-page-in-a-browser-tab.md
- **Pipeline spec:** 802-an-agents-html-page-in-a-browser-tab.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "So the options can be inline editing for agents to show
  components or static html in a scratch file or existing project for that sort of style."
- **Discovery (drafted by /spec; re-verified at promotion):** `browser_open_url`
  (`crates/marley_workbench/src/browser_tools.rs`, `open_url`; registry description names #586's
  local HTML pages); the Browser tab's CDP session in `crates/marley_browser`;
  `paths::data_dir()`; `theme::ActiveTheme`.
- **Recall (§18.3):** #586's local page (`address::local_page`, the `Fs::metadata` file check,
  `open_url_tab` reusing a tab on the same URL through `show_tab_where`); L-claude-586 (a path or a
  `file:` URL, both read); #801's `Fs::watch` loop and its `smol::unblock` lesson (gate:21's
  `async_block_without_await`); `Fs::watch` on a file loses a file replaced by rename (`sed -i`, an
  editor's atomic save), so the watch is on the folder, filtered to the file.
- **Promoted 2026-10-17**; seams re-read on the tree #801 left: `browser_tools::run` answers
  `browser_open_url` before the browser settles, `caller_scope` gives the caller's workspace,
  `BrowserHub::reload(target)`, `BrowserView::target()`, `live_views`, `theme::GlobalTheme`
  (observed by `markdown` for theme changes), `ThemeSettings::ui_font`/`buffer_font`.

### Design
- **`marley_mcp/src/registry.rs` (Marley):** `browser_show_page`, a browser write (idempotent:
  a second call with the same page leaves what the first did), its description saying when to
  choose it over `component_show`; schemas: `html` + `name`, or `path`, optional `theme`; the
  answer `opened`, `project`, `tab`, `path`, `stylesheet`, `reason`.
- **`marley_workbench/src/browser_pages.rs` (Marley, new: the page writer is a component of its
  own):** `show_page` (the caller's scope; with `html`, the name checked, the stylesheet link put
  in the `<head>` unless `theme: false`, the file written to `pages_dir_in(data_dir, key)`; with
  `path`, inside one of the caller's workspace's folders after `canonicalize`, `.html`/`.htm`, a
  file); opens it as `open_in_tab` does and waits up to 5 seconds for the tab's page id; a watch
  per page file (its folder's `Fs::watch`, events for the file only) that reloads every tab on the
  file's URL; `stylesheet_text(cx)` from `cx.theme()` and
  `ThemeSettings`, written when its text changes, at init and on each `GlobalTheme` change, after
  which every watched page's tabs reload.
- **`marley_workbench/src/browser.rs` (Marley):** `reload_tabs_on(url, cx)`: reload each live tab
  whose address is `url`.
- **`browser_tools.rs` (Marley):** route `browser_show_page` before the browser settles, as
  `browser_open_url`; `open_in_tab` gives the view it opened.

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_mcp/src/registry.rs` | Marley | the spec, the schemas, `IDEMPOTENT` |
| `crates/marley_workbench/src/browser_pages.rs` | Marley | new: writer, watch, stylesheet |
| `crates/marley_workbench/src/browser.rs` | Marley | `reload_tabs_on` |
| `crates/marley_workbench/src/browser_tools.rs` | Marley | the route; `open_in_tab` shared |
| `crates/marley_workbench/src/marley_workbench.rs` | Marley | the module, its `init` |
| `docs/marley/guide.md`, `CHANGELOG.md` | docs | at Complete |

### Visual check plan
| REQ | Set-up and action | Shot |
|---|---|---|
| REQ-001, REQ-003 | stand-in in the repo's terminal calls `html` + `name: status` (body on `var(--marley-surface)`, `var(--marley-text)`) | `802-01-scratch` |
| REQ-002 | the same `name`, changed text | `802-02-reload` (one tab, new text) |
| REQ-002, REQ-004 | `path` to `repo/report.html`, then `sed -i` on it | `802-03-path` |
| REQ-003 | `theme selector: toggle` to One Light | `802-04-theme` |
| REQ-005 | `path` to a page outside the repo, and to `README.md` | `802-05-outside` |

### Risks and decisions
- A `file:` page loading a `file:` stylesheet: Chromium allows it (subresource, not fetch).
- The tab id needs the page attached; the answer leaves `tab` out after 5 seconds rather than
  failing the call, since the tab is open.

## Phase 2 — Code
- **Built:** as the design. `open_in_tab`'s window and tab half became `open_in_workspace`,
  shared by `browser_open_url` and the new tool; `open_answer` and `declined` are shared too.
- **Review of the diff:** the page is written before the tab opens and the stylesheet before
  either; `path` is checked after `canonicalize`, so a link out of the project is refused; no
  file is written for `path`; nothing reads an entity while it is updated (`reload_tabs_on`
  collects the targets, then updates the hub once). One race found and fixed: a watch that ended
  when a change found no tab could end just as a new call opened a tab on the file, leaving that
  tab unwatched, so each watch now lasts the session (a reload of no tabs does nothing).
- **Deviation:** the stylesheet sits in `pages/`, beside the project folders, not at the data
  directory's top (the spec now says so).
- **Clippy:** `TaskExt` for `detach_and_log_err`; two `&mut AsyncApp` made `&AsyncApp`
  (`needless_pass_by_ref_mut`). **Gate run 1 (`802-gate-1.log`):** every gate PASS; the receipt
  failed because the scenario was written during the run.

## Phase 3 — Test
- **Scenario:** `script/e2e/802-an-agents-html-page-in-a-browser-tab.sh` (`compositor sway`, the
  browser fixture's offline Chromium): a stand-in agent calls `browser_show_page` through the
  plugin's bridge from the repository's folder, and at the end from the project's terminal.
- **Run 1:** every check passed until the last step, whose click on the rail's terminal row
  (y 136) hit the group header; the row is at y 172. Shot 01's stylesheet named the fonts
  `.ZedSans` and `.ZedMono`, Zed's own aliases, which Chromium cannot find: a real bug, fixed by
  `gpui::font_name_with_fallbacks` (IBM Plex Sans, Lilex), with a check added.
- **Run 2: every check passed.** The shots:
  - `802-01-scratch` (REQ-001, REQ-003): a Browser tab "Build status" of `repo`, its address
    `file:///…/pages/<key>/status.html`; the card on the theme's surface with its border, the
    heading in the accent blue, "3 passing" green, "1 failing" red, "1 skipped" amber, the
    command in the monospace font and muted color, all on One Dark's editor background.
  - `802-02-reload` (REQ-002): the same tab, now "Run 2: 4 passing, 0 failing"; `browser_tabs`
    lists one tab.
  - `802-04-theme` (REQ-003): after `theme selector: toggle` to One Light, the same page in light
    colors (light background and card, darker green, red and amber) without another call; the
    stylesheet says `color-scheme: light`.
  - `802-03-path` (REQ-002, REQ-004): a second tab "Report" on `repo/report.html`, showing "The
    second draft, written by sed." after `sed -i` replaced the file; the folder's listing is the
    same before and after.
  - `802-05-outside` (REQ-005): the terminal: `show_page path …/outside.html` answers
    `{"opened": false, "reason": "the file is outside your project"}` and `show_page path
    README.md` answers `"not an .html or .htm file"`; still two Browser tabs.
- **Gate run 2 (`802-gate-2.log`)**, on the tree after the font fix and the scenario: **GATE GREEN [diff]**.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` ("An agent's HTML page in a Browser tab"); `docs/marley/guide.md`
  ("An agent's HTML page (#802)", and the tool's row in the browser table);
  `docs/marley_architecture/marley_workbench.md` (the module, a known limit) and `marley_mcp.md`.
- **Knowledge:** F-claude-802-a-stylesheet-named-zeds-font-aliases-001,
  F-claude-802-a-file-watch-that-ended-raced-a-new-tab-001,
  L-claude-802-zed-font-names-are-aliases-001, L-claude-802-watch-the-folder-to-follow-one-file-001.
- **Brain:** no decision recorded: the spec's decisions D1 to D3 stand as written.
