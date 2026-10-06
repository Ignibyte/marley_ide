# Favourites and bookmarks from Rusty — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-662-rusty-favourites.md
- **Pipeline spec:** 662-rusty-favourites.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** the Queue's top after #661, from Chad's 2026-10-06 "adding the last things missing".
- **Classification / tier:** feature, R4b; Marley crates only (and the Marley keymap).
- **Pre-flight:** green; no active pipeline; another session's cargo was running, so planning only.
- **Recall (§18.3):**
  - AD-644: the Brain view is its own entity; search on Enter only; every vault change a Rusty
    tool. AD-654: the picker matches in Marley and keeps its own recent list.
  - F-644: a `MultiWorkspace` call from inside the rail's update panics; defer.
  - The brain (consultation `aff0dccff92241e8a7dac6003cc431ab`): nothing on this seam.
- **Discovery** (an Explore map, 2026-10-06):
  - `rusty.rs`: `Vault` (189-196), `VaultReads` (205-214), `connected` re-reads (466), announcement
    (469-475), the switch clearing `Vault` (266-268). `project.rs:55-158`: `ProjectPages`, the model.
  - `brain.rs` (1315 lines): `Render` (1284-1315; the group goes between the search 1303 and the body
    1304); `Line` (134-140) and `step` (426-464) unchanged; `reveal` (338-347) plus
    `self.open.insert`; `search_for` (545-582) with the field's text set as `knowledge_panel.rs:529-534`
    does; `deploy_menu` (852-943, 92 lines: split per kind); `Edit` (143-148), `start_edit` (635-680),
    `commit_edit` (684-748); `open_page` (101-117), `clicked` (401-415).
  - `page.rs` (1701 lines): `render_header` (1030-1091; the star between the breadcrumb and Edit);
    `slug()` (359-362); `_subscriptions: [Subscription; 2]` (287, 322-333); `open_later` (172-189),
    `open` (191-226, an existing tab only brought forward); `scroll_to_heading_line` (916-928);
    the outline's `Heading { level, text, line }`; the tab's `actions!` (84-100).
  - `page_picker.rs`: `empty_order` call (326-343), `render_match` (434-484), one separator
    (298-300); `switcher.rs`: `Order` (94-102), `empty_order` (110-145).
  - The stand-in: `TOOL_NAMES` (39-78; `brain_follow_up` listed twice: drop one), `VAULT_TOOLS`
    (1106-1177), `stamp()` skipping dot folders (1264-1290), `rename` (654-684), `delete_page` and
    `delete_folder` (730-745).
  - Keys: `RustyPage` binds only `alt-left` and `alt-right`; Ctrl+D is free there in Read.

### Design
- **`marley_rusty/src/bookmarks.rs`** (Marley): `BOOKMARK_LIST`, `BOOKMARK_ADD`, `BOOKMARK_REMOVE`,
  `BOOKMARK_SET`; `BookmarkKind { File, Folder, Search, Heading, Other(String) }`; `Bookmark { kind,
  title, path, query, heading }` (`Deserialize`, defaults), `kind()`, `key()`, `is_page(slug)`,
  `page(slug)`, `with_title`; `bookmarks_from_answer`; `BookmarkWrite { Add, Remove, Set }` with
  `tool()` and `arguments()` (the fields Rusty takes, empty ones left out).
- **The stand-in**: the four tools over `vault/.rusty/bookmarks.json` with Rusty's rules; `rename`
  and the deletes carry and drop bookmarks; `stamp()` watches the file.
- **`marley_workbench/src/rusty/favourites.rs`** (new, Marley): the `Bookmarks` global (read on
  need, on `Announced`, on connect, after writes; cleared when off); `pub(super) fn write`;
  `ToggleBookmark` handling; `render_group(view, cx)` for the Brain view (header and rows, the kind
  icons `FileMarkdown`, `Folder`, `MagnifyingGlass`, `Hash`); the group's menu (Rename…, Remove).
- **`brain.rs`**: draws the group between the search and the body; `deploy_menu` split per kind
  under the cap; `Edit::BookmarkTitle(key)` through the existing inline edit; `open_folder(path)`,
  `run_search(query)` for the group's clicks; re-read bookmarks after its renames and deletes on the
  service connection.
- **`page.rs`**: the star button (`Star`/`StarFilled`), a third subscription (the `Bookmarks`
  global), `rusty::ToggleBookmark` in its `actions!` and on the workspace for the palette; an
  `open_at_heading_later` that opens or brings forward the tab and scrolls to the heading matched
  in the outline.
- **`page_picker.rs` / `switcher.rs`**: `empty_order` takes the favourite slugs; separators after
  the favourites and after the recent; a star on favourite rows.
