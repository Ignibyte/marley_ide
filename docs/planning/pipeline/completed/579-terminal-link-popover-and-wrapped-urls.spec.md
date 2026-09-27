---
pipeline_id: 02c7294c-a58b-4086-8b0b-34d034ca1cd1
ticket: docs/planning/tickets/open/TICKET-579-terminal-link-popover-and-wrapped-urls.md
status: Phase 4 — Complete PASS
title: "A menu on a clicked terminal link, the default offered until chosen, and wrapped URLs joined"
type: feature
slice: prong 3 with prong 1, B6b; #503's slice 2
references: [docs/planning/pipeline/completed/503-terminal-urls-open-in-the-browser.spec.md, docs/planning/pipeline/completed/503-terminal-urls-open-in-the-browser.notes.md, docs/orca_architecture/03-browser-and-design-mode.md, docs/orca_architecture/05-terminal-and-workspace.md]
---

## Title
#503 routes a Ctrl+clicked terminal URL and offers a dev server's URL. This slice makes a plain
click on a URL useful, puts the same choices in the terminal's right-click menu, and joins a URL
that a program wrapped at the right edge or drew inside a box, which today opens cut in two.

## Scope
### In
- **The link menu on a plain click.**
  - When: a plain left click (one click, no modifier, no drag, no selection, the program not
    taking the mouse) on an http or https URL found in the text.
  - What opens at the pointer: Zed's `ContextMenu`, with the URL as a header and three entries:
    Open in Browser Tab, Open in System Browser and Copy Link.
  - A terminal over SSH (#503 D4) offers no Browser tab.
  - An OSC 8 link's plain click keeps opening it directly (#503 REQ-007): the program made it a
    link on purpose.
- **The right-click menu.** When the right-click lands on a link (a URL in the text, or an OSC 8
  link whose target is http or https), the terminal's menu starts with the same entries and a
  separator. Copy Link copies an OSC 8 link's hidden target.
- **The default, offered until chosen.** While the user's settings do not name
  `marley.terminal_links`, both menus end with "Always Open Local Links in a Browser Tab" and
  "Always Open Local Links in the System Browser". Choosing one writes `marley.terminal_links`
  (`local_in_browser_tab` or `system_browser`) to the user's settings, and from then on neither
  entry shows.
- **Wrapped URLs joined.**
  - A program that wraps a URL itself at the right edge, with a newline where the terminal would
    have soft-wrapped: the URL runs on in the row below, and the row it leaves is filled to the
    last column.
  - A program that draws a URL inside a box frame (`│ ┃ ║ ╎ ╏ ┆ ┇ ┊ ┋ |`), each row between the
    same frame columns.
  - A Ctrl+click, the link menu and the right-click menu on any of its rows take the whole URL.
  - The hover underline covers every row of an edge-wrapped URL, and the clicked row of a framed
    one.
- **The e2e scenario** `script/e2e/579-terminal-link-menus-and-wrapped-urls.sh`.

### Out (explicitly deferred)
- A setting that turns the plain-click menu off or makes a plain click open the URL (Orca's
  `terminalLinkClickBehavior`). Escape or a click elsewhere dismisses the menu, and the terminal
  keeps its click-to-select everywhere else.
- Middle-click on a link (Orca opens it).
- The link menu for `file://`, `mailto:` and other schemes: they keep Zed's Ctrl+click.
- A framed URL's hover drawn on every row. It needs a hover of several ranges in
  `terminal_element.rs`, where Zed's hover is one range.
- URLs wrapped by a program inside a tmux or screen pane, where the pane's own borders are the
  frame: covered when they look like a frame and otherwise not.

## Reference (§20)
Warp's published docs (docs.warp.dev/terminal/more-features/files-and-links, read 2026-09-25, as
cited by #503): a plain click on a link shows an "Open Link" tooltip, and a right-click copies
the URL. Orca (report 03 §2.3 and item 5; the files, MIT, read at `1c2cf120e3`):
- a plain click opens `LinkActionPopover` at the pointer, with the destination, Orca Browser or
  System Browser, and "Copy link", which for an OSC 8 link copies the hidden target;
- a one-time dialog asks which browser is the default, and its answer goes to the settings;
- `edge-wrapped-terminal-http-links.ts` and `hard-wrapped-terminal-http-links.ts` join a URL a
  program wrapped at the edge or inside a frame.

Marley matches the plain-click choices and the right-click copy. The one-time question becomes
two entries of the same menus, shown until one is chosen. The joining follows Orca's rules: an
edge-filled row, no new scheme, no label row, and frame columns kept. Upstream Zed's terminal
(`crates/terminal`, `crates/terminal_view`) keeps finding links, joins soft-wrapped rows, draws
the hover and owns the menu. Marley adds the menu's entries and the joining of rows Zed treats as
separate.

### Prior art
- **Behavior maps and reports.** Report 03 §2.3 (link routing and the popover) and report 05
  §2.4. Orca's files, read at `1c2cf120e3`:
  - `terminal-link-activation.ts:21-30`: a plain left click with no modifier;
  - `terminal-link-pointer-gesture.ts:4, 58-59`: no popover after 4 px of movement or with a
    selection;
  - `LinkActionPopover.tsx:56-57, 78-192`: the destination header, "Copy link" kept open, the
    two action rows, dismissal;
  - `link-routing-preference-dialog.tsx:125-244` and `use-terminal-pane-startup-actions.ts:173-201`:
    the one-time dialog, whose answer and a "prompted" flag go to the settings;
  - `edge-wrapped-terminal-http-links.ts:8-19, 63-182`: the start row's scheme with no word
    character before it, a row that reaches the last column, no next-row scheme, no label row
    (`^[^\s:][^:]*:\s`, `^HTTP/\d`), rows with a frame character left to the framed rule, a
    2048-character limit;
  - `hard-wrapped-terminal-http-links.ts:10-15, 32-135`: frame characters, the same prefix before
    the scheme, a frame at the same column on each row, a fragment that ends in `[/?&=#%+:-]` or
    fills 80% of the width, three rows at least;
  - their tests, as the case list for this ticket's scenario.
- **Published material.** Warp's files-and-links page; OSC 8 (`ESC ] 8 ; ; URI ST`).
- **The code we already ship.**
  - `crates/terminal/src/alacritty/hyperlinks.rs`: `find_from_grid_point` (91-151) with the OSC 8
    branch (98-123) and the regex branch between `line_search_left` and `line_search_right`,
    which follow `WRAPLINE` only (`vendor/alacritty_terminal/src/term/search.rs:592-616`);
    `sanitize_url_punctuation` (237-286); `HyperlinkMatch { text, is_url, range }` (33-38).
  - `crates/terminal/src/terminal.rs`: `find_hyperlink_at_point` (private, 2153-2161),
    `mouse_down` (2899-2978, a plain click starts a zero-width selection), `mouse_drag` (a 2 px
    threshold), `mouse_up` (2980-3060, with #503's OSC 8 hunk), `mouse_mode`,
    `last_content.selection_text`.
  - `crates/terminal_view/src/terminal_element.rs`: the left `on_mouse_up` listener (1037-1047);
    the hover drawn only with the platform modifier held, one `Range` over cells in reading
    order (937-947, 1400-1417), so a range that spans rows underlines each of them.
  - `crates/terminal_view/src/terminal_view.rs`: `deploy_context_menu` (578-650), which has no
    link entries; the right-click handler (1491-1513); `MarleyTerminalUrl` (147-154).
  - `crates/ui`'s `ContextMenu` (`header`, `entry`, `separator`, dismissal on Escape and outside
    clicks), and #503's `links.rs` (`destination`, `open`, `offer_button`'s three entries).
  - `settings::update_settings_file`, as the Settings window writes a value.
  - Does a crate we build own the seam? Zed owns the link finding, the hover and the menu.
    Nothing joins rows without `WRAPLINE`, so that is new: pure code in `marley_terminal`, called
    from `hyperlinks.rs`.

## UI proof
UI-AFFECTING: a menu on a click, new right-click entries, and a Ctrl+click opening a whole URL.
`script/e2e/579-terminal-link-menus-and-wrapped-urls.sh` (`compositor sway`, offline Chromium, a
fake `xdg-open` first on every PATH, the clipboard read with `wl-paste`). Shots:
`579-01-link-menu`, `579-02-menu-tab`, `579-03-menu-system`, `579-04-right-click-osc8`,
`579-05-default-chosen`, `579-06-edge-wrapped`, `579-07-framed`. The run log holds the
`xdg-open` calls, the clipboard reads, `mcp_agent tabs` and the settings file's
`terminal_links`. #503's scenario runs unchanged in the golden set.

## Locked-In Decisions
- D1 — The menus are Zed's `ContextMenu`, deployed by the terminal view where its right-click menu
  is, and filled by one Marley hook (`MarleyTerminalLinkMenu`), so the menu's look, keys and
  dismissal are Zed's and the entries are Marley's.
- D2 — A plain click opens the link menu only for an http or https URL found in the text, on one
  click with no modifier, no selection and no mouse mode. An OSC 8 link's plain click opens it,
  as #503 made it, and the right-click menu offers its copy.
- D3 — The default is offered as two menu entries, not a dialog, shown while the user's settings
  file does not name `marley.terminal_links`. A dialog on the first click would stop the click it
  came from. Choosing writes the setting, which also ends the offer, so no second flag is kept.
- D4 — Rows join only by the rules Orca's tests pin:
  - an edge-wrapped row fills to the last column;
  - the next row starts with URL characters, no new scheme and no `label:` form;
  - framed rows keep their frame columns and the prefix before the scheme;
  - at most 2048 characters.
  The joining is a pure function over row text in `marley_terminal`, and Zed's `hyperlinks.rs`
  calls it when a row-local match reaches the edge or its row has a frame.
- D5 — The menus route through #503's `destination` and `open`: Open in Browser Tab and Open in
  System Browser pick the place explicitly, over SSH the Browser tab is not offered, and a local
  URL is normalized (`0.0.0.0` to `127.0.0.1`) either way.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user plainly clicks an http or https URL in a terminal's text, the terminal shall show a menu at the pointer with the URL, Open in Browser Tab, Open in System Browser and Copy Link. | Shot `579-01-link-menu` |
| REQ-002 | WHEN the user chooses Open in Browser Tab, Marley shall open the URL in a Browser tab of the terminal's project. | Shot `579-02-menu-tab`; `mcp_agent tabs` |
| REQ-003 | WHEN the user chooses Open in System Browser, Marley shall hand the URL to the system browser. | Shot `579-03-menu-system`; the `xdg-open` log |
| REQ-004 | WHEN the user chooses Copy Link, the URL shall be on the clipboard; for an OSC 8 link, its target. | The run log's `wl-paste` reads |
| REQ-005 | WHEN the user right-clicks a link, the terminal's menu shall start with Open in Browser Tab, Open in System Browser and Copy Link. | Shot `579-04-right-click-osc8` |
| REQ-006 | WHILE the user's settings do not name `marley.terminal_links`, the link menus shall offer the two Always entries, and choosing one shall write it and end the offer. | The settings file after the choice; shot `579-05-default-chosen` |
| REQ-007 | WHEN the user Ctrl+clicks any row of a URL a program wrapped at the right edge, Marley shall open the whole URL. | Shot `579-06-edge-wrapped`; the tab's URL |
| REQ-008 | WHEN the user Ctrl+clicks any row of a URL drawn inside a box frame, Marley shall open the whole URL. | Shot `579-07-framed`; the tab's URL |
| REQ-009 | WHEN the user plainly clicks an OSC 8 link or text that is no link, the terminal shall behave as before: the link opens, the text gets no menu. | The run log; #503's scenario in the golden set |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code**
  - The rows in `docs/marley/zed-touchpoints.md` first.
  - Zed side: `terminal.rs` (a public link at a point), `hyperlinks.rs` (the join call),
    `terminal_view.rs` (the hook, the plain-click menu, the right-click entries) and
    `terminal_element.rs` (the plain-click listener).
  - Marley side: `marley_terminal::links` (the joining) and `marley_workbench::links` (the
    entries, the default).
  - fmt and clippy clean, and a review of the diff (re-entrancy on the menus, the drag and
    selection guards).
- **P3 Test** — write and run the scenario and read every shot; #503's scenario in the golden
  set; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, the plan's B6b, the crate notes, the touchpoint rows, the ledger,
  close, archive, commit.
