# A menu on a clicked terminal link, the default offered until chosen, and wrapped URLs joined — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-579-terminal-link-popover-and-wrapped-urls.md
- **Pipeline spec:** 579-terminal-link-popover-and-wrapped-urls.spec.md

## Phase 1 — Plan
- **Request:** #503's slice 2, minted at #503's Complete (2026-09-26), from #503's Out: the
  popover on a plain-clicked link, asking once which place is the default, and joining URLs a
  program wrapped at the edge or drew in a box. Chad decided on 2026-09-26 that every remaining
  Orca and Warp finding gets built.
- **Classification / tier:** feature, M. Four Zed files get small hunks: `terminal.rs`,
  `hyperlinks.rs`, `terminal_view.rs` and `terminal_element.rs`. The logic lives in
  `marley_terminal` (pure) and `marley_workbench`.
- **Recall (§18.3):**
  - AD-claude-503: Marley decides where a URL goes, and Zed finds it. `destination`, `open` and
    the offer's three entries are reused here.
  - PR-claude-defer-a-pane-change-out-of-an-items-own-event-001: a menu entry runs inside the
    menu's update, and a Browser tab goes through `open`, which defers it.
  - L-claude-503-gpui-falls-back-to-the-desktop-portal-when-every-open-command-fails-001 and
    L-claude-561-pythons-webbrowser-tries-every-browser-it-knows-after-a-failed-one-001: the
    scenario's fake `xdg-open` goes first on every PATH and exits 0.
  - #503's notes, the split: the popover needs a link-at-a-point and a plain-click path. The
    stitching is about 300 lines in Orca, and Zed's hover is one range.
  - Brain consultation 3817649c63184b7abefc0edf0c321384: nothing on this seam.