- **`keymap.json`**: `"ctrl-d": "rusty::ToggleBookmark"` in `RustyPage`.
- **`guide/index.html`**: the Brain view article names Favourites and the star.
- **File manifest:** `crates/marley_rusty/src/{bookmarks.rs, marley_rusty.rs}`, the stand-in,
  `crates/marley_workbench/src/rusty/{favourites.rs, brain.rs, page.rs, page_picker.rs}`,
  `crates/marley_rusty/src/switcher.rs`, `crates/marley_workbench/src/rusty.rs`,
  `crates/marley_workbench/keymap.json`, the guide page (all Marley);
  `script/e2e/662-rusty-favourites.sh` (Test). No Zed crate.

### Visual check plan
One scenario, `compositor sway`, as the spec's UI proof; positions from #644's and #656's
scenarios and the first run's shots.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | The Brain view | `662-01-favourites` |
| REQ-003 | Open alpha; click its star | `662-02-starred` |
| REQ-005 | Ctrl+Alt+U | `662-03-picker` |
| REQ-006 | Click the folder favourite | `662-04-folder` |
| REQ-007 | Click the search favourite | `662-05-search` |
| REQ-008 | Click the heading favourite | `662-06-heading` |
| REQ-009 | Right-click alpha's favourite, Rename…, type, Enter | `662-07-renamed` |
| REQ-003, 004 | Ctrl+D in alpha's tab | `662-08-ctrl-d` |
| REQ-011 | Rewrite `bookmarks.json` from outside | `662-09-outside` |
| REQ-010 | Right-click the search favourite, Remove | `662-10-removed` |
| REQ-002, 012 | Not shot | Review |

### Risks
- **Two highlights**: a folder favourite and its tree row may both show selected; the group keeps
  no selection of its own.
- **A heading that renders differently** (a wikilink in it): matched against the outline's text,
  Rusty's way; no match opens the page at the top.
- **The service connection** announces nothing: the group re-reads after Marley's own renames and
  deletes, and when the Brain view takes the focus.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; the Explore map.
- [x] Mint the pair; the BACKLOG row removed; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request.

## Phase 2 — Code (2026-10-06)
### Built
- **`marley_rusty/src/bookmarks.rs`**: the four tool names, `BookmarkKind`, `Bookmark` (`page`,
  `kind`, `is_page`, `key`, `shown_title`, `arguments` with empty fields left out),
  `bookmarks_from_answer`, `retitled`, and `BookmarkWrite { Add, Remove, Set }` with `tool` and
  `arguments`.
- **The stand-in**: `bookmark_list`, `bookmark_add`, `bookmark_remove` (refuses an unknown key with
  `No bookmark …`) and `bookmark_set` over `vault/.rusty/bookmarks.json`, deduped by key; `rename`
  carries bookmarks to the new path, the deletes drop them, and the watcher's stamp covers the
  file so a change from outside is announced. The duplicate `brain_follow_up` in its tool list
  is gone.
- **`rusty/favourites.rs`**: the `Bookmarks` global and its reads (on first need, on Rusty's
  announcement, when the connection comes up, after every write; one more read queued behind a
  running one; cleared while Rusty is off), `write` (a refusal toasts its first line and
  re-reads), `toggle_page` and `retitle`.
- **`brain.rs`**: the Favourites group between the search and the tree (folder, magnifying glass,
  hash and page icons; the path, query or heading in the tooltip); a click opens a page as a row
  does, opens and selects a folder (clearing a search first), runs a search through the field, or
  opens a page at a heading; the right-click menu has Rename… (the existing inline edit, as
  `Edit::BookmarkTitle`) and Remove. The write helper re-reads the list after Marley's own
  renames, moves and deletes, which the service connection does not announce.
- **`page.rs`**: the star between the page's name and Edit (`Star`, `StarFilled`), its tooltip
  naming `rusty: toggle bookmark`; the action on the tab; `open_at_heading_later`, and `open`
  taking a `Visit` so an open tab goes to the heading too.
- **`switcher.rs` and `page_picker.rs`**: `empty_order` takes the favourite slugs and lists them
  after the page in front and before the recent pages, with separators after each group; a star
  in the end slot of favourite rows.
- **`keymap.json`**: `ctrl-d` in `RustyPage`. **The guide**: the Brain view article names
  Favourites and the star.

### Deviations
- `with_title` became `retitled(list, key, title)`: a rename sends the whole list, so the helper
  works on the list.
- The group is drawn in `brain.rs`, beside the tree it sits over; `favourites.rs` keeps the data.
- `rusty: toggle bookmark` acts on the focused Page tab only, not the workspace: with no page in
  front there is nothing to star. The palette lists it while a Page tab has the focus.
- A heading visit puts the heading at the top through the outline's line for every heading visit,
  not only the favourite's (`outline_line`, `scroll_to_heading_line`); before the outline is
  parsed it falls back to the old scroll.
- No re-read when the Brain view takes the focus: the re-read after each of Marley's writes
  covers the service connection's silence.

