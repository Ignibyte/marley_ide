---
pipeline_id: a98df00e-cd49-4963-80ba-f0f787cb7b9c
ticket: docs/planning/tickets/open/TICKET-654-rusty-open-page-picker.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Open a brain page by name, or make it, from a picker"
type: feature
slice: Rusty in Marley R3a (`rusty: open page`, R-D3's QuickSwitcher row, R-D10)
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/queued/645-brain-page-tab.spec.md, docs/planning/pipeline/queued/644-brain-view-in-the-rail.spec.md, docs/planning/pipeline/queued/643-rusty-switch-and-connection.spec.md]
---

## Title
`rusty: open page` opens a picker over every page of Rusty's brain: Zed's `picker` crate in the
workspace's modal layer, over one `brain_list_pages` read, matching titles and slugs on the machine
with Zed's `fuzzy_nucleo`, the pages Marley opened most recently first, and a "Create page" row at
the end when the query names no page, which makes it through `brain_new_page` with the query split
into `folder` and `name`. An unresolved link in a Page tab creates its page the same way.
`rusty::OpenPage`'s slug becomes optional, so the palette lists the action, and `secondary-alt-u`
opens it from anywhere. It is the screen Rusty's Qt app draws as `QuickSwitcher.qml`, which the
plan's R-D3 maps to "`rusty: open page`, a picker over `brain_list_pages` by title". Chad,
2026-10-03: "lets make a plan to begin the work and spec out the tickets" and "lets make sure we use
the gpui components we found here".

## Scope
### In
- **Order: after #645, which it extends.** It builds on #643 (the `marley.rusty` switch, the
  `Rusty` global with its connection state and its `call`, `crates/marley_rusty` with the Python
  stand-in named in `MARLEY_RUSTY_MCP`), #644 (the stand-in's vault tools, `brain_new_page` among
  them with Rusty's refusals) and #645 (`rusty::OpenPage { slug, preview }`,
  `rusty::page::open_later`, `PageView` and its navigation, `marley_rusty::page`'s `PageLink`,
  `split_fragment` and `normalise_target`), and takes their names as they ship. It needs nothing
  from #646 or #647.