- **Discovery:** an Explore agent read Orca's files and Zed's link paths; the spec's prior art
  lists the lines. Four facts the design rests on:
  - soft-wrapped rows already join, and a hover range spanning rows already underlines each row;
  - a plain click starts a zero-width selection and opens only OSC 8 links (#503);
  - the right-click handler selects the word under the pointer, then `deploy_context_menu`
    builds the menu and keeps it in `context_menu`;
  - the left mouse-up listener in `terminal_element.rs` runs `Terminal::mouse_up` in the
    terminal's update, and the element holds the view.
- **Decisions:** D1 to D5 in the spec.

### Design
- **Approach.**
  1. *The link at a point (Zed, `terminal.rs`).* `pub fn marley_link_at(&mut self, position:
     Point<Pixels>) -> Option<MarleyLink>` takes the position as the mouse handlers do (less the
     bounds' origin, as a grid point) and runs `find_hyperlink_at_point`. `MarleyLink { target,
     osc8 }`: the URL (an OSC 8 link's hidden target), and whether it came from OSC 8. It gives
     only URLs (`is_url`), never paths.
  2. *The join (Marley, `marley_terminal::links`).* `joined_url(rows, row, column) ->
     Option<JoinedUrl>`. The rows are built from the grid by the caller: `MarleyRow { text: Vec<char>,
     columns: Vec<usize>, last_column, wrapped }`, spacer cells skipped. It returns `{ url,
     start: (row, column), end: (row, column), framed }` when a URL running over two rows or more
     holds the point.
     - *Edge rule.* Scan up from the clicked row while the row above fills to the last column,
       is not `wrapped` (Zed joins those), holds no frame character and does not start a new
       scheme. The start is the nearest `http://` or `https://` with no word character before
       it. Join rows forward while each one fills to the last column and the next starts with
       no scheme, no space, no `label:` form and no `HTTP/`.
     - *Frame rule.* The clicked row has a frame character before the URL's column and one
       after it. The rows above and below join when they have frame characters at the same two
       columns, the same text before the URL's column, and a fragment that ends in
       `[/?&=#%+:-]` or fills 80% of the width. The fragment is the row's text between the
       URL's column and the right frame, trimmed.
     - The joined text is cut where a URL ends: at a space, a quote, `<`, `>`, a backtick or
       a bracket not opened. It is at most 2048 characters and must parse as http or https.
  3. *The call (Zed, `hyperlinks.rs`).* In `find_from_grid_point`'s regex branch, after the
     match: when the match reaches the row's last column, or the clicked row holds a frame
     character, build up to 40 rows around the point, call `joined_url`, and prefer its URL
     when it holds the point and is longer. The range is `start..=end` for an edge join, and
     the clicked row's part for a framed one.
  4. *The hook (Zed, `terminal_view.rs`).* `pub struct MarleyTerminalLinkMenu(pub Arc<dyn
     Fn(&MarleyFooterContext, &MarleyLink, ContextMenu, &mut Window, &mut App) -> ContextMenu>)`
     and its `Global`.
     - `deploy_context_menu` asks `marley_link_at(position)` first, and in its build closure
       passes the menu through the hook before Zed's entries when there is a link.
     - A new `pub fn marley_deploy_link_menu(position, window, cx)` builds a menu of the hook's
       entries alone, for a plain click on a link that is not OSC 8. It is kept in
       `context_menu` as Zed's menu is, with the same dismissal subscription.
  5. *The plain click (Zed, `terminal_element.rs`).* A second left `on_mouse_up` listener runs
     when the click count is 1, no modifier is held, the terminal is not in mouse mode and
     `last_content.selection_text` is empty. It updates the view (not the terminal) with
     `marley_deploy_link_menu`, deferred so it runs after the terminal's own `mouse_up`.
  6. *The entries (Marley, `links.rs`).* `link_menu(context, link, menu, window, cx)`:
     - a header of the URL, shortened;
     - Open in Browser Tab (`Destination::BrowserTab`, the URL normalized), hidden over SSH;
     - Open in System Browser;
     - Copy Link (`cx.write_to_clipboard` of the target);
     - while `marley.terminal_links` is not in the user's settings file (the raw user
       content), a separator and the two Always entries, which write it with
       `update_settings_file`.
     Only http and https links get entries; any other link leaves the menu as Zed's.
- **File manifest.**
  - Zed crates: `crates/terminal/src/terminal.rs` (`MarleyLink`, `marley_link_at`),
    `crates/terminal/src/alacritty/hyperlinks.rs` (the join call and the row builder),
    `crates/terminal_view/src/terminal_view.rs` (the hook, `deploy_context_menu`'s call,
    `marley_deploy_link_menu`) and `crates/terminal_view/src/terminal_element.rs` (the
    plain-click listener).
  - Marley crates: `crates/marley_terminal/src/links.rs` (new: `MarleyRow`, `joined_url`),
    `crates/marley_terminal/src/marley_terminal.rs` (the module) and
    `crates/marley_workbench/src/links.rs` (`link_menu` and the hook set at `init`).
  - The scenario `script/e2e/579-terminal-link-menus-and-wrapped-urls.sh`.
- **Ledger rows (before the code):** `terminal.rs`, `terminal_view.rs` (existing rows
  widen), `terminal_element.rs` and `hyperlinks.rs` (rows to find or add).