### Review
- Clippy: `iter_on_single_items` (`std::iter::once`), more than three bools (`Reading` replaced
  two flags), `needless_pass_by_value` (`write` takes `&BookmarkWrite`), a needless `&mut App`
  (`write` and the toggles take `&App`), `match` to `if let` in `go_to_heading`.
- No entity is read while it updates: the group's clicks run inside the view's listener and call
  out through `favourites`, which touches globals only.

### Gate
`just gate-diff`: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-06)
Scenario `script/e2e/662-rusty-favourites.sh` under `compositor sway`: the stand-in over a scratch
vault (`home`, `notes/alpha`, `notes/beta` with First, Second and Third, `projects/atlas` and
`ideas/seed` tagged `idea`, `ideas/plain`) and `.rusty/bookmarks.json` with the folder
`projects`, the search `tag:idea` and the heading `notes/beta#Second` (blank title); the recent
pages' scope deleted from the run's database. Three runs; every check passed in each.

### What each shot shows (third run)
- `662-01-favourites` (REQ-001): Favourites above the tree: Projects (folder), Ideas (magnifying
  glass), Second (hash; the blank title falls back to the heading); the tree below.
- `662-02-starred` (REQ-003): alpha opened from the tree, its star clicked: the star filled in the
  accent colour, alpha fourth in the group; `bookmark_add {"kind": "file", "path":
  "notes/alpha"}` in the log. No tooltip left over (see the fixes).
- `662-03-picker` (REQ-005): Home (the page in front), Alpha with a star and selected, a
  separator, Atlas (recent), a separator, then Plain, Seed and Beta.
- `662-04-folder` (REQ-006): `projects` open in the tree with atlas, its row selected.
- `662-05-search` (REQ-007): `tag:idea` in the field; Seed and Atlas listed, Plain not.
- `662-06-heading` (REQ-008): Beta opened with Second three lines under the top, where #656's
  outline puts a heading.
- `662-07-renamed` (REQ-009): "Alpha notes" in alpha's place, the search still open;
  `bookmark_set` sent the four in order with the new title.
- `662-08-ctrl-d` (REQ-003, 004): Ctrl+D in alpha's tab: the star empty, alpha gone from the
  group, the stand-in's list down to three.
- `662-09-outside` (REQ-011): the file rewritten from outside: Alpha from outside, Ideas,
  Projects, in that order; alpha's star filled with no click.
- `662-10-removed` (REQ-010): Remove on Ideas: Alpha from outside and Projects left.
- REQ-002 and REQ-012 by review: the group returns `None` on an empty list; the list is dropped
  while Rusty is off and nothing reads it then, and #661's filter hides `rusty: toggle bookmark`.

### Fixes, each rebuilt and run again
- **The rename cleared the search** (first run, `662-07`): `start_edit` emptied the search for
  every edit, since the tree's edits are typed in the tree. A favourite's title is typed in the
  group, so `start_edit` now leaves the search, the rebuild and the tree's scroll alone for it.
- **The star's tooltip kept its words** (first and second runs, `662-02`): GPUI builds a tooltip
  once, on hover, so after a click it still said "Add to Favourites" beside a filled star. The
  starred button now has an id of its own, which drops the shown tooltip.
- **The header against the rail's edge** (`662-01`): `ListSubHeader` now `inset`, as the
  Knowledge panel's are, in line with the rows' icons.
- **The scenario**: the tree's first row is at 215, not 223; beta's fixture had too little after
  Second for the heading to leave the bottom, so its second part runs 40 lines.

The gate runs on the final tree at Complete, after the fixes and the docs.

## Phase 4 — Complete (2026-10-06)
- **Docs:** `CHANGELOG.md` (Added); `docs/marley/rusty-in-marley.md` (R4b shipped, the QuickSwitcher
  and BookmarksPane rows, the batch line); `docs/marley/guide.md` (Favourites in the Brain view,
  the star in a Page tab, favourites first in the picker); `docs/marley/walkthrough.md` (stop
  2.13c, `rusty: toggle bookmark` in Appendix B); `docs/marley_architecture/marley_rusty.md`
  (`bookmarks`, `empty_order`'s favourites, the stand-in's bookmark tools);
  `docs/marley_architecture/marley_workbench.md` (the picker's favourites, the Favourites
  section). No Zed crate touched, so no zed-touchpoints row.
- **Knowledge:** F-claude-662-renaming-a-favourite-cleared-the-brain-search-001,
  F-claude-662-the-star-tooltip-kept-its-old-words-after-a-click-001,
  L-claude-662-a-tooltip-that-names-a-state-needs-an-id-per-state-001,
  AD-claude-662-favourites-are-rustys-bookmarks-read-whole-001.
- **Brain:** `brain decide` on consultation `aff0dccff92241e8a7dac6003cc431ab`, follow up by
  2026-11-06: `decisions/marleys-favourites-are-rustys-bookmarks-read-whole`.
- **Closed:** the ticket in `tickets/closed/`; this pair in `completed/`.
