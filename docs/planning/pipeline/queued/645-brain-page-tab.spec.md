---
pipeline_id: 02df160f-7254-46eb-ad34-c04ca70458fd
ticket: docs/planning/tickets/open/TICKET-645-brain-page-tab.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A brain page in a center tab"
type: feature
slice: Rusty in Marley R2 (the Page tab), decision R-D4; prong 2 (D11 as amended)
references: [docs/marley/rusty-in-marley.md, docs/marley/three-prong-plan.md]
---

## Title
A page of Rusty's brain opens as a center tab (`impl workspace::Item`, as `DecisionsView` and
`AgentView` do): its title and properties above its body, the body rendered by Zed's own `markdown`
crate after a pure pass in `marley_rusty` turns wikilinks into links Marley can follow, Back and
Forward within the tab, an Edit toggle that shows the page's vault file in a Zed editor, and Zed's
preview-tab rule for the rail's clicks. It is the screen Rusty's Qt app draws as `NoteTab.qml`, and
the action every later click calls.

## Scope
### In
- **Builds on #643 and #644.** #643 makes `crates/marley_rusty` (pure, with `fixtures/` and the
  Python stand-in `stand_in/rusty-mcp`, which a scenario names in `MARLEY_RUSTY_MCP`), the client in
  `marley_workbench::rusty` (Zed's `ContextServer`) with its connection state and its signal on
  `notifications/resources/list_changed`, and the `marley.rusty` switch; #644 makes the rail's Brain
  view, its one door for opening a page (`rusty::brain::open_page(slug, preview)`), and the
  stand-in's vault tools, `setting_get brain_vault_path` among them. This ticket adds to both and
  takes their names as they ship.
- **The action and the opener.** `rusty::OpenPage { slug: String, preview: bool }` (`preview`
  defaults to false) in a new `rusty` action namespace, registered on each workspace while
  `marley.rusty.enabled` is on, and `rusty::page::open_later(workspace, slug, preview, window, cx)`
  for Marley's own callers. Both run one opener, in `window.defer`: a Page tab of the workspace
  that shows `slug` now comes forward (and, without `preview`, is kept and focused); else, with
  `preview`, a new tab replaces the active pane's preview tab and the focus stays where it was;
  else a new kept tab opens with the focus. The slug is required, so the palette does not list
  the action until the picker's ticket makes it optional (Out).
- **The rail's opens.** #644's `rusty::brain::open_page`, which opens the page's file in Zed's
  editor until this ticket, calls `rusty::page::open_later` instead, so the tree's clicks, Enter,
  Today and a new page open Page tabs. The click rule stays #644's (its D6): one click previews
  where `preview_tabs.enabled` and `enable_preview_from_project_panel` allow, a double click keeps.
- **The tab.** `PageView`: the page's title as the tab's text, italic while it is the preview tab,
  `IconName::FileMarkdown`, the slug as its tooltip; a header row with Back and Forward icon
  buttons, the slug's folder and name, and an Edit button; the title in a large label; the
  properties; the body, scrolling under the fixed header.
- **The properties.** Every entry of `brain_render`'s `properties`, in file order, read only: a key
  in a muted label column and its value beside it, a list as `ui::Chip`s, a number or boolean as
  its text, an object as compact JSON. The layout is Ely's `DescriptionList`, ported (D7).
- **The body.** `marley_rusty::page` takes `raw`, cuts the frontmatter by Rusty's own rule, and
  rewrites each wikilink pulldown-cmark 0.13 finds (Rusty's parse options, `ENABLE_WIKILINKS` on)
  into an ordinary Markdown link: `[text](<rusty:page/SLUG#HEADING>)` when `links` resolves its
  target, `[text](<rusty:new/TARGET>)` when not, with the alias or the target as written as the
  text. An embed (`![[x]]`) becomes the same link. Zed's `Markdown` entity, with the project's
  language registry and heading slugs on, draws the result through `MarkdownElement` with Zed's
  Preview style; `link_callback` draws a `rusty:new/` link in the muted text colour.
- **Links.** `marley_rusty::page::PageLink::parse` sorts a clicked address: `rusty:page/` opens the
  page in this tab, pushed on its history, and scrolls to the heading when one is named;
  `rusty:new/` shows a toast that the page does not exist yet and leaves the tab as it is; a
  `#heading` alone scrolls this page; a local target with no scheme (`other.md`) is a page target,
  normalised as Rusty does; anything else goes to `cx.open_url`.
- **Back and Forward.** The tab's own history (`marley_rusty::page::PageHistory`), with
  `rusty::PageBack` and `rusty::PageForward` on Alt-Left and Alt-Right in the tab's key context
  (`RustyPage`, in Marley's own keymap), and the header's buttons disabled at either end.
- **Edit.** `rusty::TogglePageEdit` and the Edit button: the page's file, `<vault>/<slug>.md` with
  the vault from `setting_get brain_vault_path` or Rusty's default `~/.rusty/brain`, opens through
  `Project::open_local_buffer` into a Zed `Editor` shown in the tab's body. While it shows, the tab
  passes `is_dirty`, `has_conflict`, `can_save`, `save`, `reload`, `for_each_project_item` and the
  editor's item events to the editor, as Zed's Markdown preview and `ProjectDiff` do, so Ctrl-S is
  Zed's save and an edit keeps a preview tab. Read, Back and Forward save unsaved edits first,
  unformatted, then show the page rendered as saved; a navigation always lands in Read.
- **Live.** The tab re-reads its page on #643's change signal while it shows Read, and drops an
  answer that comes back after a newer navigation.
- **States.** Loading; "No page SLUG in the brain." when `brain_render` answers `null`; the error
  text when the call fails; and, while there is no connection, the page last drawn with a line
  saying Marley is not connected to Rusty.
- **The stand-in** (#643's, with #644's vault tools) gains `brain_render` over its scratch vault on
  disk: `raw`, `title`, `page_type`, `properties` from the frontmatter, `links` and `unresolved` by
  a `<target>.md` file of the vault, `null` for a missing page. Its `list_changed` on `SIGUSR1`
  (#644's) stands in for Rusty's watcher. Fixtures: three pages.
- **Zed's namespace test.** `"rusty"` joins `test_action_namespaces` in `crates/zed/src/zed.rs`,
  unless #643 or #644 added it first.
- The in-app guide page's Brain article line, and `script/e2e/645-brain-page-tab.sh`.

### Out (explicitly deferred)
- **Rusty's TICKET-036, the structured render.** Typed blocks with resolved links from Rusty would
  replace this ticket's pass. Until then, what the pass leaves to Zed's parser: callouts beyond
  GFM's five kinds, embeds drawn as links, `%%comments%%`, `==highlights==`, inline `#tags` as
  links, and images from the vault. The live vault has 814 pages, 4514 wikilinks, 1964 piped and
  182 with a heading, and no callout, embed or image; one page holds a comment.
- **The second slice, its own ticket: `rusty: open page`'s picker.** `OpenPage`'s slug becomes
  optional and, with none, a picker over `brain_list_pages` by title (favourites first once Rusty's
  TICKET-037 lands) opens the page chosen, or creates the one typed through `brain_new_page` with
  its folder and name. A click on an unresolved link creates its page the same way (not the Qt
  app's `folder: ""`, which turns `decisions/foo` into `decisions-foo`). #646 leaves the picker out
  too (its Out list), so it is R3's remaining half.
- **Inline title and property edits** (`brain_rename`, `brain_set_property`,
  `brain_remove_property`; Ely's `PropertyGrid` and `InlineEdit`): an editor per value kind, not
  small.
- **Task boxes that toggle** (the Qt app writes the file on a click), property values as links (the
  `consulted` slugs, `url`), the favourites star (Rusty's TICKET-037 is not built), the rename, move
  and delete menu (#644's tree), the local graph (#647), backlinks and outgoing links (#646).
- **An outline in the tab, its own ticket:** `brain_render`'s `outline` beside the body, a click
  scrolling through Zed's heading slugs. Ely's `TableOfContents` marks the section in view from
  heading positions its own renderer measures, which `MarkdownElement` does not expose (Prior
  art). #646's D4 counts on an outline here; it is not in this slice.
- **Search in the page** (`SearchableItem`), a key for Edit (Ctrl-E is Zed's file finder on Linux),
  Ctrl-click to open a link in a new tab, splitting the tab, and Rusty's per-section LIVE mode.
- **Restoring Page tabs after a restart** (`SerializableItem`, as AD-609 deferred for the Agent
  tab).
- **Rusty's TICKET-035, the change cursor.** Nothing here depends on it. Until it lands, a write
  that touches only Rusty's database from another process is not announced; page files are watched.
  A `service` connection hears no `list_changed` (#643's D5), so there a change made outside shows
  at the next navigation or signal.
- **Two Rusty-side requests**, filed in Rusty when Chad confirms: a tool that names the vault root
  (no tool returns it, and Rusty never stores its default), and a commit of the outside edits that
  `sync_all` picks up (D6).
- **Edit in a remote project.** The vault is local; in an SSH project Edit says it needs a local
  project.

## Reference (§20)
Upstream Zed: the workspace's preview tabs (`workspace::Pane`'s `replace_preview_item_id`,
`unpreview_item_if_preview`, `TabContentParams::preview`, the tab's double click, and the project
panel's rule that a click count of 1 previews and a higher one keeps and focuses) and Zed's
Markdown preview (`markdown_preview::MarkdownPreviewView`: a `MarkdownElement` in an item, link
clicks taken by `on_url_click` and acted on in `window.defer`, save passed to an editor) are kept
as they are; Marley adds an item that uses them. The screen's content follows Rusty's own
`NoteTab.qml` (MIT, the same owner): back and forward over a per-tab history, the folder and name,
the properties above the body, a Read and Edit toggle that saves before it leaves the source. No
Warp behavior applies.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` §1.3 (the
  pane's `preview_item_id`, "the single italic preview tab", and its per-pane `nav_history`) and §4
  (the file finder's `open_path_preview` gated by `PreviewTabsSettings`). Orca's changelog records a
  setting to turn preview tabs off (`docs/orca_architecture/07-engineering-and-changelog.md:1531`),
  which Zed's `preview_tabs.enabled` is here. Nothing in `docs/warp_architecture/` covers a notes
  page.
- **Published material:** pulldown-cmark 0.13's `Options::ENABLE_WIKILINKS` and
  `LinkType::WikiLink { has_pothole }` ("Obsidian-style Wikilinks"); Obsidian's link forms
  (`[[page]]`, `[[page|alias]]`, `[[page#heading]]`, `![[embed]]`); CommonMark's angle-bracket link
  destinations, which allow spaces. Rusty's TICKET-036 (`docs/planning/tickets/open/` in rusty-v3).
- **The code we already ship:**
  - `markdown` (Zed) owns this seam: `MarkdownElement` draws headings, lists, `ui::Checkbox` task
    boxes, code blocks highlighted through the language registry, tables, GFM callouts and
    footnotes, with selection and copy, heading slugs and `scroll_to_heading`; `on_url_click`
    replaces `cx.open_url`, and `MarkdownStyle::link_callback` styles a link by its address. Its
    parser leaves wikilinks off on purpose (`parser.rs:984-987`, held by a test). Taken unchanged.
  - `markdown_preview` (Zed): the item around a `MarkdownElement`, its deferred link handling and
    its save passed to an editor; the pattern, not the type (it is bound to a source editor).
  - `workspace` (Zed): the preview API above; `open_project_item`'s two-step add
    (`workspace.rs:5524-5541`). `git_ui::ProjectDiff` passes save and dirty state to its editor.
  - `pulldown-cmark` 0.13.4 (Cargo.lock), the version Rusty parses with: the pass finds wikilinks
    with Rusty's own options, so both sides see the same links.
  - Marley: `decisions.rs` and `agent_tab.rs` (an item, its opener, `window.defer`), the Browser
    tab's own history and Alt-Left and Alt-Right in its key context (`keymap.json:133-147`),
    `markdown_commands.rs` (#530's hook in the same crates, which this ticket does not need).
  - Rusty: `brain_render` returns `raw`, `title`, `page_type`, `properties`, `links` (`target`,
    `slug`, `alias`, `embed`) and `unresolved` in one call; its renderer writes `rusty:page/` and
    `rusty:new/` addresses, normalises a target and splits a heading off it, which the pass copies.
  - Ely GPUI Components (story
    `markdownrenderer-tableofcontents-documentoutline-readingprogress`): `MarkdownRenderer`
    (`src/documents/render.rs`) over its own tree (`src/documents/tree.rs`) was read and not ported:
    it is a second renderer, colouring code with Ely's own `code_colors` rather than a language
    registry, with no selection, Ely's theme tokens, Lucide icons and `Latex`, and a parser that
    `expect`s and `unreachable!`s where Marley's lint table denies both; porting its 780 lines draws
    less than Zed's crate does. `TableOfContents` (`src/navigation/toc.rs`) needs Ely's `motion`
    module and anchors its own renderer places, so it waits with the outline (Out).
    `ReadingProgress` is not needed.
  - Ely story `pagecover-pageicon-pageproperties`: `PageCover` and `PageIcon`
    (`src/documents/pages.rs`) are not ported, since no page of the vault sets a cover or an icon;
    `PropertyGrid` (`src/data_display/records.rs:117-222`) is an inspector of editors in folding
    groups, for the editing that is Out. Its sibling `DescriptionList` (`records.rs:12-86`), labels
    in a column with values beside them that drop under the label when short of room, is ported for
    the read-only properties: no component of Zed's `ui` crate lays out keys and values (`chip.rs`
    draws the list values; `data_table` is a grid with columns).

## UI proof
`script/e2e/645-brain-page-tab.sh` (`compositor sway`: it clicks rows and links). Setup writes a
scratch vault under `$E2E_WORK/vault` with three pages: `projects/atlas` (frontmatter with `title`,
`type`, `status` and a `tags` list; headings, a list, `- [x]` and `- [ ]` tasks, a fenced `rust`
block, a table, a piped wikilink to the decision, a heading link to its `## Why`, an unresolved
`[[ideas/later]]` and a web link), `decisions/use-zeds-renderer` (long enough that `## Why` is below
the fold) and `people/sam`. It links #643's stand-in as `$E2E_WORK/bin/rusty-mcp`, names it in
`MARLEY_RUSTY_MCP` and points it at that vault, sets `marley.rusty` on with the embedded connection
and `preview_tabs.enabled` and `preview_tabs.enable_preview_from_project_panel` true in the run's
copy of the settings (the user's own may turn previews off), and binds Ctrl+Alt+Shift+O to
`["rusty::OpenPage", {"slug": "ideas/nowhere"}]` in the run's keymap. Never the user's brain
(R-D8). Shots:
- `645-01-preview`: one click on Atlas in the Brain view: an italic Atlas tab; header, title,
  properties with tag chips, headings, the list, one task box checked and one not, resolved links in
  the link colour and `ideas/later` muted.
- `645-02-body`: the body scrolled: the `rust` block highlighted, the table.
- `645-03-replaced`: one click on Sam: one tab, italic Sam.
- `645-04-kept`: a double click on Atlas: one tab, Atlas upright.
- `645-05-wikilink`: a click on the alias "the renderer decision": the decision page, Back enabled.
- `645-06-back`: Alt-Left: Atlas again, Forward enabled.
- `645-07-forward`: Alt-Right: the decision page.
- `645-08-heading`: back to Atlas, a click on the heading link: the decision page with `Why` at the
  top.
- `645-09-unresolved`: back to Atlas, a click on `ideas/later`: the toast; the tab still Atlas.
- `645-10-found`: one click on Sam (a second, italic tab), then one click on Atlas: Atlas's tab
  comes forward; two tabs.
- `645-11-edit`: Sam's tab, Edit: the file's source with its frontmatter in an editor; still italic.
- `645-12-edit-keeps`: a line typed: the title upright with Zed's unsaved dot.
- `645-13-read-saves`: Read: the new line rendered, no dot; the scenario checks the file on disk
  holds the line and its other lines byte for byte.
- `645-14-live`: a line appended to `people/sam.md` from outside and the stand-in sent `SIGUSR1`
  (its `list_changed`, as Rusty's watcher sends one): shown with no input.
- `645-15-missing`: Ctrl+Alt+Shift+O: a tab saying there is no page `ideas/nowhere`.
- `645-16-not-connected`: `marley.rusty.enabled` set false from outside: the tab keeps its page and
  says it is not connected.

## Locked-In Decisions
- D1: **Rendering: Marley's pass, Zed's renderer.** A pure pass in `marley_rusty` over
  pulldown-cmark, with Rusty's options and `ENABLE_WIKILINKS`, rewrites each wikilink's byte range
  (from `into_offset_iter`) into an ordinary link whose address is Rusty's own `rusty:page/SLUG#H`
  or `rusty:new/TARGET`, inside angle brackets with `<`, `>` and `%` percent-encoded; Zed's
  `markdown` crate draws the result unchanged. Zed's crate already draws everything the page
  shows, highlights code through the language registry, selects and copies, scrolls to a heading,
  and takes link clicks and per-link styles, and it is what Zed's Markdown preview draws with, so
  the page looks like the rest of Marley. No Zed touchpoint. Rejected: (a) a node tree of Marley's
  own drawn as elements, Ely's way, which is a second renderer with less in it; (b) a switch in
  Zed's parser (`parser.rs:984-987`), a touchpoint in the option sets, their test,
  `MarkdownOptions` and the parse call, after which `on_url_click` would still receive only
  `foo` for both `[[foo]]` and `[x](foo)`; (c) porting Ely's `MarkdownRenderer` and
  `TableOfContents` (Prior art); (d) `brain_render`'s `html`, Qt rich text GPUI cannot draw.
- D2: **Links resolve by target, not by position.** The pass splits a heading or block part off at
  the first `#` or `^` and normalises the target as Rusty's `split_fragment` and
  `normalise_target` do, then looks it up among `links`' targets; an empty target is the page
  itself. A heading is found through Zed's heading slugs (`util::markdown::generate_heading_slug`),
  and a page without that heading opens at its top.
- D3: **The action lands here; the picker is the second slice.** `rusty::OpenPage` and
  `page::open_later` are the one way to open a page: #644's `open_page` (the tree, Enter, Today, a
  new page), #646's backlinks, #647's graph, keymaps and, once its ticket makes the slug optional,
  the palette's picker over `brain_list_pages` with create on a miss. The picker is a search screen
  of its own (Zed's `picker` crate or Ely's `SearchPalette`) with a write behind it, and #646 leaves
  it out as well, so it is its own ticket rather than a third half of this one. The `rusty`
  namespace matches the plan's `rusty: open page`; its cost is one line in Zed's namespace test.
- D4: **Preview tabs are Zed's.** The opener calls `Pane::replace_preview_item_id` and then
  `Pane::add_item` (as `open_project_item` does), and `unpreview_item_if_preview` to keep;
  `preview_tabs.enabled` off gives kept tabs, as for files. A preview opens without taking the
  focus, a keep takes it. The tab title is italic while previewed (`editor/src/items.rs:827`), and
  an edit in Edit keeps the tab through `ItemEvent::Edit` (`Pane::handle_item_edit`). Zed's tab
  double click, pin and `TogglePreviewTab` work as for any item. A tab is found by the page it shows
  now, not the one it opened on (PR-claude-599).
- D5: **Back and Forward are the tab's.** A per-tab history, as Rusty's `NoteTab.qml` keeps and as
  Marley's Browser tab binds Alt-Left and Alt-Right. Rejected: Zed's pane `nav_history`, whose
  entries cross tabs, so Back could leave the page, and which cannot reopen a closed non-file
  preview (`pane.rs:4938`, `workspace.rs:3290-3300`).
- D6: **Edit is the vault file in a Zed editor, inside the tab.** The toggle R-D3 names, one tab per
  page: Zed's whole editor, with Zed's save. Leaving the source saves first, as Rusty's page does,
  with `SaveOptions { format: false, autosave: true }` so prettier, which Zed allows for Markdown,
  does not rewrite a brain page Marley saved by itself; Ctrl-S follows the user's settings. The save
  lands on disk as an Obsidian edit does: `rusty-mcp`'s watcher announces it about 0.6 s later and
  its indexer re-reads the vault 5 s after that. Nothing commits it: `sync_all` writes no commit,
  and the next tool write's `git add -A` takes it under that tool's message, the same as an Obsidian
  edit. R-D4's "reindexes and commits them" is half right; the rest is a Rusty-side request (Out).
  Rejected: a separate editor tab beside the page (two tabs and no toggle); saving through
  `brain_write_page` from a buffer with no file (R-D4 chose disk; Zed's reload and conflict checks
  need a file).
- D7: **Properties are read only, in Ely's `DescriptionList` layout**, ported onto Zed's theme and
  the `ui` crate with Ely's MIT notice on the file; list values are `ui::Chip`s.
- D8: **Live by re-reading.** On #643's change signal the tab calls `brain_render` again and keeps
  its scroll; each read carries the navigation's generation and a stale answer is dropped. In Edit
  the buffer follows the file through Zed's own watching.
- D9: **Every opener and every link acts in `window.defer`.** A link click arrives inside the
  `Markdown` entity's update (Zed's preview defers for this reason,
  `markdown_preview_view.rs:1254-1257`), and the opener reads every Page tab, so it may not run
  inside one (AD-609, F-503).
- D10: **Not restored after a restart** (AD-609's reason: no store of the tab's history yet).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a page row of the rail's Brain view is clicked once, the system shall show that page in the active pane's preview tab, its title in italics. | Shot `645-01-preview` |
| REQ-002 | WHEN another page row is clicked once while a Page tab is the preview tab, the system shall show the new page in its place, in one preview tab. | Shot `645-03-replaced` |
| REQ-003 | WHEN a page row is double-clicked, the system shall keep that page's tab, its title upright. | Shot `645-04-kept` |
| REQ-004 | WHEN `rusty::OpenPage` names a page a tab of the workspace shows, the system shall bring that tab forward instead of opening another. | Shot `645-10-found` |
| REQ-005 | The Page tab shall show the page's title and its properties, in file order with list values as chips, above the body. | Shot `645-01-preview` |
| REQ-006 | The Page tab shall render the body's headings, lists and task boxes, each box checked as written. | Shot `645-01-preview` |
| REQ-007 | The Page tab shall render a fenced code block highlighted for its language, and a table in columns. | Shot `645-02-body` |
| REQ-008 | The Page tab shall draw a wikilink that resolves to no page in the muted text colour, and a resolved one in the link colour. | Shot `645-01-preview` |
| REQ-009 | WHEN a resolved wikilink is clicked, the tab shall show the linked page. | Shot `645-05-wikilink` |
| REQ-010 | WHEN a wikilink that names a heading is clicked, the tab shall show the linked page scrolled to that heading. | Shot `645-08-heading` |
| REQ-011 | WHEN Back is pressed, the tab shall show the page it showed before. | Shot `645-06-back` |
| REQ-012 | WHEN Forward is pressed after Back, the tab shall show the page it went back from. | Shot `645-07-forward` |
| REQ-013 | WHEN an unresolved wikilink is clicked, the system shall say the page does not exist yet and leave the tab as it is. | Shot `645-09-unresolved` |
| REQ-014 | WHEN Edit is pressed, the tab shall show the page's vault file in a Zed editor. | Shot `645-11-edit` |
| REQ-015 | WHEN the source is edited in a preview tab, the system shall keep the tab. | Shot `645-12-edit-keeps` |
| REQ-016 | WHILE the source has unsaved edits, the tab shall show Zed's unsaved dot, and closing it shall ask as Zed asks for a file. | Shot `645-12-edit-keeps`; review |
| REQ-017 | WHEN Read is pressed with unsaved edits, the system shall save them unformatted and show the page rendered as saved. | Shot `645-13-read-saves`; the scenario's file check |
| REQ-018 | WHEN Rusty announces a change while the tab shows Read, the tab shall show the page as it now is. | Shot `645-14-live` |
| REQ-019 | WHEN `rusty::OpenPage` names a slug with no page, the tab shall say there is no such page. | Shot `645-15-missing` |
| REQ-020 | WHILE Marley has no connection to Rusty, the tab shall keep the page it shows and say it is not connected. | Shot `645-16-not-connected` |
| REQ-021 | WHEN a web link in a page is clicked, the system shall open it with the system's handler. | Review (a scenario would reach the user's browser) |
| REQ-022 | WHILE `marley.rusty.enabled` is off, `rusty::OpenPage` shall open no Page tab. | Review |

## Phase Plan
- **P1 Plan**: promote after #643 and #644 complete; re-verify every seam the notes cite against
  the code as they shipped it (the client's call, connection state and `list_changed` signal, the
  stand-in, `rusty::brain::open_page`, whether `"rusty"` is in `test_action_namespaces`); ask the
  brain the rendering question; Chad confirms the rendering decision.
- **P2 Code**: the `README.md` marker first; the `zed.rs` row widened (if the namespace line is
  this ticket's); `marley_rusty::page` and its fixtures; the stand-in's additions; the actions, the
  opener, `PageView`, the properties port, the keymap block, #644's `open_page` pointed at the
  opener, the in-app guide line; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test**: write and run the scenario, read every shot.
- **P4 Complete**: CHANGELOG (Added) and architecture docs (§21), the user docs the notes list,
  R-D4's wording corrected in the plan, ledger capture (§19), the brain decision, close the ticket,
  archive, commit.