### E2E plan
Setup:
- offline Chromium and a served site (`index.html` "Link page", `osc8.html`);
- fakes `xdg-open`, `gio`, `google-chrome` and `firefox` first on every PATH;
- a `.bashrc` that sets `BROWSER` to the fake unless it is Marley's opener;
- fakes on the terminal's PATH: `urls` (a local and a non-local URL, then a wait), `osc8`,
  `edge` (with `tput cols`, a URL longer than the width printed with a newline exactly at the
  edge, then a wait) and `framed` (a box three rows high holding one URL, at the width
  `tput cols` gives, then a wait).

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `urls`; a plain click on the local URL | `579-01-link-menu`: the menu at the pointer with the URL and the entries |
| REQ-002 | Open in Browser Tab | `579-02-menu-tab`; `tabs` holds the Link page in `repo` |
| REQ-003 | back to the terminal; a plain click on the non-local URL; Open in System Browser | `579-03-menu-system`; the `xdg-open` log |
| REQ-004 | a plain click; Copy Link; `wl-paste` | the log: the clipboard is the URL |
| REQ-005, REQ-004 | `osc8`; a right-click on the label; Copy Link | `579-04-right-click-osc8`: the link entries first; the clipboard is the target |
| REQ-006 | a plain click on the local URL; Always Open Local Links in the System Browser | the settings file names `system_browser`; `579-05-default-chosen`: the next menu has no Always entries |
| REQ-007 | `edge`; Ctrl+click on the second row | `579-06-edge-wrapped`; the `xdg-open` log (now `system_browser`) holds the whole URL |
| REQ-008 | `framed`; Ctrl+click on the second row of the box | `579-07-framed`; the log holds the whole URL |
| REQ-009 | a plain click on plain text; #503's scenario | no menu in the next shot; the golden set |

Not reached by a scenario: an SSH terminal's menu without the Browser tab entry (the review
checks `over_ssh`), and a TUI in mouse mode (the review checks `mouse_mode`).

### Risks
- A plain click on a URL opened a menu where it used to only place the selection. Escape and
  outside clicks dismiss it, and a drag still selects. This is Orca's default, and the Out lists
  a setting.
- The deferred listener must run after `Terminal::mouse_up`. Otherwise a click that ends a
  drag could see the selection before the terminal clears it. The guard reads the synced
  `selection_text`, which a drag fills before the release.
- The edge rule could join a URL with an unrelated next row that happens to start with URL
  characters. The label and scheme guards follow Orca's tests, and the row above must fill to
  the last column.
- `hyperlinks.rs` is upstream's link finder, so the hunk stays one call after the match, with
  the row builder beside it.