- **The action.** `rusty::OpenPage`'s `slug` becomes `Option<String>` with `#[serde(default)]`.
  With a slug it does what #645 ships. With none it opens the picker, and `preview` is not read.
  gpui now builds the action with no arguments, so the command palette lists it as
  `rusty: open page` (D8). While Rusty is off it shows #644's toast ("Rusty is off. Turn it on in
  the Rusty section of the Marley settings.") and opens nothing; while Rusty is on and not
  connected, a toast with #643's state, and nothing opens. A second run while the picker is open
  closes it (Zed's `toggle_modal`).
- **The key.** `secondary-alt-u` to `rusty::OpenPage` in the Marley keymap's `Workspace` block
  (Ctrl+Alt+U on Linux, Cmd+Alt+U on macOS), free in every keymap Zed ships and in Marley's (D9).
  From a terminal it takes the key from the program, and Marley's shortcut note (#563) says so the
  first time, as `marley::NewAgent` does.
- **The list.** On each open, one `brain_list_pages { limit: 100000 }` through #643's `call`, as
  Rusty's app asks for every page, off the main thread, into the typed view
  `marley_rusty::switcher::PageSummary` (`slug`, `title`, `page_type`, `updated_at`). While it
  reads, the picker says "Reading the brain…"; a failure shows its first line and no create row.
  The query is matched in Marley, so no Rusty call is made per keystroke and nothing typed leaves
  the machine (D2).
- **The rows.** Each page row shows `IconName::FileMarkdown`, the title, and the slug in muted text
  beside it, through `ui::ListItem` and `ui::HighlightedLabel`, with the matched letters lit in the
  field that matched (D3).
- **The order.** With an empty query: the recently opened pages that the list holds, newest first,
  a separator, then the rest in Rusty's order (`updated_at`, newest first). When the active item is
  a Page tab, its page is the first row and the selection starts on the second, so Enter goes back
  to the page shown before, as Zed's file finder does (D4). With a query: title matches and slug
  matches from `fuzzy_nucleo::match_strings_async`, each page at its better score, ties to the
  more recently opened and then to Rusty's order, at most 100 rows (D3).
- **Recently opened.** A page counts as opened when #645's opener is asked for it, from any caller,
  or when a Page tab moves to it by a link, Back or Forward. Kept newest first, at most 20, in Zed's
  key-value store under Marley's scope `marley-rusty-recent-pages`, so the order survives a restart
  (D4).
- **Opening.** Enter or a click on a page row closes the picker and calls
  `rusty::page::open_later(workspace, slug, false, ..)`: a kept tab with the focus, or the tab that
  shows the page brought forward (D5). Ctrl+Enter does what Enter does.
- **Create on a miss.** When the query, read as a page path (D6), names no listed page, the list
  ends with "Create page: PATH". Its confirm sends `brain_new_page { folder, name }`, the folder the
  text before the path's last `/` (empty for the root) and the name the rest, never a `name` that
  holds `/`; the picker stays open while Rusty answers; on success it closes and opens the slug
  Rusty returns in a kept tab, with a toast when that slug is not the path asked for; on a refusal
  (a folder that does not exist, `..`) Rusty's message shows in a toast and the picker keeps its
  query.
- **Unresolved links create.** In a Page tab, a click on a `rusty:new/TARGET` link drops the
  target's heading (`split_fragment`), normalises it (`normalise_target`), splits it as above and
  calls `brain_new_page`; the tab then shows the returned slug, pushed on its history as a resolved
  link's page is. A refusal shows Rusty's message in a toast and leaves the tab as it is. This
  replaces #645's "does not exist yet" toast (its REQ-013), as #645's Out assigns it here (D7).
- **The pure core** (`marley_rusty`, no gpui): `switcher` (the `brain_list_pages` view, the recent
  list's visit, cap, store form and filter, the empty-query order, the merge of title and slug
  matches) and `page::NewPage` (a target or a query split into `folder` and `name`, with #645's
  normalising).
- **The stand-in** (#643's, with #644's tools) gains `brain_list_pages` over its scratch vault:
  the title from the frontmatter, else the file name; `page_type` from the frontmatter, else the top
  folder; `updated_at` from the file's mtime; newest first; `limit` honoured; each call logged.
- **Docs in the crate:** the in-app guide page's Brain article gains the picker's line
  (`crates/marley_workbench/guide/index.html`, under `crates/marley_*`, so changed in the Code
  phase, before the gate).
- `script/e2e/654-rusty-open-page-picker.sh`.

### Out (explicitly deferred)
- **Favourites first.** R-D3 says "favourites first"; Rusty serves no favourites until its
  TICKET-037 (bookmarks in a git-tracked vault file, with tools) lands. Until then the picker puts
  the recently opened first. The follow-up, one line: favourites from TICKET-037's tools as the
  first group on an empty query, starred, before the recent pages (#644's Out holds the same wait
  for the Brain view).
- **Aliases.** Obsidian's switcher searches "by name or alias"; Rusty indexes aliases
  (`brain_aliases`) and resolves links by them, but `brain_list_pages` returns four fields and no
  aliases, and only `brain_read_page`'s frontmatter holds them. A Rusty-side request, filed when
  Chad confirms: aliases in `brain_list_pages`' summaries. `brain_resolve_slug` is not used (D2).
- **Making a missing folder.** Rusty's TICKET-041 (REQ-002) makes the folder in the back end; until
  it lands, `brain_new_page` refuses "No folder X" and the picker shows that. New folders come from
  the Brain view's New Folder (#644). Once TICKET-041 lands, its REQ-003 also returns an existing
  page's slug instead of numbering a second one; the picker opens whatever slug Rusty returns.
- **Rusty's TICKET-040.** Until it lands, pages of a deleted folder may come back under `archive/`
  in `brain_list_pages`; the picker lists what Rusty lists, as #644's tree lists `archive/`.
- **Rusty's TICKET-035, the change cursor.** Nothing here depends on it. The list is read when the
  picker opens; a page made elsewhere while it is open shows at the next open.
- **Opening in a split or a new tab** (Zed's file finder splits on Ctrl+Enter; Obsidian opens a new
  tab): #645 leaves splitting a Page tab out.
- **Rusty's Ctrl+O.** Zed binds it to `workspace::OpenFiles` with no context, and a terminal passes
  it to its program (Claude Code's transcript key, bash's operate-and-get-next). A user who wants it
  binds it in their own keymap, which wins over Marley's.
- **A preview tab from the picker** (Zed's `preview_tabs.enable_preview_from_file_finder`, off by
  default): #645's preview open keeps the focus where it was, which suits a click in the tree and
  not a picker (D5).
- **Following a rename** of a recently opened page: a renamed page's old slug drops out of the list
  because Rusty no longer lists it.
- **Restoring the picker's query** between opens, and the picker on the Brain view's New Page
  button.

## Reference (§20)
Upstream Zed, kept as it is: the `picker` crate (`Picker::uniform_list`, `PickerDelegate`'s
`update_matches`, `confirm`, `separators_after_indices`, `no_matches_text`, `render_match`; the
`Picker` key context with `menu::Confirm`, `SelectNext`, `SelectPrevious` and `Cancel`), the
workspace's `ModalLayer` (`Workspace::toggle_modal`, which closes the same modal on a second toggle
and gives the focus back), and the file finder's behavior: history first on an empty query, the
active item first with the selection on the next row (`skip_focus_for_active_in_search`, on by
default), "Create File: path" as the last row when no file is at the typed path, and Enter opening
a kept tab while `enable_preview_from_file_finder` is off (its default). `fuzzy_nucleo` ranks, as
it does for Zed's file finder, command palette and tab switcher. Marley adds a delegate of its own;
no file finder code is carried (§20, the code-layer wall). The screen's content follows Rusty's own
`QuickSwitcher.qml` (MIT, the same owner): every page by title and slug, the best of the two scores,
create when nothing matches. No Warp behavior applies: Warp keeps no notes vault.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` §2 (`Picker`
  and `PickerDelegate`: "New overlays are nearly free"), §3.1 (`CommandPaletteFilter`, considered:
  #642's D4 keeps an off feature's action listed with a toast pointing at its switch, and this
  ticket does the same) and §4.1 (the file finder's `Match::CreateNew`, history from
  `recent_navigation_history`, `fuzzy_nucleo` with a cancel flag, the cap at 100).
  `docs/orca_architecture/05-terminal-and-workspace.md` §2.9: Orca's Quick Open, 50 results,
  name-first rows, "Zed's file finder matches it". `docs/warp_architecture/crates/fuzzy_match.md`:
  Warp's palettes rank with a Skim matcher and highlight the matched indices; read as a map only.
  The plan's R-D3, R-D10 and Rusty's triage (TICKET-037, TICKET-040, TICKET-041).
- **Published material:** Obsidian's Quick switcher (`obsidian.md/help/Plugins/Quick+switcher`):
  Ctrl+O; "If the search term is empty, the Quick switcher shows the most recent notes"; Enter
  creates when nothing matches and Shift+Enter when something does; Ctrl+Enter opens a new tab;
  search "by name or alias". Rusty (MIT, read at `295565c`): `brain_list_pages`
  (`rusty-mcp/src/main.rs:921-931`, params `:98-105`) returns `BrainPageSummary { slug, page_type,
  title, updated_at }` (`rusty-core/src/brain/mod.rs:54-65`) ordered by `updated_at DESC`, 50 by
  default and uncapped (`:573-626`), from SQLite with no embedding; `brain_new_page`
  (`main.rs:1264-1276`, params `:593-601`; `mod.rs:2438-2474`: "No folder X", `/` turned into `-`
  in the name, a clash numbered `X 1`, the slug returned); `brain_resolve_slug` (`main.rs:
  1651-1657`, `mod.rs:2275-2307`); `QuickSwitcher.qml:31-86` (its scoring, favourites first on an
  empty query, Enter and Shift+Enter) and `Main.qml:372, 506, 642` (`limit: 100000`, `createPage`
  with `folder: ""`, Ctrl+O); TICKET-041's REQ-002 and REQ-003.
- **The code we already ship:**
  - `picker` (Zed) owns this seam: `PickerDelegate` (`picker.rs:164-442`), `Picker::uniform_list`
    (`:460`), `separators_after_indices` (`:172`), `confirm(secondary)` (`:255`). Taken unchanged.
  - `file_finder` (Zed), read for behavior and not depended on: the create row pushed when no entry
    is at the typed path (`file_finder.rs:1235-1246`) and sorted last (`:624-628`), labelled
    "Create File: …" (`:1339-1340`); the selection skipping the active file
    (`calculate_selected_index`, `:1528-1545`; `default.json:1556`); `allow_preview` from
    `enable_preview_from_file_finder` (`:1568`; `default.json:1501`, false); a second Toggle while
    open cycles the selection (`:98-103`), which this picker does not copy (D8).
  - `fuzzy_nucleo` (Zed; `nucleo` 0.5.0 in Cargo.lock): `match_strings_async` (`strings.rs:100`)
    builds a nucleo pattern from the query split on whitespace (`fuzzy_nucleo.rs:67-87`), so "dec
    rend" finds `decisions/use-zeds-renderer`. Taken. `fuzzy` (Zed, the matcher Marley's three
    pickers use, `agents.rs:762`) matches the query's characters in order with its spaces
    (`fuzzy/src/strings.rs:145-146`), so the same query misses a slug: considered, not taken (D3).
  - `workspace` (Zed): `toggle_modal` (`workspace.rs:8614`, `modal_layer.rs:152-172`),
    `ModalView`, `Toast`. `db::kvp::KeyValueStore::scoped` (`db/src/kvp.rs:89-118`). gpui:
    `available_actions` builds each action with no arguments (`key_dispatch.rs:363-380`,
    `action.rs:336-368`), so an action with a required field is never in the palette.
  - Marley: `agents.rs` (`NewAgentPicker`, `:575-661`: `toggle_modal`, `Picker::uniform_list`, a
    key context of its own, the shortcut note from a terminal), `shortcut_note.rs` (`taken`, its
    note kept in Zed's key-value store, `:1-80`), the Marley keymap's `Workspace` block
    (`keymap.json:7-15`), #644's off toast and stand-in, #645's opener, `PageView` and `PageLink`.
  - Ely GPUI Components (story `searchpalette-spotlightsearch-quicklauncher`, HEAD `2f8b2f6`):
    `src/navigation/palette/kinds.rs` and `mod.rs` read and not ported. `SearchPalette` asks a
    synchronous `Fn(&str)` for its rows on every render (`kinds.rs:401-404`, `:440`), so an MCP read
    would need a cache outside it and nothing cancels a long match; `Palette` draws every row as a
    child of one scrolling `div` (`mod.rs:236-311`, `:366-377`), no virtual list, and Ely's own
    `QuickOpen` caps at 50 rows for it (`kinds.rs:196-197`) where the vault holds 814 pages; the
    overlay is Ely's `Backdrop`, `FocusScope`, `take_focus` and `motion` (`mod.rs:154-169`,
    `:391-395`) where Zed has `ModalLayer`; its rows are only the values given, so a create row has
    no place (`mod.rs:221-231`); it calls `expect` (`mod.rs:34`), `panic!` (`kinds.rs:137`) and
    `assert!` (`:324`, `:448`), which Marley's lint table denies; and it marks substrings
    (`Highlight::matching`, `:458`) with Ely's theme, i18n strings and icons. What it confirms:
    `QuickOpen` leads with recent paths on an empty query and draws a path as name then folder
    (`:199-305`), and `QuickSwitcher` starts on its second item so Enter goes back (`:358-365`), the
    same two rules Zed's file finder keeps. Zed's own wins: nothing taken, no notice needed.
  - The sweep's answer: `picker`, `fuzzy_nucleo` and the modal layer own the screen; Rusty's tools
    own the data and the write; nothing new is needed but a delegate and a pure module.

## UI proof
`script/e2e/654-rusty-open-page-picker.sh` (`compositor sway`: it clicks links in a Page tab).
Setup writes a scratch vault under `$E2E_WORK/vault` with made-up pages, their mtimes set so
Rusty's order is known (newest first): `home`, `people/sam` ("Sam"), `notes/2026-q3` ("Quarterly
review"), `decisions/use-zeds-renderer` ("Use Zed's renderer"), `projects/atlas` ("Atlas", with
the unresolved links `[[ideas/later]]` and `[[drafts/soon]]`), `projects/marley/shell` ("Shell"),
`ideas/seed` ("Seed"); there is no `drafts/` folder. It links #643's stand-in as
`$E2E_WORK/bin/rusty-mcp`, names it in `MARLEY_RUSTY_MCP`, points it at that vault, and sets
`marley.rusty.enabled` true with the embedded connection in the run's settings; it deletes the
`marley-rusty-recent-pages` scope and the shortcut note's `rusty::OpenPage` row from the run's copy
of the database, so the run starts with nothing opened and the user's own pages never show (R-D8).
Shots:
- `654-01-listed`: Ctrl+Alt+U from the center terminal: the picker with the seven pages in that
  order, title and slug on each row, the first selected; the shortcut note naming Ctrl+Alt+U.
- `654-02-by-title`: "quart rev" typed: Quarterly review first, letters lit in its title; "Create
  page: quart rev" last.
- `654-03-by-slug`: "peop sam": Sam, letters lit in `people/sam`.
- `654-04-exact`: "people/sam": Sam alone, no create row.
- `654-05-opened`: "quart", Enter: the picker gone; a kept Quarterly review tab with the focus.
- `654-06-recent`: Sam and then Atlas opened the same way, Ctrl+Alt+U: Atlas, Sam, Quarterly
  review, a separator, then the other four in Rusty's order; the selection on Sam.
- `654-07-back`: Enter: Sam's tab in front, still three tabs.
- `654-08-create-row`: "projects/marley/review": "Create page: projects/marley/review" alone and
  selected.
- `654-09-created`: Enter: a kept "review" tab; the stand-in's log holds `brain_new_page` with
  `folder` `projects/marley` and `name` `review`.
- `654-10-refused`: "drafts/soon", Enter: a toast with "No folder drafts"; the picker open with
  "drafts/soon".
- `654-11-link-created`: Escape; Atlas's tab; a click on `ideas/later`: the tab shows "later", Back
  enabled; the log holds `folder` `ideas` and `name` `later`.
- `654-12-link-resolved`: Alt-Left: Atlas, `ideas/later` now in the link colour.
- `654-13-link-refused`: a click on `drafts/soon`: the toast; Atlas still shown.
- `654-14-palette`: the command palette, "open page" typed: `rusty: open page` with Ctrl+Alt+U.
- `654-15-from-palette`: Enter: the picker.
- `654-16-not-connected`: Escape; the stand-in's stop file made and its process killed; Ctrl+Alt+U:
  the toast with #643's reason; no picker.
- `654-17-off`: `marley.rusty.enabled` set false from outside; Ctrl+Alt+U: "Rusty is off…"; no
  picker.

## Locked-In Decisions
- D1: **Zed's `Picker` in the modal layer, a delegate of Marley's.** `PagePicker`, a `ModalView`
  holding `Picker::uniform_list(PagePickerDelegate)`, opened by `Workspace::toggle_modal`, with the
  key context `RustyPagePicker`, as Marley's `NewAgentPicker` and Zed's file finder are built. The
  picker owns the query editor, the list keys, the virtual list, dismissal and the focus's return.
  Rejected: porting Ely's `SearchPalette` (Prior art: a synchronous search on every render, no
  virtual list, its own overlay, panics the lint table denies, no place for a create row); a list
  drawn by Marley, as #646's search is (there Rusty ranks; here Marley ranks, which is the picker's
  job).
- D2: **One `brain_list_pages` read per open, matched in Marley.** `{ limit: 100000 }`, as Rusty's
  app asks: Rusty's default is 50 and it has no upper bound. The read is SQLite only, and the query
  never reaches Rusty, so nothing typed goes to an embedding provider, the reason #644's and #646's
  searches send on Enter only. Matching fields: title and slug, which is all the list serves.
  Rejected: `brain_search` (it ranks bodies, and with an embedding provider set it embeds the
  query); `brain_resolve_slug` for aliases (it answers slugs only, without the alias that matched,
  by a substring `LIKE`, ten at most, and would be a call per keystroke); a cache kept between opens
  (the read is a few milliseconds, and a cache would need the change signal to stay true).
- D3: **Zed's `fuzzy_nucleo`, over titles and over slugs.** Two `match_strings_async` calls on the
  background executor, `Case::smart_if_uppercase_in(query)`, `LengthPenalty::On`, 100 results each;
  each page keeps its better score and the positions of the field that gave it, as Rusty's switcher
  scores the best of title and slug. Ties go to the more recently opened, then to Rusty's order;
  the merged list stops at 100 rows, and the picker's `match_count` is the same number, so the
  selection never reaches a row not drawn (BF-symbol-picker-render-cap). A new query drops the
  older match task (the `Picker` replaces its pending update). Rejected: `fuzzy::match_strings`,
  Marley's other pickers' matcher, whose query keeps its spaces as characters to match in order, so
  "dec rend" misses `decisions/use-zeds-renderer`; one candidate string per page joining title and
  slug (positions would need splitting back, and a match across the join is noise).
- D4: **Recently opened first, kept by Marley.** Rusty serves no recent pages (no tool, and
  `updated_at` is when the index last saw a change, so a page read every day but never edited
  sinks). Marley keeps the slugs its opener was asked for and the pages Page tabs moved to by a
  link, Back or Forward, newest first, at most 20, as one JSON array under the key `pages` in Zed's
  key-value store, scope `marley-rusty-recent-pages`, written in the background on a change, read
  once at init into a global (the store `shortcut_note.rs` uses). A recent slug is drawn only when
  the list holds it. On an empty query the recent group comes first with a separator after it
  (`separators_after_indices`, the file finder's separate history); when the active item is a Page
  tab its page heads the group and the selection starts on the next row, as the file finder's
  `skip_focus_for_active_in_search` default and Ely's `QuickSwitcher` both do. Rejected: in memory
  only (Page tabs are not restored after a restart, #645's D10, so every start would forget); per
  workspace (the brain is one store for every project on the box); Rusty's order alone.
- D5: **Enter opens a kept tab with the focus.** Through `rusty::page::open_later(workspace, slug,
  false, ..)` after the picker dismisses, which defers itself (#645's D9), so a page already shown
  in a tab comes forward and nothing opens twice. Rusty's switcher opens with `openPage(slug,
  false)`, and Zed's file finder opens kept while `enable_preview_from_file_finder` is off, its
  default. Ctrl+Enter (`menu::SecondaryConfirm`) does what Enter does. Rejected: reading
  `enable_preview_from_file_finder` (#645's preview open leaves the focus where it was, right for a
  click in the tree, wrong for a picker that has just closed).
- D6: **The create row and the split.** The query read as a page path: trimmed, a leading `/` or
  `./` and a `.md` suffix dropped, as #645's `normalise_target` does. The row "Create page: PATH"
  ends the list when that path is not empty, does not end in `/`, and is no listed slug (compared
  exactly: the vault's files are case-sensitive), as Zed's file finder ends its list with "Create
  File: path" whenever no file is there, so Enter on the first row still opens the best match.
  Confirm splits PATH at its last `/` into `folder` and `name` and sends `brain_new_page { folder,
  name }`: Rusty's TICKET-041 turns a slashed `name` into a root page `a-b`, and the split stays
  right after it lands. The picker stays open with the row reading "Creating…" and a second confirm
  does nothing until Rusty answers; then it closes and opens the returned slug kept, with a toast
  naming that slug when it is not PATH (Rusty numbers a clash `X 1`); a refusal shows Rusty's
  message in a toast of its own id (L-587) and keeps the query. Rejected: `brain_new_folder` before
  the page (a second write and commit that TICKET-041's REQ-002 makes redundant, and a typo would
  make a folder); `picker::ConfirmInput` (Alt+Enter) to force a create while pages match, the role
  of Obsidian's and Rusty's Shift+Enter (the row is always last, as in the file finder); hiding the
  row when the query equals a page's title, as Rusty's hint does (`atlas` is a root page `atlas`,
  not `projects/atlas`).
- D7: **An unresolved link makes its page.** #645's `rusty:new/TARGET` click: the heading split off
  (`split_fragment`), the target normalised, split as in D6 and sent to `brain_new_page`; the tab
  then shows the returned slug, pushed on its history. No prompt, as Obsidian and Rusty's `NoteTab`
  create on a click; a page made by mistake is deleted from the Brain view (#644). Rejected: the Qt
  app's `brain_new_page { folder: "" }` (TICKET-041's bug); keeping #645's "does not exist yet"
  toast (#645's Out names this slice for the create).
- D8: **The action keeps its name; the slug becomes optional.** `OpenPage { #[serde(default)] slug:
  Option<String>, #[serde(default)] preview: bool }`, so gpui builds it from `{}` and the palette
  lists `rusty: open page`, the plan's name (#645's D3). The handler checks the switch each time it
  runs: off, #644's toast; on and not connected, a toast with #643's state; connected, the picker.
  A second run closes the picker, `toggle_modal`'s rule. Rejected: a second action
  `rusty::OpenPagePicker` (two palette entries for one screen); hiding the `rusty` namespace from
  the palette while off (#642's D4: an off feature's action stays listed and points at its switch);
  the file finder's second press cycling the selection (Zed's own palette closes on a second press,
  and the picker's list keys move the selection).
- D9: **The key is `secondary-alt-u`.** In the Marley keymap's `Workspace` block, beside
  `secondary-alt-n` (New Agent) and #644's `secondary-alt-v` (the Brain view). Swept as AD-450's
  chord was: `ctrl-alt-u` is unbound in every context of Zed's Linux and Windows defaults and in
  Vim's keymap and the Atom, Cursor, Emacs, JetBrains, Sublime Text and VS Code base keymaps;
  `cmd-alt-u` is unbound in macOS's defaults and base keymaps (TextMate's `ctrl-alt-u` is another
  chord there, `secondary` being Cmd); Marley's keymap binds neither; Omarchy's Hyprland bindings
  use Ctrl+Alt only with Delete and Tab. No context binds it deeper, so it reaches from the editor,
  a terminal, a Page tab and the rail. Rusty's Ctrl+O is not taken: Zed binds it with no context to
  `workspace::OpenFiles`, which outranks a `Workspace` binding (a binding with no context matches
  at the deepest depth, `keymap.rs:246-250`), and a terminal passes it to its program. Rejected:
  `secondary-alt-o` (Zed's `projects::OpenRecent` in `Workspace` on Linux and macOS; making it
  Rusty's only while Rusty is on, by propagating while off as L-637 allows, gives one key two
  meanings and the palette two labels for it); `secondary-alt-p` (`agent::ManageProfiles` in
  `AcpThread`, `picker::TogglePreview` in pickers with a preview); `secondary-shift-o` (the
  outline); `secondary-alt-shift-o` (`projects::OpenRemote`, a frame-overlay key); `secondary-alt-q`
  (a slip of Alt quits through `ctrl-q`); `secondary-alt-m` and `secondary-alt-w` (taken in macOS's
  agent options and search bar; Ctrl+M is Enter in a terminal, Ctrl+W closes a tab).
- D10: **Scenarios never touch the real brain** (R-D8). The scenario runs #643's stand-in over a
  scratch vault, named in `MARLEY_RUSTY_MCP`; it clears the recent pages' scope and the note's row
  from the run's copy of the database (the harness copies the user's, `script/e2e.sh:643`), so the
  run starts from nothing and no real slug can show.