## Phase 2 — Code
- **Built:**
  - `marley_terminal::links` (new, pure):
    - `LinkRow` (`from_cells`: a wide character's spacer skipped, trailing blanks trimmed) and
      `JoinedUrl`;
    - `joined_url(rows, row, column, last_column)`, which takes the frame rule when the clicked
      row has a frame character and the edge rule otherwise;
    - the edge rule tries the clicked row's URLs before the point, nearest first, then the last
      URL of each row above that runs on into the next. A row the terminal wrapped always runs
      on; any other runs on when it fills to the last column (or one short of a wide character)
      and the next row holds no frame and does not start afresh (a space, a new scheme, a
      `label:` form, `HTTP/2:`);
    - the frame rule has the same prefix, a frame at the same right column, parts with no
      spaces, and each part ending in `/ ? & = # % + : -` or filling 80% of the width;
    - URL characters as Zed's `URL_REGEX` takes them, trailing punctuation and unopened
      brackets trimmed, at most 2048 characters.
  - `hyperlinks.rs`: `marley_joined_url(term, point)` builds 20 rows either side into `LinkRow`s
    and turns the result back into a `Match`. The regex branch keeps it when it is longer than
    the row's own match.
  - `terminal.rs`: `MarleyLink { target, osc8 }` and `marley_link_at(position)`, which gives
    http and https links only.
  - `terminal_view.rs`:
    - the global `MarleyTerminalLinkMenu`;
    - `deploy_context_menu` passes the menu through it, then a separator, when the right-click
      is on a link;
    - `marley_deploy_link_menu`, the entries alone for a link that is not OSC 8, with the
      dismissal subscription `deploy_context_menu` uses.
  - `terminal_element.rs`: a second left mouse-up listener. When the click count is 1, no
    modifier is held, there is no mouse mode and no selected text, it defers the view's
    `marley_deploy_link_menu`.
  - `links.rs` (workbench): `link_menu`, which gives:
    - a header of the URL, shortened to 60 characters;
    - Open in Browser Tab (not over SSH, the URL normalized);
    - Open in System Browser;
    - Copy Link (the target);
    - while the user's settings file names no `terminal_links`, the two Always entries, which
      write it with `update_settings_file`.
    The hook is set at `init`.
- **Deviations from the plan, and why:**
  - The trailing separator is `deploy_context_menu`'s rather than the hook's, so the plain-click
    menu does not end in one.
  - The edge rule tries several starts, so a continuation row that also starts a URL of its own
    still finds the one from the row above.
  - The label test follows Orca's `^[^\s:][^:]*:\s`: the name may hold spaces
    (`Not Found: …`).
  - A framed join needs two rows, not Orca's three; the start part must fill or end on a
    continuation character.
- **Review:**
  - Re-entrancy:
    - the element's listener reads the terminal after the terminal's own listener has finished
      its update, and defers the menu;
    - `marley_deploy_link_menu` updates the terminal inside the view's update;
    - the hook reads the project, the terminal and the settings inside the menu's build;
    - the entries open tabs through `open`, which defers, and write settings asynchronously.
  - Guards: a double click selects as before, a drag leaves text selected, and mouse mode is
    left to the program. An OSC 8 plain click still goes through `mouse_up`'s #503 branch.
  - Panics: every index in the joiner is behind `get`, or behind a length the code has checked.
  - Checks: `cargo check` on each crate, fmt, and `cargo clippy -p marley_terminal -p terminal
    -p terminal_view -p marley_workbench --all-targets -- -D warnings`. Its findings are fixed:
    unused results in the trim, a long doc paragraph, a `const fn`, `then_some`, and `&App` in
    `choose_default`.

## Phase 3 — Test
- **The scenario:** `script/e2e/579-terminal-link-menus-and-wrapped-urls.sh` (`compositor
  sway`, offline Chromium, a served site).
  - Fakes named `xdg-open`, `gio`, `google-chrome` and `firefox` go first on every PATH and exit
    0; the last three log to `leak.log`.
  - Fakes on the terminal's PATH: `urls` (the site's URL, which listens, so #503's strip shows,
    and a URL that is not local), `osc8`, `edge` (a URL a row and a half long, cut with a newline
    at `tput cols`) and `framed` (a URL over three rows of a 44-column box).
  - Coordinates come from #503's measured geometry.
  - 9 `expect` checks: through `mcp_agent tabs`, the `xdg-open` log, `wl-paste` and the settings
    file.
- **Green, the debug build, first run:** all 9 pass. The edge-wrapped URL came out whole
  (`https://example.com/edge/a…a/end`), and so did the framed one
  (`https://example.com/framed/b…b/end`). `leak.log` stayed empty.
- **Red, the installed build (`bd2b1a0066`, before the change):** `check the menu opened a Browser
  tab of the project: FAIL`. With no menu, the Return went to the waiting program.
- **Shots, read:**
  - `579-01-link-menu`: a plain click on `local: http://127.0.0.1:<port>/` opens the menu at the
    pointer, upward near the bottom. It has the URL as its header, Open in Browser Tab, Open in
    System Browser and Copy Link, then a separator and the two Always entries (REQ-001; REQ-006's
    offer).
  - `579-02-menu-tab`: Open in Browser Tab opened a "Site page" tab of `repo` at the URL
    (REQ-002).
  - `579-03-menu-system`: the menu for `https://example.com/docs`. Open in System Browser sent it
    to the log (REQ-003).
  - The run log: Copy Link put `http://127.0.0.1:<port>/` on the clipboard (REQ-004). A plain
    click on `local` gave no menu: Down and Return reached the program, which printed `^[[B` and
    ended, and the log stayed as it was (REQ-009).
  - `579-04-right-click-osc8`: a right-click on "Open the OSC 8 page" opens Zed's menu. It starts
    with the OSC 8 target as its header, then the three link entries and the Always pair, and
    Zed's entries follow (New Terminal … Close Terminal Tab). Copy Link put the target
    `…/osc8.html` on the clipboard (REQ-005, REQ-004).
  - `579-05-default-chosen`: after "Always Open Local Links in the System Browser", the settings
    file holds `"terminal_links": "system_browser"`, and the next menu has only the header and
    the three entries (REQ-006).
  - `579-06-edge-wrapped`: the URL fills the first row and ends on the second. Ctrl+click on the
    second row sent the whole URL to the system browser, which the setting now picks (REQ-007).
  - `579-07-framed`: the box drawn with its URL over three rows. Ctrl+click on the middle row sent
    the whole URL (REQ-008).