- D11: **Where it lives.** The view in `marley_workbench::rusty::page_picker` (a public module, as
  #645's `rusty::page` calls it to record an open, L-574); the pure parts in
  `marley_rusty::switcher` and `marley_rusty::page::NewPage`. No Zed crate changes; one new
  dependency edge, `marley_workbench` on `fuzzy_nucleo`; no new external crate and no spawn site.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `rusty::OpenPage` runs without a slug while Rusty is on and connected, the system shall open a picker listing every page `brain_list_pages` returns, each row with its title and its slug. | Shot `654-01-listed` |
| REQ-002 | WHILE the query is empty and no page has been opened, the picker shall list the pages in Rusty's order, the most recently updated first. | Shot `654-01-listed` |
| REQ-003 | WHEN the user types a query, the picker shall list the pages whose title matches it, best first, with the matched letters lit in the title. | Shot `654-02-by-title` |
| REQ-004 | WHEN the user types a query that matches a page's slug, the picker shall list that page with the matched letters lit in the slug. | Shot `654-03-by-slug` |
| REQ-005 | WHEN the user confirms a page row, the system shall close the picker and show that page in a kept Page tab with the focus. | Shot `654-05-opened` |
| REQ-006 | WHILE the query is empty, the picker shall list the recently opened pages first, newest first, then a separator, then the other pages in Rusty's order. | Shot `654-06-recent` |
| REQ-007 | WHEN the picker opens while the active item is a Page tab, the system shall select the row after that page's row. | Shot `654-06-recent` |
| REQ-008 | WHEN the user confirms a page that a tab already shows, the system shall bring that tab forward and open no other. | Shot `654-07-back` |
| REQ-009 | The system shall keep at most 20 recently opened pages in Zed's key-value store, so their order holds after a restart. | Review |
| REQ-010 | WHERE the query, read as a page path, names no listed page, the picker shall end its list with a row offering to create the page at that path. | Shots `654-02-by-title`, `654-08-create-row` |
| REQ-011 | WHERE the query names a listed page by its slug, the picker shall offer no create row. | Shot `654-04-exact` |
| REQ-012 | WHEN the user confirms the create row, the system shall call `brain_new_page` with the path's text before its last `/` as `folder` and the rest as `name`, and show the slug Rusty returns in a kept Page tab. | Shot `654-09-created`; the stand-in's log |
| REQ-013 | IF Rusty refuses to create the page, THEN the system shall show Rusty's message in a toast and keep the picker open with its query. | Shot `654-10-refused` |
| REQ-014 | WHEN the user clicks an unresolved link in a Page tab, the system shall create the linked page the same way and show it in that tab. | Shots `654-11-link-created`, `654-12-link-resolved`; the log |
| REQ-015 | IF Rusty refuses to create a linked page, THEN the system shall show Rusty's message in a toast and leave the tab as it is. | Shot `654-13-link-refused` |
| REQ-016 | WHEN Ctrl+Alt+U is pressed with the focus in a terminal, an editor, a Page tab or the rail, the system shall run `rusty::OpenPage` without a slug. | Shots `654-01-listed`, `654-06-recent`; review of the keymap |
| REQ-017 | WHEN the key takes a terminal's keystroke for the first time on this data directory, the system shall show Marley's shortcut note naming it. | Shot `654-01-listed` |
| REQ-018 | The command palette shall list `rusty: open page`, and running it shall open the picker. | Shots `654-14-palette`, `654-15-from-palette` |
| REQ-019 | WHEN `rusty::OpenPage` runs without a slug while Rusty is on and not connected, the system shall show a toast saying so with the connection's reason, and open no picker. | Shot `654-16-not-connected` |
| REQ-020 | WHEN `rusty::OpenPage` runs without a slug while Rusty is off, the system shall show the toast saying Rusty is off and where to turn it on, and open no picker. | Shot `654-17-off` |
| REQ-021 | WHILE the page list is being read, the picker shall say so, and IF the read fails, THEN it shall show the failure's first line and no create row. | Review |
| REQ-022 | WHEN Rusty returns a slug other than the path asked for, the system shall show the returned page and name its slug in a toast. | Review (the stand-in's numbering is #644's; no shot) |
| REQ-023 | WHEN `rusty::OpenPage` names a slug, the system shall open it as #645 ships it, whether or not the picker exists. | Review |

## Phase Plan
- **P1 Plan**: promote after #645 completes (L-540): re-verify every name this spec takes from
  #643, #644 and #645 as they shipped (the `Rusty` global, `call`, the stand-in and its stop file,
  `OpenPage`, `open_later`, `PageView`'s navigation path, `PageLink::Missing`, `normalise_target`,
  `split_fragment`, the off toast's wording); check whether Rusty's TICKET-037 or TICKET-041 has
  landed (favourites first; a folder made by Rusty and an existing slug returned), and re-plan what
  they change; sweep the keymaps again for `secondary-alt-u`; ask the brain (`brain_ask`) on D4 and
  D9; Chad confirms the key.
- **P2 Code**: the `README.md` marker first; `marley_rusty::switcher` and `page::NewPage`; the
  stand-in's `brain_list_pages`; `OpenPage`'s optional slug and its handler; `PagePicker` and its
  delegate; the recent pages' global, its store and #645's two call sites; #645's `rusty:new/`
  click; the keymap line; `fuzzy_nucleo` in `marley_workbench`'s manifest; the in-app guide line; a
  review of the diff; `script/gates.sh --diff` green.
- **P3 Test**: write and run the scenario, read every shot.
- **P4 Complete**: CHANGELOG (Added; Changed: an unresolved link now makes its page) and the
  architecture and user docs the notes list (§21), the plan's R-D3 row and slices table, ledger
  capture (§19), `brain_decide` for D4 and D9, close the ticket, archive, commit.