- **Not reached by a scenario:** the SSH form of the menu without Open in Browser Tab (the
  review reads `over_ssh`), and a program in mouse mode (the listener's `mouse_mode` guard).
- **The golden set, with 579 added (`just regress`, the debug build):** all 24 pass. #503
  (OSC 8's plain click, Ctrl+clicks and the offer) and #549 (plain clicks on the rail) pass
  unchanged. 579 took 72 s.
- **The focus report:** each sway run stopped with its Marley, pointer and keyboard. The
  Hyprland check found no Marley window before or after the run, and nothing was added or
  reloaded.
- **The gate, first run:** `GATE RED`, gate:21 dylint.
  - Stable clippy had made `in_url` a `const fn`. Dylint's own nightly (`nightly-2026-03-21`) has
    no const `char::is_control`, so `marley_terminal` did not compile there
    (L-claude-579-clippys-const-fn-and-dylints-nightly-disagree-001).
  - Fixed at the source: the control characters are tested as the code ranges Zed's `URL_REGEX`
    names (`U+0000` to `U+001F`, `U+007F` to `U+009F`), which is the same set `is_control`
    tests.
  - Afterwards, dylint on the two Marley crates and stable clippy on `marley_terminal` are clean,
    and the scenario was rerun on a fresh build: all 9 checks pass.
- **The gate, second run:** `script/gates.sh --diff` gave `GATE GREEN [diff]`, 16 passed and 0
  failed. The receipt matches the tree.
- **Verdict:** PASS. REQ-001 to REQ-009 are shown by the shots and the checks.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md`, under Added: "A menu on a clicked terminal link, and wrapped URLs opened
    whole".
  - `docs/marley/three-prong-plan.md`: B6b (#579, shipped).
  - `docs/marley_architecture/marley_workbench.md`: the link menus.
  - `docs/marley_architecture/terminal_blocks.md`, the `marley_terminal` note: `links.rs`, and
    #561's `BROWSER` setting, which #561 left out.
  - The touchpoint rows were checked against what shipped: `terminal.rs` (`MarleyLink`,
    `marley_link_at`), `terminal_view.rs` (the hook, the call, `marley_deploy_link_menu`),
    `terminal_element.rs` (the listener), and the new `alacritty/hyperlinks.rs` row
    (`marley_joined_url` after the regex match).
- **Knowledge appended:**
  - `L-claude-579-a-program-waiting-on-read-proves-no-menu-opened-001`
  - `L-claude-579-clippys-const-fn-and-dylints-nightly-disagree-001`
  - `AD-claude-579-a-clicked-terminal-link-gets-zeds-menu-with-marleys-entries-001`
  - No F block: the red gate was a toolchain mismatch, caught before any commit and recorded as
    a lesson.
- **Brain:** consultation `3817649c63184b7abefc0edf0c321384` was closed by `brain decide`
  (`decisions/a-clicked-terminal-link-gets-zeds-menu-with-marleys-entries-and-wrapped-urls-are-joined`,
  follow-up 2026-10-26).
- **Closed:** TICKET-579 moved to `tickets/closed/`, and its BACKLOG row went at promotion.
